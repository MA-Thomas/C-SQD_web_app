import Link from "next/link";
import { notFound } from "next/navigation";

import { ActionRail } from "../../../components/action-rail";
import {
  AuditObjectDisclosure,
  WorkRecordDisclosure,
} from "../../../components/audit-object-disclosure";
import { BearingPill } from "../../../components/bearing-pill";
import { CriterionCluster } from "../../../components/criterion-cluster";
import { CrweCoverageMatrix } from "../../../components/crwe-coverage-matrix";
import { DissentBlock } from "../../../components/dissent-block";
import { FactTimeline } from "../../../components/fact-timeline";
import { GatedAction } from "../../../components/gated-action";
import { ReportReader } from "../../../components/report-reader";
import { StatusPill } from "../../../components/status-pill";
import { ContestReportForm, CwePetitionForm } from "../../../components/subject-actions";
import { TupleBadge } from "../../../components/tuple-badge";
import {
  formatLabel,
  getArticleAccess,
  getDomainInstantiation,
  getScholarlyObject,
  getSynthesisReviewRelations,
  getWorkAuditInvolvements,
  type SynthesisReviewRelation,
  type WorkAuditInvolvement,
} from "../../../lib/csqd-api";
import {
  factKind,
  getAcademicCweNodes,
  getPublicAuditSummaryForObject,
  groupedElementReviewsByCriterion,
  payloadRecord,
  publicAuditStatusLabel,
  stringValue,
} from "../../../lib/public-audit";
import { evalTupleValues } from "../../../lib/tuple-items";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

