import Link from "next/link";

import {
  browseProblemAreaWorks,
  formatLabel,
  getDomainInstantiation,
  getDomainInstantiations,
  type ProblemAreaWorkSummary,
  type ScholarlyObjectSummary,
} from "../../lib/csqd-api";

type PageProps = {
  searchParams: Promise<{
    problem_area?: string;
    q?: string;
  }>;
};

export default async function BrowsePage({ searchParams }: PageProps) {
  const { problem_area, q } = await searchParams;
  const problemAreaId = problem_area?.trim() ?? "";
  const query = q?.trim() ?? "";
  const [cweNodes, problemAreaWorks] = await Promise.all([
    getAcademicCweNodes(),
    browseProblemAreaWorks({ cweNodeId: problemAreaId, query }),
  ]);
  const selectedNode =
    cweNodes.find((node) => node.id === problemAreaId) ?? null;
  const workGroups = groupProblemAreaWorks(problemAreaWorks);
  const factLinkedWorkCount = workGroups.filter(
    (group) => group.problemFactCount > 0,
  ).length;
  const problemFactCount = workGroups.reduce(
    (sum, group) => sum + group.problemFactCount,
    0,
  );

  return (
          <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Academic Peer Review domain</p>
            <h1>Browse Problem Areas</h1>
          </div>
          <div className="status-pill">
            {selectedNode ? selectedNode.label : "All problem areas"}
          </div>
        </header>

        <form className="retrieval-form browse-form" action="/browse">
          <label htmlFor="problem-area-query">Problem area</label>
          <div className="retrieval-controls">
            <input
              defaultValue={query}
              id="problem-area-query"
              name="q"
              placeholder="Problem, topic, claim, method, or evidence concern"
              type="search"
            />
            <button type="submit">Browse</button>
          </div>
          <label className="browse-select-label" htmlFor="problem-area-select">
            <span>Peer-review criterion</span>
            <select
              defaultValue={selectedNode?.id ?? ""}
              id="problem-area-select"
              name="problem_area"
            >
              <option value="">Any criterion</option>
              {cweNodes.map((node) => (
                <option key={node.id} value={node.id}>
                  {node.label}
                </option>
              ))}
            </select>
          </label>
        </form>

        <section className="metric-grid" aria-label="Browse metrics">
          <div className="metric">
            <span>Problem areas</span>
            <strong>{cweNodes.length}</strong>
          </div>
          <div className="metric">
            <span>Related works</span>
            <strong>{workGroups.length}</strong>
          </div>
          <div className="metric">
            <span>Criterion facts</span>
            <strong>{problemFactCount}</strong>
          </div>
        </section>

        <section className="problem-area-grid" aria-label="Peer-review problem areas">
          <Link
            className={`problem-area-card${selectedNode ? "" : " active"}`}
            href={browseHref(query, "")}
          >
            <strong>All problem areas</strong>
            <span>{factLinkedWorkCount} works with criterion-linked facts</span>
          </Link>
          {cweNodes.map((node) => (
            <Link
              className={`problem-area-card${
                selectedNode?.id === node.id ? " active" : ""
              }`}
              href={browseHref(query, node.id)}
              key={node.id}
            >
              <strong>{node.label}</strong>
              <span>{node.description}</span>
            </Link>
          ))}
        </section>

        <section className="object-list" aria-label="Problem-area work list">
          {workGroups.length === 0 ? (
            <div className="empty-state">
              <h2>No related works found</h2>
              <p>
                {selectedNode
                  ? `${selectedNode.label} has no matching local works yet.`
                  : "No local works match this problem area yet."}
              </p>
            </div>
          ) : (
            workGroups.map((group) => {
              return (
                <article className="object-card work-card" key={group.id}>
                  <div className="object-main">
                    <div className="object-kicker">
                      <span>Problem-area work</span>
                      <span>{formatLabel(group.relevance)}</span>
                      {group.problemFactCount > 0 ? (
                        <span>
                          {group.problemFactCount} criterion{" "}
                          {group.problemFactCount === 1 ? "fact" : "facts"}
                        </span>
                      ) : null}
                      {group.primaryVersion.publication_year ? (
                        <span>{group.primaryVersion.publication_year}</span>
                      ) : null}
                    </div>
                    <h2>{group.title}</h2>
                    <p>{group.primaryVersion.authors.join(", ")}</p>
                    <div className="object-actions">
                      <Link href={`/scholarly-objects/${group.primaryVersion.id}`}>
                        Open audit record
                      </Link>
                      {group.primaryVersion.audit_subject_id ? (
                        <Link
                          href={`/commission?subject_id=${group.primaryVersion.audit_subject_id}`}
                        >
                          Commission audit
                        </Link>
                      ) : null}
                      <Link
                        href={`/scholarly-objects/${group.primaryVersion.id}/review?criterion=${problemAreaId}`}
                      >
                        Review this criterion
                      </Link>
                      <Link
                        href={`/sign-in?return_to=${encodeURIComponent(
                          `/scholarly-objects/${group.primaryVersion.id}`,
                        )}&intent=watch`}
                      >
                        Watch
                      </Link>
                    </div>
                    <div className="version-context-list work-version-list">
                      {group.versions.map((version) => (
                        <Link
                          className="version-context-row"
                          href={`/scholarly-objects/${version.id}`}
                          key={version.id}
                        >
                          <div>
                            <strong>
                              {formatLabel(versionKindForObject(version))}
                            </strong>
                            <span>
                              {version.source_name}
                              {version.publication_year
                                ? ` - ${version.publication_year}`
                                : ""}
                            </span>
                          </div>
                          <span>{formatLabel(version.audit_status)}</span>
                        </Link>
                      ))}
                    </div>
                  </div>
                  <dl className="object-facts">
                    <div>
                      <dt>Status</dt>
                      <dd>{formatLabel(group.auditStatus)}</dd>
                    </div>
                    <div>
                      <dt>Facts</dt>
                      <dd>{group.factCount}</dd>
                    </div>
                    <div>
                      <dt>Problem facts</dt>
                      <dd>{group.problemFactCount}</dd>
                    </div>
                  </dl>
                </article>
              );
            })
          )}
        </section>
      </section>
  );
}

