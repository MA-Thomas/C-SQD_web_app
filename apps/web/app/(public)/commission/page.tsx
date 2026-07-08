import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import {
  commissionAuditEpisode,
  createAuditSubject,
  formatLabel,
  getAuditSubjects,
  getDomainInstantiation,
  getDomainInstantiations,
  type DomainInstantiationSummary,
} from "../../lib/csqd-api";

type PageProps = {
  searchParams: Promise<{
    subject_id?: string;
  }>;
};

/// The scoped claim leads: papers, models, and datasets are audited as
/// artifacts only when the target genuinely is the artifact. For claim
/// audits, papers attach to the episode as evidence artifacts instead
/// (claim-scoped audits memo).
const subjectTypes = [
  "scoped_claim",
  "research_manuscript",
  "preprint",
  "dataset",
  "code_repository",
  "clinical_trial_protocol",
  "ai_model_evaluation",
  "benchmark",
  "policy_document",
  "grant_proposal",
  "technical_report",
  "other",
];

const organizationTypes = [
  "biotech",
  "venture_capital",
  "foundation",
  "university",
  "journal",
  "regulator",
  "other",
];

/// Public commission path: the form is fully public up to submission (zero
/// friction before intent). Submitting creates/reuses the AuditSubject and
/// commissions the episode.
export default async function CommissionPage({ searchParams }: PageProps) {
  const { subject_id } = await searchParams;
  const selectedSubjectId = subject_id?.trim() ?? "";
  const [domains, subjects] = await Promise.all([
    getDomainInstantiations(),
    getAuditSubjects(),
  ]);
  const selectedSubject =
    subjects.find((subject) => subject.id === selectedSubjectId) ?? null;
  const selectedDomainId =
    selectedSubject?.domain_instantiation_id ??
    preferredDomain(domains)?.id ??
    domains[0]?.id ??
    "";
  const domainDetail = selectedDomainId
    ? await getDomainInstantiation(selectedDomainId)
    : null;
  const cweNodes = domainDetail?.cwe_nodes ?? [];

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Commission</p>
          <h1>Commission an Audit</h1>
          <p>
            Fund a structured, decomposed audit of a bounded claim — or of a
            paper, model, dataset, or protocol directly. State the target
            claim and its scope conditions, attach papers as evidence
            artifacts to be inspected, and scope the audit by criterion.
            Delivery lands as an audit report on the public or private record.
          </p>
        </div>
      </header>

      <div className="pub-commission-layout">
        <form action={commissionAction} className="pub-panel audit-form">
          <input name="domain_instantiation_id" type="hidden" value={selectedDomainId} />

          <div className="pub-panel-head">
            <div>
              <p className="pub-kicker">Audit subject</p>
              <h2>Subject + Sponsor</h2>
            </div>
            <span className="pub-panel-count">
              {domainDetail ? domainDetail.name : "No domain"}
            </span>
          </div>

          <label>
            Existing subject
            <select defaultValue={selectedSubjectId} name="subject_id">
              <option value="">Create a new audit subject</option>
              {subjects.map((subject) => (
                <option key={subject.id} value={subject.id}>
                  {subject.title ?? subject.id} · {formatLabel(subject.subject_type)}
                </option>
              ))}
            </select>
          </label>

          <div className="pub-form-row">
            <label>
              New subject type
              <select
                defaultValue={selectedSubject?.subject_type ?? "scoped_claim"}
                name="subject_type"
              >
                {subjectTypes.map((subjectType) => (
                  <option key={subjectType} value={subjectType}>
                    {formatLabel(subjectType)}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Subject title
              <input
                defaultValue={selectedSubject?.title ?? ""}
                name="subject_title"
                placeholder="Short title for the audit subject"
                type="text"
              />
            </label>
          </div>

          <label>
            Target claim
            <textarea
              defaultValue={selectedSubject?.claim_statement ?? ""}
              name="claim_statement"
              placeholder='The bounded claim under audit, e.g. "Biomarker X predicts response to treatment Y in population Z"'
              rows={3}
            />
          </label>

          <label>
            Scope conditions
            <textarea
              name="scope_conditions"
              placeholder={
                "One per line, as label: value\npopulation: adults aged 40-70\noutcome: 12-month treatment response"
              }
              rows={4}
            />
            <small className="muted-copy">
              The explicit conditions under which the claim is evaluated —
              population, intervention, measurement, outcome, timeframe.
              Required for scoped-claim subjects along with the target claim.
            </small>
          </label>

          <label>
            Episode label
            <input
              defaultValue={
                selectedSubject?.title
                  ? `Commissioned audit: ${selectedSubject.title}`
                  : ""
              }
              name="label"
              placeholder="Commissioned audit episode"
              required
              type="text"
            />
          </label>

          <div className="pub-form-row">
            <label>
              Sponsor organization
              <input
                defaultValue="Northstar Bio Diligence"
                name="sponsor_organization_name"
                required
                type="text"
              />
            </label>
            <label>
              Sponsor type
              <select defaultValue="biotech" name="sponsor_organization_type">
                {organizationTypes.map((organizationType) => (
                  <option key={organizationType} value={organizationType}>
                    {formatLabel(organizationType)}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="pub-form-row">
            <label>
              Funding amount
              <input
                defaultValue="7500"
                min="1"
                name="funding_amount"
                required
                step="0.01"
                type="number"
              />
            </label>
            <label>
              Currency
              <input defaultValue="USD" name="funding_currency" required type="text" />
            </label>
          </div>

          <div className="pub-form-row">
            <label>
              Deadline
              <input name="deadline" type="datetime-local" />
            </label>
            <label className="pub-checkbox-inline">
              <input name="confidential" type="checkbox" value="true" />
              Confidential commission
            </label>
          </div>

          <fieldset>
            <legend>Audit scope</legend>
            {cweNodes.length === 0 ? (
              <p className="muted-copy">No criteria are configured for this domain.</p>
            ) : (
              <div className="pub-scope-options">
                {cweNodes.map((node, index) => (
                  <label key={node.id}>
                    <input
                      defaultChecked={index < 2}
                      name="scope_cwe_node_ids"
                      type="checkbox"
                      value={node.id}
                    />
                    <span>
                      <strong>{node.label}</strong>
                      <small>{node.description}</small>
                    </span>
                  </label>
                ))}
              </div>
            )}
          </fieldset>

          <label>
            Notes
            <textarea
              name="notes"
              placeholder="Scope boundaries, sponsor constraints, or delivery notes"
              rows={5}
            />
          </label>

          <button className="primary-action" type="submit">
            Commission audit
          </button>
        </form>

        <aside className="pub-action-rail" aria-label="Selected subject">
          <div className="pub-panel">
            <h3>Selection</h3>
            <dl className="pub-facts">
              <div>
                <dt>Subject</dt>
                <dd>{selectedSubject?.title ?? "New audit subject"}</dd>
              </div>
              {selectedSubject?.claim_statement ? (
                <div>
                  <dt>Target claim</dt>
                  <dd>{selectedSubject.claim_statement}</dd>
                </div>
              ) : null}
              <div>
                <dt>Type</dt>
                <dd>
                  {selectedSubject
                    ? formatLabel(selectedSubject.subject_type)
                    : "Selected in form"}
                </dd>
              </div>
              <div>
                <dt>Domain</dt>
                <dd>{domainDetail?.name ?? "Not configured"}</dd>
              </div>
              <div>
                <dt>Criteria</dt>
                <dd>{cweNodes.length}</dd>
              </div>
              <div>
                <dt>Registered</dt>
                <dd>
                  {selectedSubject
                    ? formatDate(selectedSubject.registered_at)
                    : "On submit"}
                </dd>
              </div>
            </dl>
          </div>
          <div className="pub-panel">
            <h3>How it works</h3>
            <p className="muted-copy">
              Commissioning creates a sponsor record, an AuditEpisode, and a
              provenance-bearing commission fact. For claim audits, attach
              papers to the episode as evidence artifacts afterwards — they
              are inspected, not counted as votes. Delivery state is tracked
              in the sponsor console; public visibility follows the
              confidentiality setting.
            </p>
          </div>
        </aside>
      </div>
    </>
  );
}

async function commissionAction(formData: FormData) {
  "use server";

  const existingSubjectId = stringField(formData, "subject_id");
  const domainInstantiationId = stringField(formData, "domain_instantiation_id");
  let subjectId = existingSubjectId;

  if (!subjectId) {
    const subject = await createAuditSubject({
      domain_instantiation_id: domainInstantiationId,
      subject_type: stringField(formData, "subject_type") || "scoped_claim",
      title: nullableStringField(formData, "subject_title"),
      claim_statement: nullableStringField(formData, "claim_statement"),
      scope_conditions: parseScopeConditions(
        stringField(formData, "scope_conditions"),
      ),
    });

    subjectId = subject?.id ?? "";
  }

  if (!subjectId) {
    redirect("/commission");
  }

  const amount = Number(stringField(formData, "funding_amount"));
  const result = await commissionAuditEpisode(subjectId, {
    label: stringField(formData, "label"),
    sponsor_organization_name: stringField(formData, "sponsor_organization_name"),
    sponsor_organization_type:
      stringField(formData, "sponsor_organization_type") || "other",
    funding: {
      amount: Number.isFinite(amount) ? amount : 0,
      currency: (stringField(formData, "funding_currency") || "USD").toUpperCase(),
    },
    scope_cwe_node_ids: formData
      .getAll("scope_cwe_node_ids")
      .map((value) => String(value).trim())
      .filter(Boolean),
    deadline: rfc3339Field(formData, "deadline"),
    confidential: formData.get("confidential") === "true",
    notes: nullableStringField(formData, "notes"),
  });

  if (!result) {
    redirect(`/commission?subject_id=${subjectId}`);
  }

  revalidatePath("/");
  revalidatePath("/commission");
  revalidatePath("/sponsor-console");
  revalidatePath("/reviewer-queue");
  revalidatePath("/library");
  revalidatePath("/register");
  revalidatePath(`/claims/${subjectId}`);
  revalidatePath(`/audit-episodes/${result.episode.id}`);

  // Claim audits land on the claim page — the epistemic center — where
  // evidence artifacts can be attached to the new episode.
  if (!existingSubjectId && stringField(formData, "subject_type") === "scoped_claim") {
    redirect(`/claims/${subjectId}`);
  }

  redirect(`/audit-episodes/${result.episode.id}`);
}

function preferredDomain(domains: DomainInstantiationSummary[]) {
  return (
    domains.find((domain) => domain.domain_type === "academic_publishing") ??
    domains[0] ??
    null
  );
}

function stringField(formData: FormData, name: string) {
  return String(formData.get(name) ?? "").trim();
}

/// One condition per line as "label: value".
function parseScopeConditions(raw: string) {
  return raw
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .flatMap((line) => {
      const separator = line.indexOf(":");

      if (separator <= 0) {
        return [];
      }

      const label = line.slice(0, separator).trim();
      const value = line.slice(separator + 1).trim();

      return label && value ? [{ label, value }] : [];
    });
}

function nullableStringField(formData: FormData, name: string) {
  const value = stringField(formData, name);

  return value ? value : null;
}

/// datetime-local values lack a timezone; the API expects RFC3339.
function rfc3339Field(formData: FormData, name: string) {
  const value = stringField(formData, name);

  if (!value) {
    return null;
  }

  const date = new Date(value);

  return Number.isNaN(date.getTime()) ? null : date.toISOString();
}

function formatDate(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}