/// Full-coverage page for one audit subject: report as the lead story,
/// reviews clustered by criterion, dissent kept distinct, provenance in an
/// advanced-gated trail, and every action on a sticky rail.
export default async function WorkPage({ params }: PageProps) {
  const { id } = await params;
  const [object, articleAccess] = await Promise.all([
    getScholarlyObject(id),
    getArticleAccess(id),
  ]);

  if (!object || !articleAccess) {
    notFound();
  }

  const [summary, fallbackNodes, involvements] = await Promise.all([
    getPublicAuditSummaryForObject(object),
    getAcademicCweNodes(),
    getWorkAuditInvolvements(object.id),
  ]);
  const domain = object.audit_subject_id
    ? await getDomainInstantiation(
        summary.episodes[0]?.domain_instantiation_id ?? "",
      ).catch(() => null)
    : null;
  const nodes = domain?.cwe_nodes?.length ? domain.cwe_nodes : fallbackNodes;
  const reviewGroups = groupedElementReviewsByCriterion(summary.facts, nodes);
  const populatedGroups = reviewGroups.filter((group) => group.reviews.length > 0);
  const status = publicAuditStatusLabel(object, summary);
  const subjectPath = `/works/${object.id}`;
  const commissionHref = object.audit_subject_id
    ? `/commission?subject_id=${object.audit_subject_id}`
    : "/commission";
  const sourceUrl =
    articleAccess.preferred_source?.url ?? articleAccess.canonical_url;
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
    <div className="pub-work-layout">
      <div className="pub-work-main">
        <header className="pub-work-header">
          <div className="pub-card-kicker">
            <StatusPill status={status} />
            <span>{formatLabel(object.object_type)}</span>
            <span>Scholarly work record</span>
          </div>
          <h1>{object.title}</h1>
          <p className="pub-work-authors">{object.authors.join(", ")}</p>
          <div className="pub-work-idline">
            {object.source_name ? <span>{object.source_name}</span> : null}
            {object.publication_year ? <span>{object.publication_year}</span> : null}
            {object.doi ? <span>DOI {object.doi}</span> : null}
            {object.versions.length > 1 ? (
              <span>{object.versions.length} versions</span>
            ) : null}
            <a href={sourceUrl} rel="noreferrer" target="_blank">
              Source
            </a>
            <Link href={`${subjectPath}/view`}>Read here</Link>
          </div>
          {object.abstract_text ? (
            <p className="pub-work-abstract">{object.abstract_text}</p>
          ) : null}
          <WorkRecordDisclosure />
          <div className="pub-work-tuple">
            <TupleBadge showVerdict tuple={summary.tuple} />
          </div>
        </header>

        <section className="pub-panel" id="latest-report">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">SynthesisReview</p>
              <h2>Latest Public Audit Report</h2>
            </div>
            {summary.latestReport ? (
              <span className="pub-panel-count">
                {summary.synthesisReviewCount} total
              </span>
            ) : null}
          </div>
          {summary.latestReport ? (
            <>
              <ReportReader review={summary.latestReport} />
              <ContestReportForm
                episodeId={summary.latestReport.episode_id}
                reviewId={summary.latestReport.id}
                subjectPath={subjectPath}
              />
            </>
          ) : (
            <div className="pub-empty">
              <h3>No public SynthesisReview yet</h3>
              <p>
                ElementReviews and audit facts can be synthesized into a public
                report once a public episode has enough review depth.
              </p>
            </div>
          )}
        </section>

        <section className="pub-panel" id="crwe-coverage">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Coverage</p>
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

        <section className="pub-panel" id="element-reviews">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">ElementReviews</p>
              <h2>Focused Reviews By Criterion</h2>
            </div>
            <GatedAction className="secondary-action" href={`${subjectPath}/review`}>
              Review one criterion
            </GatedAction>
          </div>
          {populatedGroups.length === 0 ? (
            <div className="pub-empty">
              <h3>No public ElementReviews yet</h3>
              <p>
                Be the first to submit a focused criterion-level review after
                signing in.
              </p>
            </div>
          ) : (
            populatedGroups.map((group) => (
              <CriterionCluster
                episodeId={openEpisode?.id ?? null}
                group={group}
                key={group.node.id}
                subjectPath={subjectPath}
              />
            ))
          )}
        </section>

        <section className="pub-panel pub-dissent" id="challenges">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Dissent</p>
              <h2>Challenges And Contestations</h2>
            </div>
            <span className="pub-panel-count">
              {summary.challengeCount} recorded
            </span>
          </div>
          <DissentBlock
            challengeResponses={challengeResponses}
            contestations={contestations}
          />
        </section>

        <section className="pub-panel" id="audit-involvements">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Registry</p>
              <h2>Audits Involving This Work</h2>
            </div>
            <span className="pub-panel-count">{involvements.length} audits</span>
          </div>
          <p className="muted-copy">
            Every audit this work participates in — as the subject of a
            single-work audit, or attached as an evidence artifact to a
            claim-scoped audit. Attachment is not endorsement: an artifact
            earns its bearing through audited warrants.
          </p>
          {involvements.length === 0 ? (
            <div className="pub-empty">
              <h3>No audit involvements yet</h3>
              <p>
                This work has not been attached to any claim-scoped audit or
                audited directly.
              </p>
            </div>
          ) : (
            <div className="pub-facts">
              {involvements.map((involvement) => (
                <div
                  className="pub-location-row"
                  key={`${involvement.episode.id}-${involvementLabel(involvement)}`}
                >
                  <div>
                    <strong>
                      {involvement.audit_target.claim_statement ??
                        involvement.audit_target.title ??
                        involvement.episode.label}
                    </strong>
                    <span>
                      {involvementMeta(involvement)}
                      {roleBearing(involvement) ? (
                        <>
                          {" "}
                          <BearingPill bearing={roleBearing(involvement)!} />
                        </>
                      ) : null}
                    </span>
                    <span>{involvementCounts(involvement)}</span>
                    <AuditObjectDisclosure
                      subjectType={involvement.audit_target.subject_type}
                      workRole={involvement.work_role}
                    />
                    {involvement.audit_state.tuple ? (
                      <TupleBadge
                        size="compact"
                        tuple={evalTupleValues(involvement.audit_state.tuple)}
                      />
                    ) : null}
                  </div>
                  {involvement.audit_target.subject_type === "scoped_claim" ? (
                    <Link href={`/claims/${involvement.audit_target.subject_id}`}>
                      Open claim audit
                    </Link>
                  ) : null}
                </div>
              ))}
            </div>
          )}
        </section>

        <section className="pub-panel" id="audit-trail">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Provenance</p>
              <h2>Audit Trail</h2>
            </div>
            <span className="pub-panel-count">{summary.facts.length} facts</span>
          </div>
          <details className="advanced-details">
            <summary>Show full audit trail and source context</summary>
            <dl className="pub-metadata-grid">
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
                <dt>AuditSubject</dt>
                <dd>{object.audit_subject_id ?? "Not registered"}</dd>
              </div>
              <div>
                <dt>Public episodes</dt>
                <dd>{summary.episodes.length}</dd>
              </div>
            </dl>
            <div>
              {articleAccess.external_locations.map((location) => (
                <div className="pub-location-row" key={location.id}>
                  <div>
                    <strong>{formatLabel(location.location_type)}</strong>
                    <span>
                      {location.is_canonical ? "Canonical" : "Alternate"} ·{" "}
                      {location.license ?? "Unspecified license"}
                    </span>
                  </div>
                  <a href={location.url} rel="noreferrer" target="_blank">
                    Open
                  </a>
                </div>
              ))}
            </div>
            <FactTimeline facts={summary.facts} />
          </details>
        </section>
      </div>

      <ActionRail
        auditSubjectId={object.audit_subject_id}
        commissionHref={commissionHref}
        episodes={summary.episodes}
        facts={[
          { label: "Status", value: status },
          {
            label: "Coverage",
            value: `${summary.crweCoverageCount}/${nodes.length}`,
          },
          { label: "ElementReviews", value: summary.elementReviewCount },
          { label: "Reports", value: summary.synthesisReviewCount },
          { label: "Challenges", value: summary.challengeCount },
          { label: "Episodes", value: summary.episodes.length },
        ]}
        openEpisodeId={openEpisode?.id ?? null}
        sourceUrl={sourceUrl}
        subjectPath={subjectPath}
        subjectTitle={object.title}
      />
    </div>
  );
}

function involvementLabel(involvement: WorkAuditInvolvement) {
  switch (involvement.work_role.kind) {
    case "direct_subject":
      return "Direct audit subject";
    case "evidence":
      return "Attached as evidence";
    case "background":
      return "Attached as background";
  }
}

function involvementMeta(involvement: WorkAuditInvolvement) {
  return [
    involvementLabel(involvement),
    involvement.audit_state.status_label,
  ].join(" · ");
}

function roleBearing(involvement: WorkAuditInvolvement) {
  switch (involvement.work_role.kind) {
    case "direct_subject":
      return null;
    case "evidence":
    case "background":
      return involvement.work_role.bearing;
  }
}

/// Counts from the server-derived audit_state, so the card answers "how much
/// scrutiny has this audit seen?" without another fetch. For evidence roles,
/// warrant/review counts scoped to this artifact are appended.
function involvementCounts(involvement: WorkAuditInvolvement) {
  const { audit_state: state, work_role: role } = involvement;
  const parts = [
    `${state.element_review_count} reviews`,
    `${state.synthesis_review_count} reports`,
    `${state.challenge_count} challenges`,
  ];

  if (role.kind === "evidence" || role.kind === "background") {
    parts.push(
      `this work: ${role.warrant_count} warrants · ${role.review_count} reviews`,
    );
  }

  return parts.join(" · ");
}
