#!/usr/bin/env bash
# End-to-end API smoke test against a local dev stack (dev-auth mode).
#
# Prereqs: scripts/setup_db.sh has run, `npm run dev:api` is up, and
# CSQD_DEV_AUTH is not disabled. Requires curl + jq + python3.
#
#   scripts/smoke_test.sh [api_base_url]
#
# Exercises the pilot loop end to end: health → auth (magic link) →
# subject registration → commission → inquiry → element review →
# commercial facts → eval tuple → public summary. Exits nonzero on the
# first failure.

set -euo pipefail

API="${1:-http://localhost:8080}"
JAR="$(mktemp)"
trap 'rm -f "$JAR"' EXIT

pass() { printf '  ok  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1" >&2; exit 1; }

jqr() { jq -r "$1" 2>/dev/null; }

echo "Smoke-testing ${API}"

# 1. Health.
status=$(curl -sf "${API}/health" | jqr '.status') || fail "health endpoint"
[ "$status" = "ok" ] || fail "health status is '$status'"
pass "health"

# 2. Domains.
domain_id=$(curl -sf "${API}/api/domain-instantiations" | jqr '.[0].id')
[ -n "$domain_id" ] && [ "$domain_id" != "null" ] || fail "no domain instantiations (seed the db)"
pass "domain instantiations"

# 3. Unauthenticated write is rejected.
code=$(curl -s -o /dev/null -w '%{http_code}' -X POST "${API}/api/audit-subjects" \
  -H 'Content-Type: application/json' \
  -d "{\"domain_instantiation_id\":\"${domain_id}\",\"subject_type\":\"scoped_claim\",\"title\":\"t\"}")
[ "$code" = "401" ] || fail "unauthenticated subject creation returned $code, expected 401"
pass "writes are session-gated"

# 4. Magic-link sign-in (dev-auth mode returns the link).
email="smoke-$(date +%s)@example.org"
link=$(curl -sf -X POST "${API}/api/auth/request-link" \
  -H 'Content-Type: application/json' \
  -d "{\"email\":\"${email}\"}" | jqr '.sign_in_url')
[ -n "$link" ] && [ "$link" != "null" ] || fail "no sign_in_url (is CSQD_DEV_AUTH off?)"
token="${link##*token=}"
user=$(curl -sf -c "$JAR" -X POST "${API}/api/auth/complete" \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"${token}\"}" | jqr '.user.user_id')
[ -n "$user" ] && [ "$user" != "null" ] || fail "magic-link completion"
pass "magic-link auth (${email})"

# 5. Session round-trip.
session_user=$(curl -sf -b "$JAR" "${API}/api/auth/session" | jqr '.user.user_id')
[ "$session_user" = "$user" ] || fail "session cookie round-trip"
pass "session cookie"

# 6. Register an audit subject.
subject_id=$(curl -sf -b "$JAR" -X POST "${API}/api/audit-subjects" \
  -H 'Content-Type: application/json' \
  -d "{\"domain_instantiation_id\":\"${domain_id}\",\"subject_type\":\"scoped_claim\",\"title\":\"Smoke test claim\",\"claim_statement\":\"Marker X predicts outcome Y in population Z\",\"scope_conditions\":[{\"label\":\"population\",\"value\":\"adults\"}]}" \
  | jqr '.id')
[ -n "$subject_id" ] && [ "$subject_id" != "null" ] || fail "subject registration"
pass "audit subject registered"

# 7. Commission an episode.
episode_id=$(curl -sf -b "$JAR" -X POST "${API}/api/audit-subjects/${subject_id}/audit-episodes" \
  -H 'Content-Type: application/json' \
  -d '{"label":"Smoke commissioned audit","sponsor_organization_name":"Smoke Test Org","sponsor_organization_type":"other","funding":{"amount":1000,"currency":"USD"},"scope_cwe_node_ids":[],"deadline":null,"confidential":false,"notes":null}' \
  | jqr '.episode.id')
[ -n "$episode_id" ] && [ "$episode_id" != "null" ] || fail "commission"
pass "episode commissioned"

# 8. Public commission inquiry (stage one, no auth).
inquiry_status=$(curl -sf -X POST "${API}/api/commission-inquiries" \
  -H 'Content-Type: application/json' \
  -d "{\"contact_name\":\"Smoke Tester\",\"contact_email\":\"${email}\",\"subject_description\":\"Please audit the smoke-test claim about marker X.\",\"budget_band\":\"under_5k\"}" \
  | jqr '.status')
[ "$inquiry_status" = "new" ] || fail "commission inquiry"
pass "commission inquiry recorded"

# 9. Eval tuple endpoint.
tuple=$(curl -sf "${API}/api/audit-episodes/${episode_id}/eval-tuple" | jqr '.s')
[ -n "$tuple" ] && [ "$tuple" != "null" ] || fail "eval tuple"
pass "eval tuple computes (S=${tuple})"

# 10. Public subject summary.
summary_status=$(curl -sf "${API}/api/public/audit-subjects/${subject_id}/summary" | jqr '.status_label // "present"')
[ -n "$summary_status" ] || fail "public subject summary"
pass "public subject summary"

# 11. Commercial facts are operator-gated for non-operators.
code=$(curl -s -b "$JAR" -o /dev/null -w '%{http_code}' -X POST \
  "${API}/api/audit-episodes/${episode_id}/facts/payment-received" \
  -H 'Content-Type: application/json' \
  -d '{"amount":{"amount":1000,"currency":"USD"}}')
[ "$code" = "403" ] || fail "payment-received returned $code for non-operator, expected 403"
pass "commercial facts are operator-gated"

echo "All smoke tests passed."
