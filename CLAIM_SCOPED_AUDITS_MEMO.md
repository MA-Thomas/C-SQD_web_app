# Claim-Scoped Audits In C-SQD

## Thesis

C-SQD audits scoped claims by inspecting their warrants. Attached papers are evidence artifacts to be examined, not votes to be counted.

In the Academic Peer Review MVP, this means the default audit object should not be "a paper" or "the literature on X." The audit object should be a bounded claim, with explicit scope conditions and a traceable relation to the artifacts used to evaluate it.

## The Drift Risk

A platform centered on scholarly works can easily slide into a familiar but weaker question:

> How much does the literature support claim X?

That framing is dangerous for C-SQD. It encourages auditors to treat each paper's conclusion as if it had already been epistemically verified, then aggregate those conclusions into a higher-level judgment. The result is a literature summary, not an audit.

C-SQD should resist that move. Publication does not convert a paper's claims into validated evidence tokens. A paper may contain useful evidence, but its measurements, assumptions, methods, causal moves, statistical inferences, and relevance to the target claim still need inspection.

## The Correct Unit

The audit object is the scoped target claim.

That claim should be stated with enough precision that reviewers can ask what would count as support, challenge, limitation, or non-applicability. A good audit subject should therefore identify:

- the target claim
- the scope conditions under which the claim is being evaluated
- the evidence artifacts attached to the audit
- the artifact-level claims that appear to bear on the target claim
- the warrant links explaining why those artifacts are supposed to matter

The central audit question is not whether many papers appear to favor the claim. It is whether the relevant warrant links survive scrutiny.

## Contrast

| Weak frame | C-SQD frame |
| --- | --- |
| How much does the literature support X? | Which warrants for X survive audit? |
| Papers are counted as evidence units. | Papers are inspected as evidence artifacts. |
| Paper conclusions are treated as inputs. | Paper conclusions are themselves auditable claims. |
| Aggregation happens over publications. | Evaluation happens over claim-warrant-evidence relations. |
| The output is a literature-level impression. | The output is an audited status of a scoped claim. |

## How Papers Function

Papers remain important, but their role is subordinate to the claim under audit.

A paper is a structured container of assertions, measurements, methods, data, models, assumptions, limitations, and conclusions. It may support the target claim, challenge it, narrow it, fail to bear on it, or support only a weaker proxy claim.

Auditors should therefore ask:

- What claim does this artifact actually make?
- What data, measurement, or method supports that claim?
- Which assumptions are required for the artifact claim to bear on the target claim?
- Is the inference statistical, causal, mechanistic, external-validity-based, or something else?
- Does the artifact support the target claim directly, or only a nearby proxy?
- Which parts of the artifact survive audit, and which should receive little or no weight?

This preserves the skeptical core of C-SQD: papers do not transfer credibility wholesale. Their evidentiary contribution must be earned through audited warrants.

## Paper Discoverability Without Paper-Centered Audits

This does not mean papers should disappear from the product model.

Papers should remain searchable, citable, and linkable evidence artifacts. A user should be able to open a paper page and see every audit that involves that paper: audits that use it as evidence, challenge it, inspect a claim made within it, or evaluate a broader target claim for which it is relevant.

The distinction is conceptual, not navigational. A paper can be a first-class discovery surface without being the audit's epistemic target.

## Product Implications

For the Academic Peer Review MVP:

- `AuditSubject` should usually be a scoped claim or claim-warrant bundle.
- `ScholarlyObject` should identify and describe papers or versions that may be attached to audits.
- Evidence artifacts should be linked to audits without being treated as already verified.
- `ElementReview` should evaluate criteria, warrant links, artifact-level claims, methods, assumptions, or relevance.
- `SynthesisReview` should report the audited status of the target claim, not the apparent opinion of the literature.
- Paper pages should list all audits that involve the paper, even when the paper is not itself the audit subject.

This keeps the public registry intuitive while protecting the ontology from drifting back into paper-centered peer review.

## Example

Suppose a funder asks C-SQD to audit a group of papers that appear to support the claim:

> Biomarker X predicts response to treatment Y in population Z.

The weak framing is:

> Do the papers support this claim?

The C-SQD framing is:

> Under the specified population, treatment, measurement, and outcome conditions, which warrant links for this claim survive audit?

The attached papers then become evidence artifacts. Reviewers inspect whether each artifact measured biomarker X appropriately, whether treatment response was operationalized correctly, whether confounding was addressed, whether the model was validated, whether the population matches Z, and whether the paper's conclusion actually bears on the target claim.

The output is not a count of supporting papers. It is an audited account of which parts of the claim are warranted, which are not, and where the evidence chain breaks.

## Evaluation Tuple

The C-SQD evaluation tuple remains useful under this framing, but its anchor should be the scoped claim under audit, not the manuscript as a publication object.

The tuple should summarize audit state:

> Given this scoped claim, this reviewer community, and this evaluation time, what has the audit found?

For a single-paper audit, the target claim may be very close to a manuscript claim. For a multi-paper audit, the tuple summarizes the audit of the target claim across inspected warrants and attached artifacts. In both cases, it is not a paper score and not a count of supporting publications.

The tuple can therefore be read as:

- `N`: audited non-ethical problems in the claim's warrants
- `M`: audited ethical or normative concerns
- `S`: stakes of the target claim
- `L`: scrutiny depth, meaning how much relevant audit work has been done
- `U`: uptake or downstream use and attention

Public-facing language should avoid implying that the tuple measures how good a manuscript is. Better labels include "claim audit tuple" or "audit state tuple." The old tuple survives; its conceptual anchor moves from manuscript-as-object to claim-under-audit.

## Rule Of Thumb

Papers do not vote. Claims do not inherit credibility from publication. Warrants must be audited.
