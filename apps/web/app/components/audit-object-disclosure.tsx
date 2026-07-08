import {
  formatLabel,
  type ClaimAuditRole,
  type WorkRoleInAudit,
} from "../lib/csqd-api";

type AuditObjectDisclosureProps = {
  subjectType: string;
  claimRole?: ClaimAuditRole;
  workRole?: WorkRoleInAudit;
};

export function AuditObjectDisclosure({
  claimRole,
  subjectType,
  workRole,
}: AuditObjectDisclosureProps) {
  return (
    <dl className="pub-facts">
      {claimRole ? (
        <div>
          <dt>Claim role</dt>
          <dd>{claimRoleStatement(subjectType, claimRole)}</dd>
        </div>
      ) : null}
      <div>
        <dt>What is being audited?</dt>
        <dd>{auditObjectStatement(subjectType)}</dd>
      </div>
      {workRole ? (
        <div>
          <dt>This work&apos;s role</dt>
          <dd>{workRoleStatement(subjectType, workRole)}</dd>
        </div>
      ) : null}
    </dl>
  );
}

export function WorkRecordDisclosure() {
  return (
    <dl className="pub-facts">
      <div>
        <dt>What is this page?</dt>
        <dd>
          This is a scholarly-work record. The work may be the direct audit
          object in one audit, evidence in another, and background context in a
          third.
        </dd>
      </div>
    </dl>
  );
}

function auditObjectStatement(subjectType: string) {
  if (subjectType === "scoped_claim") {
    return "This audit evaluates a scoped claim. Attached papers are evidence, not the audit object.";
  }

  if (isScholarlyWorkSubject(subjectType)) {
    return `This audit evaluates the ${subjectLabel(subjectType)} itself as the audit object.`;
  }

  return `This audit evaluates a ${subjectLabel(subjectType)} as the audit object.`;
}

function workRoleStatement(subjectType: string, role: WorkRoleInAudit) {
  switch (role.kind) {
    case "direct_subject":
      return isScholarlyWorkSubject(subjectType)
        ? "In this audit, this work is the direct audit object and is fulfilling the auditable-claim role."
        : "In this audit, this work is the audit target itself.";
    case "evidence":
      return subjectType === "scoped_claim"
        ? "In this audit, this work is evidence for the scoped claim, not the audit object."
        : `In this audit, this work is evidence for the ${subjectLabel(subjectType)}, not the audit object.`;
    case "background":
      return "In this audit, this work is background context, not the audit object.";
  }
}

function claimRoleStatement(subjectType: string, role: ClaimAuditRole) {
  switch (role.kind) {
    case "explicit_scoped_claim":
      return "The claim is explicitly stated as a bounded audit subject.";
    case "work_as_claim":
      return `The ${subjectLabel(subjectType)} is directly fulfilling the auditable-claim role.`;
  }
}

function isScholarlyWorkSubject(subjectType: string) {
  return [
    "research_manuscript",
    "preprint",
    "dataset",
    "code_repository",
    "clinical_trial_protocol",
    "technical_report",
  ].includes(subjectType);
}

function subjectLabel(subjectType: string) {
  return formatLabel(subjectType).toLowerCase();
}
