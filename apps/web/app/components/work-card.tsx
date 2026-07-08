import Link from "next/link";

import { formatLabel } from "../lib/csqd-api";
import {
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../lib/public-audit";
import { StatusPill } from "./status-pill";
import { TupleBadge } from "./tuple-badge";

/// Registry card with strict news-card hierarchy: status kicker → headline →
/// source line → tuple + counts. The headline is the primary link into the
/// full-coverage page.
export function WorkCard({
  group,
  summary,
}: {
  group: ScholarlyWorkGroup;
  summary: PublicAuditSummary | null;
}) {
  const object = group.primaryVersion;
  const status = publicAuditStatusLabel(object, summary);
  const workHref = `/works/${object.id}`;
  const elementReviews = summary?.elementReviewCount ?? group.elementReviewCount;
  const reports = summary?.synthesisReviewCount ?? group.synthesisReviewCount;
  const challenges = summary?.challengeCount ?? 0;

  return (
    <article className="pub-card">
      <div className="pub-card-kicker">
        <StatusPill status={status} />
        <span>{formatLabel(object.version_kind)}</span>
        <span>Work record</span>
        {group.versionCount > 1 ? <span>{group.versionCount} versions</span> : null}
      </div>
      <h3>
        <Link href={workHref}>{group.title}</Link>
      </h3>
      <p className="pub-source-line">
        {object.authors.join(", ") || object.source_name}
        {object.source_name && object.authors.length > 0 ? ` · ${object.source_name}` : ""}
        {object.publication_year ? ` · ${object.publication_year}` : ""}
      </p>
      <TupleBadge size="compact" tuple={summary?.tuple ?? null} />
      <div className="pub-card-meta">
        <span>
          <strong>{elementReviews}</strong> ElementReview{elementReviews === 1 ? "" : "s"}
        </span>
        <span>
          <strong>{reports}</strong> report{reports === 1 ? "" : "s"}
        </span>
        {challenges > 0 ? (
          <span>
            <strong>{challenges}</strong> challenge{challenges === 1 ? "" : "s"}
          </span>
        ) : null}
      </div>
      <div className="pub-card-actions">
        <Link href={workHref}>Open work record</Link>
        <Link href={`${workHref}/review`}>Review one criterion</Link>
      </div>
    </article>
  );
}
