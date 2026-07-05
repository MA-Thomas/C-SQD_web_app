import Link from "next/link";

import {
  formatDate,
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../lib/public-audit";
import { StatusPill } from "./status-pill";

/// Dense list row for rails: clamped title, then a quiet meta line. The
/// `detail` slot lets each rail say why the row is in it (challenge count,
/// scrutiny depth, review need).
export function WorkListRow({
  group,
  summary,
  detail,
}: {
  group: ScholarlyWorkGroup;
  summary: PublicAuditSummary | null;
  detail?: React.ReactNode;
}) {
  const object = group.primaryVersion;

  return (
    <li>
      <p className="pub-row-title">
        <Link href={`/works/${object.id}`}>{group.title}</Link>
      </p>
      <p className="pub-row-meta">
        <StatusPill status={publicAuditStatusLabel(object, summary)} />
        {detail}
        {summary?.latestReport ? (
          <span>{formatDate(summary.latestReport.authored_at)}</span>
        ) : null}
      </p>
    </li>
  );
}

/// Titled rail card wrapping a `pub-rail-list`.
export function SectionRail({
  title,
  footerHref,
  footerLabel,
  children,
}: {
  title: string;
  footerHref?: string;
  footerLabel?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="pub-rail" aria-label={title}>
      <h3>{title}</h3>
      <ul className="pub-rail-list">{children}</ul>
      {footerHref ? (
        <Link className="pub-rail-footer" href={footerHref}>
          {footerLabel ?? "View all"}
        </Link>
      ) : null}
    </section>
  );
}
