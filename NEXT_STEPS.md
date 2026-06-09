# NEXT_STEPS

## Current Direction

C-SQD has pivoted from a narrow academic peer-review MVP to the more general epistemic audit infrastructure described in `CSQD_NEW.pdf` / `FEN_Schema_for_CSQD.pdf`.

Academic Peer Review remains the first active product surface and first demo domain. It should be treated as one `DomainInstantiation` of the broader system, not as the root ontology of C-SQD.

The live app should continue to make this visible:

- Global C-SQD surfaces: Domains, Library.
- Active domain surfaces: Browse, Scholarly Search, Assignments, Review Episodes, Bounties, Payments.
- Planned domains remain visible but inactive until their backend adapters and domain configs are meaningful.
- Use "causal & statistical" for evaluation language that might otherwise be narrower.

## Current State

The general audit backbone is now present in the codebase:

- Rust domain types exist for `DomainInstantiation`, `AuditObject`, `ReviewEvent`, `ReviewEventMembership`, `ERSolicitation`, `SolicitationEvent`, `SynthesisSection`, `SynthesisReviewRelation`, `Challenge`, and evaluation tuple structures.
- PostgreSQL migrations create the audit backbone tables.
- A default Academic Publishing / Academic Peer Review domain instantiation is seeded with base CWE nodes.
- Existing scholarly demo records are mirrored into `audit_objects`.
- Article retrieval paths ensure imported DOI, arXiv, PubMed/PMC, and title-search results have matching audit objects.
- `/api/domain-instantiations` and `/api/audit-objects` expose the new backbone.
- Legacy `/api/scholarly-objects` endpoints still work as the Academic Peer Review adapter.
- Native article display is rights-gated by trusted access signals; a random PDF URL alone is not enough.
- The Library is now explicit user curation through `user_library_items`, not viewed-history spillover.
- `/api/library-items` supports listing and manually adding works to the Library.
- The review page can create an Academic Peer Review element review as a `ReviewEvent` with a matching `ReviewEventMembership`.
- Creating an element review automatically adds the target audit object to Library with `added_reason = 'review_created'`.
- Scholarly object list/detail/library status and counts now prefer active `review_events` over legacy review scaffolding.
- `/api/peer-review/problem-area-works` exposes a first problem-area browse adapter over Academic Peer Review CWE criteria plus free-text matching.
- `/browse` lets users browse works by peer-review problem area, using CWE nodes as the current browse facets.
- The UI now includes a Domains page and a sidebar domain context showing C-SQD as multi-domain epistemic audit infrastructure.

## Existing Code To Preserve

The current codebase has useful Academic Peer Review infrastructure:

- Rust workspace at the repo root.
- Rust API crate: `services/api`.
- Rust domain crate: `crates/domain`.
- Next.js web app: `apps/web`.
- PostgreSQL schema and migrations in `db/migrations`.
- Demo seed data in `db/seeds/000001_demo_data.sql`.
- Docker Postgres config: `infra/docker-compose.yml`.
- Local database setup script: `scripts/setup_db.sh`.
- DOI, arXiv, PubMed/PMC, and title retrieval repository layers.
- Article access and external-location logic.
- Article work/version grouping.
- Scholarly Search frontend flow.
- Library page and explicit library membership API.
- Scholarly object detail, viewer, and review-shell pages.
- Peer Review problem-area browse page.
- Domains page and active-domain sidebar framing.

These should remain the Academic Peer Review adapter while the backend grows the general audit infrastructure.

## Conceptual Model

Use these mappings when continuing the refactor:

- `ScholarlyObject` remains an Academic Peer Review adapter over the more general `AuditObject`.
- `scholarly_objects` should gradually become academic metadata plus compatibility views around `audit_objects`.
- `EvaluationFact` should become `ReviewEventPayload::ElementReview` where appropriate.
- `ReviewEpisode` should become `ReviewEvent` for most review workflow records.
- `ReviewAssignment` should become `ERSolicitation` plus append-only `SolicitationEvent`.
- `SynthesisReview` should become `ReviewEventPayload::SynthesisReview` plus `SynthesisSection`.
- `ErrorClaim` / bounty workflows should become `ReviewEvent` payload variants plus challenge/bounty relations.
- `scholarly_work_groups` / `scholarly_work_versions` should become the Academic Peer Review lineage/version adapter over `AuditObjectRelation`.

Do not treat these as mere label changes. The new schema changes where provenance, state, review membership, and derived status live.

Also keep UI concepts separate from domain concepts:

- Domain concepts: audit objects, review events, review modes, solicitations, synthesis, challenges, evaluation tuples.
- UI surfaces: Scholarly Search, Library, Assignments, review workspaces, domain dashboards.

Library is cross-domain user workspace infrastructure. Scholarly Search is an Academic Peer Review surface, not a general C-SQD primitive.

For now, Academic Peer Review problem areas are represented by CWE nodes. This is a practical browse adapter, not a complete research-topic ontology. Future topic clustering can sit beside the CWE taxonomy, but CWE criteria remain the review/evaluation facets.

## Product/UI Direction

The Domains page should remain the place where the broader C-SQD substrate is legible:

