import type { Fact, SynthesisReviewRelation } from "../lib/csqd-api";
import { formatDate, payloadRecord, stringValue } from "../lib/public-audit";

/// Visually distinct dissent block: contestations and challenge responses,
/// kept apart from the coverage so disagreement reads as first-class signal
/// rather than a footnote. The challenged artifacts stay on the record.
export function DissentBlock({
  contestations,
  challengeResponses,
}: {
  contestations: SynthesisReviewRelation[];
  challengeResponses: Fact[];
}) {
  if (contestations.length === 0 && challengeResponses.length === 0) {
    return (
      <div className="pub-empty">
        <h3>No public challenges recorded</h3>
        <p>
          Challenges contest specific ElementReviews or SynthesisReviews while
          preserving the historical audit trail.
        </p>
      </div>
    );
  }

  return (
    <div>
      {contestations.map((relation) => (
        <article className="pub-dissent-entry" key={relation.id}>
          <p className="pub-dissent-kind">
            Report contested
            {typeof relation.relation_type === "object"
              ? ` · ${relation.relation_type.contests.scope}`
              : null}
          </p>
          {typeof relation.relation_type === "object" &&
          relation.relation_type.contests.rationale ? (
            <p>{relation.relation_type.contests.rationale}</p>
          ) : null}
          <p className="pub-dissent-meta">{formatDate(relation.asserted_at)}</p>
        </article>
      ))}
      {challengeResponses.map((fact) => {
        const payload = payloadRecord(fact, "submitter_response");

        return (
          <article className="pub-dissent-entry" key={fact.id}>
            <p className="pub-dissent-kind">ElementReview challenged</p>
            <p>{stringValue(payload?.content)}</p>
            <p className="pub-dissent-meta">{formatDate(fact.occurred_at)}</p>
          </article>
        );
      })}
    </div>
  );
}
