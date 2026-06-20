import Link from "next/link";

import { formatLabel, type ScholarlyObjectSummary } from "../lib/csqd-api";
import {
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../lib/public-audit";
import { GatedAction } from "./gated-action";
import { StatusPill } from "./status-pill";
import { TupleBadge } from "./tuple-badge";

type PublicWorkCardProps = {
  group: ScholarlyWorkGroup;
  summary: PublicAuditSummary | null;
  primaryActionLabel?: string;
};

export function PublicWorkCard({
  group,
  primaryActionLabel = "Open audit record",
  summary,
}: PublicWorkCardProps) {
  const object = group.primaryVersion;
  const status = publicAuditStatusLabel(object, summary);

  return (
    <article className="object-card work-card public-work-card">
      <div className="object-main">
        <div className="object-kicker">
          <StatusPill status={status} />
          <span>{formatLabel(object.version_kind)}</span>
          {object.publication_year ? <span>{object.publication_year}</span> : null}
        </div>
        <h2>{group.title}</h2>
        <p>{object.authors.join(", ") || object.source_name}</p>
        <TupleBadge size="compact" tuple={summary?.tuple ?? null} />
        <div className="object-actions">
          <Link href={`/scholarly-objects/${object.id}`}>{primaryActionLabel}</Link>
          <Link href={`/scholarly-objects/${object.id}/review`}>
            Review one criterion
          </Link>
          {object.audit_subject_id ? (
            <Link href={`/commission?subject_id=${object.audit_subject_id}`}>
              Commission deeper audit
            </Link>
          ) : (
            <Link href={`/commission`}>Commission deeper audit</Link>
          )}
          <GatedAction
            className="object-action-link"
            explain="Watching a subject saves it to your library."
            href="/library"
          >
            Watch
          </GatedAction>
        </div>
        <VersionStrip versions={group.versions} />
      </div>
      <dl className="object-facts public-work-facts">
        <div>
          <dt>CRWE coverage</dt>
          <dd>{summary?.crweCoverageCount ?? 0}</dd>
        </div>
        <div>
          <dt>ElementReviews</dt>
          <dd>{summary?.elementReviewCount ?? group.elementReviewCount}</dd>
        </div>
        <div>
          <dt>SynthesisReviews</dt>
          <dd>{summary?.synthesisReviewCount ?? group.synthesisReviewCount}</dd>
        </div>
        <div>
          <dt>Challenges</dt>
          <dd>{summary?.challengeCount ?? 0}</dd>
        </div>
      </dl>
    </article>
  );
}

function VersionStrip({ versions }: { versions: ScholarlyObjectSummary[] }) {
  if (versions.length <= 1) {
    return null;
  }

  return (
    <div className="compact-version-strip" aria-label="Known versions">
      {versions.map((version) => (
        <Link href={`/scholarly-objects/${version.id}`} key={version.id}>
          {formatLabel(version.version_kind)}
          {version.publication_year ? ` ${version.publication_year}` : ""}
        </Link>
      ))}
    </div>
  );
}
