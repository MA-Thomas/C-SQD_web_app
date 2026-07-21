import Link from "next/link";
import { notFound } from "next/navigation";

import { AuditObjectDisclosure } from "../../../components/audit-object-disclosure";
import { BearingPill } from "../../../components/bearing-pill";
import { CriterionCluster } from "../../../components/criterion-cluster";
import { EvidenceManager } from "../../../components/evidence-manager";
import { GatedAction } from "../../../components/gated-action";
import { ReportReader } from "../../../components/report-reader";
import { TupleBadge } from "../../../components/tuple-badge";
import {
  formatLabel,
  getAuditEpisodesForSubject,
  getAuditSubject,
  getDomainInstantiation,
  getEvalTuple,
  getEvidenceArtifacts,
  getFactsForSubject,
  getSynthesisReviews,
  type EvidenceArtifactSummary,
  type SynthesisReview,
} from "../../../lib/csqd-api";
import {
  factKind,
  getAcademicCweNodes,
  groupedElementReviewsByCriterion,
} from "../../../lib/public-audit";
import { evalTupleValues } from "../../../lib/tuple-items";
import { warrantsByArtifact, warrantsFromFacts, type WarrantSummary } from "../../../lib/warrants";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

/// The claim audit page: the epistemic center of a claim-scoped audit
/// (CLAIM_SCOPED_AUDITS_MEMO.md). The bounded target claim leads, its scope
/// conditions are explicit, and attached papers appear as evidence artifacts
/// whose bearing is earned through audited warrants — never as votes. The
/// audit's findings (tuple, element reviews, synthesis) live here too, so the
/// page answers the audit question rather than just posing it.
export default async function ClaimAuditPage({ params }: PageProps) {
  const { id } = await params;
  const subject = await getAuditSubject(id);

  if (!subject) {
    notFound();
  }

  const [episodes, facts, domain, fallbackNodes] = await Promise.all([
    getAuditEpisodesForSubject(subject.id),
    getFactsForSubject(subject.id),
    getDomainInstantiation(subject.domain_instantiation_id).catch(() => null),
    getAcademicCweNodes(),
  ]);
  const evidenceByEpisode = await Promise.all(
    episodes.map(async (episode) => ({
      episode,
      artifacts: await getEvidenceArtifacts(episode.id),
      syntheses: await getSynthesisReviews(episode.id),
    })),
  );
  const allArtifacts = evidenceByEpisode.flatMap((entry) => entry.artifacts);
  const warrants = warrantsFromFacts(facts);
  const warrantsForArtifact = warrantsByArtifact(warrants);
  const openEpisode =
    episodes.find((episode) => episode.status === "active") ?? null;
  const tupleEpisode = openEpisode ?? episodes[0] ?? null;
  const tuple = tupleEpisode ? await getEvalTuple(tupleEpisode.id) : null;
  const latestReport = latestSynthesis(
    evidenceByEpisode.flatMap((entry) => entry.syntheses),
  );
  const openEpisodeArtifacts = openEpisode
    ? (evidenceByEpisode.find((entry) => entry.episode.id === openEpisode.id)
        ?.artifacts ?? [])
    : [];
  const nodes = domain?.cwe_nodes?.length ? domain.cwe_nodes : fallbackNodes;
  const reviewGroups = groupedElementReviewsByCriterion(facts, nodes);
  const populatedGroups = reviewGroups.filter((group) => group.reviews.length > 0);
  const elementReviewCount = facts.filter(
    (fact) => factKind(fact) === "element_review",
  ).length;
  const subjectPath = `/claims/${subject.id}`;

  return (
    <div className="pub-work-layout">
      <div className="pub-work-main">
        <header className="pub-work-header">
          <div className="pub-card-kicker">
            <span>{formatLabel(subject.subject_type)}</span>
            <span>Claim under audit</span>
          </div>
          <h1>
            {subject.claim_statement ?? subject.title ?? "Untitled audit subject"}
          </h1>
          {subject.claim_statement && subject.title ? (
            <p className="pub-work-authors">{subject.title}</p>
          ) : null}
          <AuditObjectDisclosure subjectType={subject.subject_type} />
          {subject.scope_conditions.length > 0 ? (
            <dl className="pub-facts">
              {subject.scope_conditions.map((condition) => (
                <div key={`${condition.label}-${condition.value}`}>
                  <dt>{formatLabel(condition.label)}</dt>
                  <dd>{condition.value}</dd>
                </div>
              ))}
            </dl>
          ) : (
            <p className="muted-copy">
              No explicit scope conditions recorded for this claim.
            </p>
          )}
          <div className="pub-work-tuple">
            <TupleBadge showVerdict tuple={evalTupleValues(tuple)} />
          </div>
        </header>

        <section className="pub-panel" id="latest-report">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">SynthesisReview</p>
              <h2>Latest Audit Report</h2>
            </div>
          </div>
          {latestReport ? (
            <ReportReader review={latestReport} />
          ) : (
            <div className="pub-empty">
              <h3>No SynthesisReview yet</h3>
              <p>
                ElementReviews of the attached artifacts and their warrants can
                be synthesized into a report once the record has enough depth.
              </p>
            </div>
          )}
        </section>

        <section className="pub-panel" id="evidence-artifacts">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Evidence</p>
              <h2>Attached Evidence Artifacts</h2>
            </div>
            <span className="pub-panel-count">
              {allArtifacts.length} artifacts · {warrants.length} warrants
            </span>
          </div>
          <p className="muted-copy">
            Papers do not vote. Each artifact&apos;s bearing on the target
            claim is earned through warrant links that survive element-review
            scrutiny — and shown here as a derived status, never a stored
            endorsement.
          </p>
          {allArtifacts.length === 0 ? (
            <div className="pub-empty">
              <h3>No evidence artifacts attached yet</h3>
              <p>
                Participants can attach scholarly works to an episode of this
                audit for inspection.
              </p>
            </div>
          ) : (
            evidenceByEpisode
              .filter((entry) => entry.artifacts.length > 0)
              .map((entry) => (
                <div key={entry.episode.id}>
                  {evidenceByEpisode.length > 1 ? (
                    <p className="pub-kicker">{entry.episode.label}</p>
                  ) : null}
                  {entry.artifacts.map((artifact) => (
                    <EvidenceArtifactRow
                      artifact={artifact}
                      key={artifact.artifact.id}
                      subjectPath={subjectPath}
                      warrants={
                        warrantsForArtifact.get(artifact.artifact.id) ?? []
                      }
                    />
                  ))}
                </div>
              ))
          )}
          {openEpisode ? (
            <EvidenceManager
              artifacts={openEpisodeArtifacts}
              episodeId={openEpisode.id}
            />
          ) : null}
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
              <h3>No ElementReviews yet</h3>
              <p>
                Reviews here can target the whole claim, one attached artifact,
                or a single warrant link — the unit the audit question turns on.
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

        <section className="pub-panel" id="episodes">
          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Episodes</p>
              <h2>Audit Episodes For This Claim</h2>
            </div>
            <span className="pub-panel-count">{episodes.length} episodes</span>
          </div>
          {episodes.length === 0 ? (
            <div className="pub-empty">
              <h3>No audit episodes yet</h3>
              <p>Commission an audit of this claim to open the record.</p>
            </div>
          ) : (
            <div className="pub-facts">
              {episodes.map((episode) => (
                <div className="pub-location-row" key={episode.id}>
                  <div>
                    <strong>{episode.label}</strong>
                    <span>
                      {formatLabel(episode.status)} ·{" "}
                      {formatDate(episode.authored_at)}
                    </span>
                  </div>
                  <Link href={`/audit-episodes/${episode.id}`}>
                    Open episode console
                  </Link>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>

      <aside className="pub-action-rail" aria-label="Claim audit state">
        <div className="pub-panel">
          <h3>Audit state</h3>
          <dl className="pub-facts">
            <div>
              <dt>Episodes</dt>
              <dd>{episodes.length}</dd>
            </div>
            <div>
              <dt>Evidence artifacts</dt>
              <dd>{allArtifacts.length}</dd>
            </div>
            <div>
              <dt>Warrant links</dt>
              <dd>{warrants.length}</dd>
            </div>
            <div>
              <dt>ElementReviews</dt>
              <dd>{elementReviewCount}</dd>
            </div>
            <div>
              <dt>Facts on record</dt>
              <dd>{facts.length}</dd>
            </div>
            <div>
              <dt>Registered</dt>
              <dd>{formatDate(subject.registered_at)}</dd>
            </div>
          </dl>
          <GatedAction className="primary-action" href={`${subjectPath}/review`}>
            Review one criterion
          </GatedAction>
          <Link
            className="secondary-action"
            href={`/commission?subject_id=${subject.id}`}
          >
            Commission an episode
          </Link>
        </div>
        <div className="pub-panel">
          <h3>The audit question</h3>
          <p className="muted-copy">
            Not &quot;how much does the literature support this claim?&quot;
            but: under the stated scope conditions, which warrant links for
            this claim survive audit?
          </p>
        </div>
      </aside>
    </div>
  );
}

function EvidenceArtifactRow({
  artifact,
  subjectPath,
  warrants,
}: {
  artifact: EvidenceArtifactSummary;
  subjectPath: string;
  warrants: WarrantSummary[];
}) {
  return (
    <div className="pub-location-row">
      <div>
        <strong>
          <Link href={`/works/${artifact.scholarly_object_id}`}>
            {artifact.title}
          </Link>
        </strong>
        <span>
          {artifact.authors.slice(0, 3).join(", ")}
          {artifact.authors.length > 3 ? " et al." : ""}
          {artifact.publication_year ? ` · ${artifact.publication_year}` : ""} ·{" "}
          {artifact.source_name}
        </span>
        <span>
          <BearingPill bearing={artifact.bearing} /> {artifact.warrant_count}{" "}
          warrants · {artifact.review_count} reviews
          {artifact.artifact.role === "background" ? " · Background" : ""}
        </span>
        {warrants.length > 0 ? (
          <details className="advanced-details">
            <summary>
              {warrants.length} warrant link{warrants.length === 1 ? "" : "s"} —
              why this artifact is supposed to bear on the claim
            </summary>
            {warrants.map((warrant) => (
              <WarrantCard
                key={warrant.factId}
                subjectPath={subjectPath}
                warrant={warrant}
              />
            ))}
          </details>
        ) : null}
      </div>
      <a href={artifact.canonical_url} rel="noreferrer" target="_blank">
        Source
      </a>
    </div>
  );
}

/// One warrant assertion, displayed for scrutiny: the artifact's own claim,
/// the inference type connecting it to the target claim, and the assumptions
/// that inference needs. The review link pre-targets this warrant so "does
/// this link survive?" is one click from reading it.
function WarrantCard({
  subjectPath,
  warrant,
}: {
  subjectPath: string;
  warrant: WarrantSummary;
}) {
  return (
    <article className="pub-review-card" id={`warrant-${warrant.factId}`}>
      <div className="pub-card-kicker">
        <span>{formatLabel(warrant.inferenceType)} inference</span>
        <span>Warrant assertion</span>
      </div>
      <p>{warrant.artifactClaim}</p>
      {warrant.assumptions ? (
        <small>Required assumptions: {warrant.assumptions}</small>
      ) : null}
      {warrant.rationale ? <small>Rationale: {warrant.rationale}</small> : null}
      <div className="pub-review-card-actions">
        <Link
          className="secondary-action"
          href={`${subjectPath}/review?warrant=${encodeURIComponent(warrant.factId)}`}
        >
          Scrutinize this warrant
        </Link>
      </div>
    </article>
  );
}

function latestSynthesis(reviews: SynthesisReview[]): SynthesisReview | null {
  if (reviews.length === 0) {
    return null;
  }

  return [...reviews].sort((left, right) =>
    right.authored_at.localeCompare(left.authored_at),
  )[0];
}

function formatDate(value: string) {
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
