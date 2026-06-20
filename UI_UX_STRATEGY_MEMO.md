# C-SQD UI/UX Strategy Memo

Date: June 10, 2026

## Purpose

This memo defines the next UI/UX direction for the C-SQD MVP.

The previous app shell was oriented around internal audit operations: audit console, sponsor console, reviewer queue, and episode workspace. Those surfaces remain important, but they should not be the first thing a public visitor sees.

The MVP website should first communicate:

> C-SQD is a public registry and method for epistemic audits.

For the Academic Peer Review domain, users should be able to discover scholarly works, inspect public audit activity, understand evaluation tuples, read SynthesisReviews, dive into ElementReviews, and learn the method behind C-SQD without needing an account.

Authenticated sponsor and reviewer workflows should exist behind login and role permissions. They are backstage operational tools, not the public face of the MVP.

## Core Product Framing

C-SQD should present as two connected layers over one audit substrate:

```text
Public audit registry
  -> public audit subjects
  -> public evaluation tuples
  -> public ElementReviews
  -> public SynthesisReviews / audit reports
  -> public challenges and provenance

Authenticated audit operations
  -> sponsor consoles
  -> reviewer queues
  -> commissioned audit workspaces
  -> private or draft audit records
```

The underlying product model remains:

```text
AuditSubject
  -> AuditEpisode
  -> Facts
  -> ElementReviews
  -> SynthesisReviews
  -> EvaluationTuple / audit report
  -> Challenges / responses / contestations
```

But the public MVP should not teach users through database-shaped or admin-shaped pages. It should teach through public audit artifacts.

## MVP Center Of Gravity

The MVP homepage should not be an internal `Audit Console`.

It should be a public entry point into C-SQD as an audit registry:

- search or browse public audit subjects
- discover public audits by domain
- inspect evaluation tuples
- read public audit reports
- understand the method
- commission or request an audit
- submit an unsolicited ElementReview after login

The public user should quickly understand:

1. What has been audited?
2. What did the audit find?
3. How much scrutiny has the subject received?
4. Which criteria or weaknesses were reviewed?
5. Who can add further review, challenge, or commission deeper work?

## Public Information Architecture

Recommended public navigation:

```text
C-SQD
  Discover
  Public Audits
  Domains
  Method
  Commission an Audit

Academic Peer Review
  Scholarly Works
  CRWE
  ElementReviews
  SynthesisReviews
  Challenges
```

Recommended authenticated navigation:

```text
Account
  Library / Watchlist

Sponsor
  Sponsor Console
  Commissioned Audits
  Audit Deliverables

Reviewer
  Reviewer Queue
  Submitted ElementReviews
  Completed Reviews

Operations / Admin
  Audit Operations
  Episode Workspace
  Solicitation Management
  Draft Reports
```

The public navigation should be visible by default. Sponsor, reviewer, account, and operations areas should require login and appropriate role state.

## Public Discover

The Discover page should be the main public exploration surface.

Users should be able to browse and search scholarly works without logging in.

Discover should support:

- title, DOI, arXiv, PubMed, author, venue, and keyword search
- filtering by domain
- filtering by audit status
- filtering by CRWE category
- sorting by scrutiny depth, report availability, recent audit activity, and uptake
- quick access to works with public SynthesisReviews
- quick access to works needing review

For each scholarly work, show a compact public audit summary:

```text
Title
Authors / source / year
Audit status
Evaluation tuple
CRWE coverage
ElementReview count
SynthesisReview count
Challenge count
```

Recommended status labels:

- `Unaudited`
- `ElementReviews submitted`
- `In synthesis`
- `Audit report available`
- `Challenged`
- `Superseded`

The evaluation tuple should use friendly labels first:

```text
Problems
Ethical concerns
Stakes
Scrutiny depth
Uptake
```

The symbolic notation `E(A | R, T_eval) -> (N, M, S, L, U)` can appear in Method pages or advanced detail views, not as the default public label.

Discover and Search/Register should be distinct surfaces, but they should route to the same registered-work action model. Once a scholarly work exists in C-SQD and is registered as an `AuditSubject`, audit actions should be visible alongside that work wherever it appears, including Discover results, Search/Register results, library/watchlist surfaces, CRWE views, and public subject pages.

## Academic Peer Review Search And Registration

