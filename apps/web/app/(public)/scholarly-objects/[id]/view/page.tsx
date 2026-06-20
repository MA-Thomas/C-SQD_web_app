import Link from "next/link";
import { notFound } from "next/navigation";

import {
  formatLabel,
  getArticleAccess,
  getScholarlyObject,
} from "../../../../lib/csqd-api";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

export default async function ScholarlyObjectViewerPage({ params }: PageProps) {
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
    <main className="viewer-shell">
      <header className="viewer-topbar">
        <div>
          <p className="eyebrow">Native viewer</p>
          <h1>{object.title}</h1>
          <span>{formatLabel(articleAccess.display_strategy)}</span>
        </div>
        <nav className="viewer-actions" aria-label="Viewer actions">
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
        </nav>
      </header>

      <section className="viewer-body">
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
      </section>
    </main>
  );
}
