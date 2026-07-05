# C-SQD Web App

C-SQD is being built as **general epistemic audit infrastructure** for public and commissioned audits of important scientific and technical claims.

The MVP now presents first as a public audit registry and method. Visitors can discover scholarly works, inspect public audit activity, read SynthesisReviews, browse ElementReviews by CRWE criterion, and commission deeper work. Sponsor, reviewer, and operator workflows remain backstage behind identity and role state.

## Current Stack

- Backend: Rust + `axum`
- Frontend: Next.js + TypeScript + React
- Database: PostgreSQL
- Local database runtime: Docker Compose
- Database access: `sqlx`

## Local Shape

- Web app: `http://localhost:3000`
- API: `http://localhost:8080`
- PostgreSQL: `localhost:55432`

The database intentionally uses `55432` to avoid colliding with a normal local Postgres on `5432`.

## First-Time Setup

Install project dependencies:

```sh
npm install
cargo fetch
```

Start PostgreSQL and apply migrations:

```sh
scripts/setup_db.sh
```

To rebuild the local database from scratch:

```sh
scripts/setup_db.sh --reset
```

Run the API:

```sh
npm run dev:api
```

Run the web app:

```sh
npm run dev:web
```

Open:

```text
http://localhost:3000
```

## Useful Routes

- `/` briefing homepage (lead audit report + activity rails)
- `/discover` search + filtered discovery of public audit subjects
- `/audits` delivered public audit reports, review depth, contested audits
- `/works/:id` public audit subject page ("full coverage")
- `/works/:id/review` ElementReview / SynthesisReview submission
- `/register` external metadata search + audit subject registration
- `/criteria` active domain criterion taxonomy (CRWE) reference
- `/method` C-SQD method explainer
- `/commission` public path for commissioning a deeper audit
- `/domains` C-SQD domain overview
- `/audit-episodes/:id` authenticated episode workspace gate
- `/sponsor-console` authenticated sponsor console gate
- `/reviewer-queue` authenticated reviewer queue gate
- `/operations` authenticated operations gate
- `/library` authenticated library/watchlist gate

## Key Documents

- `C_SQD_NEW_GTM.pdf`: current go-to-market strategy.
- `FEN_for_CSQD_GTM.pdf`: rendered current FEN schema / GTM ontology.
- `FEN_Schema_for_CSQD_GTM.tex`: source for the current FEN schema.
- `interpretation.md`: working interpretation and source precedence.
- `NEXT_STEPS.md`: current engineering roadmap.
- `build_decisions.md`: stack and architecture decisions; partly MVP-era, now superseded by `interpretation.md`, `NEXT_STEPS.md`, and the new GTM/FEN documents for ontology and roadmap.
- `README_FOR_EUNICE.md`: concise onboarding guide for new collaborators.

Older documents, including `CSQD_NEW.pdf`, `FEN_Schema_for_CSQD.pdf`, and MVP documents in `old_mvp_docs/`, are useful history but no longer authoritative when they conflict with the GTM/FEN documents above.

## Verification

```sh
cargo fmt --all
cargo test --workspace
cargo check --workspace
npm run build:web
```

For a quick API smoke test:

```sh
curl http://localhost:8080/health
curl http://localhost:8080/api/domain-instantiations
curl http://localhost:8080/api/audit-episodes
```
