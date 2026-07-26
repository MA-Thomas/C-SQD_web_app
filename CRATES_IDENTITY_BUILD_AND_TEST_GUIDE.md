# Building and Testing `crates/identity`

Audience: future ChatGPT/Codex sessions implementing the C-SQD identity architecture.

Architectural contract: [`CSQD_IDENTITY_ARCHITECTURE.md`](CSQD_IDENTITY_ARCHITECTURE.md).

Reference implementation: `IDENTITY/identity-model`.

The reference implementation is design and code source material. Do not move it into the Rust workspace or copy it wholesale. Build a smaller C-SQD-native crate that uses the existing C-SQD FEN model, UUIDs, `chrono` timestamps, Axum API, SQLx repositories, PostgreSQL migrations, and application conventions.

## 1. Target Outcome

Create a pure Rust workspace crate:

```text
crates/identity/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs
    ├── events.rs
    ├── projection.rs
    ├── policy.rs
    └── error.rs
```

Likely later additions:

```text
services/api/src/repositories/identity.rs
services/api/src/routes/identity.rs
db/migrations/000005_identity_authority.sql
apps/web/app/components/...
```

The crate should own identity and authorization rules. SQL queries, HTTP details, cookies, email delivery, and web UI remain in their existing layers.

The first production-capable slice must support:

- a durable human principal linked to an existing account;
- direct sponsorship by a human acting in their own capacity;
- organization principals linked to existing organizations;
- sponsorship by an organization through an authorized human representative;
- verified and self-asserted identity claims;
- organization membership;
- scoped authority grants and revocations;
- deterministic current-state projection;
- authorization decisions using action, resource, grant, validity, and session assurance;
- compatibility with current global roles while routes are migrated;
- provenance references to the sponsor, actor, optional represented organization, authority grant, and policy decision.

It must not initially implement:

- biometrics or selfie liveness;
- identity documents;
- patient, payer, clinical, or HIPAA concepts;
- iOS App Attest;
- a second HTTP runtime;
- a second general-purpose FEN fact graph;
- mandatory Keycloak deployment;
- encryption before there is sensitive evidence to encrypt.

## 2. Rules for Every Future Session

Before changing code:

1. Read this guide and `CSQD_IDENTITY_ARCHITECTURE.md`.
2. Read `README.md`, `NEXT_STEPS.md`, `interpretation.md`, and any applicable `AGENTS.md`.
3. Inspect `git status` and preserve unrelated user changes.
4. Inspect the current identity implementation status and recent migrations.
5. Run or record the baseline checks before attributing failures to new work.
6. Keep the session limited to one milestone below unless the user explicitly broadens scope.

During implementation:

- Use `apply_patch` for source edits.
- Never rewrite an applied migration. Add a new numbered migration.
- Keep the identity crate independent of Axum, React, and direct database access.
- Avoid circular crate dependencies.
- Prefer typed identifiers, actions, scopes, statuses, and policy outcomes over strings.
- Keep authorization enforcement on the server. Frontend gates are explanatory only.
- Do not expose private identity evidence in public API types.
- Add tests with every new rule or state transition.
- Preserve current login behavior until its replacement is complete and verified.
- Do not silently interpret legacy roles as stronger authority than they currently provide.
- Do not collect identity data merely because the reference model supports it.

At the end of every session:

1. Run the milestone's required checks.
2. Inspect the final diff.
3. Update `IDENTITY_IMPLEMENTATION_STATUS.md` once that file exists.
4. Record schema or policy decisions that future sessions must not rediscover.
5. Report incomplete work and known failures precisely.
6. Do not commit, push, or open a pull request unless the user asks.

## 3. Known Baseline Conditions

Future sessions should not confuse these existing issues with identity regressions:

- The production web build passes.
- The Rust workspace tests pass, with several existing unused-function warnings in the API.
- The separate CI TypeScript command currently uses an incorrect workspace-relative path.
- The current `next lint` script opens an interactive configuration prompt.
- `scripts/smoke_test.sh` currently sends an invalid `other` organization enum and an empty CWE criterion list. The application flow passes when valid values are supplied.
- `IDENTITY/identity-model` cannot build in place because it is nested under the C-SQD workspace but is not a workspace member.
- The reference crate passes its isolated all-features test suite; some live-provider and live-PostgreSQL tests are environment-gated.

Unless the user asks otherwise, identity implementation should not bundle repairs to these unrelated baseline items into the same change.

## 4. Dependency Direction

Recommended dependency direction:

```text
crates/domain ────────┐
                     ├──> crates/identity
                     └──> services/api
crates/identity ─────────> services/api
```

`crates/domain` may define shared typed IDs needed by audit provenance, such as:

- `IdentityPrincipalId`
- `IdentityAssertionId`
- `OrganizationMembershipId`
- `AuthorityGrantId`
- `AccessDecisionId`
- `PolicyId`

`crates/identity` may depend on `csqd-domain` for those shared IDs, `Timestamp`, and existing organization or episode identifiers. `csqd-domain` must not depend on `csqd-identity`; otherwise the workspace will acquire a cycle.

If sharing the existing domain types would force identity-specific behavior into `crates/domain`, define identity-owned IDs in `crates/identity` and use their UUID wire representation at the provenance boundary. Do not create a new common-types crate unless the actual dependency graph requires it.

Initial crate dependencies should remain small:

- `chrono`
- `serde`
- `serde_json` only if structured policy metadata genuinely requires it
- `uuid`
- `csqd-domain` if the dependency direction above remains acyclic

Do not add SQLx, Axum, Reqwest, cryptography, or an OIDC library to the pure identity crate during the first milestones.

## 5. Proposed Domain Types

The exact names may change, but the model should cover the following semantics.

### Principals and account links

```rust
pub enum IdentityPrincipalKind {
    Human,
    Organization,
    SystemAgent,
}

pub enum IdentityPrincipalStatus {
    Active,
    Disputed,
    Superseded { by: IdentityPrincipalId },
    Deactivated,
}

pub struct AccountPrincipalLink {
    pub account_id: UserId,
    pub principal_id: IdentityPrincipalId,
    pub status: LinkStatus,
    pub established_by: Principal,
    pub established_at: Timestamp,
}
```

One account should normally resolve to one active human principal. Do not assume email is the durable principal identifier.

### Assertions and assurance

```rust
pub enum IdentityAssertionKind {
    VerifiedEmail,
    OidcSubject,
    Orcid,
    InstitutionalAffiliation,
    OrganizationMembership,
    ReviewerExpertise,
    ConflictDisclosure,
    AuthenticatorAssertion,
    Other(String),
}

pub enum AssuranceLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}
```

Assertions should distinguish:

- the subject principal;
- the claim;
- who asserted or verified it;
- evidence or external reference;
- assurance;
- validity period;
- active, superseded, revoked, disputed, or expired state.

### Sponsoring parties

```rust
pub enum SponsoringParty {
    Individual(IdentityPrincipalId),
    Organization(IdentityPrincipalId),
}

pub enum SponsorVisibility {
    Named,
    Generic,
    Confidential,
}
```

The sponsor is the principal that owns or funds the commission. The actor is the authenticated human who performs the commissioning action. For personal sponsorship they are the same principal. For organization sponsorship they are different, and the actor must hold an active grant to represent the organization.

Current C-SQD already permits a user or organization in the general `Principal` and `AuditCommission.commissioned_by` types. The organization-only restriction is in `CommissionAuditEpisodeRequest` and the repository flow, which always create or select an organization and currently overload the episode's `authored_by` value as the sponsor. Session 6 should generalize that boundary while keeping existing organization-sponsored facts readable.

### Authority

