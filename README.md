# C-SQD Web App

C-SQD is being built as **general epistemic audit infrastructure** for commissioned, decomposed audits of important scientific and technical claims.

The current app still contains useful Academic Publishing / Scholarly Search infrastructure, but the active GTM direction is no longer "build a better journal." The first product wedge is sponsored epistemic audits: organizations commission audits of claims, papers, models, datasets, reports, and related artifacts; reviewers complete scoped ElementReviews; synthesis authors integrate those facts into audit-level interpretations.

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

- `/` Audit Console for commissioned audit episodes
- `/commission` sponsor/scope form for commissioning an audit
- `/audit-episodes/:id` episode workspace for facts and scoped element reviews
- `/intake` Scholarly Intake / Academic Publishing metadata adapter
- `/browse` Academic Publishing problem-area browse
- `/domains` C-SQD domain overview
- `/library` saved audit subjects

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
