import Link from "next/link";

import { WorkCard } from "../../components/work-card";
import {
  browseProblemAreaWorks,
  getScholarlyObjects,
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

/// The consumption surface: search + criterion/status/sort filters over the
/// registry card grid. Criterion filtering is domain-scoped (the active
/// domain's CWE nodes), not hard-coded.
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

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Discover</p>
          <h1>{query ? `Results for “${query}”` : "Discover Scholarly Works"}</h1>
          <p>
            Browse and search public audit subjects. Every result links to a
            full audit record.
          </p>
        </div>
        <Link className="secondary-action" href="/register">
          Register a missing work
        </Link>
      </header>

      <form action="/discover" className="pub-filterbar">
        <label>
          Search
          <input
            defaultValue={query}
            name="q"
            placeholder="Title, DOI, arXiv, PubMed, author, venue, or keyword"
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
        <div className="pub-grid">
          {filteredGroups.map((group) => (
            <WorkCard
              group={group}
              key={group.id}
              summary={summaries.get(group.primaryVersion.id) ?? null}
            />
          ))}
        </div>
      )}
    </>
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

function slug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "");
}
