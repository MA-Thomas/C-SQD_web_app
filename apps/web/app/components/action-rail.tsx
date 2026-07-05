import { GatedAction } from "./gated-action";
import { EpisodeParticipationActions } from "./subject-actions";
import { TupleRecomputePanel } from "./tuple-recompute";
import type { AuditEpisode } from "../lib/csqd-api";

/// Sticky right rail on the full-coverage page: every public action in one
/// place, visible to everyone, write actions auth-gated. Money (commission)
/// and reading (source) sit alongside contribution actions.
export function ActionRail({
  subjectPath,
  subjectTitle,
  auditSubjectId,
  commissionHref,
  sourceUrl,
  openEpisodeId,
  episodes,
  facts,
}: {
  subjectPath: string;
  subjectTitle: string;
  auditSubjectId: string | null;
  commissionHref: string;
  sourceUrl: string;
  openEpisodeId: string | null;
  episodes: AuditEpisode[];
  facts: Array<{ label: string; value: string | number }>;
}) {
  return (
    <aside className="pub-action-rail" aria-label="Audit actions">
      <div className="pub-panel">
        <h3>Participate</h3>
        <div className="pub-action-stack">
          <GatedAction
            className="primary-action"
            href={`${subjectPath}/review`}
            explain="ElementReviews are focused reviews of one criterion and carry provenance."
          >
            Submit ElementReview
          </GatedAction>
          <GatedAction
            className="secondary-action"
            href={
              openEpisodeId
                ? `${subjectPath}/review?synthesis=1`
                : `${subjectPath}/review`
            }
            explain="Unsolicited SynthesisReviews require starting or joining the public episode first."
          >
            Submit SynthesisReview
          </GatedAction>
          <EpisodeParticipationActions
            auditSubjectId={auditSubjectId}
            openEpisodeId={openEpisodeId}
            subjectPath={subjectPath}
            subjectTitle={subjectTitle}
          />
          <GatedAction
            className="secondary-action"
            href="/library"
            explain="Saving to your library requires an account."
          >
            Save to library
          </GatedAction>
        </div>
      </div>

      <div className="pub-panel">
        <h3>Deepen</h3>
        <div className="pub-action-stack">
          <a className="secondary-action" href={sourceUrl} rel="noreferrer" target="_blank">
            Open source material
          </a>
          <GatedAction className="secondary-action" href={commissionHref}>
            Commission deeper audit
          </GatedAction>
        </div>
      </div>

      <div className="pub-panel">
        <h3>Audit record</h3>
        <dl className="pub-facts">
          {facts.map((fact) => (
            <div key={fact.label}>
              <dt>{fact.label}</dt>
              <dd>{fact.value}</dd>
            </div>
          ))}
        </dl>
      </div>

      {episodes.length > 0 ? (
        <div className="pub-panel">
          <TupleRecomputePanel episodes={episodes} />
        </div>
      ) : null}
    </aside>
  );
}
