# C-SQD UI/UX Strategy Memo

Date: June 10, 2026

## Purpose

This memo defines the next UI/UX direction for C-SQD after the shift from an MVP-era publishing/review platform toward general commissioned epistemic audit infrastructure.

The product should make one thing clear:

> C-SQD is general audit infrastructure. Domains provide specialized intake, language, criteria, and workflows on top of that shared audit substrate.

The immediate design goal is to preserve the useful Academic Peer Review manuscript search and review experience while preventing the product from feeling like a better journal or author-submission platform.

## Core Product Model

The shared product substrate is:

```text
AuditSubject
  -> AuditEpisode
  -> Facts
  -> ElementReviews
  -> SynthesisReview
  -> EvalTuple / audit deliverable
```

This model should remain visible in the generic audit operations surfaces:

- Audit Console
- Commission Audit
- Audit Episode Workspace
- Synthesis / Deliverable views
- Domain configuration

Domain-specific workflows should sit on top of this model rather than replace it.

## Keep The Domain Switcher

The website should keep a domain switcher.

The domain switcher should not imply that C-SQD is multiple unrelated products. It should communicate that users are choosing a domain lens over the same underlying audit machinery.

Recommended domain switcher behavior:

- Show the active domain clearly.
- Identify the domain scope.
- Separate active domains from planned domains.
- Avoid centering the whole app on Academic Peer Review, even while it is the first active domain.

Example:

```text
Active domain
Academic Peer Review
Manuscripts, preprints, datasets, code, protocols

Planned
Clinical Trial Review
AI Model Auditing
Policy Review
```

## Academic Peer Review Domain

Academic Peer Review should remain the first concrete domain workspace.

It should keep:

- DOI, title, arXiv, PubMed, and manuscript retrieval
- scholarly object and version grouping
- article/manuscript viewer
- problem-area browse
- academic CWE / CRWE criteria
- manuscript-specific ElementReview UX
- library/watchlist behavior

But the framing should change from publishing to audit intake.

The primary purpose of manuscript search should be:

> Find an existing scholarly work and turn it into an audit subject.

This supports sponsors as the sharp initial wedge, but it is broader than sponsor procurement alone.

Supported users include:

- sponsors finding a paper or preprint they want audited
- funders checking the evidence behind a grant or program
- journals or conferences commissioning outside review
- companies doing technical diligence
- researchers registering their own work for scrutiny
- reviewers and analysts finding existing audit activity

Recommended naming:

- Prefer `Scholarly Intake` or `Find Work to Audit`.
- Avoid `Submit Manuscript` as a primary label.
- `Search Manuscripts` is acceptable if it appears clearly inside the Academic Peer Review domain.

Recommended intake flow:

```text
Search DOI / title / arXiv / PubMed
  -> Select work/version
  -> Register as AuditSubject
  -> Commission audit / save to library / view audit activity
```

## Sponsor And Reviewer Flows

The UI should distinguish Sponsor flows from Reviewer flows.

This should be done as modes, queues, and views over shared audit records, not as separate products.

Both sponsors and reviewers operate on the same underlying `AuditEpisode` and `Fact` graph, but their jobs-to-be-done are different enough that they need different affordances.

## Sponsor Flow

Sponsor users want to:

- find or register an audit subject
- define audit scope
- fund the audit
- track reviewer assignment progress
- monitor review coverage
- receive a synthesis/deliverable
- understand risk, uncertainty, and next actions

Sponsor UX should emphasize:

- sponsor organization
- audit scope
- funding
- deadlines
- assignment coverage
- ElementReview progress
- synthesis status
- audit report/deliverable
- decision relevance

Recommended sponsor labels:

- `Sponsor Console`
- `Find Work to Audit`
- `Commission Audit`
- `Audit Brief`
- `Scope`
- `Funding`
- `Assignment Progress`
- `Delivery`
- `Audit Report`

## Reviewer Flow

Reviewer users want to:

- see assigned ElementReviews
- understand the scoped criterion
- inspect source materials
- submit a focused review
- track payment/completion state
- see how their review contributes to synthesis
- build a durable record of expertise

Reviewer UX should emphasize:

- assigned reviews
- criterion-specific task brief
- due dates
- source materials
- review form
- compensation
- completion status
- later: reputation and public record

