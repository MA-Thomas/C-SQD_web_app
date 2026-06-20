import type { Fact } from "../lib/csqd-api";
import { factKind, formatDate, payloadRecord, stringValue } from "../lib/public-audit";

const KIND_LABEL: Record<string, string> = {
  audit_commission: "Audit commissioned",
  element_review: "ElementReview submitted",
  er_solicitation: "Review solicited",
  solicitation_event: "Solicitation update",
  submitter_response: "Submitter response",
  episode_participation: "Joined public episode",
  feature_petition: "Petition to feature",
  cwe_petition: "CRWE petition",
  curation_decision: "Curation decision",
};

function factHeadline(fact: Fact) {
  const kind = factKind(fact);
  const label = KIND_LABEL[kind] ?? kind;

  if (kind === "element_review") {
    const payload = payloadRecord(fact, "element_review");
    const finding = stringValue(payload?.finding).replaceAll("_", " ");

    return finding ? `${label} · ${finding}` : label;
  }

  if (kind === "submitter_response") {
    const payload = payloadRecord(fact, "submitter_response");
    const responseType = stringValue(payload?.response_type).replaceAll("_", " ");

    return responseType ? `${label} · ${responseType}` : label;
  }

  return label;
}

function factStatusNote(fact: Fact) {
  if (typeof fact.status === "string") {
    return fact.status === "active" ? null : fact.status;
  }

  const [state] = Object.keys(fact.status as Record<string, unknown>);

  return state ?? null;
}

/// Vertical, interleaved audit trail. Collapsed by default on public pages
/// (provenance is advanced detail); the data is ordered oldest-first.
export function FactTimeline({ facts }: { facts: Fact[] }) {
  const ordered = [...facts].sort(
    (left, right) =>
      new Date(left.occurred_at).getTime() - new Date(right.occurred_at).getTime(),
  );

  if (ordered.length === 0) {
    return <p className="timeline-empty">No public facts recorded yet.</p>;
  }

  return (
    <ol className="fact-timeline">
      {ordered.map((fact) => {
        const statusNote = factStatusNote(fact);

        return (
          <li className="timeline-entry" id={`fact-${fact.id}`} key={fact.id}>
            <span className={`timeline-dot timeline-${factKind(fact)}`} aria-hidden />
            <div className="timeline-body">
              <p className="timeline-headline">
                {factHeadline(fact)}
                {statusNote ? (
                  <span className="timeline-status"> · {statusNote}</span>
                ) : null}
              </p>
              <p className="timeline-meta">{formatDate(fact.occurred_at)}</p>
            </div>
          </li>
        );
      })}
    </ol>
  );
}
