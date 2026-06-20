import { notFound } from "next/navigation";

import { CrweCoverageMatrix } from "../../../components/crwe-coverage-matrix";
import { FactTimeline } from "../../../components/fact-timeline";
import { GatedAction } from "../../../components/gated-action";
import { ReportReader } from "../../../components/report-reader";
import { StatusPill } from "../../../components/status-pill";
import { TupleBadge } from "../../../components/tuple-badge";
import type { CWENode, Fact, SynthesisReview } from "../../../lib/csqd-api";

// ── Seed-shaped demo data ───────────────────────────────────────

const DEMO_NODES: CWENode[] = [
  {
    id: "node-methodology",
    domain_instantiation_id: "demo-domain",
    parent: null,
    label: "Methodological soundness",
    description: "Design, analysis, and inferential validity.",
    source: "canonical",
  },
  {
    id: "node-reproducibility",
    domain_instantiation_id: "demo-domain",
    parent: null,
    label: "Reproducibility",
    description: "Code, data, and materials availability.",
    source: "canonical",
  },
  {
    id: "node-ethics",
    domain_instantiation_id: "demo-domain",
    parent: null,
    label: "Research ethics",
    description: "Consent, welfare, and disclosure.",
    source: "canonical",
  },
];

function demoFact(
  id: string,
  occurredAt: string,
  payload: Record<string, unknown>,
  status: string = "active",
): Fact {
  return {
    id,
    subject_id: "demo-subject",
    domain_instantiation_id: "demo-domain",
    occurred_at: occurredAt,
    payload,
    status,
    provenance: { recorded_by: { user: "demo-user" } },
    external_refs: [],
  };
}

const DEMO_FACTS: Fact[] = [
  demoFact("fact-commission-1", "2026-01-12T09:00:00Z", {
    audit_commission: { sponsor: "org-foundation" },
  }),
  demoFact("fact-solicitation-1", "2026-01-19T10:30:00Z", {
    er_solicitation: { brief: "Statistical review of the primary endpoint." },
  }),
  demoFact("fact-review-method", "2026-02-02T16:45:00Z", {
    element_review: {
      finding: "non_ethical_problem",
      severity: "major",
      cwe_criterion: { node_id: "node-methodology" },
    },
  }),
  demoFact("fact-review-repro", "2026-02-09T11:15:00Z", {
    element_review: {
      finding: "no_problem_found",
      severity: "none",
      cwe_criterion: { node_id: "node-reproducibility" },
    },
  }),
  demoFact(
    "fact-review-superseded",
    "2026-02-12T08:00:00Z",
    {
      element_review: {
        finding: "no_problem_found",
        severity: "none",
        cwe_criterion: { node_id: "node-methodology" },
      },
    },
    "superseded",
  ),
  demoFact("fact-response-1", "2026-02-20T14:00:00Z", {
    submitter_response: { response_type: "contests" },
  }),
  demoFact("fact-petition-1", "2026-03-01T12:00:00Z", {
    feature_petition: { element_review: "fact-review-method" },
  }),
];

const DEMO_REPORT: SynthesisReview = {
  id: "review-demo-1",
  episode_id: "episode-demo-1",
  submitted_by: "demo-reviewer",
  authored_at: "2026-03-04T09:00:00Z",
  status: "final",
  summary:
    "The primary endpoint analysis is underpowered relative to the claimed effect size; reproducibility materials are complete and well organized.",
  featured: true,
  unsolicited: false,
  sections: [
    {
      id: "section-1",
      review_id: "review-demo-1",
      section_type: "methodological_assessment",
      content:
        "The pre-registered analysis plan was followed, but the power calculation assumed an effect size twice what the pilot data support.",
      referenced_facts: ["fact-review-method"],
    },
    {
      id: "section-2",
      review_id: "review-demo-1",
      section_type: "evidence_integration",
      content:
        "Replication archives include runnable code and raw data; an independent rerun reproduced all main tables.",
      referenced_facts: ["fact-review-repro"],
    },
    {
      id: "section-3",
      review_id: "review-demo-1",
      section_type: "recommendations",
      content:
        "Re-estimate the primary effect with the corrected power assumptions before the result is used for clinical guidance.",
      referenced_facts: [],
    },
  ],
};

const DEMO_TUPLE = {
  problems: 1,
  ethicalConcerns: 0,
  stakes: 2,
  scrutinyDepth: 3.5,
  uptake: 12,
};

const STATUS_LABELS = [
  "Unaudited",
  "Registered for audit",
  "ElementReviews submitted",
  "In synthesis",
  "Audit report available",
  "Challenged",
  "Superseded",
];

// ── Page ────────────────────────────────────────────────────────

/// Dev-only component gallery (F2 exit criterion): every core component
/// rendered against seed-shaped data, with no bespoke markup. 404s in
/// production builds.
export default function ComponentGalleryPage() {
  if (process.env.NODE_ENV === "production") {
    notFound();
  }

  return (
    <section className="workspace">
      <header className="topbar">
        <div>
          <p className="eyebrow">Development</p>
          <h1>Component gallery</h1>
        </div>
      </header>
      <p>
        Core design-system components rendered against seed-shaped data.
        Toggle Advanced mode in the navbar to see symbolic tuple notation.
      </p>

      <article className="panel">
        <h2>TupleBadge</h2>
        <p>Populated, compact, and empty (unaudited) states.</p>
        <TupleBadge tuple={DEMO_TUPLE} />
        <TupleBadge size="compact" tuple={DEMO_TUPLE} />
        <TupleBadge tuple={null} />
      </article>

      <article className="panel">
        <h2>StatusPill</h2>
        <p>One pill per status label; colors come from the token layer.</p>
        <div className="pill-row">
          {STATUS_LABELS.map((status) => (
            <StatusPill key={status} status={status} />
          ))}
        </div>
      </article>

      <article className="panel">
        <h2>CrweCoverageMatrix</h2>
        <p>
          Problems on methodology, clear on reproducibility, ethics
          unreviewed.
        </p>
        <CrweCoverageMatrix
          facts={DEMO_FACTS}
          nodes={DEMO_NODES}
          reviewHrefBase="/scholarly-objects/demo/review"
        />
        <h3>Empty state</h3>
        <CrweCoverageMatrix facts={[]} nodes={[]} />
      </article>

      <article className="panel">
        <h2>FactTimeline</h2>
        <p>
          Interleaved commission, solicitation, reviews (one superseded),
          submitter response, and petition.
        </p>
        <FactTimeline facts={DEMO_FACTS} />
        <h3>Empty state</h3>
        <FactTimeline facts={[]} />
      </article>

      <article className="panel">
        <h2>ReportReader</h2>
        <p>
          Sectioned SynthesisReview with inline fact citations linking into
          the timeline above.
        </p>
        <ReportReader authorName="Demo Reviewer" review={DEMO_REPORT} />
      </article>

      <article className="panel">
        <h2>GatedAction</h2>
        <p>
          Signed out, these route through sign-in with <code>return_to</code>{" "}
          preserved; signed in, they link straight through.
        </p>
        <div className="pill-row">
          <GatedAction
            explain="Sign in to submit an ElementReview"
            href="/scholarly-objects/demo/review"
          >
            Submit ElementReview
          </GatedAction>
          <GatedAction
            className="primary-action"
            explain="Sign in to start a public episode"
            href="/scholarly-objects/demo"
          >
            Start public episode
          </GatedAction>
        </div>
      </article>
    </section>
  );
}
