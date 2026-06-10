# README For Eunice

This repo contains the C-SQD web app: a Next.js frontend, Rust API, and local PostgreSQL database for commissioned epistemic audit infrastructure.

The current app still has an Academic Publishing / Scholarly Search adapter, but the current GTM direction is broader: organizations commission decomposed audits of claims, papers, models, datasets, reports, and other artifacts.

## 1. Clone The Repo

```sh
git clone <repo-url>
cd C-SQD_web_app
```

Replace `<repo-url>` with the GitHub URL Marcus gives you.

## 2. Install Local Tools

Required:

- **Git**: clone and manage the repo.
- **Docker Desktop**: run the local PostgreSQL database.
- **Node.js + npm**: run the web app.
- **Rust / cargo**: run the backend API.
- **PostgreSQL client tools**: provide `psql` for database setup.

Optional:

- **Playwright**: useful for browser/UI smoke tests later. It is not required for the basic setup right now.

On macOS, Homebrew is the simplest path for most tools:

```sh
brew install node rustup postgresql@16
rustup-init
```

Install Docker Desktop separately, then open it before setting up the database.

## 3. Install Project Dependencies

From the repo root:

```sh
npm install
cargo fetch
```

## 4. Start The Database

```sh
scripts/setup_db.sh
```

This starts the Dockerized PostgreSQL database, applies migrations, and loads demo data.

To reset the local database:

```sh
scripts/setup_db.sh --reset
```

Database URL:

```text
postgresql://csqd:csqd@localhost:55432/csqd
```

The repo uses port `55432` to avoid colliding with a normal local Postgres on `5432`.

## 5. Run The App

Use two terminals.

Terminal 1, backend API:

```sh
npm run dev:api
```

Terminal 2, web app:

```sh
npm run dev:web
```

Open:

```text
http://localhost:3000
```

Useful pages:

- `/` Audit Console for commissioned audit episodes
- `/commission` commission a scoped audit
- `/audit-episodes/:id` episode workspace for facts and element reviews
- `/intake` Scholarly Intake / Academic Publishing metadata adapter
- `/browse` Academic Publishing problem-area browse
- `/domains` C-SQD domain overview
- `/library` saved audit subjects

API health check:

```sh
curl http://localhost:8080/health
```

## 6. If Ports Are Busy

Use alternate ports:

```sh
CSQD_API_PORT=18080 npm run dev:api
NEXT_PUBLIC_API_BASE_URL=http://localhost:18080 CSQD_WEB_PORT=3001 npm run dev:web
```

Then open:

```text
http://localhost:3001
```

## 7. Core Documents

Read these first:

- `C_SQD_NEW_GTM.pdf`: current go-to-market strategy.
- `FEN_for_CSQD_GTM.pdf`: rendered current FEN schema / GTM ontology.
- `FEN_Schema_for_CSQD_GTM.tex`: source for the current FEN schema.
- `NEXT_STEPS.md`: current engineering roadmap.
- `build_decisions.md`: stack and architecture decisions.
- `interpretation.md`: notes connecting earlier source materials.

Older documents, including the previous `CSQD_NEW.pdf`, `FEN_Schema_for_CSQD.pdf`, and MVP docs, live in:

```text
old_mvp_docs/
```

They are useful history, but the project has moved beyond the narrow MVP.

## 8. Repo Map

```text
apps/web/          Next.js frontend
services/api/      Rust API service
crates/domain/     Shared Rust domain types
db/migrations/     Database migrations
db/seeds/          Demo seed data
infra/             Docker Compose config
scripts/           Local setup scripts
images_and_logos/  Logo/source visual assets
```

## 9. Before Handing Off Changes

Run:

```sh
cargo fmt --all
cargo test --workspace
cargo check --workspace
npm run build:web
```

For a quick database/API smoke test:

```sh
scripts/setup_db.sh
curl http://localhost:8080/api/domain-instantiations
curl http://localhost:8080/api/audit-objects
```

## 10. Mental Model

C-SQD is not just an academic peer review app.

It is general commissioned epistemic audit infrastructure:

- **DomainInstantiation**: a configured audit domain.
- **AuditSubject**: referenced metadata for the artifact or claim being evaluated.
- **Fact**: an atomic epistemic or administrative act.
- **AuditEpisode**: a coherent commissioned audit question over time.
- **EpisodeMembership**: the provenance-bearing link between a fact and an episode.
- **SynthesisReview**: an authored interpretation of an audit episode.
- **Evaluation tuple**: a derived view of scrutiny and uptake.

Academic Publishing is the first implemented adapter. Future domains should reuse the same FEN infrastructure.
