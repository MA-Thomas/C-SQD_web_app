import { AppSidebar } from "../components/app-sidebar";
import {
  formatLabel,
  getDomainInstantiations,
  type DomainInstantiationSummary,
} from "../lib/csqd-api";

type DomainCard = {
  id: string;
  name: string;
  registryName: string;
  domainType: string;
  status: "active" | "planned";
  description: string;
  auditSubjects: string;
  auditModes: string;
  evaluationBasis: string;
  sharedPrimitives: string;
  liveSurfaces: string | null;
};

const plannedDomains: DomainCard[] = [
  {
    id: "clinical-trial-protocol-review",
    name: "Clinical Trial Protocol Review",
    registryName: "Planned",
    domainType: "clinical_trial_review",
    status: "planned",
    description:
      "Protocol, endpoint, ethics, and causal & statistical review before and during clinical studies.",
    auditSubjects: "Protocols, amendments, endpoints, evidence packets",
    auditModes:
      "Methodological audit, ethics audit, causal & statistical audit, protocol response",
    evaluationBasis:
      "Public health consequentiality, feasibility, risk-benefit, endpoint validity",
    sharedPrimitives:
      "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
    liveSurfaces: null,
  },
  {
    id: "ai-system-auditing",
    name: "AI System Auditing",
    registryName: "Planned",
    domainType: "ai_auditing",
    status: "planned",
    description:
      "Structured review of model behavior, deployment claims, evaluations, and risk controls.",
    auditSubjects: "Model cards, eval reports, deployment claims",
    auditModes:
      "Evaluation audit, risk synthesis, red-team response, mitigation follow-up",
    evaluationBasis:
      "Deployment risk, downstream uptake, evidence quality, causal & statistical validity",
    sharedPrimitives:
      "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
    liveSurfaces: null,
  },
  {
    id: "policy-evidence-review",
    name: "Policy Evidence Review",
    registryName: "Planned",
    domainType: "policy_review",
    status: "planned",
    description:
      "Audit trails for evidence packages, policy claims, implementation assumptions, and dissent.",
    auditSubjects: "Evidence packets, policy briefs, claims, responses",
    auditModes: "Evidence integration, claim audit, response, synthesis review",
    evaluationBasis:
      "Downstream adoption, causal & statistical adequacy, implementation assumptions",
    sharedPrimitives:
      "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
    liveSurfaces: null,
  },
];

const sharedPrimitives = [
  "Audit subjects",
  "Facts",
  "Audit episodes",
  "Episode memberships",
  "Synthesis reviews",
  "Evaluation tuples",
  "Solicitations",
];

export default async function DomainsPage() {
  const registryDomains = await getDomainInstantiations();
  const activeDomains =
    registryDomains.length > 0
      ? registryDomains.map(domainCardFromRegistry)
      : [fallbackAcademicDomain()];
  const domainCards = [...activeDomains, ...plannedDomains];

  return (
    <main className="app-shell">
      <AppSidebar activeItem="domains" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">C-SQD domains</p>
            <h1>Epistemic Audit Domains</h1>
          </div>
          <div className="status-pill">
            {activeDomains.length} active + {plannedDomains.length} planned
          </div>
        </header>

        <section className="metric-grid" aria-label="Domain metrics">
          <div className="metric">
            <span>Active domains</span>
            <strong>{activeDomains.length}</strong>
          </div>
          <div className="metric">
            <span>Planned domains</span>
            <strong>{plannedDomains.length}</strong>
          </div>
          <div className="metric">
            <span>Shared primitives</span>
            <strong>{sharedPrimitives.length}</strong>
          </div>
        </section>

        <section className="domain-substrate" aria-label="Shared C-SQD primitives">
          {sharedPrimitives.map((primitive) => (
            <span key={primitive}>{primitive}</span>
          ))}
        </section>

        <section className="domain-grid" aria-label="C-SQD domain list">
          {domainCards.map((domain) => (
            <article
              className={`domain-card domain-card-${domain.status}`}
              key={domain.id}
            >
              <div className="domain-card-main">
                <div className="object-kicker">
                  <span>{formatLabel(domain.domainType)}</span>
                  <span>{formatLabel(domain.status)}</span>
                </div>
                <h2>{domain.name}</h2>
                <p>{domain.description}</p>
                <dl className="domain-concept-list">
                  <div>
                    <dt>Audit modes</dt>
                    <dd>{domain.auditModes}</dd>
                  </div>
                  <div>
                    <dt>Shared C-SQD primitives</dt>
                    <dd>{domain.sharedPrimitives}</dd>
                  </div>
                  {domain.liveSurfaces ? (
                    <div>
                      <dt>Live surfaces</dt>
                      <dd>{domain.liveSurfaces}</dd>
                    </div>
                  ) : null}
                </dl>
              </div>
              <dl className="domain-facts">
                <div>
                  <dt>Registry</dt>
                  <dd>{domain.registryName}</dd>
                </div>
                <div>
                  <dt>Audit subjects</dt>
                  <dd>{domain.auditSubjects}</dd>
                </div>
                <div>
                  <dt>Status</dt>
                  <dd>{formatLabel(domain.status)}</dd>
                </div>
                <div>
                  <dt>Evaluation basis</dt>
                  <dd>{domain.evaluationBasis}</dd>
                </div>
              </dl>
            </article>
          ))}
        </section>
      </section>
    </main>
  );
}

function domainCardFromRegistry(domain: DomainInstantiationSummary): DomainCard {
  if (domain.domain_type === "academic_publishing") {
    return {
      id: domain.id,
      name: "Academic Peer Review",
      registryName: domain.name,
      domainType: domain.domain_type,
      status: "active",
      description:
        "Scholarly works and preprints enter as audit subjects; facts, episodes, and synthesis reviews hold the audit trail.",
      auditSubjects: "Articles, preprints, datasets, software, protocols, reports",
      auditModes: "Element review facts, synthesis review, submitter response",
      evaluationBasis:
        "Methodological adequacy, causal & statistical adequacy, interpretation strength",
      sharedPrimitives:
        "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
      liveSurfaces: "Audit Console, Commission Audit, Library, Scholarly Intake",
    };
  }

  return {
    id: domain.id,
    name: formatLabel(domain.domain_type),
    registryName: domain.name,
    domainType: domain.domain_type,
    status: "active",
    description:
      "Configured C-SQD domain with shared audit subjects, facts, audit episodes, and criteria.",
    auditSubjects: "Configured by domain schema",
    auditModes: "Configured by domain schema",
    evaluationBasis: "Configured evaluation tuple",
    sharedPrimitives:
      "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
    liveSurfaces: null,
  };
}

function fallbackAcademicDomain(): DomainCard {
  return {
    id: "academic-peer-review-fallback",
    name: "Academic Peer Review",
    registryName: "Academic Publishing Demo",
    domainType: "academic_publishing",
    status: "active",
    description:
      "Scholarly works and preprints enter as audit subjects; facts, episodes, and synthesis reviews hold the audit trail.",
    auditSubjects: "Articles, preprints, datasets, software, protocols, reports",
    auditModes: "Element review facts, synthesis review, submitter response",
    evaluationBasis:
      "Methodological adequacy, causal & statistical adequacy, interpretation strength",
    sharedPrimitives:
      "Audit subjects, facts, audit episodes, memberships, synthesis, evaluation tuples",
    liveSurfaces: "Audit Console, Commission Audit, Library, Scholarly Intake",
  };
}
