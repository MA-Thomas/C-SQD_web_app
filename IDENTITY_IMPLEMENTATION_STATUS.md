# C-SQD Identity Implementation Status

This file records completed identity milestones and the decisions future sessions must preserve. The architectural contract is `CSQD_IDENTITY_ARCHITECTURE.md`; the implementation sequence is `CRATES_IDENTITY_BUILD_AND_TEST_GUIDE.md`.

## 2026-07-26 — Session 1: pure workspace crate and typed model

### Completed

- Added the `csqd-identity` package at `crates/identity`.
- Added it to the root Cargo workspace.
- Kept the crate pure: it has no Axum, SQLx, HTTP, cookie, UI, provider, or cryptography dependencies.
- Added shared identity identifier newtypes to `csqd-domain` so the dependency remains one-way: `csqd-identity` depends on `csqd-domain`, never the reverse.
- Added typed models for:
  - human, organization, system-agent, and device principals;
  - account-to-principal links;
  - authentication identities and methods;
  - identity assertions and assurance levels;
  - validity periods;
  - organization membership;
  - individual and organization sponsorship;
  - sponsor visibility;
  - authority grants and revocations;
  - resource scopes and authorized actions;
  - authorization outcomes and access decisions.
- Added validation constructors for required labels, evidence references, validity periods, sponsorship invariants, non-empty authority actions, revocation reasons, and access-decision reason codes.
- Added stable snake-case serialization tests and a compile-fail test proving that typed identity IDs cannot be interchanged.

### Decisions

- Shared identity IDs live in `csqd-domain::ids`; behavioral identity rules live in `csqd-identity`.
- IDs remain string-backed, serde-transparent newtypes to match current C-SQD database and wire conventions.
- `Timestamp` and existing audit entity IDs come from `csqd-domain`.
- Personal sponsorship records the same human principal as sponsor and actor and carries no organization authority.
- Organization sponsorship requires a distinct human actor, a represented organization principal, and an authority grant.
- Credential secrets and raw authentication tokens are explicitly outside the identity model.
- Database persistence, event replay, projections, and policy evaluation are deferred to their documented milestones.

### Files and migrations

- Modified `Cargo.toml`.
- Modified `Cargo.lock`.
- Modified `crates/domain/src/ids.rs`.
- Modified `crates/domain/src/lib.rs`.
- Added `crates/identity/Cargo.toml`.
- Added `crates/identity/src/lib.rs`.
- Added `crates/identity/src/error.rs`.
- Added `crates/identity/src/model.rs`.
- No database migration was added.
- The reference `IDENTITY/` directory was not modified.

### Verification

