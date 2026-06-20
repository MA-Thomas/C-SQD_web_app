import Link from "next/link";
import { notFound } from "next/navigation";

import { CrweCoverageMatrix } from "../../../components/crwe-coverage-matrix";
import { FactTimeline } from "../../../components/fact-timeline";
import { GatedAction } from "../../../components/gated-action";
import { ReportReader } from "../../../components/report-reader";
import { StatusPill } from "../../../components/status-pill";
import {
  ChallengeElementReviewForm,
  ContestReportForm,
  CwePetitionForm,
  EpisodeParticipationActions,
  FeaturePetitionForm,
} from "../../../components/subject-actions";
import { TupleBadge } from "../../../components/tuple-badge";
import { TupleRecomputePanel } from "../../../components/tuple-recompute";
import {
  formatLabel,
  getArticleAccess,
  getDomainInstantiation,
  getScholarlyObject,
  getSynthesisReviewRelations,
  type CWENode,
  type Fact,
  type SynthesisReviewRelation,
} from "../../../lib/csqd-api";
import {
  factKind,
  formatDate,
  getAcademicCweNodes,
  getPublicAuditSummaryForObject,
  groupedElementReviewsByCriterion,
  payloadRecord,
  publicAuditStatusLabel,
  stringValue,
} from "../../../lib/public-audit";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

