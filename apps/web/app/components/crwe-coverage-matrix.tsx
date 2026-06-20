import Link from "next/link";

import type { CWENode, Fact } from "../lib/csqd-api";
import { criterionNodeId, payloadRecord } from "../lib/public-audit";

type RowState = "unreviewed" | "clear" | "problems" | "contested";

function rowState(reviews: Fact[]): RowState {
  if (reviews.length === 0) {
    return "unreviewed";
  }

  const findings = reviews
    .map((fact) => payloadRecord(fact, "element_review"))
    .map((payload) => (payload?.finding as string) ?? "");

  if (
    findings.some(
      (finding) =>
        finding === "non_ethical_problem" || finding === "ethical_problem",
    )
  ) {
    return "problems";
  }

  return "clear";
}

const STATE_LABEL: Record<RowState, string> = {
  unreviewed: "Unreviewed",
  clear: "No problems found",
  problems: "Problems identified",
  contested: "Contested",
};

/// Shape accompanies color so state is legible without color vision; the
/// adjacent text label carries the meaning for assistive tech.
const STATE_GLYPH: Record<RowState, string> = {
  unreviewed: "·",
  clear: "✓",
  problems: "!",
  contested: "⚖",
};

/// One row per CWE criterion, colored by review state, with a per-row review
/// CTA. Teaches decomposed judgment by showing exactly which criteria have
/// been scrutinized.
export function CrweCoverageMatrix({
  nodes,
  facts,
  reviewHrefBase,
  anchorPrefix = "criterion",
}: {
  nodes: CWENode[];
  facts: Fact[];
  /// e.g. `/scholarly-objects/<id>/review` — `?criterion=<node_id>` is added.
  reviewHrefBase?: string;
  anchorPrefix?: string;
}) {
  if (nodes.length === 0) {
    return (
      <p className="coverage-empty">
        No CRWE criteria are configured for this domain yet.
      </p>
    );
  }

  return (
    <ul className="coverage-matrix" aria-label="CRWE coverage by criterion">
      {nodes.map((node) => {
        const reviews = facts.filter((fact) => {
          const payload = payloadRecord(fact, "element_review");

          return criterionNodeId(payload) === node.id;
        });
        const state = rowState(reviews);

        return (
          <li className={`coverage-row coverage-${state}`} key={node.id}>
            <span className="coverage-state-dot" aria-hidden>
              {STATE_GLYPH[state]}
            </span>
            <div className="coverage-main">
              <a href={`#${anchorPrefix}-${node.id}`} className="coverage-label">
                {node.label}
              </a>
              <span className="coverage-meta">
                {STATE_LABEL[state]}
                {reviews.length > 0
                  ? ` · ${reviews.length} ElementReview${reviews.length === 1 ? "" : "s"}`
                  : ""}
              </span>
            </div>
            {reviewHrefBase ? (
              <Link
                className="coverage-action"
                href={`${reviewHrefBase}?criterion=${node.id}`}
              >
                Review this criterion
              </Link>
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}