Academic Peer Review should include a public search and registration surface for scholarly works.

This surface is more than search. It should be the entry ramp for expanding and participating in the Academic Peer Review audit graph:

```text
Find scholarly work
  -> create or find scholarly metadata when needed
  -> register linked AuditSubject when needed
  -> inspect public audit activity
  -> inspect applicable CRWE
  -> submit reviews, synthesis, challenges, or petitions
```

In backend terms, registering a scholarly work means creating or reusing a durable `AuditSubject` for that work. The `ScholarlyObject` record is Academic Peer Review adapter metadata: title, authors, DOI, arXiv, PubMed, source links, version grouping, rights/access signals, and retrieval provenance. Audit actions attach to the linked `AuditSubject`, not to the scholarly metadata record itself.

Recommended registration semantics:

```text
External scholarly work
  -> ScholarlyObject metadata record
  -> AuditSubject with source_entity_type = scholarly_object
  -> Facts, public AuditEpisodes, ElementReviews, SynthesisReviews, challenges, petitions
```

Users should be able to:

- search existing C-SQD scholarly work records
- search external sources such as DOI, arXiv, PubMed, publisher metadata, journal records, author, venue, title, and keyword search
- add a missing work to the C-SQD scholarly work repository when metadata can be retrieved or supplied
- register the work as an `AuditSubject` when needed
- inspect existing public audit activity for the work
- inspect the applicable CRWE criteria for the work
- see the same audit actions shown for registered works in Discover and public subject pages
- submit an unsolicited `ElementReview` against an existing CRWE criterion after login
- start or join a public `AuditEpisode` before submitting an unsolicited `SynthesisReview`
- submit direct challenges to existing `ElementReviews` or `SynthesisReviews`
- petition for an existing `ElementReview` by another author to be included in the featured `ElementReview` set
- petition for a new CRWE element when the current taxonomy lacks a criterion needed to audit the work
- petition that an existing CRWE element should be considered applicable to the work

Unsolicited `SynthesisReviews` should not float independently from the audit graph. A user must first start or join a public `AuditEpisode`; their submission should then be marked by the backend type system as unsolicited rather than commissioned.

The search and registration UX should distinguish contribution types:

```text
Unsolicited ElementReview
  focused review of one existing CRWE criterion
  attached to the public audit subject and relevant public episode context
  discoverable after submission, subject to moderation and visibility rules

Unsolicited SynthesisReview
  integrative interpretation of a public AuditEpisode
  requires starting or joining that public episode first
  marked as unsolicited in backend domain types

Direct challenge
  contests the content, status, or interpretation of an existing ElementReview or SynthesisReview
  preserves the challenged artifact and adds a provenance-bearing contestation record

Petition to feature
  asks that someone else's existing ElementReview be placed in the small featured set
  does not contest the review
  does not ask C-SQD, moderators, or the community to open a challenge
  affects default UX prominence, not discoverability

CRWE petition
  asks for a new CRWE element or applicability of an existing CRWE element
  explains why the current taxonomy does not cover the audit need for this work
```

All `ElementReviews` should remain discoverable, but only a small subset should be featured in default UX surfaces. Featured status should be treated as a provenance-bearing curation layer, not as the existence or validity of the review itself.

## Public Audit Subject Page

Clicking a registered scholarly work from Discover, Search/Register, Library, CRWE, Public Audits, or any other public surface should open a public audit subject page, not an internal operations workspace.

The public page should be backed by the registered `AuditSubject`. Scholarly metadata, version grouping, article access, and source links should support that audit subject; they should not become the place where audit actions or audit state are conceptually stored.

This page should be the flagship public artifact for the MVP.

Recommended structure:

```text
Subject summary
  title, authors, identifiers, source links, versions

Evaluation tuple
  Problems
  Ethical concerns
  Stakes
  Scrutiny depth
  Uptake

Latest public audit report
  SynthesisReview summary
  key findings
  recommendations
  open questions

CRWE coverage
  reviewed criteria
  unreviewed criteria
  problem areas

ElementReviews
  grouped by CRWE criterion
  collapsed by default

Challenges
  open challenges
  responses
  superseded or contested claims

Audit trail
  provenance
  public facts
  advanced / collapsed by default
```

The page should have clear actions:

- `Submit ElementReview`
- `Review this criterion`
- `Start public audit episode`
- `Join public audit episode`
- `Submit SynthesisReview`
- `Commission deeper audit`
- `Challenge this review`
- `Petition to feature ElementReview`
- `Petition CRWE change`
- `Save to library` or `Watch`

Actions that require identity should be visible but auth-gated.

## Submit ElementReview CTA

The public audit subject page should invite participation.

Recommended primary or secondary action:

```text
Submit ElementReview
```

Possible friendlier variant:

```text
Review one criterion
```

This action creates an unsolicited ElementReview and should require login.

If the user is logged out:

- show a sign-in prompt
- preserve the return URL
- preserve the selected audit subject and criterion if applicable
- explain briefly that ElementReviews are focused reviews of one CRWE criterion

After login, the user should be able to:

- choose a CRWE criterion
- see any existing ElementReviews for that criterion
- submit finding, severity, confidence, limitations, recommendations, and review content
- optionally cite source material
- submit as an unsolicited public ElementReview

The UI should distinguish:

```text
Commissioned ElementReview
  requested through a solicitation
  may be compensated
  appears in Reviewer Queue
  tied to a commissioned AuditEpisode

Unsolicited ElementReview
  initiated from public Discover
  requires login
  uncompensated by default
  may contribute to scrutiny depth
  may be cited by later SynthesisReviews
  may be challenged, moderated, or superseded
```

## Public Method

The Method area should explain how C-SQD works.

It should be pedagogical, not marketing-heavy. It should make the audit model understandable through concrete examples and short conceptual sections.

For the Academic Peer Review domain, Method should include:

### Audit Subjects

Explain that an audit subject can be a paper, preprint, dataset, code repository, protocol, report, or scholarly claim.

### ElementReviews

Explain that an ElementReview is a focused review of one criterion or research weakness.

ElementReviews are smaller and more composable than traditional peer review. They can be commissioned, assigned, unsolicited, challenged, cited, and synthesized.

### SynthesisReviews

Explain that a SynthesisReview is an integrative audit report.

It combines ElementReviews and other facts into a higher-level interpretation:

- summary
- key findings
- evidence integration
- recommendations
- open questions
- cited ElementReviews

### CRWE

CRWE means Common Research Weakness Enumeration.

For Academic Peer Review, CRWE should function as the public taxonomy for organizing review criteria and problem areas.

Example categories may include:

- methodological adequacy
- statistical adequacy
- data and code availability
- interpretation strength
- ethical concern
- reproducibility
- evidence quality
- external validity

The exact CRWE taxonomy can evolve, but the UX should make clear that reviews are attached to explicit criteria rather than vague overall impressions.

### Evaluation Tuple

Explain the public evaluation tuple with friendly labels:

```text
Problems
Ethical concerns
Stakes
Scrutiny depth
Uptake
```

Then provide the expert notation in an expanded or technical section:

```text
E(A | R, T_eval) -> (N, M, S, L, U)
```

The tuple should be presented as a derived public summary of the audit record, not as a mysterious score.

### Challenge System

Explain that audit claims can be challenged without erasing the record.

Challenges should support:

- contesting an ElementReview
- contesting a SynthesisReview
- submitting a response
- marking a review as superseded
- linking newer evidence
- preserving the historical audit trail

The challenge system is central to making C-SQD feel like infrastructure for durable epistemic accountability rather than a one-time review platform.

### Public And Private Audits

Explain the visibility model:

```text
Public audits
  discoverable
  readable without login
  contribute to public evaluation tuples
  can be cited, challenged, watched, and expanded

Private audits
  visible only to authorized sponsors, reviewers, and operators
  may later publish a public report or public subset
  can share the same underlying audit substrate
```

## Authenticated Sponsor Flow

Sponsor Console should require login.

It should answer:

> What did we fund, and what is the delivery state?

Sponsor users should see:

- their commissioned audits
- sponsor organization
- audit subject
- scope
- funding
- deadlines
- assignment progress
- ElementReview coverage
- SynthesisReview / audit report status
- delivery state
- private vs public visibility

This should not be part of the public default experience.

Public visitors can see `Commission an Audit`, but they should not see private sponsor funding or delivery operations without authentication.

## Authenticated Reviewer Flow

Reviewer Queue should require login.

It should answer:

> What am I assigned to review?

Reviewer users should see:

- assigned commissioned ElementReviews
- criterion-specific task brief
- source materials
- due dates
- compensation state
- submission status
- completed reviews
- relationship between their ElementReview and any SynthesisReview

Unsolicited ElementReviews initiated from public pages should also appear in a logged-in user's submitted review history, but they are different from paid or assigned reviewer work.

## Public Audits

Public Audits should show delivered and visible audits across domains.

For the Academic Peer Review MVP, this may overlap with Discover, but it should emphasize completed public outputs:

- public audit reports
- public SynthesisReviews
- subjects with meaningful ElementReview depth
- challenged or contested audits
- recently updated audit records

This page is useful for showing C-SQD's value even before the user has a particular paper or claim in mind.

## Domains

The domain switcher should remain, but it should not imply separate products.

It should communicate that domains are lenses over the same audit substrate.

Active domain:

```text
Academic Peer Review
Papers, preprints, datasets, code, protocols, scholarly claims
```

Planned domains:

```text
Clinical Trial Review
AI Model Auditing
Policy Evidence Review
```

Planned domains can be visible, but they should not pretend to have live workflows before real domain configs and public artifacts exist.

## Design Principles

### Minimal By Default, Full Featured By Expansion

The public MVP should not show every operational control at once.

Default views should show:

- subject
- audit status
- evaluation tuple
- public report availability
- CRWE coverage
- next useful action

Advanced views can reveal:

- raw facts
- provenance
- audit episode memberships
- challenge history
- symbolic tuple notation
- internal workflow state

### Teach Through Artifacts

The app should teach C-SQD by showing public audit artifacts:

- public work cards
- evaluation tuples
- CRWE coverage
- ElementReview lists
- SynthesisReview reports
- challenge threads

Avoid relying on explanatory marketing copy or admin dashboards as the main teaching mechanism.

### Role-Aware Access

Public users can discover and read public audit activity.

Logged-in users can submit ElementReviews, save works, watch subjects, and challenge public claims.

Sponsors can view private funding and delivery operations.

Reviewers can view assigned and compensated work.

Operators can manage solicitations, drafts, and audit records.

### Reports Are The Destination

SynthesisReviews should feel like audit reports, not merely database records.

Public reports should be readable, citable, and shareable.

The operational system exists to create trustworthy public or private audit deliverables.

## Near-Term Implementation Recommendations

### 1. Recenter The Homepage

Replace the internal Audit Console as the default homepage with a public registry-oriented landing page.

The first screen should focus on:

- discovering public audits
- searching scholarly works
- recent public audit reports
- a short explanation of the evaluation tuple
- clear paths to Method and Commission an Audit

### 2. Build Discover

Create a public Discover page for scholarly works.

It should display evaluation tuple summaries, CRWE coverage, public report availability, ElementReview counts, and challenge counts.

### 3. Create Public Audit Subject Pages

Reframe scholarly object pages as public audit subject pages.

They should prioritize evaluation tuple, latest public SynthesisReview, CRWE coverage, ElementReviews, and challenges.

Article metadata and source access should remain, but they should support the audit subject rather than dominate the page.

### 4. Add Method

Create a Method area with Academic Peer Review sections:

- Audit Subjects
- ElementReviews
- SynthesisReviews
- CRWE
- Evaluation Tuple
- Challenges
- Public vs Private Audits

### 5. Auth-Gate Sponsor And Reviewer Consoles

Sponsor Console and Reviewer Queue should require login.

Logged-out users who attempt to access them should see a clear sign-in prompt and a short explanation of the role.

### 6. Add Submit ElementReview Flow

Expose `Submit ElementReview` or `Review one criterion` on public audit subject pages and CRWE rows.

Require login before submission.

Support unsolicited ElementReviews separately from commissioned assignments.

### 7. De-Emphasize Internal Workspace UI

Keep the internal AuditEpisode workspace, but treat it as an authenticated operations surface.

It should not be the main public representation of an audit.

## Strategic Principle

The MVP should make C-SQD legible as a public epistemic audit registry.

Academic Peer Review should be the first domain where users can discover works, inspect evaluation tuples, read public audit reports, explore ElementReviews, understand CRWE criteria, and participate by submitting focused reviews.

Sponsor and reviewer operations are essential, but they belong behind identity and role state. The public product should lead with audit artifacts, method, discoverability, and participation.