export default async function PublicAuditSubjectPage({ params }: PageProps) {
  const { id } = await params;
  const [object, articleAccess] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
  ]);

  if (!object || !articleAccess) {
    notFound();
  }

  const [summary, cweNodes] = await Promise.all([
    getPublicAuditSummaryForObject(object),
    getAcademicCweNodes(),
  ]);
  const domain = object.audit_subject_id
    ? await getDomainInstantiation(
        summary.episodes[0]?.domain_instantiation_id ?? "",
      ).catch(() => null)
    : null;
  const nodes = domain?.cwe_nodes?.length ? domain.cwe_nodes : cweNodes;
  const reviewGroups = groupedElementReviewsByCriterion(summary.facts, nodes);
  const primarySourceUrl =
    articleAccess.preferred_source?.url ?? articleAccess.canonical_url;
  const status = publicAuditStatusLabel(object, summary);
  const subjectPath = `/scholarly-objects/${object.id}`;
  const commissionHref = object.audit_subject_id
    ? `/commission?subject_id=${object.audit_subject_id}`
    : "/commission";
  const openEpisode =
    summary.episodes.find((episode) => episode.status === "active") ?? null;
  const reportRelations: SynthesisReviewRelation[] = summary.latestReport
    ? await getSynthesisReviewRelations(summary.latestReport.id)
    : [];
  const contestations = reportRelations.filter(
    (relation) => typeof relation.relation_type === "object",
  );
  const challengeResponses = summary.facts.filter(
    (fact) =>
      factKind(fact) === "submitter_response" &&
      stringValue(payloadRecord(fact, "submitter_response")?.response_type) ===
        "contests",
  );

  return (
    <section className="workspace public-subject">
      <header className="subject-sticky-header">
        <div className="subject-heading">
          <p className="eyebrow">Public audit subject</p>
          <h1>{object.title}</h1>
          <div className="subject-heading-meta">
            <StatusPill status={status} />
            <span>{formatLabel(object.object_type)}</span>
            <span>{object.source_name}</span>
            {object.publication_year ? <span>{object.publication_year}</span> : null}
          </div>
        </div>
        <div className="subject-heading-tuple">
          <TupleBadge tuple={summary.tuple} size="compact" />
          <Link className="status-pill" href="/discover">
            Back to Discover
          </Link>
        </div>
      </header>

      <section className="detail-grid">
        <article className="detail-primary subject-summary">
          <p className="author-line">{object.authors.join(", ")}</p>
          {object.abstract_text ? (
            <p className="abstract-text">{object.abstract_text}</p>
          ) : null}
          <div className="source-actions">
            <GatedAction
              className="primary-action"
              href={`${subjectPath}/review`}
              explain="ElementReviews are focused reviews of one CRWE criterion and carry provenance."
            >
              Submit ElementReview
            </GatedAction>
            <Link className="secondary-action" href={commissionHref}>
              Commission deeper audit
            </Link>
            <a
              className="secondary-action"
              href={primarySourceUrl}
              rel="noreferrer"
              target="_blank"
            >
              Open source
            </a>
            <GatedAction
              className="secondary-action"
              href="/library"
              explain="Saving to your library requires an account."
            >
              Save to library
            </GatedAction>
          </div>
          <div className="source-actions participation-actions">
            <EpisodeParticipationActions
              auditSubjectId={object.audit_subject_id}
              openEpisodeId={openEpisode?.id ?? null}
              subjectPath={subjectPath}
              subjectTitle={object.title}
            />
          </div>
        </article>

        <aside className="detail-side" aria-label="Public audit summary">
          <dl className="detail-facts">
            <div>
              <dt>Audit status</dt>
              <dd>
                <StatusPill status={status} />
              </dd>
            </div>
            <div>
              <dt>CRWE coverage</dt>
              <dd>
                {summary.crweCoverageCount}/{nodes.length}
              </dd>
            </div>
            <div>
              <dt>ElementReviews</dt>
              <dd>{summary.elementReviewCount}</dd>
            </div>
            <div>
              <dt>SynthesisReviews</dt>
              <dd>{summary.synthesisReviewCount}</dd>
            </div>
            <div>
              <dt>Challenges</dt>
              <dd>{summary.challengeCount}</dd>
            </div>
            <div>
              <dt>Public episodes</dt>
              <dd>{summary.episodes.length}</dd>
            </div>
          </dl>
          <TupleRecomputePanel episodes={summary.episodes} />
        </aside>
      </section>

      <section className="tuple-band" aria-label="Evaluation tuple">
        <TupleBadge tuple={summary.tuple} />
      </section>

      <nav className="workspace-tabs" aria-label="Public audit subject sections">
        <a href="#latest-report">Latest Report</a>
        <a href="#crwe-coverage">CRWE Coverage</a>
        <a href="#element-reviews">ElementReviews</a>
        <a href="#challenges">Challenges</a>
        <a href="#audit-trail">Audit Trail</a>
      </nav>

      <section
        className="workspace-section first-workspace-section"
        id="latest-report"
      >
        <div className="section-heading">
          <div>
            <p className="eyebrow">SynthesisReview</p>
            <h2>Latest Public Audit Report</h2>
          </div>
          <GatedAction
            className="secondary-action"
            href={openEpisode ? `${subjectPath}/review?synthesis=1` : `${subjectPath}/review`}
            explain="Unsolicited SynthesisReviews require starting or joining the public episode first."
          >
            Submit SynthesisReview
          </GatedAction>
        </div>
        {summary.latestReport ? (
          <>
            <ReportReader review={summary.latestReport} />
            <div className="report-actions">
              <ContestReportForm
                episodeId={summary.latestReport.episode_id}
                reviewId={summary.latestReport.id}
                subjectPath={subjectPath}
              />
            </div>
          </>
        ) : (
          <div className="empty-state">
            <h2>No public SynthesisReview yet</h2>
            <p>
              ElementReviews and audit facts can be synthesized into a public
              report once a public episode has enough review depth.
            </p>
          </div>
        )}
      </section>

      <section className="workspace-section" id="crwe-coverage">
        <div className="section-heading">
          <div>
            <p className="eyebrow">CRWE coverage</p>
            <h2>Criteria Reviewed And Open</h2>
          </div>
          <CwePetitionForm
            episodeId={openEpisode?.id ?? null}
            nodes={nodes.map((node) => ({ id: node.id, label: node.label }))}
            subjectPath={subjectPath}
          />
        </div>
        <CrweCoverageMatrix
          anchorPrefix="criterion"
          facts={summary.facts}
          nodes={nodes}
          reviewHrefBase={`${subjectPath}/review`}
        />
      </section>

      <section className="workspace-section" id="element-reviews">
        <div className="section-heading">
          <div>
            <p className="eyebrow">ElementReviews</p>
            <h2>Focused Reviews By Criterion</h2>
          </div>
          <GatedAction className="secondary-action" href={`${subjectPath}/review`}>
            Review one criterion
          </GatedAction>
        </div>
        <div className="element-review-group-list">
          {reviewGroups.every((group) => group.reviews.length === 0) ? (
            <div className="empty-state">
              <h2>No public ElementReviews yet</h2>
              <p>
                Be the first to submit a focused criterion-level review after
                signing in.
              </p>
            </div>
          ) : (
            reviewGroups
              .filter((group) => group.reviews.length > 0)
              .map((group) => (
                <details
                  className="review-details"
                  id={`criterion-${group.node.id}`}
                  key={group.node.id}
                >
                  <summary>
                    <span>{group.node.label}</span>
                    <strong>{group.reviews.length}</strong>
                  </summary>
                  <div className="review-card-list">
                    {group.reviews.map((fact) => (
                      <ElementReviewCard
                        episodeId={openEpisode?.id ?? null}
                        fact={fact}
                        key={fact.id}
                        subjectPath={subjectPath}
                      />
                    ))}
                  </div>
                </details>
              ))
          )}
        </div>
      </section>

      <section className="workspace-section" id="challenges">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Challenges</p>
            <h2>Contestations And Responses</h2>
          </div>
          <span className="access-badge">{summary.challengeCount} recorded</span>
        </div>
        {contestations.length === 0 && challengeResponses.length === 0 ? (
          <div className="empty-state">
            <h2>No public challenges recorded</h2>
            <p>
              Challenges contest specific ElementReviews or SynthesisReviews
              while preserving the historical audit trail.
            </p>
          </div>
        ) : (
          <div className="challenge-thread">
            {contestations.map((relation) => (
              <article className="challenge-entry" key={relation.id}>
                <p className="challenge-kind">
                  Report contested
                  {typeof relation.relation_type === "object"
                    ? ` (${relation.relation_type.contests.scope})`
                    : null}
                </p>
                {typeof relation.relation_type === "object" &&
                relation.relation_type.contests.rationale ? (
                  <p>{relation.relation_type.contests.rationale}</p>
                ) : null}
                <p className="challenge-meta">{formatDate(relation.asserted_at)}</p>
              </article>
            ))}
            {challengeResponses.map((fact) => {
              const payload = payloadRecord(fact, "submitter_response");

              return (
                <article className="challenge-entry" key={fact.id}>
                  <p className="challenge-kind">ElementReview challenged</p>
                  <p>{stringValue(payload?.content)}</p>
                  <p className="challenge-meta">{formatDate(fact.occurred_at)}</p>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-section" id="audit-trail">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Audit trail</p>
            <h2>Provenance And Source Context</h2>
          </div>
          <span className="access-badge">{summary.facts.length} facts</span>
        </div>
        <details className="advanced-details">
          <summary>Show full audit trail</summary>
          <section className="detail-panels">
            <article className="panel">
              <h2>Subject Metadata</h2>
              <dl className="article-access-grid">
                <div>
                  <dt>DOI</dt>
                  <dd>{object.doi ?? "Unspecified"}</dd>
                </div>
                <div>
                  <dt>Publication date</dt>
                  <dd>{object.publication_date ?? "Unspecified"}</dd>
                </div>
                <div>
                  <dt>License</dt>
                  <dd>{object.license ?? "Unspecified"}</dd>
                </div>
                <div>
                  <dt>Rights</dt>
                  <dd>{formatLabel(articleAccess.rights_status)}</dd>
                </div>
                <div>
                  <dt>Versions</dt>
                  <dd>{object.versions.length || 1}</dd>
                </div>
                <div>
                  <dt>AuditSubject</dt>
                  <dd>{object.audit_subject_id ?? "Not registered"}</dd>
                </div>
              </dl>
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
              <h2>Public Facts</h2>
              <FactTimeline facts={summary.facts} />
            </article>
          </section>
        </details>
      </section>
    </section>
  );
}

function ElementReviewCard({
  episodeId,
  fact,
  subjectPath,
}: {
  episodeId: string | null;
  fact: Fact;
  subjectPath: string;
}) {
  const payload = payloadRecord(fact, "element_review");
  const finding = stringValue(payload?.finding) || "inconclusive";

  return (
    <article className="review-card" id={`fact-${fact.id}`}>
      <div className="object-kicker">
        <span>{formatLabel(finding)}</span>
        {stringValue(payload?.severity) ? (
          <span>{formatLabel(stringValue(payload?.severity))}</span>
        ) : null}
        {stringValue(payload?.confidence) ? (
          <span>{formatLabel(stringValue(payload?.confidence))} confidence</span>
        ) : null}
        {payload?.solicitation ? <span>Commissioned</span> : <span>Unsolicited</span>}
      </div>
      <p>{stringValue(payload?.content) || "ElementReview fact"}</p>
      {stringValue(payload?.limitations) ? (
        <small>Limitations: {stringValue(payload?.limitations)}</small>
      ) : null}
      {stringValue(payload?.recommendations) ? (
        <small>Recommendations: {stringValue(payload?.recommendations)}</small>
      ) : null}
      <div className="object-actions review-card-actions">
        <ChallengeElementReviewForm
          episodeId={episodeId}
          factId={fact.id}
          subjectPath={subjectPath}
        />
        <FeaturePetitionForm
          episodeId={episodeId}
          factId={fact.id}
          subjectPath={subjectPath}
        />
      </div>
    </article>
  );
}