```rust
pub enum AuthorityKind {
    PlatformOperator,
    OrganizationAdministrator,
    OrganizationRepresentative,
    SponsorRepresentative,
    EpisodeSponsor,
    EpisodeReviewer,
    SynthesisAuthor,
    EpisodeOperator,
    Observer,
}

pub enum ResourceScope {
    Platform,
    Domain(DomainInstantiationId),
    Organization(OrganizationId),
    AuditSubject(AuditSubjectId),
    AuditEpisode(AuditEpisodeId),
    SynthesisReview(SynthesisReviewId),
}

pub enum AuthorizedAction {
    CommissionAudit,
    ManageOrganizationMembers,
    ViewSponsoredAudit,
    AcceptReviewAssignment,
    SubmitElementReview,
    SubmitSynthesisReview,
    ViewConfidentialEvidence,
    PublishSynthesisReview,
    RecordInvoice,
    RecordPayment,
    RecordReviewerPayout,
    ManageAccounts,
    GrantAuthority,
    RevokeAuthority,
    ExportPrivateAudit,
}
```

An authority grant should carry:

- actor principal;
- optional represented organization;
- kind;
- scope;
- permitted actions;
- issuer principal;
- issued time;
- optional expiration;
- evidence references;
- status derived from grant and revocation events.

### Authorization decisions

```rust
pub enum AuthorizationOutcome {
    Allowed,
    Denied,
    StepUpRequired,
    ManualReviewRequired,
}
```

Every decision should be explainable through stable reason codes. Human-readable messages are presentation, not policy logic.

## 6. Persistence Shape

The first persistence migration should be additive and reversible through normal forward migrations. Suggested tables:

- `identity_principals`
- `account_principal_links`
- `organization_principal_links`
- `identity_assertions`
- `organization_memberships`
- `audit_episode_sponsorships`
- `authority_grants`
- `authority_revocations`
- `identity_access_decisions`

Recommended properties:

- UUID primary keys;
- `timestamptz` timestamps;
- foreign keys to existing `users`, `organizations`, domains, subjects, and episodes where appropriate;
- check constraints for closed labels;
- partial or compound indexes for active-grant lookup;
- immutable grant and revocation records;
- explicit issuer and provenance fields;
- distinct sponsor and actor principal references;
- a nullable represented-organization and authority-grant reference;
- named, generic, or confidential public-attribution policy;
- no raw credential tokens;
- no identity documents or biometric data.

The repository may materialize active grants with SQL, but the Rust projection and policy functions remain the canonical behavioral specification.

### Legacy backfill

Backfill must be explicit and idempotent:

1. Create a human identity principal for each existing user.
2. Link each account to its human principal.
3. Create an organization principal for each existing organization.
4. Link organization business records to their identity principals.
5. Backfill existing organization-sponsored commissions as organization sponsorships, preserving their current sponsor principal and marking the source as a legacy backfill.
6. Convert legacy global roles into transitional authority grants:
   - `operator` → platform operator;
   - `sponsor` → compatibility sponsor authority with clearly limited semantics;
   - `reviewer` → reviewer eligibility, not automatic access to every episode;
   - `member` → no privileged authority.
7. Mark all generated records with a legacy-backfill provenance source.

Do not delete `users.roles` in the same migration. Remove it only after all server authorization paths use the new model and a later migration has been reviewed.

## 7. Implementation Sessions

Each session below should end in a coherent, testable state.

### Session 0 — Reconfirm architecture and integration seams

Deliverables:

- inspect current source and migrations for changes since this guide;
- write `IDENTITY_IMPLEMENTATION_STATUS.md`;
- record final names for crate, IDs, actions, scopes, and migration tables;
- confirm dependency direction;
- identify every current `require_role` call and classify its intended future policy.

No production behavior changes.

Exit criteria:

- an approved mapping from each privileged API action to a typed action and resource scope;
- no unresolved crate dependency cycle;
- legacy compatibility behavior documented.

### Session 1 — Scaffold the pure crate

Deliverables:

- add `crates/identity` with package name `csqd-identity`;
- add it to the root workspace;
- add model IDs and enums;
- implement validation constructors for non-empty or otherwise constrained values;
- add serialization boundary tests;
- document public modules.

Tests:

- every enum has stable snake-case serialization;
- typed IDs cannot be interchanged;
- invalid empty labels or invalid validity periods are rejected;
- no SQLx or Axum dependency appears in the crate.

