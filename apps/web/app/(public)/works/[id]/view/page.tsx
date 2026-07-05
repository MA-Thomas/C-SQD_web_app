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

/// Rights-aware reading view. Embeds the article when native display is
/// permitted; otherwise routes to the canonical source.
export default async function WorkViewerPage({ params }: PageProps) {
  const { id } = await params;
  const [object, articleAccess] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
  ]);

  if (!object || !articleAccess) {
    notFound();
  }

  const nativeLocation =
    articleAccess.external_locations.find(
      (location) => location.location_type === "pdf",
    ) ??
    articleAccess.external_locations.find(
      (location) =>
        location.location_type === "full_text" ||
        location.location_type === "repository",
    );
  const sourceUrl =
    nativeLocation?.url ??
    articleAccess.preferred_source?.url ??
    articleAccess.canonical_url;
  const canEmbed = articleAccess.native_display_permitted;

  return (
    <div className="pub-viewer">
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Reading view</p>
          <h1>{object.title}</h1>
          <p>{formatLabel(articleAccess.display_strategy)}</p>
        </div>
        <nav className="pub-card-actions" aria-label="Viewer actions">
          <Link href={`/works/${object.id}`}>Audit record</Link>
          <a href={articleAccess.canonical_url} rel="noreferrer" target="_blank">
            Canonical source
          </a>
        </nav>
      </header>

      {canEmbed ? (
        <iframe className="pub-viewer-frame" src={sourceUrl} title={object.title} />
      ) : (
        <div className="pub-empty">
          <h3>Native display unavailable</h3>
          <p>
            C-SQD has source metadata for this article, but no permitted native
            display target yet.{" "}
            <a href={articleAccess.canonical_url} rel="noreferrer" target="_blank">
              Open the canonical source
            </a>
            .
          </p>
        </div>
      )}
    </div>
  );
}
