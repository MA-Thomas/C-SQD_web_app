import Link from "next/link";
import { notFound } from "next/navigation";

import {
  formatLabel,
  getArticleAccess,
  getScholarlyObject,
  type ScholarlyObjectDetail,
} from "../../../lib/csqd-api";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

export default async function ScholarlyObjectReviewPage({ params }: PageProps) {
  const { id } = await params;
  const [object, articleAccess] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
  ]);

  if (!object || !articleAccess) {
    notFound();
  }

  const nativeLocation = articleAccess.external_locations.find(
    (location) => location.location_type === "pdf",
  ) ?? articleAccess.external_locations.find(
    (location) =>
      location.location_type === "full_text" || location.location_type === "repository",
  );
  const sourceUrl =
    nativeLocation?.url ??
    articleAccess.preferred_source?.url ??
    articleAccess.canonical_url;
  const canEmbed = articleAccess.native_display_permitted;

  return (
    <main className="review-shell">
      <section className="review-workspace" aria-label="Review workspace">
        <header className="review-workspace-header">
          <div>
            <p className="eyebrow">Review workspace</p>
            <h1>{object.work_group?.title ?? object.title}</h1>
          </div>
          <div className="review-workspace-actions">
            <Link className="secondary-action" href={`/scholarly-objects/${object.id}`}>
              Record
            </Link>
            <a
              className="secondary-action"
              href={articleAccess.canonical_url}
              rel="noreferrer"
              target="_blank"
            >
              Source
            </a>
          </div>
        </header>

        <section className="review-target-card">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Target</p>
              <h2>Review Scope</h2>
            </div>
            <span className="access-badge">{formatLabel(object.version_kind)}</span>
          </div>

          <div className="review-scope-options" role="group" aria-label="Review target scope">
            <label>
              <input name="review-target-scope" type="radio" value="work_group" />
              <span>
                <strong>Article work</strong>
                <small>{object.work_group?.title ?? object.title}</small>
              </span>
            </label>
            <label>
              <input name="review-target-scope" type="radio" value="specific_version" />
              <span>
                <strong>This version</strong>
                <small>{formatLabel(object.version_kind)}</small>
              </span>
            </label>
            <label>
              <input
                defaultChecked
                name="review-target-scope"
                type="radio"
                value="work_and_version"
              />
              <span>
                <strong>Work and version</strong>
                <small>Article work + selected version</small>
              </span>
            </label>
          </div>
        </section>

        <section className="review-target-card">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Versions</p>
              <h2>Article Version Set</h2>
            </div>
            <span className="access-badge">
              {object.versions.length || 1}{" "}
              {(object.versions.length || 1) === 1 ? "version" : "versions"}
            </span>
          </div>
          <VersionContextList object={object} />
        </section>

        <section className="review-placeholder-grid" aria-label="Review drafting areas">
          <article className="review-placeholder-panel">
            <h2>Claims</h2>
            <p>No claims recorded yet.</p>
          </article>
          <article className="review-placeholder-panel">
            <h2>Notes</h2>
            <p>No notes recorded yet.</p>
          </article>
          <article className="review-placeholder-panel">
            <h2>Evaluation Facts</h2>
            <p>No evaluation facts drafted yet.</p>
          </article>
          <article className="review-placeholder-panel">
            <h2>Citations</h2>
            <p>No cited evidence attached yet.</p>
          </article>
        </section>
      </section>

      <section className="review-reader" aria-label="Article reader">
        <header className="review-reader-header">
          <div>
            <p className="eyebrow">Reader</p>
            <h2>{object.title}</h2>
            <span>{formatLabel(articleAccess.display_strategy)}</span>
          </div>
          <Link className="secondary-action" href={`/scholarly-objects/${object.id}/view`}>
            Full viewer
          </Link>
        </header>

        <div className="review-reader-body">
          {canEmbed ? (
            <iframe className="article-frame" src={sourceUrl} title={object.title} />
          ) : (
            <div className="viewer-unavailable">
              <h2>Native display unavailable</h2>
              <p>
                C-SQD has source metadata for this article, but no permitted native
                display target yet.
              </p>
              <a
                className="primary-action"
                href={articleAccess.canonical_url}
                rel="noreferrer"
                target="_blank"
              >
                Open source
              </a>
            </div>
          )}
        </div>
      </section>
    </main>
  );
}

function VersionContextList({ object }: { object: ScholarlyObjectDetail }) {
  const versions =
    object.versions.length > 0
      ? object.versions
      : [
          {
            scholarly_object_id: object.id,
            title: object.title,
            version_kind: object.version_kind,
            doi: object.doi,
            source_name: object.source_name,
            canonical_url: object.canonical_url,
            native_display_permitted: object.native_display_permitted,
            is_current: true,
            is_primary: true,
          },
        ];

  return (
    <div className="version-context-list">
      {versions.map((version) => (
        <Link
          className={`version-context-row${version.is_current ? " current" : ""}`}
          href={`/scholarly-objects/${version.scholarly_object_id}/review`}
          key={version.scholarly_object_id}
        >
          <div>
            <strong>{formatLabel(version.version_kind)}</strong>
            <span>{version.source_name}</span>
          </div>
          {version.is_primary ? <span>Primary</span> : null}
        </Link>
      ))}
    </div>
  );
}
