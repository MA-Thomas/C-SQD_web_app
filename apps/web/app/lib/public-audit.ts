import {
  getDomainInstantiation,
  getDomainInstantiations,
  getFactsForSubject,
  getPublicSummariesForScholarlyObjects,
  getPublicSummaryForScholarlyObject,
  type AuditEpisode,
  type CWENode,
  type Fact,
  type PublicSubjectSummaryApi,
  type ScholarlyObjectSummary,
  type SynthesisReview,
} from "./csqd-api";

export type ScholarlyWorkGroup = {
  id: string;
  title: string;
  primaryVersion: ScholarlyObjectSummary;
  versions: ScholarlyObjectSummary[];
  versionCount: number;
  auditStatus: string;
  auditEpisodeCount: number;
  factCount: number;
  elementReviewCount: number;
  synthesisReviewCount: number;
};

export type PublicTupleSummary = {
  problems: number;
  ethicalConcerns: number;
  stakes: number;
  scrutinyDepth: number;
  uptake: number;
};

export type PublicAuditSummary = {
  episodes: AuditEpisode[];
  /// Only populated by `getPublicAuditSummaryForObject` (subject pages);
  /// batch summaries carry counts, not raw facts.
  facts: Fact[];
  tuple: PublicTupleSummary | null;
  statusLabel: string | null;
  crweCoverageCount: number;
  crweReviewedNodeIds: string[];
  elementReviewCount: number;
  synthesisReviewCount: number;
  challengeCount: number;
  latestReport: SynthesisReview | null;
};

export type CriterionReviewGroup = {
  node: CWENode;
  reviews: Fact[];
};

export async function getAcademicCweNodes() {
  const domains = await getDomainInstantiations();
  const academicDomain = domains.find(
    (domain) => domain.domain_type === "academic_publishing",
  );

  if (!academicDomain) {
    return [];
  }

  const detail = await getDomainInstantiation(academicDomain.id);

  return detail?.cwe_nodes ?? [];
}

function summaryFromApi(
  api: PublicSubjectSummaryApi | null,
  facts: Fact[] = [],
): PublicAuditSummary {
  if (!api) {
    return emptyPublicAuditSummary();
  }

  return {
    episodes: api.episodes,
    facts,
    tuple: api.tuple
      ? {
          problems: api.tuple.problems,
          ethicalConcerns: api.tuple.ethical_concerns,
          stakes: api.tuple.stakes,
          scrutinyDepth: api.tuple.scrutiny_depth,
          uptake: api.tuple.uptake,
        }
      : null,
    statusLabel: api.status_label,
    crweCoverageCount: api.crwe_reviewed_node_ids.length,
    crweReviewedNodeIds: api.crwe_reviewed_node_ids,
    elementReviewCount: api.element_review_count,
    synthesisReviewCount: api.synthesis_review_count,
    challengeCount: api.challenge_count,
    latestReport: api.latest_report,
  };
}

/// Single-subject summary, including raw public facts for criterion grouping
/// on the audit subject page. Two API calls total.
export async function getPublicAuditSummaryForObject(
  object: ScholarlyObjectSummary,
): Promise<PublicAuditSummary> {
  const api = await getPublicSummaryForScholarlyObject(object.id);
  const facts = object.audit_subject_id
    ? await getFactsForSubject(object.audit_subject_id)
    : [];

  return summaryFromApi(api, facts);
}

/// Batch summaries: one API call per 100 works (previously: 2 + 2·episodes
/// calls per work).
export async function getPublicAuditSummariesForObjects(
  objects: ScholarlyObjectSummary[],
): Promise<Map<string, PublicAuditSummary>> {
  const summaries = await getPublicSummariesForScholarlyObjects(
    objects.map((object) => object.id),
  );
  const byObjectId = new Map<string, PublicAuditSummary>();

  for (const summary of summaries) {
    if (summary.scholarly_object_id) {
      byObjectId.set(summary.scholarly_object_id, summaryFromApi(summary));
    }
  }

  for (const object of objects) {
    if (!byObjectId.has(object.id)) {
      byObjectId.set(object.id, emptyPublicAuditSummary());
    }
  }

  return byObjectId;
}

