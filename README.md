# C-SQD Web App

C-SQD is being built as **multi-domain epistemic audit infrastructure**.

The first active domain is **Academic Peer Review**. Current product surfaces include Scholarly Search, Library, Domains, Assignments, scholarly object detail pages, rights-aware native viewing, and an early review workspace.

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

- `/` Scholarly Search
- `/domains` C-SQD domain overview
- `/library` saved audit objects
- `/assignments` review assignments

## Key Documents

- `CSQD_NEW.pdf`: current C-SQD conceptual direction.
- `FEN_Schema_for_CSQD.pdf`: FEN/C-SQD schema framing.
- `interpretation.md`: working interpretation and source precedence.
- `NEXT_STEPS.md`: current engineering roadmap.
- `build_decisions.md`: stack and architecture decisions; partly MVP-era, now superseded by `interpretation.md` and `NEXT_STEPS.md` for ontology and roadmap.
- `README_FOR_EUNICE.md`: concise onboarding guide for new collaborators.

Older MVP documents live in `old_mvp_docs/`.

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
curl http://localhost:8080/api/audit-objects
```