- `cargo fmt --all -- --check`: pass.
- `cargo check -p csqd-identity`: pass.
- `cargo test -p csqd-identity`: pass — 11 unit tests and 1 compile-fail documentation test.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`: pass.
- `cargo check --workspace`: pass with the five existing API dead-code warnings.
- `cargo test --workspace`: pass — 53 unit tests plus the identity compile-fail documentation test.

A strict clippy run including dependencies reaches the pre-existing `clippy::large_enum_variant` warning in `csqd-domain::TimelineEntry`. Identity code itself passes strict clippy with `--no-deps`.

### Known limitations

- Model constructors validate normal creation paths, but persistence-boundary validation does not exist yet.
- Principal kinds are not yet resolved from a repository, so constructors cannot prove that a given principal ID points to a human or organization record.
- Authority grants are modeled but are not yet projected, evaluated, persisted, or enforced.
- Existing accounts, roles, organizations, commissions, and sessions are not connected to the new crate.
- No application behavior or database schema changed in this milestone.

### Recommended next session

- Session 2: add append-only identity events, deterministic current-state projection, as-of-time projection, revocation and expiration precedence, and replay-invariant tests.

## 2026-07-26 — Session 2: append-only events and deterministic projections

### Completed

- Added a typed, append-only identity event envelope with:
  - a distinct `IdentityEventId`;
  - an explicit append sequence;
  - a recorded timestamp and recording principal;
  - a closed, tagged payload vocabulary for principals, account links, authentication identities, assertions, memberships, sponsorships, grants, revocations, and access decisions.
- Added persistence-boundary validation so deserialized or directly constructed event payloads are rechecked before replay.
- Added a deterministic in-memory projection for current identity and authority state.
- Added an as-of projection that replays only the ledger prefix recorded at or before the caller-supplied timestamp.
- Added time-explicit queries for:
  - active account-to-principal links;
  - active assertions;
  - active organization memberships;
  - active authority grants;
  - sponsorships by sponsoring principal.
- Enforced replay invariants for:
  - unique event IDs and append sequences;
  - monotonic recorded timestamps in append order;
  - unique entity IDs;
  - valid status transitions and supersession targets;
  - principal existence and expected human or organization kind;
  - one active principal link per account;
  - organization sponsorship grants matching the represented organization and human actor;
  - existing revoking principals and one revocation per authority grant.
- Added integration tests for revocation, expiration, assertion supersession, disputed account links, membership validity windows, deterministic shuffled replay, duplicate ledger records, unknown targets, non-monotonic timestamps, and historical reconstruction.

### Decisions

- `append_sequence` is the canonical replay order. Input slices may arrive shuffled; projection sorts them before applying events.
- `recorded_at` describes ledger inclusion. As-of projection is therefore a reproducible recorded-time prefix, not an implicit call to the current clock.
- Validity windows use an inclusive start and exclusive end.
- Revocation is a separate immutable record. A grant is inactive at and after its revocation's effective timestamp.
- Supersession, dispute, and deactivation are explicit events. Current projection never silently deletes the prior record.
- Superseded and revoked states are terminal. Disputed principals, links, and assertions may transition to a different state through a later explicit event.
- Historical questions should use `project_identity_state_at`; querying a current projection with an earlier effective time does not rewind later lifecycle events.

### Files and migrations

- Added `crates/identity/src/events.rs`.
- Added `crates/identity/src/projection.rs`.
- Added `crates/identity/tests/projection.rs`.
- Modified `crates/identity/src/lib.rs`.
- Modified `crates/identity/src/model.rs`.
- Modified `crates/identity/src/error.rs`.
- Modified `crates/domain/src/ids.rs`.
- Modified `crates/domain/src/lib.rs`.
- No database migration was added.
- The reference `IDENTITY/` directory was not modified.

### Verification

- `cargo fmt --all -- --check`: pass.
- `cargo check -p csqd-identity`: pass.
- `cargo test -p csqd-identity`: pass — 14 unit tests, 7 projection integration tests, and 1 compile-fail documentation test.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`: pass.
- `cargo check --workspace`: pass with the five existing API dead-code warnings.
- `cargo test --workspace`: pass — 63 unit and integration tests plus the identity compile-fail documentation test.
- `git diff --check`: pass.

### Known limitations

- Events and projections are pure in-memory domain behavior; no database tables or repositories persist them yet.
- Projection queries identify active grants but do not yet evaluate a requested action against scope, represented organization, session assurance, or a versioned policy.
- Existing accounts, sessions, roles, organizations, commissions, and API authorization paths remain on the legacy model.
- Public identity and sponsor projections, including visibility redaction, are not implemented yet.
- No production behavior changed in this milestone.

### Recommended next session

- Session 3: add pure, typed policy inputs and outcomes; scope and action matching; represented-organization and assurance checks; stable reason codes; and the initial C-SQD authorization matrix.

## 2026-07-26 — Session 3: typed policy evaluation

### Completed

- Added a pure policy module with typed:
  - authorization context;
  - policy input;
  - versioned initial-policy configuration;
  - conflict status;
  - authorization basis;
  - policy decision;
  - stable reason codes;
  - invalid-input errors.
- Added `RegisterPublicAuditSubject` to the closed authorized-action vocabulary.
- Implemented the initial C-SQD policy matrix for:
  - registering a public audit subject;
  - commissioning personally;
  - commissioning for an organization;
  - viewing a personally or organizationally sponsored private episode;
  - accepting and submitting assigned review work;
  - submitting and publishing synthesis reviews;
  - managing organization members;
  - recording invoices, payments, and reviewer payouts;
  - managing accounts;
  - granting or revoking authority;
  - viewing confidential evidence;
  - exporting private audits.
