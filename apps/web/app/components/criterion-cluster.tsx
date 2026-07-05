import { formatLabel, type Fact } from "../lib/csqd-api";
import { payloadRecord, stringValue, type CriterionReviewGroup } from "../lib/public-audit";
import {
  ChallengeElementReviewForm,
  FeaturePetitionForm,
} from "./subject-actions";

/// "Coverage" clustering: one collapsible cluster per CRWE criterion with
/// public ElementReviews inside. The full-coverage analog of grouping a
/// story's articles by outlet.
export function CriterionCluster({
  group,
  episodeId,
  subjectPath,
}: {
  group: CriterionReviewGroup;
  episodeId: string | null;
  subjectPath: string;
}) {
  return (
    <details className="pub-cluster" id={`criterion-${group.node.id}`}>
      <summary>
        <span>{group.node.label}</span>
        <span className="pub-cluster-count">
          {group.reviews.length} review{group.reviews.length === 1 ? "" : "s"}
        </span>
      </summary>
      <div className="pub-cluster-body">
        {group.reviews.map((fact) => (
          <ElementReviewCard
            episodeId={episodeId}
            fact={fact}
            key={fact.id}
            subjectPath={subjectPath}
          />
        ))}
      </div>
    </details>
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
    <article className="pub-review-card" id={`fact-${fact.id}`}>
      <div className="pub-card-kicker">
        <span>{formatLabel(finding)}</span>
        {stringValue(payload?.severity) ? (
          <span>{formatLabel(stringValue(payload?.severity))}</span>
        ) : null}
        {stringValue(payload?.confidence) ? (
          <span>{formatLabel(stringValue(payload?.confidence))} confidence</span>
        ) : null}
        <span>{payload?.solicitation ? "Commissioned" : "Unsolicited"}</span>
      </div>
      <p>{stringValue(payload?.content) || "ElementReview fact"}</p>
      {stringValue(payload?.limitations) ? (
        <small>Limitations: {stringValue(payload?.limitations)}</small>
      ) : null}
      {stringValue(payload?.recommendations) ? (
        <small>Recommendations: {stringValue(payload?.recommendations)}</small>
      ) : null}
      <div className="pub-review-card-actions">
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