Recommended reviewer labels:

- `Reviewer Queue`
- `Assignments`
- `Review Workspace`
- `Scope Criterion`
- `Source Materials`
- `Submit ElementReview`
- `Compensation`
- `Completed Reviews`

## Information Architecture

Recommended navigation structure:

```text
C-SQD
  Audit Console
  Commission Audit
  Domains

Sponsor
  Find Work to Audit
  Sponsor Console
  Audit Deliverables

Reviewer
  Reviewer Queue
  Review Workspace
  Completed Reviews

Academic Peer Review
  Scholarly Intake
  Browse Problem Areas
  Library
```

This structure can be simplified in the near term, but the conceptual separation should guide page design.

## Audit Episode Workspace

The audit episode page should become the central operations surface for a single commissioned audit.

It should support at least three modes or sections:

### Sponsor View

Focus:

- scope
- sponsor
- funding
- assignment progress
- coverage by criterion
- synthesis status
- delivery state
- evaluation tuple

### Reviewer View

Focus:

- assigned criterion
- source materials
- ElementReview form
- compensation
- due/completion state
- relevant prior facts

### Audit Record

Focus:

- all facts
- provenance
- ElementReviews
- solicitation lifecycle events
- synthesis reviews
- evaluation tuple
- historical audit trail

The same page may initially present these as panels. Over time, they can become tabs or role-aware views.

## Workflow Presentation

The audit workflow should be visually prominent:

```text
Commission -> Solicit -> Review -> Synthesize -> Deliver
```

Each step should show:

- current status
- missing requirements
- next available action
- relevant count or metric

Example:

```text
Commissioned: complete
Solicitations: 3 issued, 2 accepted, 1 completed
Reviews: 1/3 scoped criteria reviewed
Synthesis: pending
Delivery: blocked until synthesis
```

## Evaluation Tuple UX

The evaluation tuple should remain available for expert users, but user-facing labels should lead.

Recommended display:

```text
N: Problems
M: Ethical Concerns
S: Stakes
L: Scrutiny Depth
U: Uptake
```

The symbolic notation `E(A | R, Teval) -> (N, M, S, L, U)` can appear in technical or expanded views, not as the first thing most users must interpret.

## Language Guidelines

Use audit-coded language across the generic product.

Prefer:

- `Register audit subject`
- `Find work to audit`
- `Commission audit`
- `Issue solicitation`
- `ElementReview`
- `Focused review`
- `Synthesis review`
- `Audit report`
- `Audit deliverable`

Avoid or de-emphasize:

- `Submit manuscript`
- `Publish`
- `Journal`
- `Accept/reject`
- `Article review` as the global product concept

Academic Peer Review can still use domain-specific words like manuscript, preprint, citation, method, reproducibility, and peer review, but those words should appear as domain language rather than as the identity of the whole platform.

## Near-Term Implementation Recommendations

### 1. Refine Navigation

Keep the domain switcher.

Make the sidebar distinguish:

- core audit operations
- sponsor actions
- reviewer actions
- Academic Peer Review domain tools

### 2. Rename Intake Surfaces

Move manuscript retrieval/search under the Academic Peer Review domain.

Recommended label:

```text
Scholarly Intake
```

Recommended page headline:

```text
Find Work To Audit
```

### 3. Add Sponsor Console

Create a sponsor-oriented view of commissioned audits with:

- funded audits
- scope
- spend/funding
- reviewer assignment progress
- synthesis/delivery status

### 4. Add Reviewer Queue

Create a reviewer-oriented work queue with:

- assigned criterion
- audit subject title
- due state
- payment condition
- action to open review workspace

### 5. Improve Episode Workspace

Turn the current audit episode page into a clearer operational hub:

- workflow strip
- sponsor progress
- reviewer assignment table
- ElementReview intake
- synthesis report panel
- audit record timeline

### 6. Make Synthesis Feel Like The Deliverable

Present synthesis reviews as audit reports, not merely records.

Recommended sections:

- summary
- key findings
- evidence integration
- recommendations
- open questions
- cited ElementReviews

## Strategic Principle

The core product should feel like commissioned audit infrastructure.

Academic Peer Review should feel like the first working domain implementation.

Sponsor and Reviewer flows should feel distinct because the users are doing different work, but they should remain visibly connected through the same audit record.