- Evaluated account-to-principal linkage, active human-principal status, represented organization, grant kind, permitted action, resource scope, validity, revocation, session assurance, authentication age, and unresolved conflicts.
- Added a configurable high-risk authentication window with a 15-minute default and configurable confidential-evidence assurance.
- Added explicit authorization bases for authenticated-principal, personal-capacity, personal-sponsorship, and authority-grant decisions.
- Extended auditable `AccessDecision` records to retain the authority grant or personal sponsorship on which the policy decision relied.
- Added policy tests covering:
  - every documented allowed path in the minimum policy matrix;
  - personal and organization-sponsored episode access;
  - missing authority;
  - expired and revoked grants;
  - wrong organization and wrong episode;
  - insufficient assurance;
  - stale authentication;
  - operator-authority step-up;
  - unresolved-conflict manual review;
  - invalid policy input;
  - stable reason-code serialization and conversion to audit records.

### Decisions

- Policy evaluation accepts a caller-supplied evaluation timestamp and never reads the system clock.
- The application boundary resolves credentials into an `AuthorizationContext`; cookies, tokens, and session identifiers remain outside the crate.
- Account/principal mismatch and inactive principals fail closed before grant evaluation.
- Personal commissioning requires an active human principal and no fabricated organization or authority grant.
- Organization commissioning requires a matching active organization principal and sponsor-representative grant.
- Platform scope is transitive only for `PlatformOperator`; other grants require an exact scope unless the sponsoring-organization episode rule explicitly applies.
- Confidential evidence always requires an explicit grant for the exact audit episode. A platform-scoped grant does not satisfy this requirement.
- Revocation takes diagnostic precedence over expiration when the same structurally matching grant has both conditions.
- Insufficient or stale authentication produces `StepUpRequired` only after an otherwise valid authorization basis is found.
- An unresolved conflict produces `ManualReviewRequired` for review submission, synthesis publication, or confidential-evidence access after identity, authority, and assurance checks pass.
- Stable reason codes are persisted as strings in `AccessDecision`; human-readable messages remain outside policy logic.

### Files and migrations

- Added `crates/identity/src/policy.rs`.
- Added `crates/identity/tests/policy.rs`.
- Modified `crates/identity/src/lib.rs`.
- Modified `crates/identity/src/model.rs`.
- Modified `crates/identity/src/projection.rs`.
- No database migration was added.
- The reference `IDENTITY/` directory was not modified.

### Verification

