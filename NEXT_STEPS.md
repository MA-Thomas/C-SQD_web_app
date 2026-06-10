# NEXT_STEPS

## Current Direction

C-SQD is now general epistemic audit infrastructure for commissioned, decomposed audits of important scientific and technical claims.

The first objective is not to build a better journal. It is to test whether organizations will fund structured audits of claims, papers, models, datasets, reports, protocols, and related artifacts.

Academic Publishing / Scholarly Intake remains useful infrastructure and should be preserved as the first intake adapter, especially for computational biology, biomedical machine learning, and adjacent technical domains.

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

The commissioned audit path is now the primary product slice:

- `/` shows the Audit Console.
- `/commission` creates or reuses an `AuditSubject`, captures sponsor/scope/funding, and commissions an `AuditEpisode`.
- `/audit-episodes/:id` shows the episode workspace and records episode-scoped `ElementReview` facts.
- `/intake` searches/retrieves scholarly metadata and registers Academic Publishing audit subjects.
- `/browse` explores Academic Publishing problem areas from criteria and facts.
- `/library` remains a cross-domain workspace for saved audit subjects.
- `/domains` frames active and planned audit domains.

## API Backbone

- `POST /api/audit-subjects` registers an `AuditSubject`.
- `POST /api/audit-subjects/:id/audit-episodes` creates an organizational sponsor, an `AuditEpisode`, an `AuditCommission` fact, and a commission `EpisodeMembership`.
- `POST /api/audit-episodes/:id/facts/element-review` creates an episode-scoped `ElementReview` fact and membership.
- Read APIs expose domain instantiations, subjects, episodes, facts, scholarly intake records, article access, browse results, and library items.

## What To Preserve

- Rust workspace, `services/api`, and `crates/domain`.
- Next.js app in `apps/web`.
- PostgreSQL migrations/seeds and local setup scripts.
- DOI, arXiv, PubMed/PMC, and title retrieval.
- Rights-aware article access and external-location logic.
- Scholarly Intake as an Academic Publishing adapter.
- Library as a cross-domain user workspace.
- Domain registry page and sidebar framing.
- Audit console, commission flow, and episode workspace as the core operational surfaces.

## Next Implementation Order

1. Add episode-scoped synthesis review creation and read surfaces.
2. Compute and display evaluation tuples from facts and episode memberships.
3. Add solicitation facts and solicitation lifecycle events for paid reviewer work.
4. Improve episode timelines with provenance, status transitions, and membership roles.
5. Add richer sponsor/admin dashboards for commissioned audit delivery.
6. Broaden domain configuration only after real pilot needs require it.

## Visual Experience

The visual language should feel like a serious diligence and audit console:

- quiet, restrained, and operational
- dense but organized tables, forms, and timelines
- clear status markers, provenance cues, and audit trails
- restrained color with semantic highlights
- strong typography and compact controls
- no marketing-style hero treatment for the main app

The target feeling is not "publishing platform." It should feel like infrastructure for funded review, evidence work, and organizational decision support.
