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

Local setup applies the demo seeds (example audits, sponsors, reports) so
the app is browsable during development. **Pilot and production databases
must run clean:**

```sh
scripts/setup_db.sh --reset --no-seeds
```

If a shared environment does carry seed data, set `NEXT_PUBLIC_DEMO_MODE=1`
so every public page is banner-labeled as demonstration data.

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

- `C-SQD_NEW_GTM.tex`: current go-to-market strategy.
- `FEN_Schema_for_CSQD_GTM.tex`: canonical FEN schema / GTM ontology source.
- `CLAIM_SCOPED_AUDITS_MEMO.md`: claim-scoped academic audit clarification.
- `interpretation.md`: working interpretation and source precedence.
- `NEXT_STEPS.md`: current engineering roadmap.
- `build_decisions.md`: stack and architecture decisions; partly MVP-era, now superseded by `interpretation.md`, `NEXT_STEPS.md`, and the new GTM/FEN documents for ontology and roadmap.
- `README_FOR_EUNICE.md`: concise onboarding guide for new collaborators.

Older rendered PDFs and MVP documents in `old_mvp_docs/` are useful history but no longer authoritative when they conflict with the GTM/FEN sources above.

## Auth Modes And Deployment Flags

Local development defaults to **dev-auth mode**: magic sign-in links are
returned in the API response (and shown on the sign-in page) instead of being
emailed. In any deployment other people can reach, set:

```sh
CSQD_DEV_AUTH=0        # links go to the API log only, until email delivery is wired
CSQD_SECURE_COOKIES=1  # Secure session cookies (HTTPS deployments)
CSQD_API_BIND=0.0.0.0  # container/hosted bind address (default: 127.0.0.1)
```

Write endpoints (subject registration, commissioning, external article
retrieval, library) require a signed-in session. The library is scoped to the
session user. Magic-link issuance is throttled per address (3 per 15-minute
window).

Email delivery (magic links, inquiry/solicitation/review notifications) is
provider-agnostic: set `CSQD_EMAIL_PROVIDER` (resend or postmark),
`CSQD_EMAIL_API_KEY`, and `CSQD_EMAIL_FROM`. Without a provider, outbound
mail is logged, so all flows stay exercisable locally.

Container builds live in `infra/Dockerfile.api` and `infra/Dockerfile.web`;
CI (format, check, test, type-check, lint, build) runs via
`.github/workflows/ci.yml`.

## Commercial Model

Commissioning is two-stage. `/commission` shows a public inquiry form
(stage one — recorded in `commission_inquiries`, triaged in Operations);
the full scoped commission form appears for signed-in users (stage two).
Money movement is recorded as facts on the audit record — `invoice_issued`,
`payment_received`, `reviewer_payout` — via the episode workspace's
commercial panel (operator-only). An episode counts as **funded** when an
active `payment_received` fact exists: a derived view, like the evaluation
tuple, which deliberately ignores all commercial facts.

Role grants (sponsor/reviewer/operator) and account display names are
managed in Operations → Accounts and `/account` respectively.

## Smoke Test

With the local stack running (dev-auth mode):

```sh
scripts/smoke_test.sh
```

exercises health → auth → registration → commission → inquiry → eval tuple
→ public summary → operator gating, end to end.

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
