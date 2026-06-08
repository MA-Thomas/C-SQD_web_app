# NEXT_STEPS

## Current Direction

C-SQD is pivoting from a narrow academic peer-review MVP to the more general epistemic audit infrastructure described in `CSQD_NEW.pdf` / `FEN_Schema_for_CSQD.pdf`.

Academic publishing remains the first product surface and first demo domain. The backend should now treat it as an `AcademicPublishing` `DomainInstantiation`, not as the root ontology of the platform.

The new architectural backbone is:

- `DomainInstantiation`: owns domain configuration, CWE taxonomy, phase rules, and evaluation tuple configuration.
- `AuditObject`: the durable reviewable artifact inside a domain.
- `ReviewEvent`: the atomic, timestamped, provenance-bearing evaluative act.
- `ReviewEventMembership`: the provenance-bearing assignment of a review event to an audit object.
- `ERSolicitation` and `SolicitationEvent`: the assignment and lifecycle log for paid element reviews.
- `SynthesisSection` and `SynthesisReviewRelation`: authored integrative interpretation structure and relations.
- `Challenge`: the schema surface for review challenges and featured-review replacement.
- Evaluation tuple `E(A | R, Teval) -> (N, M, S, L, U)`: a derived view, not stored state.

## Existing Code To Preserve

The current codebase has useful Academic Publishing infrastructure:

- Rust workspace at the repo root
- Rust API crate: `services/api`
- Rust domain crate: `crates/domain`
- Next.js web app: `apps/web`
- PostgreSQL schema and migrations in `db/migrations`
- Demo seed data in `db/seeds/000001_demo_data.sql`
- Docker Postgres config: `infra/docker-compose.yml`
- Local database setup script: `scripts/setup_db.sh`
- DOI, arXiv, PubMed/PMC, and title retrieval repository layers
- Article access and external-location logic
- Article work/version grouping
- Search-for-work frontend flow
- Scholarly object detail, viewer, and review-shell pages

These should be retained as the Academic Publishing adapter while the backend grows the general audit backbone.

## Conceptual Renames

Use these mappings when refactoring:

- `ScholarlyObject` -> `AuditObject` with Academic Publishing metadata.
- `scholarly_objects` -> `audit_objects` plus an academic metadata/compatibility layer.
- `EvaluationFact` -> `ReviewEventPayload::ElementReview` where appropriate.
- `ReviewEpisode` -> `ReviewEvent` for most review workflow records.
- `ReviewAssignment` -> `ERSolicitation` plus append-only `SolicitationEvent`.
- `SynthesisReview` table -> `ReviewEventPayload::SynthesisReview` plus `SynthesisSection`.
- `ErrorClaim` / bounty workflow -> `ReviewEvent` payload variants and challenge/bounty relations.
- `scholarly_work_groups` / `scholarly_work_versions` -> Academic Publishing lineage/version adapter over `AuditObjectRelation`.

Do not treat these as mere label changes. The new schema changes where provenance and state live.

## Implementation Order

1. Add Rust domain types for the general C-SQD schema.
2. Add PostgreSQL tables for the audit backbone.
3. Seed a default `AcademicPublishing` domain instantiation and base CWE nodes.
4. Mirror or migrate existing demo scholarly objects into `audit_objects`.
5. Add `/api/domain-instantiations` and `/api/audit-objects` endpoints.
6. Keep legacy `/api/scholarly-objects` endpoints working during the transition.
7. Adapt article retrieval so DOI/arXiv/PubMed/title imports create or update audit objects.
8. Replace mutable review-assignment state with `ERSolicitation` and `SolicitationEvent`.
9. Replace review status derivation from `review_episodes` with review-event, solicitation, and challenge-derived status.
10. Add a first evaluation tuple computation endpoint for Academic Publishing.

## Immediate Backend Target

The first implementation slice should produce a working local backend where:

- The database has the general audit backbone tables.
- A default Academic Publishing domain exists.
- The seeded demo work has a corresponding `AuditObject`.
- The API can list domain instantiations.
- The API can list and fetch audit objects.
- Existing article/retrieval routes still compile and run.

This gives the repo a new structural center without breaking the current demo surface.

## Important Local Ports

Use these local defaults:

- Web app: `http://localhost:3000`
- API: `http://localhost:8080`
- Repo-managed Docker Postgres: `localhost:55432`

Do not assume `localhost:5432` is available. On this machine, `5432` was already occupied by an unrelated local Postgres instance.

If default app ports are occupied, use fallback demo ports:

- Web app: `http://localhost:3001`
- API: `http://localhost:18080`

## Verification

After backend changes, run:

```sh
cargo test --workspace
cargo check --workspace
npm run build:web
```

If local Postgres/Docker permissions allow it, also run:

```sh
scripts/setup_db.sh --reset
CSQD_API_PORT=18080 DATABASE_URL=postgres://csqd:csqd@localhost:55432/csqd cargo run -p csqd-api
curl -s http://127.0.0.1:18080/health
curl -s http://127.0.0.1:18080/api/domain-instantiations
curl -s http://127.0.0.1:18080/api/audit-objects
```

## Architecture Decisions To Preserve

Follow `build_decisions.md` unless it conflicts with the new FEN schema. Where it conflicts, the new schema wins.

Important commitments:

- backend: Rust + `axum`
- database: PostgreSQL
- DB access: `sqlx`
- frontend: Next.js + TypeScript + React
- Academic Publishing starts as the first domain instantiation
- article retrieval remains rights-aware and does not copy unauthorized PDFs
- payments remain provider-agnostic through an internal ledger plus adapters
- search uses FEN/C-SQD semantics with database/search projections
- Rust modules use the modern layout with no `mod.rs` files

## Best Next Product Slice

After the backbone is in place, build the first true `ReviewEvent` workflow:

- Create an ElementReview draft for an AuditObject.
- Attach it through `ReviewEventMembership`.
- Record CWE criterion, finding, severity, text, reviewer, provenance, and featured flag.
- Show the resulting ReviewEvent count on the existing academic object page.

That slice proves the new ontology while keeping the user-facing demo grounded in academic review.