- `cargo fmt --all -- --check`: pass.
- `cargo check -p csqd-identity`: pass.
- `cargo test -p csqd-identity`: pass — 14 unit tests, 7 policy integration tests, 7 projection integration tests, and 1 compile-fail documentation test.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`: pass.
- `cargo check --workspace`: pass with the five existing API dead-code warnings.
- `cargo test --workspace`: pass — 70 unit and integration tests plus the identity compile-fail documentation test.
- `git diff --check`: pass.

### Known limitations

- Policy evaluation is not yet called by API routes and its decisions are not persisted.
- The identity projection remains in memory; no PostgreSQL repository or migration exists yet.
- Granting or revoking authority identifies the target authority kind, but target-principal validation, self-grant prevention, and final-operator protection still require the later operator-administration integration slice.
- The policy does not infer domain, organization, subject, episode, or synthesis parentage. Callers must supply the intended authorization boundary explicitly.
- Organization business-record and organization-principal correspondence is not persisted yet.
- Existing role checks remain the production authorization mechanism.
- No production behavior changed in this milestone.

### Recommended next session

- Session 4: add the forward PostgreSQL migration, idempotent legacy backfill, SQLx identity repository, active-grant queries, and projection/query equivalence tests without removing legacy roles.

## 2026-07-26 — Session 3A: policy and projection hardening

### Completed

- Made entity creation, link establishment, assertion, membership, sponsorship,
  and grant issuance timestamps effective-time lower bounds.
- Added a one-to-one `OrganizationPrincipalLink` model and event projection.
- Required organization memberships, organization sponsorships, and
  organization-scoped grants to agree with the linked organization business
  record and principal.
- Replaced existence-only supersession checks with lineage validation:
  - principal replacements must be distinct active principals of the same kind;
  - account links must replace a link for the same account;
  - authentication identities must replace the same mechanism for the same
    account;
  - assertions must replace the same assertion kind for the same subject;
  - memberships must replace the same member and organization relationship;
  - replacement records must already be effective.
- Replaced `target_authority_kind` with a typed `AuthorityMutationTarget`
  containing the target actor, represented organization, authority kind, and
  resource scope.
- Added organization-administrator authority mutation, platform-only operator
  creation, episode-operator boundaries, and self-escalation denial.
- Replaced independent optional grant and sponsorship fields on
  `AccessDecision` with a typed `AuthorizationBasis`.
- Added account, represented organization, authentication method, assurance,
  and authentication time to auditable access decisions.
- Added replay validation for account/actor correspondence, represented
  organizations, personal sponsorship, authority grants, action, scope,
  effective time, and non-empty reason codes.
- Replaced the largest positional constructors with validated parameter
  structs and made invariant-bearing sponsorship, grant, and access-decision
  fields crate-private with read-only accessors.
- Expanded policy coverage so every authorized action has an allowed path where
  the initial policy permits one and a fail-closed unlinked-account case.

### Decisions

- A record without an explicit validity window becomes effective no earlier
  than its own creation, establishment, assertion, sponsorship, or issuance
  timestamp.
- Organization identity and organization business records are distinct but
  linked one-to-one; neither identifier may be treated as a wildcard for the
  other.
- Supersession represents semantic replacement within one lineage, not merely a
  pointer to any existing record.
- Organization administrators may grant or revoke approved
  organization-scoped authority for their own organization, but cannot create
  platform authority.
- Platform operator authority requires a platform-scoped platform-operator
  basis. Self-granting authority is denied.
- Final-active-operator protection still requires repository-backed knowledge
  of the complete active operator set and remains part of the later
  operator-administration integration slice.
- Denied access decisions carry no authorization basis. Allowed, step-up, and
  manual-review decisions require a validated basis.
- Access decisions are recorded at their evaluation timestamp so replay can
  validate their basis against the exact ledger prefix used by policy.

### Files and migrations

- Modified `crates/identity/src/model.rs`.
- Modified `crates/identity/src/events.rs`.
- Modified `crates/identity/src/projection.rs`.
- Modified `crates/identity/src/policy.rs`.
- Modified `crates/identity/src/error.rs`.
- Modified `crates/identity/src/lib.rs`.
- Modified `crates/identity/tests/projection.rs`.
- Modified `crates/identity/tests/policy.rs`.
- Modified `CSQD_IDENTITY_ARCHITECTURE.md`.
- Modified `CRATES_IDENTITY_BUILD_AND_TEST_GUIDE.md`.
- No database migration was added.
- The reference `IDENTITY/` directory was not modified.

### Verification

- `cargo fmt --all -- --check`: pass.
- `cargo check -p csqd-identity`: pass.
- `cargo test -p csqd-identity`: pass — 15 unit tests, 10 policy integration
  tests, 11 projection integration tests, and 1 compile-fail documentation
  test.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`: pass.
- `cargo check --workspace`: pass with the five existing API dead-code warnings.
- `cargo test --workspace`: pass — 78 unit and integration tests plus the
  identity compile-fail documentation test.
- `git diff --check`: pass.

### Known limitations

- Identity events and decisions remain in memory and are not persisted.
- API routes still use legacy role checks and do not call the identity policy.
- Final-active-operator protection is not implementable until the repository
  can atomically inspect and update the complete active operator set.
- Public identity and sponsor projections are not implemented.
- Validated parameter structs protect normal construction and event replay
  revalidates deserialized payloads; dedicated external wire DTOs may still be
  introduced when the SQLx/API boundaries are implemented.
- No production behavior changed.

### Recommended next session

- Session 4: add the forward PostgreSQL migration, idempotent legacy backfill,
  organization-principal links, SQLx identity repository, active-grant queries,
  and projection/query equivalence tests without removing legacy roles.

## 2026-07-26 — Session 4: PostgreSQL migration and repository

### Completed

- Added forward migration `000005_identity_persistence.sql` with:
  - append-only, explicitly sequenced identity events;
  - durable principals, account links, organization links, authentication
    identities, assertions, memberships, sponsorships, authority grants,
    revocations, and access decisions;
  - foreign keys, lifecycle checks, validity-window checks, and a partial
    unique index preventing duplicate active account links;
  - indexed relational fields plus the validated Rust record as JSON.
- Backfilled every current account to a human principal and active account
  link, and every organization to a one-to-one organization principal link.