Exit criteria:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test -p csqd-identity
```

### Session 2 — Events and deterministic projections

Deliverables:

- define append-only identity, assertion, membership, grant, and revocation events;
- implement projection into current identity and authority state;
- implement as-of-time projection;
- define supersession, expiration, dispute, and revocation precedence.

Required invariants:

- revoked grants never authorize later actions;
- expired grants are inactive at the evaluation time;
- superseded assertions do not contribute to current assurance;
- duplicate event replay is rejected or deterministically deduplicated;
- event order is explicit and stable;
- projection never depends on wall-clock time implicitly.

Tests:

- grant then revoke;
- grant then expire;
- assertion then supersede;
- disputed account-principal link;
- two organization memberships with different validity windows;
- replay produces the same projection;
- as-of projection reconstructs historical authority.

### Session 3 — Policy evaluation

Deliverables:

- define typed policy input and output;
- evaluate action, resource scope, authority, represented organization, and session assurance;
- return stable reason codes;
- implement initial policies for current C-SQD actions.

Minimum policy matrix:

| Action | Required authority | Suggested assurance |
|---|---|---|
| Register public audit subject | authenticated member | low |
| Commission personally | active human principal acting for self | medium |
| Commission on behalf of organization | sponsor representative for organization | medium |
| View sponsored private episode | personal episode sponsorship, episode grant, or sponsoring-organization authority | medium |
| Submit assigned review | active episode reviewer grant | medium |
| Publish SynthesisReview | episode operator or platform operator | high/recent |
| Record invoice/payment/payout | episode operator or platform operator | high/recent |
| Manage accounts | platform operator | high/recent |
| Grant operator authority | platform operator plus step-up | high/recent |
| View confidential evidence | explicit episode grant | policy-defined |

Tests must cover allowed, denied, expired, revoked, wrong organization, wrong episode, insufficient assurance, step-up-required, and manual-review outcomes.

### Session 3A — Policy hardening before persistence

This milestone is required before Session 4 so the Rust projection and policy
remain a safe canonical specification for the database implementation.

Deliverables:

- treat creation, establishment, assertion, sponsorship, and issuance timestamps
  as effective-time lower bounds;
- add the one-to-one organization-business-record to organization-principal link
  to the pure event and projection model;
- reject organization grants whose business-record scope does not match that
  link;
- validate supersession as semantic lineage for the same account, subject,
  mechanism, or organization relationship;
- replace authority-kind-only mutation inputs with a complete target actor,
  represented organization, kind, and scope;
- prevent self-escalation and prevent organization authority from creating
  platform authority;
- persist typed authorization bases together with the account, represented
  organization, authentication method, assurance, and authentication time;
- validate decision provenance during replay;
- use validated parameter structs for large domain constructors and keep
  invariant-bearing grant, sponsorship, and decision fields crate-private.

Required regression tests:

- future-issued authority never authorizes;
- organization scopes cannot cross linked business records;
- unrelated and self-referential supersession fail;
- authority mutation respects target kind and scope;
- missing or mismatched decision bases fail replay;
- every authorized action has an allowed-path test where applicable and a
  fail-closed test;
- valid policy decisions convert to replayable audit records.

### Session 4 — PostgreSQL migration and repository

Deliverables:

- add the next numbered migration without modifying older migrations;
- implement SQLx repository functions in the API layer;
- use transactions for multi-row grants, revocations, and backfills;
- implement active-grant queries and event replay;
- add database integration tests against an isolated test database.

Tests:

- migration applies to a clean database;
- migration applies to a database containing current seeds;
- backfill is idempotent;
- foreign keys and check constraints reject invalid state;
- duplicate active account links are prevented;
- revocation and grant creation are atomic;
- active-grant query agrees with the Rust projection;
- rollback leaves no partial authority state.

Do not use the developer's normal database for destructive migration tests. Use a disposable database or transaction-isolated test schema.

### Session 5 — Link current authentication to principals

Deliverables:

- resolve every authenticated session to an active human principal;
- include principal ID, authentication method, authentication time, and assurance in server-side session context;
- add “sign out all sessions” or session enumeration/revocation if not already present;
- preserve magic-link login behavior;
- reject suspended, deactivated, disputed, or unlinked identities according to policy.

Compatibility:

- existing accounts are backfilled and can still sign in;
- the frontend session payload changes only when corresponding types and consumers are updated;
- no route relies on frontend role gates for security.

Security tests:

- expired and revoked sessions fail;
- deactivated accounts fail;
- a session cannot select another account's principal;
- an unverified email does not become verified identity evidence;
- raw tokens never appear in database reads, logs, or API responses outside local dev-auth behavior.

### Session 6 — Individual sponsorship, organizations, and sponsor authority

Deliverables:

- introduce a `SponsoringParty` boundary that accepts either the authenticated human principal or an organization principal;
- support personal commissioning without creating an organization;
- link organization business records to organization identity principals;
- implement organization invitations and membership acceptance;
- implement organization administrator and sponsor representative grants;
- require organization authority only when the human commissions on behalf of an organization;
- create episode-scoped `EpisodeSponsor` authority for both personal and organization sponsorship;
- record sponsor principal, actor principal, optional represented organization, authority grant, and sponsor visibility;
- derive sponsor-console content from personal sponsorship, organization authority, and episode authority;
- preserve existing organization-sponsored commissions and API representations during migration.

Tests:

- an authenticated person can commission personally;
- personal sponsorship does not create an organization record;
- personal sponsorship records the same principal as sponsor and actor;
- a personal sponsor receives access only to that person's sponsored episodes;
- invited but unaccepted member cannot act;
- active representative can commission for the correct organization;
- representative cannot commission for another organization;
- revoked representative loses future access;
- an organization-sponsored commission records both human actor and represented organization;
- an individual cannot claim to represent an organization without active authority;
- named, generic, and confidential sponsor visibility produce the correct public projection;
- invoices and payments accept either an individual or organization sponsor principal;
- legacy organization-sponsored episodes continue to deserialize and display correctly;
- organization administrator can grant only permitted organization-scoped authority;
- organization authority cannot create platform operator authority.

### Session 7 — Reviewer identity and episode assignments

Deliverables:

- represent reviewer eligibility separately from episode assignment;
- add ORCID, affiliation, expertise, and conflict assertions as needed;
- distinguish self-asserted and verified reviewer claims;
- create an episode-scoped reviewer grant on accepted assignment;
- require the grant for commissioned/private review submission;
- preserve the public unsolicited-review policy separately.

Tests:

- reviewer eligibility alone does not expose private episodes;
- assignment is limited to one episode;
- assignment expiration and revocation work;
- reviewer submission provenance cites the assignment grant;
- conflict status can deny, step up, or require manual review;
- public participation rules do not leak confidential episode data.

### Session 8 — Operator administration, sensitive actions, and provenance

Deliverables:

- replace global role checks for high-risk operations with policy evaluation;
- record access decisions for sensitive actions;
- add recent-authentication or step-up requirements;
- enrich audit provenance with identity and authority references;
- maintain last-operator and no-self-escalation protections.

Tests:

- operator cannot grant themselves new operator authority;
- the final active operator cannot be removed without a controlled replacement;
- old authentication requires step-up for sensitive actions;
- commercial facts record actor and authority basis;
- public provenance omits private session and credential fields;
- authorization denial does not append the protected business fact.

### Session 9 — OIDC and stronger authentication, only when required

This is optional for the first pilot.

Deliverables:

- provider-neutral OIDC configuration;
- issuer, audience, expiration, nonce, algorithm, and JWKS validation;
- link verified OIDC subjects to accounts and principals;
- map `amr` and `acr` evidence into C-SQD assurance policy;
- add passkey or MFA support for high-risk actions if selected.

Use the reference crate's OIDC and assurance tests as source material, but adapt them to the existing async runtime. Do not introduce blocking HTTP calls inside Axum request handling.

Tests:

- wrong issuer, audience, nonce, key, algorithm, or expiration is rejected;
- symmetric-algorithm downgrade is rejected;
- unverified email is not promoted;
- identity-provider account linking cannot take over an existing account;
- key rotation and JWKS refresh behavior are deterministic.

### Session 10 — Encrypted evidence, only when collection becomes necessary

Do not start this session merely to make the system appear sophisticated.

Prerequisites:

- a concrete identity-evidence workflow;
- data-retention requirements;
- key-management provider selection;
- incident and recovery procedures;
- privacy review.

Deliverables:

- envelope encryption for sensitive identity evidence;
- authenticated associated data binding ciphertext to principal, type, and version;
- managed key identifiers and rotation;
- materialization policy evaluation before decryption;
- immutable materialization audit;
- deletion or cryptographic-erasure policy where legally required.

Tests:

- wrong key, retired key, changed associated data, and tampered ciphertext fail closed;
- nonce reuse is prevented;
- policy denial occurs before key access;
- materialization is audited without logging plaintext;
- key rotation preserves authorized historical reads;
- backups and restores retain encryption metadata.

## 8. API Integration Pattern

Routes should eventually use a shared authorization context:

```rust
pub struct AuthorizationContext {
    pub account_id: UserId,
    pub actor_principal_id: IdentityPrincipalId,
    pub represented_organization_id: Option<OrganizationId>,
    pub authentication_assurance: AssuranceLevel,
    pub authenticated_at: Timestamp,
}
```

Conceptual route flow:

```text
cookie/session
    -> authenticated account
    -> active principal link
    -> authorization context
    -> typed action + resource
    -> policy evaluation
    -> allow / deny / step-up / manual review
    -> business transaction with provenance
