# README For Eunice

This repo contains the C-SQD web app: a Next.js frontend, Rust API, and local PostgreSQL database for public and commissioned epistemic audit infrastructure.

The current MVP presents C-SQD first as a public audit registry and method for claim-scoped audits. Academic Peer Review is the first implemented adapter and discovery surface; sponsor, reviewer, and operations workflows still exist as backstage surfaces behind identity and role state.

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

## 5. View A Local Live Version

After the database setup finishes, use two terminal windows from the repo root.

Leave both terminals open while viewing the site.

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

The local site is live while those two commands are running. When frontend files change, the browser usually refreshes automatically. If the page shows API errors, check that Terminal 1 is still running.

Useful pages:

- `/` public registry home
- `/claims` claim audit index
- `/claims/:id` scoped claim audit page
- `/discover` directed discovery of public audit records and works
- `/audits` delivered public SynthesisReviews and ElementReview depth
- `/method` C-SQD method explainer
- `/commission` commission a deeper audit
- `/register` Search / Register scholarly work metadata adapter
- `/criteria` CRWE criterion reference
- `/works/:id` public work page and audit involvement surface
- `/works/:id/review` ElementReview submission for a work
- `/audit-episodes/:id` authenticated episode workspace gate
- `/sponsor-console` authenticated sponsor console gate
- `/reviewer-queue` authenticated reviewer queue gate
- `/operations` authenticated operations gate
- `/library` authenticated library/watchlist gate
- `/domains` C-SQD domain overview

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

- `C-SQD_NEW_GTM.tex`: current go-to-market strategy.
- `FEN_Schema_for_CSQD_GTM.tex`: canonical FEN schema / GTM ontology source.
- `CLAIM_SCOPED_AUDITS_MEMO.md`: claim-scoped academic audit clarification.
- `NEXT_STEPS.md`: current engineering roadmap.
- `build_decisions.md`: stack and architecture decisions.
- `interpretation.md`: notes connecting earlier source materials.

Older rendered PDFs and MVP docs live in:

```text
old_mvp_docs/
```

They are useful history, but the project has moved beyond the narrow MVP.

## 8. Repo Map

```text
apps/web/          Next.js frontend
services/api/      Rust API service
crates/domain/     Shared Rust domain types
crates/academic-adapter/
                   Academic publishing adapter types
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
curl http://localhost:8080/api/audit-subjects
curl http://localhost:8080/api/claim-audits
```

## 10. Mental Model

C-SQD is not just an academic peer review app.

It is general commissioned epistemic audit infrastructure:

- **DomainInstantiation**: a configured audit domain.
- **AuditSubject**: referenced metadata for the scoped claim, claim-warrant bundle, or artifact-attached claim under evaluation.
- **Fact**: an atomic epistemic or administrative act.
- **Evidence artifact**: a paper or work attached to an audit episode for inspection; attachment is neutral and does not count as support.
- **AuditEpisode**: a coherent commissioned audit question over time.
- **EpisodeMembership**: the provenance-bearing link between a fact and an episode.
- **SynthesisReview**: an authored interpretation of an audit episode.
- **Evaluation tuple**: a derived view of scrutiny and uptake.

Academic Publishing is the first implemented adapter. Papers remain searchable and linkable, but they are usually evidence surfaces rather than the audit's epistemic target. Future domains should reuse the same FEN infrastructure.
