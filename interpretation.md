# Interpretation Of The C-SQD / FEN Direction

## Purpose

This document records the working interpretation for product and implementation decisions.

C-SQD is now being built as **general epistemic audit infrastructure** for commissioned, decomposed audits. Academic Publishing / Scholarly Search remains the first implemented intake adapter, but the first GTM wedge is organization-funded epistemic auditing, especially in computational biology, biomedical machine learning, and adjacent technical domains.

## Source Precedence

Use this order when documents conflict:

1. `C_SQD_NEW_GTM.pdf`
2. `FEN_for_CSQD_GTM.pdf`
3. `FEN_Schema_for_CSQD_GTM.tex`
4. `NEXT_STEPS.md` for current engineering execution
5. `build_decisions.md` for stack and architecture choices
6. Older documents: `CSQD_NEW.pdf`, `FEN_Schema_for_CSQD.pdf`, and `old_mvp_docs/`

Older documents are useful history, especially for platform ambition and marketplace framing, but they should not override the current commissioned-audit GTM or the GTM FEN schema.

## Core Interpretation

C-SQD should be understood first as epistemic audit infrastructure.

The central product asset is not a document reader, PDF host, publication venue, or manuscript marketplace. The central asset is the audit graph:

- domain instantiations
- audit subjects
- facts
- audit episodes
- episode memberships
- synthesis reviews
- provenance
- evaluation tuples

Academic Publishing remains the first implemented adapter. Its scholarly intake, library, article access, and episode workspace should be treated as domain-specific surfaces over the general audit infrastructure.

## Core Ontology

Use the following interpretation when designing backend and UI behavior:

- `DomainInstantiation`: a configured epistemic audit domain, such as Academic Peer Review or Clinical Trial Protocol Review.
- `AuditSubject`: referenced metadata for the paper, model, dataset, protocol, report, or claim under evaluation.
- `Fact`: an atomic, timestamped, provenance-bearing epistemic or administrative act.
- `AuditEpisode`: a coherent commissioned audit question over time.
- `EpisodeMembership`: a provenance-bearing claim that a fact belongs to an episode.
- `SynthesisReview`: an authored integrative interpretation of an audit episode.
- Evaluation tuple `E(A | R, T_eval) -> (N, M, S, L, U)`: a derived view over episode memberships, not ordinary stored state.

The old scholarly/review terms should be treated as Academic Peer Review adapters, not as the universal platform model.

## Academic Peer Review As First Domain

The current app may still use domain-specific language:

- Scholarly Intake
- scholarly objects
- articles/preprints
- manuscript or paper review

But these should be interpreted as Academic Publishing surfaces over the general C-SQD substrate.

For example:

- A `ScholarlyObject` is an Academic Publishing metadata adapter over `AuditSubject`.
- An `ElementReview` is a `FactPayload` attached to an `AuditEpisode` through `EpisodeMembership`.
- A commission is an `AuditCommission` fact and an episode membership, not a separate product primitive.
- A synthesis review is a first-class `SynthesisReview` attached to an `AuditEpisode`.

## Multi-Domain Product Framing

The live product should make clear that C-SQD has multiple possible domains.

Current domain:

- Academic Peer Review

Planned domains may include:

- Clinical Trial Protocol Review
- AI System Auditing
- Policy Evidence Review

Do not create fake workflows for planned domains. It is fine to show them as planned, but they should become operational only when real domain configs, audit subject types, and adapters exist.

The Domains page should separate:

- domain semantics: audit subjects, facts, audit episodes, episode memberships, synthesis reviews, evaluation basis
- implemented UI surfaces: Scholarly Intake, Library, commissioned audit workspaces

Library is cross-domain user workspace infrastructure. Scholarly Intake is an Academic Publishing surface, not a global C-SQD primitive.

Use "causal & statistical" for evaluation language that might otherwise be narrower.

## Hosting And Publisher-Controlled Content

C-SQD may host or natively render content when it has the rights to do so, such as:

- author-submitted manuscripts
- preprints
- permissively licensed works
- conference submissions
- datasets
- code
- protocols
- other authorized materials

For copyrighted or publisher-controlled works, C-SQD should preserve the external publisher or repository as the authoritative source.

In those cases, C-SQD should store:

- metadata
- canonical identifiers
- links
- access-rights signals
- audit subjects
- facts
- audit episodes
- synthesis reviews

C-SQD should not host or embed unauthorized copies of publisher-controlled articles. A random PDF URL alone is not enough for native display. Native display should require trustworthy rights signals.

## Visibility And Subscriptions

The existence of facts, episode metadata, subject identities, and public discovery signals should be visible enough to support network effects.

Full fact text, advanced analytics, custom community-filtered evaluations, institutional reports, and other high-value views may be subscription-gated.

The platform should avoid hiding so much audit information that facts and synthesis outputs cease to be discoverable, citable, or reusable.

## Revenue Framing

Older MVP docs remain useful for revenue framing where they do not conflict with the C-SQD/FEN model.

Near-term revenue should come from commissioned audits funded by organizations that already spend money on diligence, validation, review, or assessment. Long-term sustainability may also come from:

- subscriptions
- institutional services
- community services
- AI assistant products
- verified tags
- nonstandard evaluations

Revenue design should support the integrity and usefulness of the audit graph, not distort it.

## Implementation Implications

Product and engineering decisions should optimize for low-friction participation in the audit graph.

Prioritize:

- durable audit-subject identity
- audit commission facts
- episode-scoped fact creation
- solicitation lifecycle as facts
- synthesis review workflows
- provenance
- evaluation tuple computation
- domain-scoped search/intake
- rights-aware handling of native versus externally controlled content
- browser-accessible participation

Do not prioritize a sophisticated document reader ahead of the core audit workflow, except where native rendering directly improves review creation or evaluation for content C-SQD is allowed to display.

## Short Form

When in doubt, build C-SQD as multi-domain epistemic audit infrastructure.

Treat Academic Publishing as the first adapter. Treat articles and manuscripts as important audit subjects, not as the product itself. Use native article display when rights permit, link out when rights require it, and keep facts, audit episodes, provenance, synthesis reviews, and evaluation tuples at the center of the platform.