```

For state-changing sensitive actions, the access decision and business fact should be committed atomically when practical. A denial must never append the protected business fact.

Avoid replacing `require_role` everywhere at once. Migrate route families in coherent slices:

1. account administration;
2. organizations and commissions;
3. reviewer assignment and private review;
4. commercial actions;
5. confidential evidence and report publication.

## 9. Provenance Integration

Do not replace C-SQD's existing `Principal` enum prematurely. Introduce an additive provenance context or new principal variant only after serialization and seeded-data compatibility are understood.

Possible additive structure:

```rust
pub struct AuthorizationProvenance {
    pub sponsor_principal_id: Option<IdentityPrincipalId>,
    pub actor_principal_id: IdentityPrincipalId,
    pub account_id: UserId,
    pub represented_organization_id: Option<OrganizationId>,
    pub authority_grant_id: Option<AuthorityGrantId>,
    pub access_decision_id: Option<AccessDecisionId>,
    pub authentication_assurance: AssuranceLevel,
    pub policy_ref: Option<String>,
}
```

Requirements:

- old facts continue to deserialize;
- new provenance is optional during migration;
- public serializers expose only approved fields;
- no cookie, raw token, OIDC token, session identifier, or private evidence appears in fact payloads;
- sponsor, actor, and represented organization remain distinguishable;
- personal sponsorship records the same human principal as sponsor and actor without fabricating an organization.

## 10. Test Strategy

### Pure crate tests

Fast, deterministic, and exhaustive:

- serialization contracts;
- type validation;
- event replay;
- historical projection;
- authority scope matching;
- policy matrix;
- expiration and revocation;
- reason-code stability;
- no implicit system time.

### Property and invariant tests

Add targeted generative tests if complexity warrants them:

- revocation never increases authority;
- reducing assurance never changes a denial into an allow;
- adding an unrelated grant does not authorize another resource;
- replay order is deterministic for an explicit append sequence;
- public projection never contains private fields.

### Database integration tests

Use disposable PostgreSQL state:

- clean migration;
- upgrade from current schema;
- backfill;
- transaction rollback;
- uniqueness and foreign keys;
- projection/query equivalence;
- concurrent grant or invitation acceptance.

### API tests

Exercise authentication and authorization together:

- unauthenticated;
- authenticated but unlinked;
- linked without authority;
- individual acting on their own behalf;
- correct authority;
- wrong organization;
- wrong episode;
- revoked and expired authority;
- insufficient assurance;
- operator override only where policy explicitly permits it.

### End-to-end smoke coverage

Once the existing smoke script is repaired, extend it rather than creating multiple competing scripts. It should eventually exercise:

1. sign in;
2. account-to-principal resolution;
3. personal sponsorship and commission creation;
4. personal sponsor-console access;
5. organization invitation and acceptance;
6. organization sponsor authority and commission creation;
7. reviewer assignment and acceptance;
8. authorized review submission;
9. unauthorized cross-episode rejection;
10. operator-sensitive action and step-up;
11. named, generic, and confidential public sponsor projections;
12. public provenance projection.

Smoke tests must use generated identities and clean up through a disposable database or a documented reset. They must not depend on hardcoded seed UUIDs when the API can discover required identifiers.

## 11. Verification Commands

Run the smallest relevant set during development, followed by the full set before completing a milestone.

Pure crate:

```sh
cargo fmt --all -- --check
cargo check -p csqd-identity
cargo test -p csqd-identity
```

Rust workspace:

```sh
cargo check --workspace
cargo test --workspace
```

Web type checking with the currently correct local path:

```sh
npx --workspace apps/web tsc --noEmit -p tsconfig.json
```

Production web build:

```sh
npm run build:web
```

Database and API smoke checks should run only after the project PostgreSQL container and current API binary are confirmed:

```sh
scripts/setup_db.sh
npm run dev:api
scripts/smoke_test.sh
```

Until the known smoke-test payload issue is repaired, record that baseline limitation rather than editing it incidentally during an identity milestone.

## 12. Security Review Checklist

Before declaring the identity layer pilot-ready:

- production dev-auth is disabled;
- magic links are actually delivered by the configured provider;
- cookies are `Secure`, `HttpOnly`, and appropriately `SameSite`;
- session rotation, expiration, revocation, and sign-out-all behavior are tested;
- login and invitation endpoints are rate-limited;
- no account discovery leaks through authentication responses;
- organization invitations are single-use and expire;
- identity-provider linking prevents account takeover;
- operator authority requires controlled bootstrap and strong authentication;
- grants are scoped, expiring where appropriate, and revocable;
- confidential episode access is tested across individuals and organizations;
- authorization occurs server-side on every protected action;
- logs contain identifiers and reason codes, not tokens or private evidence;
- public APIs return only public identity projections;
- data retention and account deactivation behavior are documented;
- backup, restore, and incident procedures include identity and authority data.

## 13. Definition of Done for the Initial Identity Program

The initial program is complete when:

- all current accounts resolve to durable human principals;
- all organizations resolve to organization principals;
- a human can sponsor directly without creating or joining an organization;
- organization sponsorship requires explicit authority from the represented organization;
- sponsor access is scoped to the sponsoring person or organization and the relevant episode;
- reviewer authority is episode-scoped;
- operator authority is explicit and auditable;
- grants can expire and be revoked;
- protected routes evaluate typed policies on the server;
- high-risk actions can require step-up;
- audit provenance distinguishes sponsor, actor, and optional represented organization;
- legacy roles are no longer the authoritative permission source;
- public identity projection is privacy-safe;
- clean migration, upgrade migration, Rust workspace, web build, API authorization, and end-to-end tests pass;
- production configuration and operational procedures are documented.

## 14. Handoff Template for Future Sessions

Each session should leave a short entry in `IDENTITY_IMPLEMENTATION_STATUS.md`:

```md
## YYYY-MM-DD — Session N: milestone name

### Completed
- ...

### Decisions
- ...

### Files and migrations
- ...

### Verification
- command: pass/fail

### Known limitations
- ...

### Recommended next session
- ...
```

Future sessions should trust verified status entries but still inspect the current code and Git state. If implementation and this guide diverge, update the guide or record a deliberate architectural decision rather than allowing undocumented drift.