- Active domains should be backed by real `domain_instantiations`.
- Planned domains can be shown as planned but should not pretend to be operational.
- Domain cards should use one shared grammar: audit objects, review modes, shared primitives, evaluation basis, and live surfaces.
- Academic Peer Review can list "Browse, Scholarly Search, Library, Assignments" only under "Live surfaces".
- Clinical Trial Protocol Review, AI System Auditing, and Policy Evidence Review should remain planned until they have real schema/config/adapter work.

For future navigation:

- Library should remain global but eventually filter by active domain by default.
- Active domain context should be visible throughout domain-specific pages.
- The app should avoid making planned domains clickable into empty fake workflows.

## Next Implementation Order

1. Add read-side `ReviewEvent` APIs and UI surfaces so users can see actual element reviews on scholarly object detail, review workspace, and problem-area browse.
2. Replace the remaining review status derivation from legacy `review_episodes` / `review_assignments` with `ReviewEvent`, `ERSolicitation`, and `Challenge`-derived status.
3. Replace mutable assignment state with `ERSolicitation` plus append-only `SolicitationEvent`.
4. Add the first evaluation tuple computation endpoint for Academic Peer Review.
5. Generalize the problem-area browse/query adapter into domain-scoped object search abstractions so future domains can define their own search/intake surfaces.
6. Make Library and audit-object APIs user-aware instead of fixed to the demo user.
7. Add domain-aware counts to the Domains page from real `audit_objects`, `review_events`, and library rows.
8. Add first synthesis-review creation/read surfaces after element-review reads are stable.
9. Seed planned domains only when their `DomainConfig`, audit object types, and CWE base taxonomy are meaningful.

## Immediate Product Slice

Build review-event visibility for Academic Peer Review.

Target behavior:

- Add a `GET` endpoint for review events attached to an audit object / scholarly object.
- Return typed `ReviewEventSummary` records with CWE criterion, finding, severity, confidence, status, provenance, and occurred-at time.
- Show recent element reviews on the scholarly object detail page.
- Show existing element reviews in the review workspace, grouped by peer-review criterion where useful.
- Let problem-area browse distinguish works with direct criterion-linked review activity from text-only matches.
- Keep object status derived from active review events, not by mutating legacy state.
- Legacy review tables remain compatibility scaffolding until the new read/write path is stable.

This slice makes the new ontology visible to users, not only present in the database.

## Backend Transition Notes

The old and new models will coexist for a while. Prefer additive migration and compatibility views over destructive refactors.

Important constraints:

- `AuditObject` is the durable reviewed artifact.
- `ReviewEvent` is the atomic, timestamped, provenance-bearing evaluative act.
- `ReviewEventMembership` records how a review event attaches to one or more audit objects.
- `ERSolicitation` and `SolicitationEvent` should become the source of truth for paid review assignment lifecycle.
- Evaluation tuple `E(A | R, Teval) -> (N, M, S, L, U)` is a derived view, not stored state.
- Rights-aware article retrieval must not copy or embed unauthorized PDFs.

## Important Local Ports

Use these local defaults:

- Web app: `http://localhost:3000`
- API: `http://localhost:8080`
- Repo-managed Docker Postgres: `localhost:55432`

Do not assume `localhost:5432` is available. On this machine, `5432` has been occupied by an unrelated local Postgres instance.

If default app ports are occupied, use fallback demo ports:

- Web app: `http://localhost:3001` or another open 300x port.
- API: `http://localhost:18080` or another open high port.

Recent local demos have used:

- Web app: `http://127.0.0.1:3012`
- API: `http://127.0.0.1:18081`

## Verification

After backend changes, run:

```sh
cargo fmt --all
cargo test --workspace
cargo check --workspace
npm run build:web
```

If local Postgres/Docker permissions allow it, also run:

```sh
scripts/setup_db.sh
CSQD_API_PORT=18080 DATABASE_URL=postgres://csqd:csqd@localhost:55432/csqd cargo run -p csqd-api
curl -s http://127.0.0.1:18080/health
curl -s http://127.0.0.1:18080/api/domain-instantiations
curl -s http://127.0.0.1:18080/api/audit-objects
curl -s http://127.0.0.1:18080/api/library-items
curl -s 'http://127.0.0.1:18080/api/peer-review/problem-area-works?cwe_node_id=00000000-0000-0000-0000-000000000602'
```

For frontend/domain UI changes, verify:

```sh
npm run build:web
curl -s http://127.0.0.1:3000/domains
curl -s http://127.0.0.1:3000/browse
curl -s http://127.0.0.1:3000/library
curl -s http://127.0.0.1:3000/
```

Use the actual running port if the default web port is occupied.

## Architecture Decisions To Preserve

Follow `build_decisions.md` unless it conflicts with the FEN schema. Where it conflicts, the FEN schema wins.

Important commitments:

- backend: Rust + `axum`
- database: PostgreSQL
- DB access: `sqlx`
- frontend: Next.js + TypeScript + React
- Academic Peer Review starts as the first domain instantiation
- article retrieval remains rights-aware and does not copy unauthorized PDFs
- payments remain provider-agnostic through an internal ledger plus adapters
- search uses C-SQD/FEN semantics with database/search projections
- Rust modules use the modern layout with no `mod.rs` files
