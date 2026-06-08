import Link from "next/link";
import { notFound } from "next/navigation";

import { AppSidebar } from "../../components/app-sidebar";
import {
  formatLabel,
  getArticleAccess,
  getScholarlyObject,
} from "../../lib/csqd-api";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

function sourceActionLabel(displayStrategy: string) {
  switch (displayStrategy) {
    case "permitted_native_display":
      return "Open source";
    case "external_publisher_page":
      return "Open publisher page";
    case "external_repository_page":
      return "Open repository page";
    case "external_landing_page":
      return "Open landing page";
    case "external_full_text":
      return "Open full text";
    case "external_pdf":
      return "Open PDF";
    default:
      return "Open canonical source";
  }
}

export default async function ScholarlyObjectPage({ params }: PageProps) {
  const { id } = await params;
  const [object, articleAccess] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
  ]);

  if (!object || !articleAccess) {
    notFound();
  }

  const primarySourceUrl =
    articleAccess.preferred_source?.url ?? articleAccess.canonical_url;
  const canonicalLocation = articleAccess.canonical_location;
  const canViewNatively = articleAccess.native_display_permitted;

  return (
    <main className="app-shell">
      <AppSidebar activeItem="search" />

      <section className="workspace">
        <header className="topbar detail-topbar">
          <div>
            <p className="eyebrow">Academic Peer Review audit object</p>
            <h1>{object.title}</h1>
          </div>
          <Link className="status-pill" href="/">
            Back to objects
          </Link>
        </header>

        <section className="detail-grid">
          <article className="detail-primary">
            <div className="object-kicker">
              <span>{formatLabel(object.object_type)}</span>
              <span>{object.source_name}</span>
              {object.publication_year ? (
                <span>{object.publication_year}</span>
              ) : null}
            </div>
            <p className="author-line">{object.authors.join(", ")}</p>
            {object.abstract_text ? (
              <p className="abstract-text">{object.abstract_text}</p>
            ) : null}

            <div className="source-actions">
              <Link
                className="primary-action"
                href={`/scholarly-objects/${object.id}/review`}
              >
                Start review
              </Link>
              {canViewNatively ? (
                <Link
                  className="secondary-action"
                  href={`/scholarly-objects/${object.id}/view`}
                >
                  View in C-SQD
                </Link>
              ) : null}
              <a
                className="secondary-action"
                href={primarySourceUrl}
                rel="noreferrer"
                target="_blank"
              >
                {sourceActionLabel(articleAccess.display_strategy)}
              </a>
              {canonicalLocation && canonicalLocation.url !== primarySourceUrl ? (
                <a
                  className="secondary-action"
                  href={canonicalLocation.url}
                  rel="noreferrer"
                  target="_blank"
                >
                  Open {formatLabel(canonicalLocation.location_type)}
                </a>
              ) : null}
            </div>
          </article>

          <aside className="detail-side" aria-label="Object facts">
            <dl className="detail-facts">
              <div>
                <dt>Status</dt>
                <dd>{formatLabel(object.review_status)}</dd>
              </div>
              <div>
                <dt>Evaluation facts</dt>
                <dd>{object.evaluation_fact_count}</dd>
              </div>
              <div>
                <dt>Display</dt>
                <dd>{formatLabel(articleAccess.display_strategy)}</dd>
              </div>
              <div>
                <dt>Version</dt>
                <dd>{formatLabel(object.version_kind)}</dd>
              </div>
              <div>
                <dt>Article work</dt>
                <dd>{object.work_group?.title ?? object.title}</dd>
              </div>
              <div>
                <dt>Rights</dt>
                <dd>{formatLabel(articleAccess.rights_status)}</dd>
              </div>
              <div>
                <dt>License</dt>
                <dd>{articleAccess.license ?? "Unspecified"}</dd>
              </div>
              <div>
                <dt>DOI</dt>
                <dd>{object.doi ?? "Unspecified"}</dd>
              </div>
              <div>
                <dt>Published</dt>
                <dd>{object.publication_date ?? "Unspecified"}</dd>
              </div>
            </dl>
          </aside>
        </section>

        <section className="detail-panels">
          <article className="panel article-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Article</p>
                <h2>Published Source</h2>
              </div>
              <span className="access-badge">
                {formatLabel(articleAccess.display_strategy)}
              </span>
            </div>

            <dl className="article-access-grid">
              <div>
                <dt>Canonical source</dt>
                <dd>
                  <a href={articleAccess.canonical_url} rel="noreferrer" target="_blank">
                    {articleAccess.canonical_url}
                  </a>
                </dd>
              </div>
              <div>
                <dt>Publication venue</dt>
                <dd>{articleAccess.source_name}</dd>
              </div>
              <div>
                <dt>DOI</dt>
                <dd>{articleAccess.doi ?? "Unspecified"}</dd>
              </div>
              <div>
                <dt>License</dt>
                <dd>{articleAccess.license ?? "Unspecified"}</dd>
              </div>
              <div>
                <dt>Rights status</dt>
                <dd>{formatLabel(articleAccess.rights_status)}</dd>
              </div>
              <div>
                <dt>Native display</dt>
                <dd>
                  {articleAccess.native_display_permitted
                    ? "Permitted"
                    : "Not permitted"}
                </dd>
              </div>
            </dl>

            <div className="source-actions">
              <Link
                className="primary-action"
                href={`/scholarly-objects/${object.id}/review`}
              >
                Start review
              </Link>
              {canViewNatively ? (
                <Link
                  className="secondary-action"
                  href={`/scholarly-objects/${object.id}/view`}
                >
                  View in C-SQD
                </Link>
              ) : null}
              <a
                className="secondary-action"
                href={primarySourceUrl}
                rel="noreferrer"
                target="_blank"
              >
                {sourceActionLabel(articleAccess.display_strategy)}
              </a>
            </div>

            <h3 className="panel-subhead">External Locations</h3>
            <div className="location-list">
              {articleAccess.external_locations.length === 0 ? (
                <p className="muted-copy">No external locations recorded.</p>
              ) : (
                articleAccess.external_locations.map((location) => (
                  <div className="location-row" key={location.id}>
                    <div>
                      <strong>{formatLabel(location.location_type)}</strong>
                      <span>
                        {location.is_canonical ? "Canonical" : "Alternate"} -{" "}
                        {location.license ?? "Unspecified license"}
                      </span>
                    </div>
                    <a href={location.url} rel="noreferrer" target="_blank">
                      Open
                    </a>
                  </div>
                ))
              )}
            </div>
          </article>

          <article className="panel">
            <h2>Review Graph</h2>
            <dl className="review-graph">
              <div>
                <dt>Current state</dt>
                <dd>{formatLabel(object.review_status)}</dd>
              </div>
              <div>
                <dt>Recorded facts</dt>
                <dd>{object.evaluation_fact_count}</dd>
              </div>
              <div>
                <dt>Review surface</dt>
                <dd>{formatLabel(articleAccess.display_strategy)}</dd>
              </div>
              <div>
                <dt>Target version</dt>
                <dd>{formatLabel(object.version_kind)}</dd>
              </div>
              <div>
                <dt>Article work</dt>
                <dd>{object.work_group?.title ?? object.title}</dd>
              </div>
            </dl>

            <h3 className="panel-subhead">Versions</h3>
            <div className="version-context-list">
              {object.versions.length === 0 ? (
                <p className="muted-copy">No sibling versions recorded yet.</p>
              ) : (
                object.versions.map((version) => (
                  <Link
                    className={`version-context-row${
                      version.is_current ? " current" : ""
                    }`}
                    href={`/scholarly-objects/${version.scholarly_object_id}`}
                    key={version.scholarly_object_id}
                  >
                    <div>
                      <strong>{formatLabel(version.version_kind)}</strong>
                      <span>{version.source_name}</span>
                    </div>
                    {version.is_primary ? <span>Primary</span> : null}
                  </Link>
                ))
              )}
            </div>
          </article>
        </section>
      </section>
    </main>
  );
}
