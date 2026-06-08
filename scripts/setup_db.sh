#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATABASE_URL="${DATABASE_URL:-postgresql://csqd:csqd@localhost:55432/csqd}"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.yml"
MIGRATION_FILE="$ROOT_DIR/db/migrations/000001_initial_schema.sql"
MIGRATIONS_DIR="$ROOT_DIR/db/migrations"
SEED_FILE="$ROOT_DIR/db/seeds/000001_demo_data.sql"
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

schema_exists="$(
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
    "SELECT to_regclass('public.scholarly_objects') IS NOT NULL;"
)"

if [[ "$schema_exists" == "t" ]]; then
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

seed_exists="$(
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -At -c \
    "SELECT EXISTS (SELECT 1 FROM scholarly_objects WHERE doi = '10.0000/csqd.demo.001');"
)"

if [[ "$seed_exists" == "t" ]]; then
  echo "Demo seed data already exists; skipping seed."
else
  echo "Applying demo seed data..."
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$SEED_FILE"
fi

echo "Local database is ready at $DATABASE_URL"
