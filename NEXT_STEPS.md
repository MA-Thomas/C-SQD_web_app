# NEXT_STEPS

## Current Direction

C-SQD is now general epistemic audit infrastructure for commissioned, decomposed audits of important scientific and technical claims.

The first objective is not to build a better journal. It is to test whether organizations will fund structured audits of claims, papers, models, datasets, reports, protocols, and related artifacts.

Academic Publishing / Search-Register remains useful infrastructure and should be preserved as the first metadata adapter, especially for computational biology, biomedical machine learning, and adjacent technical domains.

## Source Precedence

Use this order when documents conflict:

1. `C_SQD_NEW_GTM.pdf`
2. `FEN_for_CSQD_GTM.pdf`
3. `FEN_Schema_for_CSQD_GTM.tex`
4. `interpretation.md`
5. `build_decisions.md` for stack and local architecture choices
6. Older documents in `old_mvp_docs/`

Older documents are historical only when they conflict with the commissioned-audit GTM or GTM FEN schema.

## Current Ontology

The active backend vocabulary is:

- `DomainInstantiation`: configured audit domain and CWE/evaluation-tuple owner.
- `AuditSubject`: durable referenced metadata for the artifact or claim under evaluation.
- `Fact`: immutable atomic epistemic or administrative act.
- `AuditEpisode`: coherent commissioned audit question over time.
- `EpisodeMembership`: provenance-bearing claim that a fact belongs to an episode.
- `SynthesisReview`: authored interpretation of an audit episode.
- `EvaluationTuple`: derived view `E(A | R, T_eval) -> (N, M, S, L, U)`.

Academic Publishing concepts such as `ScholarlyObject` are intake/access adapters over this substrate, not universal C-SQD primitives.

## Current Product Slice

The public audit registry path is the primary MVP slice. The public frontend
was rebuilt (July 2026) around a news-briefing information architecture; see
`PUBLIC_FRONTEND_REBUILD_PLAN.md` for the full design record.

- `/` is a briefing homepage: the latest public audit report as the lead
  story, plus rails for recently challenged, gaining scrutiny, and awaiting
  review.
- `/discover` searches and filters public scholarly works (criterion, status,
  sort), absorbing the old CRWE browse as a filter dimension.
- `/audits` lists delivered public outputs: reports, review depth, contested
  audits (replaces `/public-audits`).
- `/works/:id` is the public audit subject "full coverage" page: report,
  criterion clusters, dissent block, audit trail, sticky action rail
  (replaces `/scholarly-objects/:id`).
- `/register` searches/retrieves scholarly metadata and registers Academic
  Publishing audit subjects (replaces `/intake` and `/retrieve`).
- `/criteria` is the domain-scoped criterion taxonomy reference (replaces the
  reference half of `/browse`).
- `/method` explains AuditSubjects, ElementReviews, SynthesisReviews, CRWE, evaluation tuples, challenges, and public/private audit visibility.
- `/commission` creates or reuses an `AuditSubject`, captures sponsor/scope/funding, and commissions an `AuditEpisode`. The form is public up to submission.
- `/library`, `/sponsor-console`, `/reviewer-queue`, `/operations`, and `/audit-episodes/:id` are backstage surfaces gated behind identity and role state.

Frontend structure: public styling lives in `apps/web/app/public.css`, scoped
under `.pub-shell`; `globals.css` now holds only base, shared-component, and
backstage styles. Old public routes were removed without redirects.

## API Backbone

- `POST /api/audit-subjects` registers an `AuditSubject`.
- `POST /api/audit-subjects/:id/audit-episodes` creates an organizational sponsor, an `AuditEpisode`, an `AuditCommission` fact, and a commission `EpisodeMembership`.
- `POST /api/audit-episodes/:id/facts/element-review` creates an episode-scoped `ElementReview` fact and membership. Public unsolicited review CTAs are currently auth-gated until real identity/session handling is connected.
- `GET /api/public/audit-subjects/:id/summary` and `GET /api/public/audit-subjects/summaries?ids=...` return server-side public subject summaries (status label, evaluation tuple, CRWE coverage, counts, latest report, episodes); the batch variant collapses the Discover/home/Public Audits fan-out into one call.
- Read APIs expose domain instantiations, subjects, episodes, facts, scholarly intake records, article access, browse results, and library items.

## What To Preserve

- Rust workspace, `services/api`, and `crates/domain`.
- Next.js app in `apps/web`.
- PostgreSQL migrations/seeds and local setup scripts.
- DOI, arXiv, PubMed/PMC, and title retrieval.
- Rights-aware article access and external-location logic.
- Search / Register as an Academic Publishing metadata adapter.
- Library / Watchlist as a cross-domain account workspace.
- Domain registry page and sidebar framing.
- Commission flow and backstage audit operations as role-gated operational surfaces.

## Next Implementation Order

Status as of June 16, 2026: the FEN alignment + public-registry plan
(`IMPLEMENTATION_PLAN.md`, phases B0–B6 and F0–F7) is substantially implemented
and verified against seeded data. See that file's "Implementation Status" table
for the per-phase breakdown.

Done:

1. ✅ Real authentication/session state and role permissions (magic-link, `require_role`).
2. ✅ Challenge and petition APIs, counts, and public threads (relations, `submitter_response` contests, `Feature`/`CWE` petitions).
3. ✅ Unsolicited public AuditEpisode start/join flows (`EpisodeParticipation`).
4. ✅ Authenticated ElementReview submission for public subject pages.
5. ✅ Sponsor, reviewer, and operations consoles behind role-gated backstage routes.

Remaining / deferred:

6. Broaden domain configuration only after real pilot needs require it (intentionally deferred).
   When a second domain goes live, the frontend needs: a `?domain=` param on
   `/criteria` and `/discover`, a domain chooser on `/criteria`, and
   `getAcademicCweNodes` generalized to `getCweNodesForDomain(domainId)`
   (currently the only academic-domain hard assumption in the public data
   layer). Never merge criteria across domains into one list — a criterion is
   owned by its `DomainInstantiation`.
7. Optional optimization: make the batch public-summary endpoint
   (`/api/public/audit-subjects/summaries?ids=...`) a single aggregating SQL
   pass instead of a server-side loop. Not blocking — the frontend already
   makes one call.
8. ✅ `FEN_Schema_for_CSQD_GTM.tex` already documents the participation/petition/
   curation `FactPayload` variants (field-for-field with `crates/domain/src/fact.rs`),
   so schema source and code are in sync. Only residual: re-render
   `FEN_for_CSQD_GTM.pdf` from the `.tex` if the rendered PDF predates these
   variants (the PDF outranks the tex in source precedence).

## Visual Experience

The app now has two deliberate visual registers over one substrate:

Public (news-briefing register, `public.css`):

- white cards on a near-white canvas, strict headline hierarchy
- blue links and CTAs; teal reserved for audit-semantic accents (tuple,
  status, provenance)
- "full coverage" clustering on subject pages; dissent visually distinct
- persistent search in the header; section tabs; no sidebar
- no marketing-style hero treatment

Backstage (operational register, `globals.css`, unchanged):

- quiet, restrained, dense tables, forms, and timelines
- clear status markers, provenance cues, and audit trails

The contrast is intentional: feeds invite, artifacts and operations stay
rigorous. The target feeling remains infrastructure for funded review,
evidence work, and organizational decision support — not "publishing
platform."
