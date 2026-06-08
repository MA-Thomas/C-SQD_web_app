# C-SQD Web App

C-SQD is being built as a web-native review graph and scholarly evaluation marketplace.

The first codebase milestone is the local development spine:

- Rust API service
- Next.js web app
- PostgreSQL schema and seed data
- provider-agnostic payment records
- F-E-N-native evaluation/search structure

## Local Shape

- Web app: `http://localhost:3000`
- API: `http://localhost:8080`
- PostgreSQL: `localhost:55432`

## First-Time Setup

Install dependencies:

```sh
npm install
cargo fetch
```

Start PostgreSQL:

```sh
scripts/setup_db.sh
```

To rebuild the local database from scratch, run:

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

The initial web app can render seeded/demo scholarly objects even before live ingestion and payment processing are connected.

## Documents

- `interpretation.md` reconciles the source PDFs.
- `build_decisions.md` records stack and architecture decisions.
- `C_SQD_web_app_build_plan.pdf` describes the MVP plan.
