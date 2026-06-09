import Link from "next/link";
import { revalidatePath } from "next/cache";

import { AppSidebar } from "../components/app-sidebar";
import {
  addLibraryItem,
  browseProblemAreaWorks,
  formatLabel,
  getDomainInstantiation,
  getDomainInstantiations,
  getLibraryItems,
  type ProblemAreaWorkSummary,
  type ScholarlyObjectSummary,
} from "../lib/csqd-api";

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
  const [cweNodes, libraryItems, problemAreaWorks] = await Promise.all([
    getAcademicCweNodes(),
    getLibraryItems(),
    browseProblemAreaWorks({ cweNodeId: problemAreaId, query }),
  ]);
  const selectedNode =
    cweNodes.find((node) => node.id === problemAreaId) ?? null;
  const workGroups = groupProblemAreaWorks(problemAreaWorks);
  const libraryWorkIds = new Set(
    libraryItems.map((item) => workIdentityForObject(item.scholarly_object)),
  );
  const reviewLinkedWorkCount = workGroups.filter(
    (group) => group.problemReviewEventCount > 0,
  ).length;
  const problemReviewEventCount = workGroups.reduce(
    (sum, group) => sum + group.problemReviewEventCount,
    0,
  );

  return (
    <main className="app-shell">
      <AppSidebar activeItem="browse" />

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
            <span>Criterion reviews</span>
            <strong>{problemReviewEventCount}</strong>
          </div>
        </section>

        <section className="problem-area-grid" aria-label="Peer-review problem areas">
          <Link
            className={`problem-area-card${selectedNode ? "" : " active"}`}
            href={browseHref(query, "")}
          >
            <strong>All problem areas</strong>
            <span>{reviewLinkedWorkCount} works with criterion-linked reviews</span>
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
              const isInLibrary = libraryWorkIds.has(group.id);

              return (
                <article className="object-card work-card" key={group.id}>
                  <div className="object-main">
                    <div className="object-kicker">
                      <span>Problem-area work</span>
                      <span>{formatLabel(group.relevance)}</span>
                      {group.problemReviewEventCount > 0 ? (
                        <span>
                          {group.problemReviewEventCount} criterion{" "}
                          {group.problemReviewEventCount === 1 ? "review" : "reviews"}
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
                        Open primary
                      </Link>
                      <Link
                        href={`/scholarly-objects/${group.primaryVersion.id}/review`}
                      >
                        Start review
                      </Link>
                      {isInLibrary ? (
                        <span className="library-state">In Library</span>
                      ) : (
                        <form action={addToLibraryAction}>
                          <input
                            name="scholarly_object_id"
                            type="hidden"
                            value={group.primaryVersion.id}
                          />
                          <button
                            className="secondary-action action-button"
                            type="submit"
                          >
                            Add to Library
                          </button>
                        </form>
                      )}
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
                          <span>{formatLabel(version.review_status)}</span>
                        </Link>
                      ))}
                    </div>
                  </div>
                  <dl className="object-facts">
                    <div>
                      <dt>Status</dt>
                      <dd>{formatLabel(group.reviewStatus)}</dd>
                    </div>
                    <div>
                      <dt>Events</dt>
                      <dd>{group.reviewEventCount}</dd>
                    </div>
                    <div>
                      <dt>Problem reviews</dt>
                      <dd>{group.problemReviewEventCount}</dd>
                    </div>
                  </dl>
                </article>
              );
            })
          )}
        </section>
      </section>
    </main>
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

async function addToLibraryAction(formData: FormData) {
  "use server";

  const scholarlyObjectId = String(formData.get("scholarly_object_id") ?? "");

  if (scholarlyObjectId) {
    await addLibraryItem(scholarlyObjectId);
    revalidatePath("/browse");
    revalidatePath("/library");
  }
}

type ProblemAreaWorkGroup = {
  id: string;
  title: string;
  primaryVersion: ScholarlyObjectSummary;
  versions: ScholarlyObjectSummary[];
  versionCount: number;
  reviewStatus: string;
  reviewEventCount: number;
  problemReviewEventCount: number;
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
      existing.reviewEventCount += object.review_event_count;
      existing.problemReviewEventCount += work.problem_review_event_count;

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
        reviewStatus: object.review_status,
        reviewEventCount: object.review_event_count,
        problemReviewEventCount: work.problem_review_event_count,
        relevance: work.relevance,
      });
    }
  }

  return Array.from(groups.values()).map((group) => ({
    ...group,
    versionCount: Math.max(group.versionCount, group.versions.length),
    reviewStatus: highestReviewStatus(group.versions),
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

function highestReviewStatus(versions: ScholarlyObjectSummary[]) {
  return versions
    .map((version) => version.review_status)
    .sort((left, right) => reviewStatusRank(left) - reviewStatusRank(right))[0];
}

function reviewStatusRank(status: string) {
  switch (status) {
    case "published":
      return 0;
    case "submitted":
      return 1;
    case "in_review":
      return 2;
    case "assigned":
      return 3;
    default:
      return 4;
  }
}

function relevanceRank(relevance: string) {
  switch (relevance) {
    case "review_activity":
      return 0;
    case "text_match":
      return 1;
    default:
      return 2;
  }
}
