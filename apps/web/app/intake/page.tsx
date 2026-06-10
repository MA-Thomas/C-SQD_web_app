import Link from "next/link";
import { revalidatePath } from "next/cache";

import { AppSidebar } from "../components/app-sidebar";
import {
  addLibraryItem,
  formatLabel,
  getLibraryItems,
  retrieveArticle,
  searchScholarlyObjects,
  type ScholarlyObjectSummary,
} from "../lib/csqd-api";

type PageProps = {
  searchParams: Promise<{
    include_preprints?: string;
    q?: string;
  }>;
};

export default async function IntakePage({ searchParams }: PageProps) {
  const { include_preprints, q } = await searchParams;
  const query = q?.trim() ?? "";
  const includePreprints = isCheckedSearchParam(include_preprints);
  let objects = query ? await searchScholarlyObjects(query) : [];
  const libraryItems = await getLibraryItems();
  const libraryWorkIds = new Set(
    libraryItems.map((item) => workIdentityForObject(item.scholarly_object)),
  );
  let objectGroups = groupObjectsByWork(objects);
  let retrievalError: string | null = null;
  const shouldRetrieve =
    query &&
    (objectGroups.length === 0 ||
      (includePreprints && !objectGroups.some((group) => group.hasPreprint)));

  if (shouldRetrieve) {
    const retrieval = await retrieveArticle(query, { includePreprints });

    if (retrieval.error) {
      retrievalError = retrieval.error;
    } else {
      objects = await searchScholarlyObjects(query);
      objectGroups = groupObjectsByWork(objects);
    }
  }

  return (
    <main className="app-shell">
      <AppSidebar activeItem="intake" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Academic Publishing intake</p>
            <h1>Scholarly Intake</h1>
          </div>
          <div className="status-pill">Metadata + subject registration</div>
        </header>

        <form className="retrieval-form" action="/intake">
          <label htmlFor="work-query">Work</label>
          <div className="retrieval-controls">
            <input
              defaultValue={query}
              id="work-query"
              name="q"
              placeholder="Title, DOI, PMID, PMCID, arXiv ID, article URL, author, or venue"
              type="search"
            />
            <button type="submit">Search</button>
          </div>
          <label className="retrieval-option" htmlFor="include-preprints">
            <input
              defaultChecked={includePreprints}
              id="include-preprints"
              name="include_preprints"
              type="checkbox"
              value="true"
            />
            <span>Include matching preprints</span>
          </label>
        </form>

        <section className="metric-grid" aria-label="Intake metrics">
          <div className="metric">
            <span>Matches</span>
            <strong>{objectGroups.length}</strong>
          </div>
          <div className="metric">
            <span>Versions</span>
            <strong>{objects.length}</strong>
          </div>
          <div className="metric">
            <span>Facts</span>
            <strong>{objects.reduce((sum, object) => sum + object.fact_count, 0)}</strong>
          </div>
        </section>

        <section className="object-list" aria-label="Scholarly intake list">
          {objectGroups.length === 0 ? (
            <div className="empty-state">
              <h2>{query ? "No work found" : "Search scholarly works"}</h2>
              <p>
                {query
                  ? retrievalError ?? "No local or retrievable record matched this query."
                  : "Use a title, DOI, PMID, PMCID, arXiv ID, article URL, author, or venue."}
              </p>
            </div>
          ) : (
            objectGroups.map((group) => {
              const isInLibrary = libraryWorkIds.has(group.id);
              const subjectId = group.primaryVersion.audit_subject_id;

              return (
                <article className="object-card work-card" key={group.id}>
                  <div className="object-main">
                    <div className="object-kicker">
                      <span>Audit subject intake</span>
                      <span>
                        {group.versionCount}{" "}
                        {group.versionCount === 1 ? "version" : "versions"}
                      </span>
                      {group.primaryVersion.publication_year ? (
                        <span>{group.primaryVersion.publication_year}</span>
                      ) : null}
                    </div>
                    <h2>{group.title}</h2>
                    <p>{group.primaryVersion.authors.join(", ")}</p>
                    <div className="object-actions">
                      <Link href={`/scholarly-objects/${group.primaryVersion.id}`}>
                        Open intake record
                      </Link>
                      {subjectId ? (
                        <Link href={`/commission?subject_id=${subjectId}`}>
                          Commission audit
                        </Link>
                      ) : null}
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
                      <dt>Episodes</dt>
                      <dd>{group.auditEpisodeCount}</dd>
                    </div>
                    <div>
                      <dt>Facts</dt>
                      <dd>{group.factCount}</dd>
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

async function addToLibraryAction(formData: FormData) {
  "use server";

  const scholarlyObjectId = String(formData.get("scholarly_object_id") ?? "");

  if (scholarlyObjectId) {
    await addLibraryItem(scholarlyObjectId);
    revalidatePath("/intake");
    revalidatePath("/library");
  }
}

type ScholarlyWorkGroup = {
  id: string;
  title: string;
  primaryVersion: ScholarlyObjectSummary;
  versions: ScholarlyObjectSummary[];
  versionCount: number;
  auditStatus: string;
  auditEpisodeCount: number;
  factCount: number;
  hasPreprint: boolean;
};

function groupObjectsByWork(objects: ScholarlyObjectSummary[]): ScholarlyWorkGroup[] {
  const groups = new Map<string, ScholarlyWorkGroup>();

  for (const object of objects) {
    const groupId = object.work_group?.id ?? normalizedTitle(object.title);
    const existing = groups.get(groupId);

    if (existing) {
      existing.versions.push(object);
      existing.versionCount = Math.max(
        existing.versionCount,
        object.work_group?.version_count ?? existing.versions.length,
      );
      existing.auditEpisodeCount += object.audit_episode_count;
      existing.factCount += object.fact_count;

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
        hasPreprint: versionKindForObject(object) === "preprint",
      });
    }
  }

  return Array.from(groups.values()).map((group) => ({
    ...group,
    versionCount: Math.max(group.versionCount, group.versions.length),
    auditStatus: highestAuditStatus(group.versions),
    hasPreprint: group.versions.some(
      (version) => versionKindForObject(version) === "preprint",
    ),
    versions: group.versions.sort(
      (left, right) =>
        versionRank(versionKindForObject(left)) - versionRank(versionKindForObject(right)),
    ),
  }));
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

function isCheckedSearchParam(value: string | undefined) {
  return value === "true" || value === "on" || value === "1";
}
