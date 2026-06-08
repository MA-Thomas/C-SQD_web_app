# Interpretation Of The C-SQD / FEN Direction

## Purpose

This document records the working interpretation for product and implementation decisions.

C-SQD is now being built as **general epistemic audit infrastructure**. Academic Peer Review is the first active domain, not the full scope of the platform.

## Source Precedence

Use this order when documents conflict:

1. `CSQD_NEW.pdf`
2. `FEN_Schema_for_CSQD.pdf`
3. `NEXT_STEPS.md` for current engineering execution
4. `build_decisions.md` for stack and architecture choices
5. Older MVP documents in `old_mvp_docs/`

Older MVP docs are useful history, especially for revenue and marketplace framing, but they should not override the current C-SQD/FEN model.

## Core Interpretation

C-SQD should be understood first as epistemic audit infrastructure.

The central product asset is not a document reader, PDF host, or narrow manuscript marketplace. The central asset is the audit graph:

- domain instantiations
- audit objects
- review events
- review-event memberships
- solicitations and solicitation events
- synthesis structures
- challenges
- provenance
- evaluation tuples

Academic Peer Review remains the first implemented domain. Its scholarly search, library, assignments, article access, and review workspace are domain-specific surfaces over the general audit infrastructure.

## Core Ontology

Use the following interpretation when designing backend and UI behavior:

- `DomainInstantiation`: a configured epistemic audit domain, such as Academic Peer Review or Clinical Trial Protocol Review.
- `AuditObject`: the durable thing being reviewed inside a domain.
- `ReviewEvent`: an atomic, timestamped, provenance-bearing evaluative act.
- `ReviewEventMembership`: the relationship between a review event and the audit object or objects it evaluates.
- `ERSolicitation`: a request for element-review labor.
- `SolicitationEvent`: append-only lifecycle history for a solicitation.
- `SynthesisSection`: authored integrative interpretation.
- `Challenge`: structured contestation over review events or syntheses.
- Evaluation tuple `E(A | R, Teval) -> (N, M, S, L, U)`: a derived view, not ordinary stored state.

The old scholarly/review terms should be treated as Academic Peer Review adapters, not as the universal platform model.

## Academic Peer Review As First Domain

The current app may still use domain-specific language:

- Scholarly Search
- scholarly objects
- articles/preprints
- review assignments
- manuscript or paper review

But these should be interpreted as Academic Peer Review surfaces over the general C-SQD substrate.

For example:

- A `ScholarlyObject` is an Academic Peer Review adapter over `AuditObject`.
- A legacy `ReviewEpisode` should become a `ReviewEvent`.
- A legacy `ReviewAssignment` should become an `ERSolicitation` plus `SolicitationEvent`.
- A synthesis review should become a `ReviewEvent` plus `SynthesisSection`.

## Multi-Domain Product Framing

The live product should make clear that C-SQD has multiple possible domains.

Current domain:

- Academic Peer Review

Planned domains may include:

- Clinical Trial Protocol Review
- AI System Auditing
- Policy Evidence Review

Do not create fake workflows for planned domains. It is fine to show them as planned, but they should become operational only when real domain configs, audit object types, and adapters exist.

The Domains page should separate:

- domain semantics: audit objects, review modes, shared primitives, evaluation basis
- implemented UI surfaces: Scholarly Search, Library, Assignments, review workspaces

Library is cross-domain user workspace infrastructure. Scholarly Search is an Academic Peer Review surface, not a global C-SQD primitive.

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
- audit objects
- review events
- evaluation data

C-SQD should not host or embed unauthorized copies of publisher-controlled articles. A random PDF URL alone is not enough for native display. Native display should require trustworthy rights signals.

## Review Visibility And Subscriptions

The existence of reviews, review metadata, object identities, and public discovery signals should be visible enough to support network effects.

Full review text, advanced analytics, custom community-filtered evaluations, institutional reports, and other high-value views may be subscription-gated.

The platform should avoid hiding so much review information that reviews cease to be discoverable, citable, or reusable.

## Revenue Framing

Older MVP docs remain useful for revenue framing where they do not conflict with the C-SQD/FEN model.

Submission or review fees should primarily support review activity and market functioning. Long-term sustainability may also come from:

- subscriptions
- challenge fees
- bounty fees
- verified tags
- nonstandard evaluations
- institutional services
- community services
- AI assistant products

Revenue design should support the integrity and usefulness of the audit graph, not distort it.

## Implementation Implications

Product and engineering decisions should optimize for low-friction participation in the audit graph.

Prioritize:

- durable audit-object identity
- review-event creation
- solicitation lifecycle history
- synthesis and challenge workflows
- provenance
- evaluation tuple computation
- domain-scoped search/intake
- rights-aware handling of native versus externally controlled content
- browser-accessible participation

Do not prioritize a sophisticated document reader ahead of the core audit workflow, except where native rendering directly improves review creation or evaluation for content C-SQD is allowed to display.

## Short Form

When in doubt, build C-SQD as multi-domain epistemic audit infrastructure.

Treat Academic Peer Review as the first domain. Treat articles and manuscripts as important audit objects, not as the product itself. Use native article display when rights permit, link out when rights require it, and keep review events, provenance, challenges, and evaluation tuples at the center of the platform.
