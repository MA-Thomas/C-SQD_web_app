import Link from "next/link";
import { revalidatePath } from "next/cache";
import { notFound, redirect } from "next/navigation";

import { AppSidebar } from "../../components/app-sidebar";
import {
  createEpisodeElementReview,
  createEpisodeSolicitation,
  createEpisodeSolicitationEvent,
  createSynthesisReview,
  formatLabel,
  getAuditEpisode,
  getAuditSubject,
  getDomainInstantiation,
  getEvalTuple,
  getFactsForEpisode,
  getSynthesisReviews,
  type CWENode,
  type EvalTuple,
  type Fact,
  type SynthesisReview,
} from "../../lib/csqd-api";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

const findings = [
  "inconclusive",
  "no_problems",
  "non_ethical_problem",
  "ethical_problem",
];

const severities = ["minor", "moderate", "major", "critical"];
const confidenceLevels = ["low", "moderate", "high"];
const solicitationEventTypes = ["accepted", "declined", "expired", "completed"];
const paymentConditions = ["on_submission", "on_acceptance"];
const synthesisSectionTypes = [
  "summary",
  "methodological_assessment",
  "ethical_assessment",
  "evidence_integration",
  "recommendations",
  "open_questions",
];

export default async function AuditEpisodePage({ params }: PageProps) {
  const { id } = await params;
  const episode = await getAuditEpisode(id);

  if (!episode) {
    notFound();
  }

  const [subject, facts, domain, synthesisReviews, evalTuple] = await Promise.all([
    getAuditSubject(episode.subject_id),
    getFactsForEpisode(episode.id),
    getDomainInstantiation(episode.domain_instantiation_id),
    getSynthesisReviews(episode.id),
    getEvalTuple(episode.id),
  ]);

  if (!subject || !domain) {
    notFound();
  }

  const commissionFact = facts.find((fact) => factKind(fact) === "audit_commission");
  const elementReviewFacts = facts.filter(
    (fact) => factKind(fact) === "element_review",
  );
  const solicitationFacts = facts.filter((fact) => factKind(fact) === "er_solicitation");
  const solicitationEventFacts = facts.filter(
    (fact) => factKind(fact) === "solicitation_event",
  );
  const commission = commissionPayload(commissionFact);
  const scopeNodeIds =
    commission?.scope?.map((criterion) => criterion.node_id).filter(Boolean) ?? [];
  const scopedNodes =
    scopeNodeIds.length > 0
      ? domain.cwe_nodes.filter((node) => scopeNodeIds.includes(node.id))
      : domain.cwe_nodes;
  const selectableNodes = scopedNodes.length > 0 ? scopedNodes : domain.cwe_nodes;

  return (
    <main className="app-shell">
      <AppSidebar activeItem="console" />

      <section className="workspace">
        <header className="topbar detail-topbar">
          <div>
            <p className="eyebrow">Audit episode</p>
            <h1>{episode.label}</h1>
          </div>
          <Link className="status-pill" href="/">
            Back to console
          </Link>
        </header>

        <section className="detail-grid">
          <article className="detail-primary">
            <div className="object-kicker">
              <span>{formatLabel(episode.status)}</span>
              <span>{formatLabel(subject.subject_type)}</span>
              <span>{domain.name}</span>
            </div>
            <h2>{subject.title ?? "Untitled audit subject"}</h2>
            {episode.notes ? <p className="abstract-text">{episode.notes}</p> : null}
            <div className="source-actions">
              <Link
                className="secondary-action"
                href={`/commission?subject_id=${episode.subject_id}`}
              >
                Commission related audit
              </Link>
            </div>
          </article>

          <aside className="detail-side" aria-label="Episode facts">
            <dl className="detail-facts">
              <div>
                <dt>Status</dt>
                <dd>{formatLabel(episode.status)}</dd>
              </div>
              <div>
                <dt>Funding</dt>
                <dd>{formatMoney(commission?.funding)}</dd>
              </div>
              <div>
                <dt>Assignments</dt>
                <dd>{solicitationFacts.length}</dd>
              </div>
              <div>
                <dt>Element reviews</dt>
                <dd>{elementReviewFacts.length}</dd>
              </div>
              <div>
                <dt>Synthesis</dt>
                <dd>{synthesisReviews.length > 0 ? "Current" : "Pending"}</dd>
              </div>
            </dl>
          </aside>
        </section>

        <WorkflowStrip
          elementReviewCount={elementReviewFacts.length}
          solicitationCount={solicitationFacts.length}
          synthesisCount={synthesisReviews.length}
        />

        <section className="metric-grid tuple-grid" aria-label="Evaluation tuple">
          <TupleMetric label="N" title="Non-ethical problems" value={evalTuple?.n} />
          <TupleMetric label="M" title="Ethical problems" value={evalTuple?.m} />
          <TupleMetric label="S" title="Stakes" value={evalTuple?.s} />
          <TupleMetric label="L" title="Scrutiny depth" value={evalTuple?.l} />
          <TupleMetric label="U" title="Uptake" value={evalTuple?.u} />
        </section>

        <section className="detail-panels workflow-panels">
          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Reviewer assignment</p>
                <h2>Issue Solicitation</h2>
              </div>
              <span className="access-badge">{solicitationFacts.length}</span>
            </div>

            <form className="element-review-form" action={createSolicitationAction}>
              <input name="episode_id" type="hidden" value={episode.id} />
              <input
                name="commission_fact_id"
                type="hidden"
                value={commissionFact?.id ?? ""}
              />

              <label>
                <span>Criterion</span>
                <select name="cwe_node_id" required>
                  {selectableNodes.map((node) => (
                    <option key={node.id} value={node.id}>
                      {node.label}
                    </option>
                  ))}
                </select>
              </label>

              <div className="element-review-form-row">
                <label>
                  <span>Payment amount</span>
                  <input
                    defaultValue="500"
                    min="1"
                    name="payment_amount"
                    required
                    step="0.01"
                    type="number"
                  />
                </label>
                <label>
                  <span>Currency</span>
                  <input defaultValue="USD" name="payment_currency" required type="text" />
                </label>
              </div>

              <label>
                <span>Payment condition</span>
                <select defaultValue="on_submission" name="payment_condition">
                  {paymentConditions.map((condition) => (
                    <option key={condition} value={condition}>
                      {formatLabel(condition)}
                    </option>
                  ))}
                </select>
              </label>

              <button className="primary-action action-button" type="submit">
                Issue solicitation
              </button>
            </form>
          </article>

          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Lifecycle</p>
                <h2>Update Solicitation</h2>
              </div>
              <span className="access-badge">{solicitationEventFacts.length}</span>
            </div>

            <form className="element-review-form" action={createSolicitationEventAction}>
              <input name="episode_id" type="hidden" value={episode.id} />
              <label>
                <span>Solicitation</span>
                <select name="solicitation_fact_id" required>
                  {solicitationFacts.map((fact) => (
                    <option key={fact.id} value={fact.id}>
                      {solicitationLabel(fact, domain.cwe_nodes)}
                    </option>
                  ))}
                </select>
              </label>

              <label>
                <span>Event</span>
                <select defaultValue="accepted" name="event_type">
                  {solicitationEventTypes.map((eventType) => (
                    <option key={eventType} value={eventType}>
                      {formatLabel(eventType)}
                    </option>
                  ))}
                </select>
              </label>

              <label>
                <span>Note</span>
                <textarea name="note" rows={3} />
              </label>

              <button
                className="secondary-action action-button"
                disabled={solicitationFacts.length === 0}
                type="submit"
              >
                Record event
              </button>
            </form>
          </article>
        </section>

        <section className="detail-panels workflow-panels">
          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Fact intake</p>
                <h2>Record Element Review</h2>
              </div>
              <span className="access-badge">{elementReviewFacts.length}</span>
            </div>

            <form className="element-review-form" action={createElementReviewAction}>
              <input name="episode_id" type="hidden" value={episode.id} />
              <label>
                <span>Solicitation</span>
                <select name="solicitation">
                  <option value="">Unsolicited review</option>
                  {solicitationFacts.map((fact) => (
                    <option key={fact.id} value={fact.id}>
                      {solicitationLabel(fact, domain.cwe_nodes)}
                    </option>
                  ))}
                </select>
              </label>

              <label>
                <span>Criterion</span>
                <select name="cwe_node_id" required>
                  {selectableNodes.map((node) => (
                    <option key={node.id} value={node.id}>
                      {node.label}
                    </option>
                  ))}
                </select>
              </label>

              <div className="element-review-form-row">
                <label>
                  <span>Finding</span>
                  <select defaultValue="inconclusive" name="finding">
                    {findings.map((finding) => (
                      <option key={finding} value={finding}>
                        {formatLabel(finding)}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>Severity</span>
                  <select defaultValue="" name="severity">
                    <option value="">Unspecified</option>
                    {severities.map((severity) => (
                      <option key={severity} value={severity}>
                        {formatLabel(severity)}
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <div className="element-review-form-row">
                <label>
                  <span>Confidence</span>
                  <select defaultValue="" name="confidence">
                    <option value="">Unspecified</option>
                    {confidenceLevels.map((confidence) => (
                      <option key={confidence} value={confidence}>
                        {formatLabel(confidence)}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="checkbox-field">
                  <input name="featured" type="checkbox" value="true" />
                  <span>Featured in synthesis queue</span>
                </label>
              </div>

              <label>
                <span>Finding summary</span>
                <input
                  name="content"
                  placeholder="Core evaluative claim"
                  required
                  type="text"
                />
              </label>

              <label>
                <span>Limitations</span>
                <textarea name="limitations" rows={3} />
              </label>

              <label>
                <span>Recommendations</span>
                <textarea name="recommendations" rows={3} />
              </label>

              <button className="primary-action action-button" type="submit">
                Record element review
              </button>
            </form>
          </article>

          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Narrative</p>
                <h2>Create Synthesis</h2>
              </div>
              <span className="access-badge">{synthesisReviews.length}</span>
            </div>

            <form className="element-review-form" action={createSynthesisReviewAction}>
              <input name="episode_id" type="hidden" value={episode.id} />
              <label>
                <span>Status</span>
                <select defaultValue="current" name="status">
                  <option value="draft">Draft</option>
                  <option value="current">Current</option>
                </select>
              </label>
              <label>
                <span>Summary</span>
                <textarea name="summary" required rows={4} />
              </label>
              <label>
                <span>Section type</span>
                <select defaultValue="evidence_integration" name="section_type">
                  {synthesisSectionTypes.map((sectionType) => (
                    <option key={sectionType} value={sectionType}>
                      {formatLabel(sectionType)}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Section content</span>
                <textarea name="section_content" rows={4} />
              </label>
              <fieldset className="criteria-fieldset">
                <legend>Referenced facts</legend>
                <div className="criteria-options compact-options">
                  {elementReviewFacts.length === 0 ? (
                    <p className="muted-copy">No element reviews available.</p>
                  ) : (
                    elementReviewFacts.map((fact) => (
                      <label key={fact.id}>
                        <input name="referenced_facts" type="checkbox" value={fact.id} />
                        <span>
                          <strong>{factTitle("element_review", payloadRecord(fact, "element_review"), undefined)}</strong>
                          <small>{factDescription("element_review", payloadRecord(fact, "element_review"))}</small>
                        </span>
                      </label>
                    ))
                  )}
                </div>
              </fieldset>
              <label className="checkbox-field">
                <input defaultChecked name="featured" type="checkbox" value="true" />
                <span>Featured synthesis</span>
              </label>

              <button
                className="primary-action action-button"
                disabled={elementReviewFacts.length === 0}
                type="submit"
              >
                Create synthesis
              </button>
            </form>
          </article>
        </section>

        <section className="detail-panels audit-console-secondary">
          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Commission scope</p>
                <h2>Criteria</h2>
              </div>
              <span className="access-badge">{scopedNodes.length}</span>
            </div>
            <div className="version-context-list">
              {scopedNodes.map((node) => (
                <CriterionRow key={node.id} node={node} />
              ))}
            </div>
          </article>
        </section>

        <section className="detail-panels workflow-panels">
          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Synthesis reviews</p>
                <h2>Interpretations</h2>
              </div>
              <span className="access-badge">{synthesisReviews.length}</span>
            </div>
            <div className="version-context-list fact-list">
              {synthesisReviews.length === 0 ? (
                <p className="muted-copy">No synthesis reviews have been authored yet.</p>
              ) : (
                synthesisReviews.map((review) => (
                  <SynthesisReviewRow review={review} key={review.id} />
                ))
              )}
            </div>
          </article>

          <article className="panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Episode facts</p>
                <h2>Timeline</h2>
              </div>
              <span className="access-badge">{facts.length}</span>
            </div>
            <div className="version-context-list fact-list">
              {facts.length === 0 ? (
                <p className="muted-copy">No facts have been recorded yet.</p>
              ) : (
                facts.map((fact) => (
                  <FactRow fact={fact} key={fact.id} nodes={domain.cwe_nodes} />
                ))
              )}
            </div>
          </article>
        </section>
      </section>
    </main>
  );
}

async function createSolicitationAction(formData: FormData) {
  "use server";

  const episodeId = stringField(formData, "episode_id");

  if (!episodeId) {
    redirect("/");
  }

  const amount = Number(stringField(formData, "payment_amount"));
  const currency = (stringField(formData, "payment_currency") || "USD").toUpperCase();

  await createEpisodeSolicitation(episodeId, {
    issued_to: null,
    cwe_node_id: stringField(formData, "cwe_node_id"),
    commission_fact_id: nullableStringField(formData, "commission_fact_id"),
    payment_scheme: {
      amount: {
        amount: Number.isFinite(amount) ? amount : 0,
        currency,
      },
      currency,
      condition: stringField(formData, "payment_condition") || "on_submission",
    },
  });

  revalidatePath("/");
  revalidatePath(`/audit-episodes/${episodeId}`);
  redirect(`/audit-episodes/${episodeId}`);
}

async function createSolicitationEventAction(formData: FormData) {
  "use server";

  const episodeId = stringField(formData, "episode_id");

  if (!episodeId) {
    redirect("/");
  }

  await createEpisodeSolicitationEvent(episodeId, {
    solicitation_fact_id: stringField(formData, "solicitation_fact_id"),
    event_type: stringField(formData, "event_type") || "accepted",
    principal: null,
    note: nullableStringField(formData, "note"),
  });

  revalidatePath("/");
  revalidatePath(`/audit-episodes/${episodeId}`);
  redirect(`/audit-episodes/${episodeId}`);
}

async function createElementReviewAction(formData: FormData) {
  "use server";

  const episodeId = stringField(formData, "episode_id");

  if (!episodeId) {
    redirect("/");
  }

  await createEpisodeElementReview(episodeId, {
    cwe_node_id: stringField(formData, "cwe_node_id"),
    submitted_by: null,
    solicitation: nullableStringField(formData, "solicitation"),
    finding: stringField(formData, "finding") || "inconclusive",
    severity: nullableStringField(formData, "severity"),
    confidence: nullableStringField(formData, "confidence"),
    limitations: nullableStringField(formData, "limitations"),
    recommendations: nullableStringField(formData, "recommendations"),
    content: stringField(formData, "content"),
    featured: formData.get("featured") === "true",
  });

  revalidatePath("/");
  revalidatePath(`/audit-episodes/${episodeId}`);
  redirect(`/audit-episodes/${episodeId}`);
}

async function createSynthesisReviewAction(formData: FormData) {
  "use server";

  const episodeId = stringField(formData, "episode_id");

  if (!episodeId) {
    redirect("/");
  }

  const sectionContent = nullableStringField(formData, "section_content");
  const referencedFacts = formData
    .getAll("referenced_facts")
    .map((value) => String(value).trim())
    .filter(Boolean);

  await createSynthesisReview(episodeId, {
    submitted_by: null,
    status: stringField(formData, "status") || "current",
    summary: stringField(formData, "summary"),
    sections: sectionContent
      ? [
          {
            section_type: stringField(formData, "section_type") || "evidence_integration",
            content: sectionContent,
            referenced_facts: referencedFacts,
          },
        ]
      : [],
    featured: formData.get("featured") === "true",
  });

  revalidatePath("/");
  revalidatePath(`/audit-episodes/${episodeId}`);
  redirect(`/audit-episodes/${episodeId}`);
}

function WorkflowStrip({
  elementReviewCount,
  solicitationCount,
  synthesisCount,
}: {
  elementReviewCount: number;
  solicitationCount: number;
  synthesisCount: number;
}) {
  const steps = [
    { label: "Commissioned", active: true },
    { label: "Assigned", active: solicitationCount > 0 },
    { label: "Reviewed", active: elementReviewCount > 0 },
    { label: "Synthesized", active: synthesisCount > 0 },
  ];

  return (
    <section className="workflow-strip" aria-label="Audit workflow">
      {steps.map((step) => (
        <div className={step.active ? "active" : ""} key={step.label}>
          <span>{step.label}</span>
        </div>
      ))}
    </section>
  );
}

function TupleMetric({
  label,
  title,
  value,
}: {
  label: string;
  title: string;
  value: number | undefined;
}) {
  return (
    <div className="metric tuple-metric">
      <span>
        {label} - {title}
      </span>
      <strong>{typeof value === "number" ? formatTupleValue(value) : "0"}</strong>
    </div>
  );
}

function CriterionRow({ node }: { node: CWENode }) {
  return (
    <div className="version-context-row">
      <div>
        <strong>{node.label}</strong>
        <span>{node.description}</span>
      </div>
      <span>{formatLabel(node.source)}</span>
    </div>
  );
}

function SynthesisReviewRow({ review }: { review: SynthesisReview }) {
  return (
    <div className="version-context-row fact-row">
      <div>
        <strong>{formatLabel(review.status)} synthesis</strong>
        <span>{review.summary}</span>
        {review.sections.map((section) => (
          <small className="context-small" key={section.id}>
            {formatLabel(section.section_type)}: {section.content}
          </small>
        ))}
      </div>
      <span>{formatDate(review.authored_at)}</span>
    </div>
  );
}

function FactRow({ fact, nodes }: { fact: Fact; nodes: CWENode[] }) {
  const kind = factKind(fact);
  const payload = payloadRecord(fact, kind);
  const nodeId = criterionNodeId(payload);
  const node = nodes.find((candidate) => candidate.id === nodeId);

  return (
    <div className="version-context-row fact-row">
      <div>
        <strong>{factTitle(kind, payload, node)}</strong>
        <span>{factDescription(kind, payload)}</span>
      </div>
      <span>{formatDate(fact.occurred_at)}</span>
    </div>
  );
}

function factTitle(
  kind: string,
  payload: Record<string, unknown> | null,
  node: CWENode | undefined,
) {
  if (kind === "element_review") {
    return `${node?.label ?? "Element review"} - ${formatLabel(stringValue(payload?.finding) || "inconclusive")}`;
  }

  if (kind === "audit_commission") {
    return "Audit commission";
  }

  if (kind === "er_solicitation") {
    return `${node?.label ?? "Solicitation"} assignment`;
  }

  if (kind === "solicitation_event") {
    return `${formatLabel(stringValue(payload?.event_type) || "event")} solicitation`;
  }

  return formatLabel(kind || "fact");
}

function factDescription(kind: string, payload: Record<string, unknown> | null) {
  if (kind === "element_review") {
    return stringValue(payload?.content) || "Element review fact";
  }

  if (kind === "audit_commission") {
    const scope = arrayValue(payload?.scope).length;

    return `${scope} scoped ${scope === 1 ? "criterion" : "criteria"}`;
  }

  if (kind === "er_solicitation") {
    const payment = payload?.payment_scheme;

    return formatPayment(payment);
  }

  if (kind === "solicitation_event") {
    return stringValue(payload?.note) || "Lifecycle event";
  }

  return "Recorded fact";
}

function solicitationLabel(fact: Fact, nodes: CWENode[]) {
  const payload = payloadRecord(fact, "er_solicitation");
  const node = nodes.find((candidate) => candidate.id === criterionNodeId(payload));

  return `${node?.label ?? "Solicitation"} - ${formatPayment(payload?.payment_scheme)}`;
}

function factKind(fact: Fact) {
  if (!fact.payload || typeof fact.payload !== "object") {
    return "fact";
  }

  const [kind] = Object.keys(fact.payload as Record<string, unknown>);

  return kind ?? "fact";
}

function payloadRecord(fact: Fact, kind: string) {
  if (!fact.payload || typeof fact.payload !== "object") {
    return null;
  }

  const payload = (fact.payload as Record<string, unknown>)[kind];

  return payload && typeof payload === "object"
    ? (payload as Record<string, unknown>)
    : null;
}

function commissionPayload(fact: Fact | undefined) {
  if (!fact) {
    return null;
  }

  return payloadRecord(fact, "audit_commission") as {
    scope?: Array<{ node_id?: string }>;
    funding?: { amount?: number; currency?: string };
    deadline?: string | null;
  } | null;
}

function criterionNodeId(payload: Record<string, unknown> | null) {
  const criterion = payload?.cwe_criterion;

  if (!criterion || typeof criterion !== "object") {
    return "";
  }

  return stringValue((criterion as Record<string, unknown>).node_id);
}

function arrayValue(value: unknown) {
  return Array.isArray(value) ? value : [];
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value : "";
}

function formatPayment(value: unknown) {
  if (!value || typeof value !== "object") {
    return "Unspecified payment";
  }

  const payment = value as {
    amount?: { amount?: number; currency?: string };
    currency?: string;
  };

  return formatMoney(payment.amount ?? { currency: payment.currency });
}

function formatMoney(value: { amount?: number; currency?: string } | undefined) {
  if (!value || typeof value.amount !== "number") {
    return "Unspecified";
  }

  return `${value.amount.toLocaleString("en", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  })} ${value.currency ?? "USD"}`;
}

function formatTupleValue(value: number) {
  return value.toLocaleString("en", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  });
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

function stringField(formData: FormData, name: string) {
  return String(formData.get(name) ?? "").trim();
}

function nullableStringField(formData: FormData, name: string) {
  const value = stringField(formData, name);

  return value ? value : null;
}