- Backfilled legacy operator roles as platform-operator grants.
- Backfilled reviewer roles as low-assurance eligibility assertions rather
  than episode authority.
- Backfilled legacy sponsor roles as deliberately non-representational
  compatibility grants that cannot authorize an organization commission.
- Preserved legacy organization-sponsored episodes in sponsorship rows.
  Because the old commission flow did not store the authenticated human actor,
  those rows are marked `actor_attribution_required` and are not emitted as
  canonical sponsorship events.
- Kept `users.roles` and all current route behavior unchanged.
- Added an API SQLx repository with:
  - ordered event loading and deterministic replay;
  - transaction-scoped ledger serialization;
  - replay-before-commit validation;
  - atomic principal/link creation;
  - atomic grants, revocations, and grant replacement;
  - active-grant queries aligned with Rust effective-time rules;
  - validated access-decision persistence.
- Added disposable PostgreSQL integration tests and a one-command runner.
- Updated local database setup so clean seeded databases load demo source rows
  before applying the identity persistence migration.

### Decisions

- The event ledger is canonical for semantic replay. Relational tables are
  transactionally maintained query indexes and constraint boundaries.
- Database append-sequence gaps after rolled-back transactions are valid;
  sequence values define order, not event counts.
- Repository writes serialize on one PostgreSQL advisory transaction lock so a
  mutation validates against an unchanging ledger prefix.
- Missing legacy actor provenance is represented explicitly instead of being
  assigned to a guessed user or system principal.
- Reviewer eligibility is evidence, not authority to review any episode.
- Legacy roles remain a compatibility mechanism until route authorization
  cuts over in later sessions.

### Files and migrations

- Added `db/migrations/000005_identity_persistence.sql`.
- Added `services/api/src/repositories/identity.rs`.
- Added `services/api/src/lib.rs`.
- Added `services/api/tests/identity_postgres.rs`.
- Added `scripts/test_identity_db.sh`.
- Modified `services/api/Cargo.toml`.
- Modified `services/api/src/main.rs`.
- Modified `services/api/src/repositories.rs`.
- Modified `scripts/setup_db.sh`.
- Modified `CSQD_IDENTITY_ARCHITECTURE.md`.
- Modified `CRATES_IDENTITY_BUILD_AND_TEST_GUIDE.md`.
- The reference `IDENTITY/` directory was not modified.

### Verification

- Migration applied to a clean disposable PostgreSQL database: pass.
- Migration upgraded a database containing all current seeds: pass.
- Migration executed twice with unchanged identity row/event counts: pass.
- Disposable database integration tests: pass — 3 tests covering
  upgrade/idempotence/replay, constraints/duplicate links, active-query versus
  Rust projection equivalence, atomic replacement, and rollback.
