import Link from "next/link";

import { AppSidebar } from "../components/app-sidebar";
import {
  formatLabel,
  getLibraryItems,
  type ScholarlyObjectSummary,
} from "../lib/csqd-api";

export default async function LibraryPage() {
  const libraryItems = await getLibraryItems();
  const workCount = new Set(
    libraryItems.map((item) => workIdentityForObject(item.scholarly_object)),
  ).size;
  const factCount = libraryItems.reduce(
    (sum, item) => sum + item.scholarly_object.fact_count,
    0,
  );

  return (
    <main className="app-shell">
      <AppSidebar activeItem="library" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Cross-domain workspace</p>
            <h1>Library</h1>
          </div>
          <div className="status-pill">Explicitly added</div>
        </header>

        <section className="metric-grid" aria-label="Library metrics">
          <div className="metric">
            <span>Library items</span>
            <strong>{libraryItems.length}</strong>
          </div>
          <div className="metric">
            <span>Works</span>
            <strong>{workCount}</strong>
          </div>
          <div className="metric">
            <span>Facts</span>
            <strong>{factCount}</strong>
          </div>
        </section>

        <section className="object-list" aria-label="Library items">
          {libraryItems.length === 0 ? (
            <div className="empty-state">
              <h2>No library items yet</h2>
              <p>Add works from Scholarly Search to build this view.</p>
            </div>
          ) : (
            libraryItems.map((item) => {
              const object = item.scholarly_object;

              return (
                <article className="object-card work-card" key={item.id}>
                  <div className="object-main">
                    <div className="object-kicker">
                      <span>{formatLabel(versionKindForObject(object))}</span>
                      <span>{formatLabel(item.added_reason)}</span>
                      <span>{formatAddedAt(item.added_at)}</span>
                    </div>
                    <h2>{object.title}</h2>
                    <p>{object.authors.join(", ")}</p>
                    <div className="object-actions">
                      <Link href={`/scholarly-objects/${object.id}`}>Open</Link>
                      {object.audit_subject_id ? (
                        <Link href={`/commission?subject_id=${object.audit_subject_id}`}>
                          Commission audit
                        </Link>
                      ) : null}
                      <a
                        href={object.canonical_url}
                        rel="noreferrer"
                        target="_blank"
                      >
                        Open source
                      </a>
                    </div>
                  </div>
                  <dl className="object-facts">
                    <div>
                      <dt>Status</dt>
                      <dd>{formatLabel(object.audit_status)}</dd>
                    </div>
                    <div>
                      <dt>Facts</dt>
                      <dd>{object.fact_count}</dd>
                    </div>
                    <div>
                      <dt>Source</dt>
                      <dd>{object.source_name}</dd>
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

function formatAddedAt(value: string) {
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
