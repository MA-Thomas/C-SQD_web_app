import Link from "next/link";

import {
  formatDate,
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../lib/public-audit";
import { StatusPill } from "./status-pill";
import { TupleBadge } from "./tuple-badge";

/// "Top story" treatment for the most recent public audit report: big
/// headline, report summary as the deck, then related-coverage links into
/// the full-coverage page (reviews, challenges, method).
export function LeadStoryCard({
  group,
  summary,
}: {
  group: ScholarlyWorkGroup;
  summary: PublicAuditSummary | null;
}) {
  const object = group.primaryVersion;
  const workHref = `/works/${object.id}`;
  const report = summary?.latestReport ?? null;
  const elementReviews = summary?.elementReviewCount ?? group.elementReviewCount;
  const challenges = summary?.challengeCount ?? 0;

  return (
    <article className="pub-lead-card">
      <div className="pub-card-kicker">
        <StatusPill status={publicAuditStatusLabel(object, summary)} />
        <span>Latest public audit report</span>
      </div>
      <h2>
        <Link href={workHref}>{group.title}</Link>
      </h2>
      {report ? <p className="pub-lead-deck">{report.summary}</p> : null}
      <p className="pub-lead-byline">
        {object.authors.slice(0, 3).join(", ")}
        {object.authors.length > 3 ? " et al." : ""}
        {object.source_name ? ` · ${object.source_name}` : ""}
        {report ? ` · report ${formatDate(report.authored_at)}` : ""}
      </p>
      <TupleBadge showVerdict size="compact" tuple={summary?.tuple ?? null} />
      <div className="pub-lead-related">
        <Link href={`${workHref}#latest-report`}>Read the full report</Link>
        <Link href={`${workHref}#element-reviews`}>
          {elementReviews} ElementReview{elementReviews === 1 ? "" : "s"}
        </Link>
        {challenges > 0 ? (
          <Link href={`${workHref}#challenges`}>
            {challenges} challenge{challenges === 1 ? "" : "s"}
          </Link>
        ) : null}
        <Link href={workHref}>Open work record</Link>
      </div>
    </article>
  );
}
