#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATABASE_URL="${DATABASE_URL:-postgresql://csqd:csqd@localhost:55432/csqd}"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.yml"
MIGRATION_FILE="$ROOT_DIR/db/migrations/000001_initial_schema.sql"
MIGRATIONS_DIR="$ROOT_DIR/db/migrations"
SEEDS_DIR="$ROOT_DIR/db/seeds"
RESET_DB=false

if [[ "${1:-}" == "--reset" ]]; then
  RESET_DB=true
elif [[ $# -gt 0 ]]; then
  echo "Usage: scripts/setup_db.sh [--reset]" >&2
  exit 2
fi

for command in docker psql; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

cd "$ROOT_DIR"

echo "Starting repo-managed PostgreSQL..."
docker compose -f "$COMPOSE_FILE" up -d postgres

echo "Waiting for PostgreSQL on localhost:55432..."
for attempt in {1..60}; do
  if psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c "SELECT 1;" >/dev/null 2>&1; then
    break
  fi

  if [[ "$attempt" -eq 60 ]]; then
    echo "PostgreSQL did not become ready in time." >&2
    exit 1
  fi

  sleep 1
done

if [[ "$RESET_DB" == true ]]; then
  echo "Resetting local database schema..."
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
fi

new_schema_exists="$(
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
    "SELECT
       to_regclass('public.audit_subjects') IS NOT NULL
       AND to_regclass('public.facts') IS NOT NULL
       AND to_regclass('public.audit_episodes') IS NOT NULL
       AND to_regclass('public.episode_memberships') IS NOT NULL;"
)"

legacy_schema_exists="$(
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
    "SELECT
       to_regclass('public.audit_objects') IS NOT NULL
       OR to_regclass('public.review_events') IS NOT NULL
       OR to_regclass('public.review_assignments') IS NOT NULL
       OR to_regclass('public.evaluation_facts') IS NOT NULL
       OR to_regclass('public.bounties') IS NOT NULL
       OR to_regclass('public.challenges') IS NOT NULL;"
)"

any_schema_exists="$(
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
    "SELECT
       to_regclass('public.users') IS NOT NULL
       OR to_regclass('public.scholarly_objects') IS NOT NULL;"
)"

if [[ "$RESET_DB" != true && "$legacy_schema_exists" == "t" ]]; then
  echo "Legacy MVP schema tables detected. Run scripts/setup_db.sh --reset for the clean GTM schema." >&2
  exit 1
fi

if [[ "$RESET_DB" != true && "$any_schema_exists" == "t" && "$new_schema_exists" != "t" ]]; then
  echo "Existing schema is not the clean GTM schema. Run scripts/setup_db.sh --reset." >&2
  exit 1
fi

if [[ "$new_schema_exists" == "t" ]]; then
  echo "Schema already exists; skipping migration."
else
  echo "Applying initial schema..."
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$MIGRATION_FILE"
fi

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c \
  "CREATE TABLE IF NOT EXISTS schema_migrations (
      version text PRIMARY KEY,
      applied_at timestamptz NOT NULL DEFAULT now()
  );"

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c \
  "INSERT INTO schema_migrations (version)
   VALUES ('000001_initial_schema.sql')
   ON CONFLICT (version) DO NOTHING;"

for migration in "$MIGRATIONS_DIR"/*.sql; do
  version="$(basename "$migration")"

  if [[ "$version" == "000001_initial_schema.sql" ]]; then
    continue
  fi

  already_applied="$(
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
      "SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE version = '$version');"
  )"

  if [[ "$already_applied" == "t" ]]; then
    echo "Migration $version already applied; skipping."
  else
    echo "Applying migration $version..."
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$migration"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c \
      "INSERT INTO schema_migrations (version) VALUES ('$version');"
  fi
done

# Seed files are idempotent (every statement uses ON CONFLICT), so apply all of
# them in lexical order on every run. New db/seeds/*.sql files are picked up
# automatically.
for seed in "$SEEDS_DIR"/*.sql; do
  [[ -e "$seed" ]] || continue
  echo "Applying seed $(basename "$seed")..."
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$seed"
done

echo "Local database is ready at $DATABASE_URL"