- `cargo check -p csqd-api`: pass.
- `cargo fmt --all -- --check`: pass.
- `bash -n scripts/setup_db.sh scripts/test_identity_db.sh`: pass.
- `cargo check --workspace`: pass with two pre-existing API dead-code warnings.
- `cargo test --workspace`: pass — 78 unit and integration tests plus the
  identity compile-fail documentation test; the three PostgreSQL tests are
  ignored in the default run and passed separately against disposable
  databases.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`: pass.
- `cargo clippy -p csqd-api --all-targets --no-deps`: pass with four
  pre-existing API warnings and no warnings in the new identity repository.
- `git diff --check`: pass.

### Known limitations

- API routes still use legacy role checks and do not yet call the identity
  policy or repository.
- Existing authentication sessions are not yet resolved to identity
  principals; that is Session 5.
- Seven current demo organization-sponsored episodes have no recoverable human
  actor and remain explicit compatibility rows pending Session 6 attribution
  or replacement.
- Final-active-operator protection is not yet wired into an operator
  administration route.
- Public identity and sponsor projections are not implemented.

### Recommended next session

- Session 5: resolve authenticated sessions to active human principals, carry
  authentication method/time/assurance in server-side session context, and
  preserve magic-link behavior while keeping route authorization unchanged
  until each protected route is deliberately migrated.

## 2026-07-26 — Session 4A: Rust type and authorization hardening

### Completed

- Made denied decisions replayable when denial was caused by:
  - an unlinked or mismatched account;
  - an inactive principal;
  - an inactive or unlinked represented organization.
- Added one shared organization-commissioning grant predicate used by policy
  evaluation and sponsorship replay.
- Required organization sponsorship to cite an active
  `SponsorRepresentative` grant for the linked organization business record.
- Replaced the optional, partial authority-mutation target with a typed request
  carrying the complete proposed grant or grant-plus-revocation.
- Retained the complete authorization request in both `PolicyDecision` and
  `AccessDecision`.
- Replaced string access-decision reasons with `PolicyReasonCode`.
- Replaced the independent outcome and optional-basis fields with
  `AccessDecisionResult`, whose variants structurally enforce:
  - no basis for denials;
  - a basis for allowed, step-up, and manual-review outcomes.
- Added outcome-specific reason-code validation.
- Rejected duplicate permitted actions and blank custom authentication method
  labels.
- Removed the signed-to-unsigned authentication-age cast and exposed wrapped
  model errors through standard Rust error sources.
- Replaced the boolean-heavy grant diagnostic accumulator with a typed set of
  diagnostic flags.
- Replaced ambiguous denial provenance with `AuditedPrincipalReference` and
  `AuditedRepresentation`, which explicitly distinguish known, unresolved, and
  absent representation states.
- Defined the clean access-decision schema directly in
  `000005_identity_persistence.sql`: canonical actor and representation
  references are non-null JSON values, while dedicated relation tables contain
  only resolved principal foreign keys.

### Decisions

- Failed identity resolution is valid denial provenance, not a malformed access
  decision.
- The same grant predicate is canonical for policy authorization and event
  replay; persistence must not admit authority semantics the policy rejects.
- Authority-mutation decisions prove the exact proposed mutation, including
  permitted actions and validity, rather than authorizing only a kind/scope
  outline.
- Typed policy reasons are preserved through JSON and PostgreSQL persistence.
- Rust uses enum variants for absent and unresolved identity provenance; SQL
  `NULL` is not used to encode either state in access-decision references.
- String-backed shared IDs remain the repository-wide compatibility
  convention. Moving them to UUID-backed validated newtypes is deliberately
  deferred because it affects every domain crate, seed fixture, and wire
  boundary rather than only `csqd-identity`.

### Files and migrations

- Modified `crates/identity/src/error.rs`.
- Modified `crates/identity/src/events.rs`.
- Modified `crates/identity/src/lib.rs`.
- Modified `crates/identity/src/model.rs`.
- Modified `crates/identity/src/policy.rs`.
- Modified `crates/identity/src/projection.rs`.
- Modified `crates/identity/tests/policy.rs`.
- Modified `crates/identity/tests/projection.rs`.
- Updated the new, uncommitted
  `db/migrations/000005_identity_persistence.sql` directly rather than adding a
  transitional compatibility migration.
- Modified `services/api/src/repositories/identity.rs`.
- Modified `services/api/tests/identity_postgres.rs`.
- Modified `CSQD_IDENTITY_ARCHITECTURE.md`.
- Modified `CRATES_IDENTITY_BUILD_AND_TEST_GUIDE.md`.

### Verification

- `cargo test -p csqd-identity`: pass — 15 unit tests, 11 policy integration
  tests, 12 projection integration tests, and 1 compile-fail documentation
  test.
- `cargo clippy -p csqd-identity --all-targets --no-deps -- -D warnings`:
  pass.
- `cargo test --workspace`: pass — including 19 API tests, 23 domain tests,
  15 identity unit tests, 11 policy tests, 12 projection tests, and the
  compile-fail documentation test; 4 database tests remain ignored in this
  command by design.
- `scripts/test_identity_db.sh`: pass — all 4 disposable PostgreSQL migration,
  constraint, repository, rollback, and denial-provenance tests.
- `cargo clippy -p csqd-api --all-targets --no-deps`: pass with 4 pre-existing
  warnings outside the identity repository.

### Known limitations

- Shared IDs remain unchecked string newtypes until a separately reviewed
  repository-wide UUID boundary migration.
- API routes still use legacy role checks; this hardening changes the canonical
  identity model and repository serialization but does not cut over route
  authorization.

### Recommended next session

- Session 5 remains next: resolve authenticated sessions to active human
  principals and carry authentication method, time, and assurance in the
  server-side session context.
