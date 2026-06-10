import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import { AppSidebar } from "../components/app-sidebar";
import {
  commissionAuditEpisode,
  createAuditSubject,
  formatLabel,
  getAuditSubjects,
  getDomainInstantiation,
  getDomainInstantiations,
  type DomainInstantiationSummary,
} from "../lib/csqd-api";

type PageProps = {
  searchParams: Promise<{
    subject_id?: string;
  }>;
};

const subjectTypes = [
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
    <main className="app-shell">
      <AppSidebar activeItem="commission" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Commissioned audit operations</p>
            <h1>Commission Audit</h1>
          </div>
          <div className="status-pill">
            {selectedSubject ? "Existing subject" : "New or existing subject"}
          </div>
        </header>

        <section className="detail-grid commission-layout">
          <form className="panel element-review-form audit-form" action={commissionAction}>
            <input name="domain_instantiation_id" type="hidden" value={selectedDomainId} />

            <div className="panel-heading">
              <div>
                <p className="eyebrow">Audit subject</p>
                <h2>Subject + Sponsor</h2>
              </div>
              <span className="access-badge">
                {domainDetail ? domainDetail.name : "No domain"}
              </span>
            </div>

            <label>
              <span>Existing subject</span>
              <select defaultValue={selectedSubjectId} name="subject_id">
                <option value="">Create a new audit subject</option>
                {subjects.map((subject) => (
                  <option key={subject.id} value={subject.id}>
                    {subject.title ?? subject.id} - {formatLabel(subject.subject_type)}
                  </option>
                ))}
              </select>
            </label>

            <div className="element-review-form-row">
              <label>
                <span>New subject type</span>
                <select
                  defaultValue={selectedSubject?.subject_type ?? "research_manuscript"}
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
                <span>Subject title</span>
                <input
                  defaultValue={selectedSubject?.title ?? ""}
                  name="subject_title"
                  placeholder="Audit subject title"
                  type="text"
                />
              </label>
            </div>

            <label>
              <span>Episode label</span>
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

            <div className="element-review-form-row">
              <label>
                <span>Sponsor organization</span>
                <input
                  defaultValue="Northstar Bio Diligence"
                  name="sponsor_organization_name"
                  required
                  type="text"
                />
              </label>
              <label>
                <span>Sponsor type</span>
                <select defaultValue="biotech" name="sponsor_organization_type">
                  {organizationTypes.map((organizationType) => (
                    <option key={organizationType} value={organizationType}>
                      {formatLabel(organizationType)}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            <div className="element-review-form-row">
              <label>
                <span>Funding amount</span>
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
                <span>Currency</span>
                <input defaultValue="USD" name="funding_currency" required type="text" />
              </label>
            </div>

            <div className="element-review-form-row">
              <label>
                <span>Deadline</span>
                <input name="deadline" type="datetime-local" />
              </label>
              <label className="checkbox-field">
                <input name="confidential" type="checkbox" value="true" />
                <span>Confidential commission</span>
              </label>
            </div>

            <fieldset className="criteria-fieldset">
              <legend>Audit scope</legend>
              {cweNodes.length === 0 ? (
                <p className="muted-copy">No criteria are configured for this domain.</p>
              ) : (
                <div className="criteria-options">
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
              <span>Notes</span>
              <textarea
                name="notes"
                placeholder="Scope boundaries, sponsor constraints, or delivery notes"
                rows={5}
              />
            </label>

            <button className="primary-action action-button" type="submit">
              Commission audit
            </button>
          </form>

          <aside className="detail-side" aria-label="Selected subject">
            <dl className="detail-facts">
              <div>
                <dt>Subject</dt>
                <dd>{selectedSubject?.title ?? "New audit subject"}</dd>
              </div>
              <div>
                <dt>Subject type</dt>
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
                  {selectedSubject ? formatDate(selectedSubject.registered_at) : "On submit"}
                </dd>
              </div>
            </dl>
          </aside>
        </section>
      </section>
    </main>
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
      subject_type: stringField(formData, "subject_type") || "other",
      title: nullableStringField(formData, "subject_title"),
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
    deadline: nullableStringField(formData, "deadline"),
    confidential: formData.get("confidential") === "true",
    notes: nullableStringField(formData, "notes"),
  });

  if (!result) {
    redirect(`/commission?subject_id=${subjectId}`);
  }

  revalidatePath("/");
  revalidatePath("/commission");
  revalidatePath("/library");
  revalidatePath("/intake");
  revalidatePath(`/audit-episodes/${result.episode.id}`);
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

function nullableStringField(formData: FormData, name: string) {
  const value = stringField(formData, name);

  return value ? value : null;
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
