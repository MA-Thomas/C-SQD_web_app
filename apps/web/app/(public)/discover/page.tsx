import Link from "next/link";

import { RegistryUnavailable } from "../../components/registry-unavailable";
import { StatusPill } from "../../components/status-pill";
import {
  browseProblemAreaWorks,
  formatLabel,
  getScholarlyObjects,
  isApiReachable,
  searchScholarlyObjects,
} from "../../lib/csqd-api";
import {
  getAcademicCweNodes,
  getPublicAuditSummariesForObjects,
  groupScholarlyObjects,
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../../lib/public-audit";

type PageProps = {
  searchParams: Promise<{
    criterion?: string;
    q?: string;
    sort?: string;
    status?: string;
  }>;
};

const statusOptions = [
  ["", "Any audit status"],
  ["unaudited", "Unaudited"],
  ["registered-for-audit", "Registered for audit"],
  ["elementreviews-submitted", "ElementReviews submitted"],
  ["in-synthesis", "In synthesis"],
  ["audit-report-available", "Audit report available"],
  ["challenged", "Challenged"],
  ["superseded", "Superseded"],
] as const;

const sortOptions = [
  ["recent", "Recent audit activity"],
  ["scrutiny", "Scrutiny depth"],
  ["reports", "Report availability"],
  ["uptake", "Uptake"],
] as const;

type DiscoverContext = {
  criterionLabel: string | null;
  query: string;
  selectedStatusLabel: string | null;
};

type CuratedDiscoverItem = {
  group: ScholarlyWorkGroup;
  summary: PublicAuditSummary | null;
};

type CuratedDiscoverSection = {
  deck: string;
  id: string;
  items: CuratedDiscoverItem[];
  title: string;
};

/// Directed discovery: search + filters create a curated set of audit objects
/// grouped by why they are worth inspecting, instead of another card gallery.
export default async function DiscoverPage({ searchParams }: PageProps) {
  const { criterion, q, sort, status } = await searchParams;
  const query = q?.trim() ?? "";
  const criterionId = criterion?.trim() ?? "";
  const selectedStatus = status?.trim() ?? "";
  const selectedSort = sortOptions.some(([value]) => value === sort)
    ? (sort ?? "recent")
    : "recent";
  const [cweNodes, objects] = await Promise.all([
    getAcademicCweNodes(),
    getDiscoverObjects(query, criterionId),
  ]);

  if (objects.length === 0 && !(await isApiReachable())) {
    return (
      <>
        <header className="pub-page-head">
          <div>
            <p className="pub-kicker">Discover</p>
            <h1>Directed Discovery</h1>
          </div>
        </header>
        <RegistryUnavailable />
      </>
    );
  }

  const groups = groupScholarlyObjects(objects);
  const summaries = await getPublicAuditSummariesForObjects(
    groups.map((group) => group.primaryVersion),
  );
  const filteredGroups = sortGroups(
    groups.filter((group) => {
      if (!selectedStatus) {
        return true;
      }

      return (
        slug(
          publicAuditStatusLabel(
            group.primaryVersion,
            summaries.get(group.primaryVersion.id),
          ),
        ) === selectedStatus
      );
    }),
    selectedSort,
    summaries,
  );
  const selectedNode = cweNodes.find((node) => node.id === criterionId) ?? null;
  const withReports = filteredGroups.filter(
    (group) =>
      (summaries.get(group.primaryVersion.id)?.synthesisReviewCount ??
        group.synthesisReviewCount) > 0,
  ).length;
  const discoverContext: DiscoverContext = {
    criterionLabel: selectedNode?.label ?? null,
    query,
    selectedStatusLabel: statusLabelForSlug(selectedStatus),
  };
  const curatedSections = curateDiscoverSections(
    filteredGroups,
    summaries,
    discoverContext,
  );
  const curationSource =
    query || selectedNode || selectedStatus
      ? "Curated from your topic and filters"
      : "Curated from public audit activity";

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Discover</p>
          <h1>{query ? `Directed Discovery: “${query}”` : "Directed Discovery"}</h1>
          <p>
            Describe a topic, category, or concern; C-SQD curates audit objects
            and shows why each one is surfaced.
          </p>
        </div>
        <div className="pub-head-actions">
          <Link className="secondary-action" href="/claims">
            Claims under audit
          </Link>
          <Link className="secondary-action" href="/register">
            Register a missing work
          </Link>
        </div>
      </header>

      <form action="/discover" className="pub-filterbar">
        <label>
          Topics or categories
          <input
            defaultValue={query}
            name="q"
            placeholder="e.g. tumor immunology, AI consent, causal inference"
            type="search"
          />
        </label>
        <label>
          Criterion
          <select defaultValue={criterionId} name="criterion">
            <option value="">Any criterion</option>
            {cweNodes.map((node) => (
              <option key={node.id} value={node.id}>
                {node.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Audit status
          <select defaultValue={selectedStatus} name="status">
            {statusOptions.map(([value, label]) => (
              <option key={value || "any"} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Sort by
          <select defaultValue={selectedSort} name="sort">
            {sortOptions.map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <button type="submit">Apply</button>
        {selectedNode ? (
          <p className="pub-filter-note">
            Filtering by criterion: {selectedNode.label} —{" "}
            <Link href="/criteria">about the criteria</Link>
          </p>
        ) : null}
      </form>

      <div className="pub-stat-strip" aria-label="Discover metrics">
        <span>{curationSource}</span>
        <span>
          <strong>{filteredGroups.length}</strong> matching works
        </span>
        <span>
          <strong>{withReports}</strong> with public reports
        </span>
        <span>
          <strong>{cweNodes.length}</strong> criteria in the active domain
        </span>
      </div>

      {filteredGroups.length === 0 ? (
        <div className="pub-empty">
          <h3>No public audit subjects found</h3>
          <p>
            Try a broader query, clear the criterion filter, or{" "}
            <Link href={`/register${query ? `?q=${encodeURIComponent(query)}` : ""}`}>
              register a missing scholarly work
            </Link>
            .
          </p>
        </div>
      ) : (
        <div className="directed-discovery">
          {curatedSections.map((section) => (
            <DiscoverCurationSection
              context={discoverContext}
              key={section.id}
              section={section}
            />
          ))}
        </div>
      )}
    </>
  );
}

function DiscoverCurationSection({
  context,
  section,
}: {
  context: DiscoverContext;
  section: CuratedDiscoverSection;
}) {
  return (
    <section className="pub-panel discover-section">
      <div className="pub-panel-head">
        <div>
          <p className="pub-kicker">Curated selection</p>
          <h2>{section.title}</h2>
        </div>
        <span className="pub-panel-count">{section.items.length} shown</span>
      </div>
      <p className="muted-copy discover-section-deck">{section.deck}</p>
      <ol className="discover-list">
        {section.items.map((item) => (
          <DiscoverRow
            context={context}
            item={item}
            key={`${section.id}-${item.group.id}`}
            sectionId={section.id}
          />
        ))}
      </ol>
    </section>
  );
}

function DiscoverRow({
  context,
  item,
  sectionId,
}: {
  context: DiscoverContext;
  item: CuratedDiscoverItem;
  sectionId: string;
}) {
  const { group, summary } = item;
  const object = group.primaryVersion;
  const status = publicAuditStatusLabel(object, summary);
  const reasons = itemReasons(item, context, sectionId);
  const reports = summary?.synthesisReviewCount ?? group.synthesisReviewCount;
  const reviews = summary?.elementReviewCount ?? group.elementReviewCount;
  const challenges = summary?.challengeCount ?? 0;

  return (
    <li className="discover-row">
      <div className="discover-row-main">
        <div className="discover-row-kicker">
          <StatusPill status={status} />
          <span>{formatLabel(object.object_type)}</span>
          {group.versionCount > 1 ? <span>{group.versionCount} versions</span> : null}
        </div>
        <h3>
          <Link href={`/works/${object.id}`}>{group.title}</Link>
        </h3>
        <p className="discover-row-source">{sourceLine(group)}</p>
        <ul className="discover-reasons" aria-label="Why shown">
          {reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      </div>
      <div className="discover-row-side">
        <dl>
          <div>
            <dt>Reviews</dt>
            <dd>{reviews}</dd>
          </div>
          <div>
            <dt>Reports</dt>
            <dd>{reports}</dd>
          </div>
          {challenges > 0 ? (
            <div>
              <dt>Challenges</dt>
              <dd>{challenges}</dd>
            </div>
          ) : null}
        </dl>
        <Link className="secondary-action" href={`/works/${object.id}`}>
          Open record
        </Link>
      </div>
    </li>
  );
}

async function getDiscoverObjects(query: string, criterionId: string) {
  if (criterionId) {
    const works = await browseProblemAreaWorks({ cweNodeId: criterionId, query });

    return works.map((work) => work.scholarly_object);
  }

  if (query) {
    return searchScholarlyObjects(query);
  }

  return getScholarlyObjects();
}

function sortGroups(
  groups: ScholarlyWorkGroup[],
  sort: string,
  summaries: Map<string, PublicAuditSummary>,
) {
  return [...groups].sort((left, right) => {
    const leftSummary = summaries.get(left.primaryVersion.id);
    const rightSummary = summaries.get(right.primaryVersion.id);

    if (sort === "scrutiny") {
      return (
        (rightSummary?.tuple?.scrutinyDepth ?? 0) -
        (leftSummary?.tuple?.scrutinyDepth ?? 0)
      );
    }

    if (sort === "reports") {
      return (
        (rightSummary?.synthesisReviewCount ?? right.synthesisReviewCount) -
        (leftSummary?.synthesisReviewCount ?? left.synthesisReviewCount)
      );
    }

    if (sort === "uptake") {
      return (rightSummary?.tuple?.uptake ?? 0) - (leftSummary?.tuple?.uptake ?? 0);
    }

    return (
      latestActivityTime(rightSummary?.latestReport?.authored_at) -
      latestActivityTime(leftSummary?.latestReport?.authored_at)
    );
  });
}

function latestActivityTime(value: string | undefined) {
  if (!value) {
    return 0;
  }

  const date = new Date(value);

  return Number.isNaN(date.getTime()) ? 0 : date.getTime();
}

function curateDiscoverSections(
  groups: ScholarlyWorkGroup[],
  summaries: Map<string, PublicAuditSummary>,
  context: DiscoverContext,
): CuratedDiscoverSection[] {
  const items = groups.map((group) => ({
    group,
    summary: summaries.get(group.primaryVersion.id) ?? null,
  }));
  const used = new Set<string>();
  const sections: CuratedDiscoverSection[] = [];
  const directed = Boolean(
    context.query || context.criterionLabel || context.selectedStatusLabel,
  );

  addSection(sections, used, {
    id: "closest",
    title: directed ? "Closest Audit Objects" : "Recent Audit Movement",
    deck: directed
      ? "The best starting points for the topic and filters you described."
      : "Default public curation based on recent audit reports and visible audit activity.",
    items,
    limit: 5,
  });

  addSection(sections, used, {
    id: "direct-claim-objects",
    title: "Works Serving As Claim Objects",
    deck: "Papers and preprints whose claims are being audited directly, not merely attached as evidence.",
    items: items
      .filter((item) => item.group.auditEpisodeCount > 0)
      .sort(
        (left, right) =>
          totalAuditActivity(right) - totalAuditActivity(left) ||
          latestItemActivity(right) - latestItemActivity(left),
      ),
    limit: 4,
  });

  addSection(sections, used, {
    id: "reports",
    title: "Reports Ready To Read",
    deck: "Audit objects with public synthesis available, useful when you want an interpretable audit record now.",
    items: items
      .filter((item) => reportCount(item) > 0)
      .sort(
        (left, right) =>
          reportCount(right) - reportCount(left) ||
          latestItemActivity(right) - latestItemActivity(left),
      ),
    limit: 4,
  });

  addSection(sections, used, {
    id: "scrutiny",
    title: "Contested Or High-Scrutiny Records",
    deck: "Records surfaced because they have challenges, deeper review activity, or unusually high scrutiny.",
    items: items
      .filter(
        (item) =>
          challengeCount(item) > 0 ||
          reviewCount(item) >= 2 ||
          (item.summary?.tuple?.scrutinyDepth ?? 0) >= 1,
      )
      .sort(
        (left, right) =>
          challengeCount(right) - challengeCount(left) ||
          (right.summary?.tuple?.scrutinyDepth ?? 0) -
            (left.summary?.tuple?.scrutinyDepth ?? 0) ||
          reviewCount(right) - reviewCount(left),
      ),
    limit: 4,
  });

  addSection(sections, used, {
    id: "needs-attention",
    title: "Needs Attention",
    deck: "Objects with little or no synthesis yet, where a review or audit report would materially improve the record.",
    items: items
      .filter((item) => reportCount(item) === 0)
      .sort(
        (left, right) =>
          reviewCount(right) - reviewCount(left) ||
          totalAuditActivity(right) - totalAuditActivity(left),
      ),
    limit: 4,
  });

  return sections;
}

function addSection(
  sections: CuratedDiscoverSection[],
  used: Set<string>,
  section: CuratedDiscoverSection & { limit: number },
) {
  const items = section.items.filter((item) => !used.has(item.group.id));
  const selected = items.slice(0, section.limit);

  if (selected.length === 0) {
    return;
  }

  for (const item of selected) {
    used.add(item.group.id);
  }

  sections.push({
    deck: section.deck,
    id: section.id,
    items: selected,
    title: section.title,
  });
}

function itemReasons(
  item: CuratedDiscoverItem,
  context: DiscoverContext,
  sectionId: string,
) {
  const reasons = [
    ...contextReasons(context),
    sectionReason(item, sectionId),
    activityReason(item),
  ].filter((reason): reason is string => Boolean(reason));

  return Array.from(new Set(reasons)).slice(0, 3);
}

function contextReasons(context: DiscoverContext) {
  const reasons: string[] = [];

  if (context.query) {
    reasons.push(`matches “${context.query}”`);
  }

  if (context.criterionLabel) {
    reasons.push(`connected to ${context.criterionLabel}`);
  }

  if (context.selectedStatusLabel) {
    reasons.push(`status: ${context.selectedStatusLabel}`);
  }

  return reasons;
}

function sectionReason(item: CuratedDiscoverItem, sectionId: string) {
  const objectType = formatLabel(item.group.primaryVersion.object_type).toLowerCase();

  switch (sectionId) {
    case "closest":
      return "ranked for this discovery view";
    case "direct-claim-objects":
      return `${objectType} is the direct audit object`;
    case "reports":
      return `${reportCount(item)} public report${reportCount(item) === 1 ? "" : "s"}`;
    case "scrutiny":
      if (challengeCount(item) > 0) {
        return `${challengeCount(item)} challenge${challengeCount(item) === 1 ? "" : "s"}`;
      }

      return `${reviewCount(item)} ElementReview${reviewCount(item) === 1 ? "" : "s"}`;
    case "needs-attention":
      return reviewCount(item) > 0
        ? "reviewed, but no synthesis yet"
        : "open record with no public report yet";
    default:
      return null;
  }
}

function activityReason(item: CuratedDiscoverItem) {
  if (item.group.versionCount > 1) {
    return `${item.group.versionCount} versions tracked`;
  }

  if (reviewCount(item) > 0) {
    return `${reviewCount(item)} ElementReview${reviewCount(item) === 1 ? "" : "s"}`;
  }

  if (reportCount(item) > 0) {
    return "has public synthesis";
  }

  return null;
}

function sourceLine(group: ScholarlyWorkGroup) {
  const object = group.primaryVersion;
  const authors = object.authors.slice(0, 3).join(", ");

  return [
    authors || null,
    object.source_name,
    object.publication_year?.toString() ?? null,
  ]
    .filter(Boolean)
    .join(" · ");
}

function reportCount(item: CuratedDiscoverItem) {
  return item.summary?.synthesisReviewCount ?? item.group.synthesisReviewCount;
}

function reviewCount(item: CuratedDiscoverItem) {
  return item.summary?.elementReviewCount ?? item.group.elementReviewCount;
}

function challengeCount(item: CuratedDiscoverItem) {
  return item.summary?.challengeCount ?? 0;
}

function totalAuditActivity(item: CuratedDiscoverItem) {
  return (
    reviewCount(item) +
    reportCount(item) +
    challengeCount(item) +
    item.group.auditEpisodeCount
  );
}

function latestItemActivity(item: CuratedDiscoverItem) {
  return latestActivityTime(item.summary?.latestReport?.authored_at);
}

function statusLabelForSlug(value: string) {
  return statusOptions.find(([slugValue]) => slugValue === value)?.[1] ?? null;
}

function slug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "");
}
