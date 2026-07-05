import {
  formatLabel,
  getDomainInstantiations,
  type DomainInstantiationSummary,
} from "../../lib/csqd-api";

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

/// Domains are lenses over one audit substrate, not separate products.
/// Active domains come from the registry; planned ones are shown without
/// pretending to have live workflows.
export default async function DomainsPage() {
  const registryDomains = await getDomainInstantiations();
  const activeDomains =
    registryDomains.length > 0
      ? registryDomains.map(domainCardFromRegistry)
      : [fallbackAcademicDomain()];
  const domainCards = [...activeDomains, ...plannedDomains];

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Domains</p>
          <h1>Epistemic Audit Domains</h1>
          <p>
            Every domain is a configured lens over the same audit substrate —
            shared primitives, domain-specific criteria and evaluation basis.
          </p>
        </div>
      </header>

      <div className="pub-stat-strip">
        <span>
          <strong>{activeDomains.length}</strong> active
        </span>
        <span>
          <strong>{plannedDomains.length}</strong> planned
        </span>
        <span>Shared primitives:</span>
      </div>
      <div className="pub-chip-row" style={{ marginBottom: 20 }}>
        {sharedPrimitives.map((primitive) => (
          <span className="pub-chip" key={primitive}>
            {primitive}
          </span>
        ))}
      </div>

      <div className="pub-domain-grid">
        {domainCards.map((domain) => (
          <article
            className={`pub-domain-card${domain.status === "planned" ? " planned" : ""}`}
            key={domain.id}
          >
            <div className="pub-card-kicker">
              <span>{formatLabel(domain.domainType)}</span>
              <span>{formatLabel(domain.status)}</span>
            </div>
            <h3>{domain.name}</h3>
            <p>{domain.description}</p>
            <dl>
              <div>
                <dt>Audit subjects</dt>
                <dd>{domain.auditSubjects}</dd>
              </div>
              <div>
                <dt>Audit modes</dt>
                <dd>{domain.auditModes}</dd>
              </div>
              <div>
                <dt>Evaluation basis</dt>
                <dd>{domain.evaluationBasis}</dd>
              </div>
              {domain.liveSurfaces ? (
                <div>
                  <dt>Live surfaces</dt>
                  <dd>{domain.liveSurfaces}</dd>
                </div>
              ) : null}
              <div>
                <dt>Registry</dt>
                <dd>{domain.registryName}</dd>
              </div>
            </dl>
          </article>
        ))}
      </div>
    </>
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
      liveSurfaces: "Discover, Audit Reports, Criteria, Register, Method",
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
    liveSurfaces: "Discover, Audit Reports, Criteria, Register, Method",
  };
}
