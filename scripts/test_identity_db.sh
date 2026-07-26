#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.yml"
TEST_ADMIN_URL="${CSQD_TEST_DATABASE_ADMIN_URL:-postgresql://csqd:csqd@localhost:55432/postgres}"

cd "$ROOT_DIR"
docker compose -f "$COMPOSE_FILE" up -d postgres

for attempt in {1..60}; do
  if psql "$TEST_ADMIN_URL" -v ON_ERROR_STOP=1 -q -c "SELECT 1;" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then
    echo "PostgreSQL did not become ready in time." >&2
    exit 1
  fi
  sleep 1
done

CSQD_TEST_DATABASE_ADMIN_URL="$TEST_ADMIN_URL" \
  cargo test -p csqd-api --test identity_postgres -- --ignored
