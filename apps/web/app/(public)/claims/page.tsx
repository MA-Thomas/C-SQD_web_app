import Link from "next/link";

import { RegistryUnavailable } from "../../components/registry-unavailable";
import {
  formatLabel,
  getClaimAuditIndex,
  isApiReachable,
  type ClaimAuditIndexEntry,
} from "../../lib/csqd-api";

export default async function ClaimsIndexPage() {
  const claimAudits = await getClaimAuditIndex();
  const registryDown =
    claimAudits.length === 0 && !(await isApiReachable());

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Claim audits</p>
          <h1>Claims Under Audit</h1>
          <p>
            This index includes explicit scoped claims and papers or preprints
            whose assertions are being audited directly. Papers attached only as
            evidence appear inside the claim audit they bear on.
          </p>
        </div>
        <Link className="secondary-action" href="/commission">
          Commission a claim audit
        </Link>
      </header>

      {registryDown ? (
        <RegistryUnavailable />
      ) : claimAudits.length === 0 ? (
        <div className="pub-empty">
          <h3>No claim audits yet</h3>
          <p>
            Create one from Commission by choosing Scoped claim, or audit a
            scholarly work directly when the paper itself is serving as the
            auditable claim object.
          </p>
        </div>
      ) : (
        <section className="pub-section">
          <div className="pub-grid claim-audit-grid">
            {claimAudits.map((entry) => (
              <article className="pub-card claim-audit-card" key={entry.subject.subject_id}>
                <div className="pub-card-kicker">
                  <span>{claimRoleLabel(entry)}</span>
                  <span className="pub-card-kicker-object">
                    {auditObjectLabel(entry)}
                  </span>
                </div>
                <h3>
                  <Link href={entryHref(entry)}>{entryTitle(entry)}</Link>
                </h3>
                {entrySourceLine(entry) ? (
                  <p className="pub-source-line">{entrySourceLine(entry)}</p>
                ) : null}
                <dl className="pub-facts">
                  <div>
                    <dt>Audit state</dt>
                    <dd>{entry.audit_state.status_label}</dd>
                  </div>
                  <div>
                    <dt>Activity</dt>
                    <dd>{entryCounts(entry)}</dd>
                  </div>
                </dl>
                <div className="pub-card-actions">
                  <Link href={entryHref(entry)}>{entryActionLabel(entry)}</Link>
                </div>
              </article>
            ))}
          </div>
        </section>
      )}
    </>
  );
}

function entryTitle(entry: ClaimAuditIndexEntry) {
  return (
    entry.subject.claim_statement ??
    entry.subject.title ??
    entry.scholarly_object?.title ??
    "Untitled claim audit"
  );
}

function entrySourceLine(entry: ClaimAuditIndexEntry) {
  if (entry.claim_role.kind === "explicit_scoped_claim") {
    return entry.subject.title && entry.subject.claim_statement
      ? entry.subject.title
      : null;
  }

  const object = entry.scholarly_object;
  if (!object) {
    return entry.subject.title;
  }

  const authors = object.authors.slice(0, 3).join(", ");
  return [
    authors || null,
    object.source_name,
    object.publication_year?.toString() ?? null,
  ]
    .filter(Boolean)
    .join(" · ");
}

function entryHref(entry: ClaimAuditIndexEntry) {
  if (entry.claim_role.kind === "work_as_claim" && entry.scholarly_object) {
    return `/works/${entry.scholarly_object.id}`;
  }

  return `/claims/${entry.subject.subject_id}`;
}

function entryActionLabel(entry: ClaimAuditIndexEntry) {
  return entry.claim_role.kind === "work_as_claim"
    ? "Open work claim audit"
    : "Open claim audit";
}

function claimRoleLabel(entry: ClaimAuditIndexEntry) {
  return entry.claim_role.kind === "work_as_claim"
    ? `${formatLabel(entry.subject.subject_type)} serving as claim`
    : "Explicit scoped claim";
}

function auditObjectLabel(entry: ClaimAuditIndexEntry) {
  return `Audit object: ${formatLabel(entry.subject.subject_type)}`;
}

function entryCounts(entry: ClaimAuditIndexEntry) {
  const parts = [
    `${entry.audit_state.element_review_count} reviews`,
    `${entry.audit_state.synthesis_review_count} reports`,
  ];

  if (entry.evidence_artifact_count > 0) {
    parts.push(`${entry.evidence_artifact_count} evidence artifacts`);
  }

  if (entry.audit_state.challenge_count > 0) {
    parts.push(`${entry.audit_state.challenge_count} challenges`);
  }

  return parts.join(" · ");
}