async function getAcademicCweNodes() {
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

type ProblemAreaWorkGroup = {
  id: string;
  title: string;
  primaryVersion: ScholarlyObjectSummary;
  versions: ScholarlyObjectSummary[];
  versionCount: number;
  auditStatus: string;
  factCount: number;
  problemFactCount: number;
  relevance: string;
};

function groupProblemAreaWorks(
  works: ProblemAreaWorkSummary[],
): ProblemAreaWorkGroup[] {
  const groups = new Map<string, ProblemAreaWorkGroup>();

  for (const work of works) {
    const object = work.scholarly_object;
    const groupId = workIdentityForObject(object);
    const existing = groups.get(groupId);

    if (existing) {
      existing.versions.push(object);
      existing.versionCount = Math.max(
        existing.versionCount,
        object.work_group?.version_count ?? existing.versions.length,
      );
      existing.factCount += object.fact_count;
      existing.problemFactCount += work.problem_fact_count;

      if (relevanceRank(work.relevance) < relevanceRank(existing.relevance)) {
        existing.relevance = work.relevance;
      }

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
        factCount: object.fact_count,
        problemFactCount: work.problem_fact_count,
        relevance: work.relevance,
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

function browseHref(query: string, problemAreaId: string) {
  const params = new URLSearchParams();

  if (query) {
    params.set("q", query);
  }

  if (problemAreaId) {
    params.set("problem_area", problemAreaId);
  }

  const queryString = params.toString();

  return queryString ? `/browse?${queryString}` : "/browse";
}

function versionKindForObject(object: ScholarlyObjectSummary) {
  if (object.version_kind) {
    return object.version_kind;
  }

  return object.object_type === "preprint" ? "preprint" : "publisher";
}

function normalizedTitle(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function workIdentityForObject(object: ScholarlyObjectSummary) {
  return object.work_group?.id ?? normalizedTitle(object.title);
}

function versionRank(versionKind: string) {
  switch (versionKind) {
    case "publisher":
      return 0;
    case "preprint":
      return 10;
    case "repository":
      return 20;
    default:
      return 99;
  }
}

function highestAuditStatus(versions: ScholarlyObjectSummary[]) {
  return versions
    .map((version) => version.audit_status)
    .sort((left, right) => auditStatusRank(left) - auditStatusRank(right))[0];
}

function auditStatusRank(status: string) {
  switch (status) {
    case "delivered":
      return 0;
    case "synthesis_pending":
      return 1;
    case "in_progress":
      return 2;
    case "commissioned":
      return 3;
    default:
      return 4;
  }
}

function relevanceRank(relevance: string) {
  switch (relevance) {
    case "fact_activity":
      return 0;
    case "text_match":
      return 1;
    default:
      return 2;
  }
}
