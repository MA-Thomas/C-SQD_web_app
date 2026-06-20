import type { SynthesisReview } from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

const SECTION_TITLES: Record<string, string> = {
  summary: "Summary",
  methodological_assessment: "Methodological Assessment",
  ethical_assessment: "Ethical Assessment",
  evidence_integration: "Evidence Integration",
  recommendations: "Recommendations",
  open_questions: "Open Questions",
};

/// Renders a SynthesisReview as a readable audit report: typographic
/// sections, fact citations, stable anchors. Reports are the destination.
export function ReportReader({
  review,
  authorName,
}: {
  review: SynthesisReview;
  authorName?: string;
}) {
  return (
    <article className="report-reader" id={`report-${review.id}`}>
      <header className="report-header">
        <div>
          <p className="eyebrow">
            {review.unsolicited ? "Unsolicited audit report" : "Audit report"}
            {review.status === "draft" ? " · draft" : null}
            {review.status === "superseded" ? " · superseded" : null}
          </p>
          <p className="report-byline">
            {authorName ?? "Reviewer"} · {formatDate(review.authored_at)}
          </p>
        </div>
        <a className="report-permalink" href={`#report-${review.id}`}>
          Permalink
        </a>
      </header>

      <p className="report-summary">{review.summary}</p>

      {review.sections.map((section) => (
        <section
          className="report-section"
          id={`report-${review.id}-${section.section_type}`}
          key={section.id}
        >
          <h3>{SECTION_TITLES[section.section_type] ?? section.section_type}</h3>
          <p>{section.content}</p>
          {section.referenced_facts.length > 0 ? (
            <p className="report-citations">
              Cites:{" "}
              {section.referenced_facts.map((factId, index) => (
                <span key={factId}>
                  {index > 0 ? ", " : ""}
                  <a href={`#fact-${factId}`}>ElementReview {factId.slice(0, 8)}</a>
                </span>
              ))}
            </p>
          ) : null}
        </section>
      ))}
    </article>
  );
}
