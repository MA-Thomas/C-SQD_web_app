import Link from "next/link";

const methodSections = [
  {
    id: "audit-subjects",
    title: "Audit Subjects",
    copy: "An audit subject is what the audit is epistemically about. The default subject is a scoped claim: a bounded assertion with explicit scope conditions — population, intervention, measurement, outcome. Papers, preprints, datasets, code, protocols, and reports can also be audited directly, and in a single-paper audit the target claim sits close to a manuscript claim. Academic metadata supports the subject, but audit actions attach to the subject record.",
  },
  {
    id: "evidence-artifacts",
    title: "Evidence Artifacts",
    copy: "Papers attached to a claim audit are evidence artifacts to be examined, not votes to be counted. Publication does not convert a paper's claims into validated evidence: each artifact's measurements, assumptions, methods, and relevance to the target claim still need inspection. Attachment is neutral — an artifact's bearing on the claim is derived from audited warrants, never assumed.",
  },
  {
    id: "warrants",
    title: "Warrant Links",
    copy: "A warrant link records why an attached artifact is supposed to bear on the target claim: the claim the artifact actually makes, the inference type connecting it to the target (statistical, causal, mechanistic, external validity), and the assumptions that inference requires. The central audit question is not how many papers appear to favor the claim, but which warrant links survive scrutiny.",
  },
  {
    id: "elementreviews",
    title: "ElementReviews",
    copy: "An ElementReview is a focused review of one criterion or research weakness. It can be commissioned, assigned, unsolicited, challenged, cited, and synthesized.",
  },
  {
    id: "synthesisreviews",
    title: "SynthesisReviews",
    copy: "A SynthesisReview is an integrative audit report. It combines ElementReviews and other facts into a higher-level interpretation with summary, findings, recommendations, open questions, and cited review facts.",
  },
  {
    id: "crwe",
    title: "CRWE",
    copy: "CRWE means Common Research Weakness Enumeration. For Academic Peer Review, it is the public taxonomy for review criteria and problem areas, so reviews attach to explicit criteria instead of vague overall impressions. Criteria are configured per domain: other audit domains define their own criterion sets.",
  },
  {
    id: "evaluation-tuple",
    title: "Claim Audit Tuple",
    copy: "The public tuple summarizes the audit state of the claim under audit — not how good a manuscript is, and never a count of supporting publications. Friendly labels: Problems, Ethical concerns, Stakes, Scrutiny depth, and Uptake. It is a derived view over the immutable audit record — recomputable for any reviewer community and reference time. The expert notation is E(A | R, T_eval) -> (N, M, S, L, U), anchored on the claim under audit.",
  },
  {
    id: "challenges",
    title: "Challenges",
    copy: "Challenges contest an ElementReview or SynthesisReview without erasing the record. The challenged artifact remains visible while a provenance-bearing contestation, response, or supersession is added.",
  },
  {
    id: "public-private",
    title: "Public And Private Audits",
    copy: "Public audits are readable without login and contribute to public summaries. Private audits are visible only to authorized sponsors, reviewers, and operators, and may later publish a public report or subset.",
  },
];

const crweExamples = [
  "Methodological adequacy",
  "Statistical adequacy",
  "Data and code availability",
  "Interpretation strength",
  "Ethical concern",
  "Reproducibility",
  "Evidence quality",
  "External validity",
];

export default function MethodPage() {
  return (
    <div className="pub-method">
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">C-SQD Method</p>
          <h1>How Public Epistemic Audits Work</h1>
          <p>
            C-SQD turns audit activity into durable public artifacts: subjects,
            criterion-level reviews, synthesis reports, evaluation summaries,
            and challenge trails.
          </p>
        </div>
        <Link className="secondary-action" href="/discover">
          Explore the artifacts
        </Link>
      </header>

      <nav className="pub-method-toc" aria-label="Method sections">
        {methodSections.map((section) => (
          <a href={`#${section.id}`} key={section.id}>
            {section.title}
          </a>
        ))}
      </nav>

      {methodSections.map((section) => (
        <article id={section.id} key={section.id}>
          <p className="pub-kicker">Method</p>
          <h2>{section.title}</h2>
          <p>{section.copy}</p>
          {section.id === "crwe" ? (
            <div className="pub-chip-row">
              {crweExamples.map((example) => (
                <span className="pub-chip" key={example}>
                  {example}
                </span>
              ))}
            </div>
          ) : null}
          {section.id === "evaluation-tuple" ? (
            <dl className="pub-def-list">
              <div>
                <dt>Problems</dt>
                <dd>
                  Audited non-ethical problems in the claim&apos;s warrants,
                  surfaced by ElementReviews.
                </dd>
              </div>
              <div>
                <dt>Ethical concerns</dt>
                <dd>Ethical problems surfaced by ElementReviews.</dd>
              </div>
              <div>
                <dt>Stakes</dt>
                <dd>How consequential the target claim is for the domain.</dd>
              </div>
              <div>
                <dt>Scrutiny depth</dt>
                <dd>The amount and weight of focused review activity.</dd>
              </div>
              <div>
                <dt>Uptake</dt>
                <dd>How much the audit record has been synthesized or used.</dd>
              </div>
            </dl>
          ) : null}
        </article>
      ))}
    </div>
  );
}
