import Link from "next/link";

import { PublicWorkCard } from "../../components/public-work-card";
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

export default async function DiscoverPage({ searchParams }: PageProps) {
  const { criterion, q, sort, status } = await searchParams;
  const query = q?.trim() ?? "";
  const criterionId = criterion?.trim() ?? "";
  const selectedStatus = status?.trim() ?? "";
  const selectedSort = sortOptions.some(([value]) => value === sort)
    ? sort ?? "recent"
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
        slug(publicAuditStatusLabel(group.primaryVersion, summaries.get(group.primaryVersion.id))) ===
        selectedStatus
      );
    }),
    selectedSort,
    summaries,
  );
  const selectedNode = cweNodes.find((node) => node.id === criterionId) ?? null;

  return (
          <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Public Discover</p>
            <h1>Discover Scholarly Works</h1>
          </div>
          <Link className="status-pill" href="/intake">
            Search / register missing work
          </Link>
        </header>

        <form className="retrieval-form discover-form" action="/discover">
          <label htmlFor="discover-query">Search</label>
          <div className="retrieval-controls">
            <input
              defaultValue={query}
              id="discover-query"
              name="q"
              placeholder="Title, DOI, arXiv, PubMed, author, venue, or keyword"
              type="search"
            />
            <button type="submit">Discover</button>
          </div>
          <div className="filter-grid">
            <label htmlFor="criterion-filter">
              <span>CRWE criterion</span>
              <select defaultValue={criterionId} id="criterion-filter" name="criterion">
                <option value="">Any criterion</option>
                {cweNodes.map((node) => (
                  <option key={node.id} value={node.id}>
                    {node.label}
                  </option>
                ))}
              </select>
            </label>
            <label htmlFor="status-filter">
              <span>Audit status</span>
              <select defaultValue={selectedStatus} id="status-filter" name="status">
                {statusOptions.map(([value, label]) => (
                  <option key={value || "any"} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <label htmlFor="sort-filter">
              <span>Sort by</span>
              <select defaultValue={selectedSort} id="sort-filter" name="sort">
                {sortOptions.map(([value, label]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        </form>

        <section className="metric-grid" aria-label="Discover metrics">
          <div className="metric">
            <span>Matching works</span>
            <strong>{filteredGroups.length}</strong>
          </div>
          <div className="metric">
            <span>Selected CRWE</span>
            <strong>{selectedNode ? "1" : cweNodes.length}</strong>
          </div>
          <div className="metric">
            <span>With reports</span>
            <strong>
              {
                filteredGroups.filter(
                  (group) =>
                    (summaries.get(group.primaryVersion.id)?.synthesisReviewCount ??
                      group.synthesisReviewCount) > 0,
                ).length
              }
            </strong>
          </div>
        </section>

        <section className="object-list" aria-label="Discover results">
          {filteredGroups.length === 0 ? (
            <div className="empty-state">
              <h2>No public audit subjects found</h2>
              <p>
                Try a broader query, clear the CRWE filter, or register a missing
                scholarly work.
              </p>
            </div>
          ) : (
            filteredGroups.map((group) => (
              <PublicWorkCard
                group={group}
                key={group.id}
                summary={summaries.get(group.primaryVersion.id) ?? null}
              />
            ))
          )}
        </section>
      </section>
  );
}

async function getDiscoverObjects(query: string, criterionId: string) {
  if (criterionId) {
    const works = await browseProblemAreaWorks({
      cweNodeId: criterionId,
      query,
    });

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
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}