export function groupScholarlyObjects(
  objects: ScholarlyObjectSummary[],
): ScholarlyWorkGroup[] {
  const groups = new Map<string, ScholarlyWorkGroup>();

  for (const object of objects) {
    const groupId = workIdentityForObject(object);
    const existing = groups.get(groupId);

    if (existing) {
      existing.versions.push(object);
      existing.versionCount = Math.max(
        existing.versionCount,
        object.work_group?.version_count ?? existing.versions.length,
      );
      existing.auditEpisodeCount += object.audit_episode_count;
      existing.factCount += object.fact_count;
      existing.elementReviewCount += object.element_review_fact_count;
      existing.synthesisReviewCount += object.synthesis_review_count;

      if (
        versionRank(versionKindForObject(object)) <
        versionRank(versionKindForObject(existing.primaryVersion))
      ) {
        existing.primaryVersion = object;
      }
    } else {
      groups.set(groupId, {
        id: groupId,
        title: object.work_group?.title ?? object.title,
        primaryVersion: object,
        versions: [object],
        versionCount: object.work_group?.version_count ?? 1,
        auditStatus: object.audit_status,
        auditEpisodeCount: object.audit_episode_count,
        factCount: object.fact_count,
        elementReviewCount: object.element_review_fact_count,
        synthesisReviewCount: object.synthesis_review_count,
      });
    }
  }

  return Array.from(groups.values()).map((group) => ({
    ...group,
    versionCount: Math.max(group.versionCount, group.versions.length),
    auditStatus: highestAuditStatus(group.versions),
    versions: group.versions.sort(
      (left, right) =>
        versionRank(versionKindForObject(left)) -
        versionRank(versionKindForObject(right)),
    ),
  }));
}

/// Status labels are computed server-side; this prefers the API label and
/// falls back to adapter-level work status for unsummarized objects.
export function publicAuditStatusLabel(
  object: ScholarlyObjectSummary,
  summary?: PublicAuditSummary | null,
) {
  if (summary?.statusLabel) {
    return summary.statusLabel;
  }

  switch (object.audit_status) {
    case "delivered":
      return "Audit report available";
    case "synthesis_pending":
      return "In synthesis";
    case "in_progress":
      return "ElementReviews submitted";
    case "closed":
      return "Superseded";
    case "commissioned":
      return "Registered for audit";
    default:
      return object.element_review_fact_count > 0
        ? "ElementReviews submitted"
        : "Unaudited";
  }
}

export function tupleItems(tuple: PublicTupleSummary | null) {
  return [
    ["Problems", tuple?.problems ?? 0],
    ["Ethical concerns", tuple?.ethicalConcerns ?? 0],
    ["Stakes", tuple?.stakes ?? 0],
    ["Scrutiny depth", tuple?.scrutinyDepth ?? 0],
    ["Uptake", tuple?.uptake ?? 0],
  ] as const;
}

export function groupedElementReviewsByCriterion(
  facts: Fact[],
  nodes: CWENode[],
): CriterionReviewGroup[] {
  return nodes.map((node) => ({
    node,
    reviews: facts.filter((fact) => {
      const payload = payloadRecord(fact, "element_review");

      return criterionNodeId(payload) === node.id;
    }),
  }));
}

export function factKind(fact: Fact | undefined) {
  if (!fact?.payload || typeof fact.payload !== "object") {
    return "fact";
  }

  const [kind] = Object.keys(fact.payload as Record<string, unknown>);

  return kind ?? "fact";
}

export function payloadRecord(fact: Fact | undefined, kind: string) {
  if (!fact?.payload || typeof fact.payload !== "object") {
    return null;
  }

  const payload = (fact.payload as Record<string, unknown>)[kind];

  return payload && typeof payload === "object"
    ? (payload as Record<string, unknown>)
    : null;
}

export function criterionNodeId(payload: Record<string, unknown> | null) {
  const criterion = payload?.cwe_criterion;

  if (!criterion || typeof criterion !== "object") {
    return "";
  }

  return stringValue((criterion as Record<string, unknown>).node_id);
}

export function stringValue(value: unknown) {
  return typeof value === "string" ? value : "";
}

export function formatCount(value: number) {
  return value.toLocaleString("en", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  });
}

export function formatDate(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

export function versionKindForObject(object: ScholarlyObjectSummary) {
  if (object.version_kind) {
    return object.version_kind;
  }

  return object.object_type === "preprint" ? "preprint" : "publisher";
}

export function workIdentityForObject(object: ScholarlyObjectSummary) {
  return object.work_group?.id ?? normalizedTitle(object.title);
}

function emptyPublicAuditSummary(): PublicAuditSummary {
  return {
    episodes: [],
    facts: [],
    tuple: null,
    statusLabel: null,
    crweCoverageCount: 0,
    crweReviewedNodeIds: [],
    elementReviewCount: 0,
    synthesisReviewCount: 0,
    challengeCount: 0,
    latestReport: null,
  };
}

function normalizedTitle(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

const VERSION_RANKS: Record<string, number> = {
  publisher: 0,
  accepted: 1,
  preprint: 2,
  other: 3,
};

function versionRank(kind: string) {
  return VERSION_RANKS[kind] ?? 3;
}

const STATUS_RANKS: Record<string, number> = {
  delivered: 0,
  synthesis_pending: 1,
  in_progress: 2,
  commissioned: 3,
  closed: 4,
  not_commissioned: 5,
};

function highestAuditStatus(objects: ScholarlyObjectSummary[]) {
  return objects.reduce((best, object) => {
    const current = STATUS_RANKS[object.audit_status] ?? 5;
    const bestRank = STATUS_RANKS[best] ?? 5;

    return current < bestRank ? object.audit_status : best;
  }, objects[0]?.audit_status ?? "not_commissioned");
}
