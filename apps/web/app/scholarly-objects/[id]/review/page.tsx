import Link from "next/link";
import { revalidatePath } from "next/cache";
import { notFound, redirect } from "next/navigation";

import {
  createElementReview,
  formatLabel,
  getArticleAccess,
  getDomainInstantiation,
  getDomainInstantiations,
  getScholarlyObject,
  type CWENode,
  type ScholarlyObjectDetail,
} from "../../../lib/csqd-api";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
  searchParams: Promise<{
    review_error?: string;
  }>;
};

export default async function ScholarlyObjectReviewPage({
  params,
  searchParams,
}: PageProps) {
  const { id } = await params;
  const { review_error } = await searchParams;
  const [object, articleAccess, cweNodes] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
    getAcademicCweNodes(),
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
          <article className="review-event-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Element review</p>
                <h2>Record Criterion Review</h2>
              </div>
              <span className="access-badge">ReviewEvent</span>
            </div>
            {review_error ? (
              <p className="form-error">
                The review could not be saved. Check the criterion and review text.
              </p>
            ) : null}
            <form className="element-review-form" action={createElementReviewAction}>
              <input name="scholarly_object_id" type="hidden" value={object.id} />
              <label>
                <span>CWE criterion</span>
                <select disabled={cweNodes.length === 0} name="cwe_node_id" required>
                  {cweNodes.length === 0 ? (
                    <option value="">No criteria available</option>
                  ) : (
                    cweNodes.map((node) => (
                      <option key={node.id} value={node.id}>
                        {node.label}
                      </option>
                    ))
                  )}
                </select>
              </label>
              <div className="element-review-form-row">
                <label>
                  <span>Finding</span>
                  <select name="finding" required>
                    <option value="non_ethical_problem">Non-ethical problem</option>
                    <option value="ethical_problem">Ethical problem</option>
                    <option value="no_problems">No problems</option>
                    <option value="inconclusive">Inconclusive</option>
                  </select>
                </label>
                <label>
                  <span>Severity</span>
                  <select name="severity">
                    <option value="">Unspecified</option>
                    <option value="minor">Minor</option>
                    <option value="moderate">Moderate</option>
                    <option value="major">Major</option>
                    <option value="critical">Critical</option>
                  </select>
                </label>
              </div>
              <label>
                <span>Confidence</span>
                <select name="confidence">
                  <option value="">Unspecified</option>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </select>
              </label>
              <label>
                <span>Finding text</span>
                <textarea
                  name="content"
                  placeholder="Record the focused evaluation for this criterion."
                  required
                  rows={7}
                />
              </label>
              <button
                className="primary-action action-button"
                disabled={cweNodes.length === 0}
                type="submit"
              >
                Save review event
              </button>
            </form>
          </article>
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

async function getAcademicCweNodes(): Promise<CWENode[]> {
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

async function createElementReviewAction(formData: FormData) {
  "use server";

  const scholarlyObjectId = String(formData.get("scholarly_object_id") ?? "");
  const cweNodeId = String(formData.get("cwe_node_id") ?? "");
  const finding = String(formData.get("finding") ?? "inconclusive");
  const severity = optionalFormValue(formData.get("severity"));
  const confidence = optionalFormValue(formData.get("confidence"));
  const content = String(formData.get("content") ?? "");

  if (!scholarlyObjectId) {
    return;
  }

  const reviewEvent = await createElementReview(scholarlyObjectId, {
    content,
    confidence,
    cwe_node_id: cweNodeId,
    finding,
    severity,
    solicitation: null,
  });

  if (!reviewEvent) {
    redirect(`/scholarly-objects/${scholarlyObjectId}/review?review_error=1`);
  }

  revalidatePath("/");
  revalidatePath("/library");
  revalidatePath(`/scholarly-objects/${scholarlyObjectId}`);
  revalidatePath(`/scholarly-objects/${scholarlyObjectId}/review`);
  redirect(`/scholarly-objects/${scholarlyObjectId}`);
}

function optionalFormValue(value: FormDataEntryValue | null) {
  const text = String(value ?? "").trim();

  return text === "" ? null : text;
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
