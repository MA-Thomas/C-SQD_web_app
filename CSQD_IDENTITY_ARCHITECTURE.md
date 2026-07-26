# C-SQD Identity Architecture

Status: proposed architecture for a production-oriented identity and authorization layer.

This document maps the reusable ideas in `IDENTITY/identity-model` into C-SQD's sponsor, reviewer, operator, organization, audit-episode, and provenance workflows. It is intentionally smaller than the imported identity system. It does not propose importing that crate wholesale.

## 1. Purpose

C-SQD needs to distinguish three questions:

1. **Authentication:** who successfully signed in?
2. **Identity:** what is known about that actor, and how was it established?
3. **Authorization:** what may that actor do in this specific personal, organizational, and audit context?

The current magic-link and cookie-session implementation is a reasonable authentication foundation. The next identity layer should make organizational representation, reviewer standing, episode assignments, authority grants, revocations, and sensitive access decisions explicit and auditable.

The objective is not maximal identity proofing. It is sufficient, proportionate assurance for trustworthy scientific and technical audit work.

## 2. Design Principles

- **Accounts are not identities.** An account is a way to authenticate; an identity principal is the durable actor represented in provenance.
- **Roles are contextual.** Sponsor, reviewer, and operator describe authority in a context, not different kinds of people.
- **Humans and organizations are principals.** A human may sponsor an audit personally or may act on behalf of an organization through delegated authority.
- **Authority is explicit and revocable.** Grants carry scope, issuer, validity, evidence, and status.
- **Identity evidence is append-only.** Corrections supersede or revoke earlier assertions rather than silently rewriting history.
- **Public provenance is privacy-preserving.** Public audit records expose only the identity information necessary to interpret authorship and authority.
- **Sensitive actions require proportionate assurance.** Routine reading and participation should remain easy; publishing, payment recording, operator administration, and confidential-data access may require stronger authentication.
- **Identity state is derived.** Current affiliations, assurance, and permissions are projections over active evidence and authority events.
- **The identity graph remains private.** It may reference C-SQD audit entities, but it is not mixed into the public epistemic fact stream.

## 3. Bounded Contexts

### Authentication

Authentication verifies control of a credential and creates a session.

Initial mechanisms:

- production email magic links;
- secure, hashed cookie sessions;
- session expiration and revocation;
- optional OIDC for institutional sign-in;
- passkey or MFA step-up for operators and other high-risk actions.

Authentication should return an `AccountId`, credential method, authentication time, and assurance evidence. It should not decide episode-specific authority.

### Identity

Identity connects an authenticated account to a durable `IdentityPrincipal`.

Proposed principal kinds:

- `Human`
- `Organization`
- `SystemAgent`
- `Device` only if device-bound credentials later require it

Use the name `IdentityPrincipal`, not `Subject`, to avoid collision with C-SQD's `AuditSubject`.

Identity evidence may include:

- verified email;
- OIDC issuer and subject;
- ORCID identifier;
- institutional affiliation;
- organization administrator confirmation;
- reviewer credential or expertise verification;
- conflict-of-interest disclosure;
- passkey or MFA assertion;
- operator manual verification.

Each assertion records provenance, assurance, validity, and status.

### Authorization

Authorization evaluates active authority grants against an action and resource.

An `AuthorityGrant` should identify:

- actor principal;
- optional represented organization;
- authority kind;
- resource scope;
- permitted actions;
- issuer;
- issuance and expiration times;
- supporting evidence;
- active, revoked, expired, or superseded status.

An active human principal may commission an audit on their own behalf without an organization authority grant. Acting for an organization always requires explicit organizational authority.

An authorization decision should be reproducible from the grant, session
assurance, policy version, action, and resource. Grant and revocation decisions
retain the complete proposed mutation—including the target grant, permitted
actions, validity, and revocation record—rather than only its authority kind
and scope. Denials retain the attempted identity context even when the account,
principal, or represented organization did not resolve, because that failure
is the reason being audited.

## 4. Core Model

| Concept | C-SQD purpose |
|---|---|
| `Account` | Authentication record and account lifecycle; one human may have multiple authentication identities over time. |
| `AuthenticationIdentity` | Magic-link email, OIDC subject, or later passkey credential linked to an account. |
| `IdentityPrincipal` | Durable human, organization, or system actor used by provenance. |
| `AccountPrincipalLink` | Links an authenticated account to a human principal with status and evidence. |
| `OrganizationPrincipalLink` | One-to-one correspondence between an organization business record and its organization identity principal. |
| `IdentityAssertion` | Evidence-backed claim such as verified email, ORCID, affiliation, or expertise. |
| `AssuranceLevel` | Ordinal confidence used by policy: `low`, `medium`, `high`, `very_high`. |
| `OrganizationMembership` | A human's relationship to an organization, with role label, verification, validity, and status. |
| `Sponsorship` | Links an audit episode to its human or organization sponsor, the human actor who created the commission, any represented organization, and the public-attribution policy. |
| `AuthorityGrant` | Scoped permission for a principal to act directly or on behalf of an organization. |
| `AuthorityRevocation` | Append-only revocation of a prior grant. |
| `AccessDecision` | Auditable allow, deny, step-up-required, or manual-review decision. |
| `IdentityEvent` | Append-only event used to derive current identity and authority state. |

The existing `users` table may serve as the initial `Account` representation. Existing `users.roles` should become a compatibility projection during migration, not the long-term authority source.

### Persistence boundary

PostgreSQL persistence uses two coordinated representations:

- `identity_events` is the append-only, explicitly sequenced source for
  deterministic Rust replay;
- relational identity tables are transactionally maintained query indexes for
  foreign keys, uniqueness, active-grant lookup, and operational reporting;
- each indexed row also retains the validated Rust record as JSON so enum and
  scope evolution remains owned by `csqd-identity`, rather than duplicated in
  SQL decoding code;
- access-decision JSON retains typed reason codes, a structurally valid
  outcome/basis combination, the complete authorization request, and explicit
  `Known`/`Unresolved`/`None` audited principal-reference variants while the
  relational action, scope, and outcome columns remain query indexes;
- access-decision principal references are mandatory JSON values rather than
  nullable foreign-key columns; dedicated relation tables index only references
  that resolved to known principals;
- repository mutations acquire one transaction-scoped ledger lock, append
  events, replay the complete ledger inside the transaction, update relational
  indexes, and then commit;
- failed replay or an index constraint rolls back the complete mutation while
  allowing gaps in the database sequence, which are ordering tokens rather
  than aggregate counts.

Migration `000005_identity_persistence.sql` backfills accounts, organizations,
reviewer eligibility, and compatibility authority without modifying
`users.roles`, and creates the typed access-decision schema directly. Legacy
organization-sponsored episodes identify their
organization but not the authenticated human actor. They are retained as
`actor_attribution_required` compatibility rows and are intentionally excluded
from canonical sponsorship events until that missing provenance is resolved.

## 5. Resource Scopes and Actions

Recommended resource scopes:

- platform-wide;
- domain instantiation;
- organization;
- audit subject;
- audit episode;
- synthesis review;
- commercial record;
- private evidence collection.

Representative actions:

- `commission_audit`
- `view_sponsored_audit`
- `manage_organization_members`
- `accept_review_assignment`
- `submit_element_review`
- `submit_synthesis_review`
- `view_confidential_evidence`
- `publish_synthesis_review`
- `record_invoice`
- `record_payment`
- `record_reviewer_payout`
- `manage_accounts`
- `grant_authority`
- `revoke_authority`
- `export_private_audit`

Policies should operate on typed actions and scopes rather than route names.

## 6. Workflow Mapping

### Sponsor workflow

1. A human creates or signs into an account.
2. The account is linked to a human principal.
3. The human chooses whether to sponsor personally or on behalf of an organization.
4. For personal sponsorship:
   - the human principal is both sponsor and actor;
   - no organization is created or required;
   - no organization authority grant is required.
5. For organization sponsorship:
   - an organization principal is created or matched;
   - representation is established through an invitation, verified domain, operator review, or organization administrator approval;
   - an active authority grant permits the human to commission on behalf of that organization;
   - the organization principal is the sponsor and the authenticated human is the actor.
6. The commission records the sponsor principal, actor principal, optional represented organization, applicable authority grant, and sponsor-attribution preference.
7. Both paths create episode-scoped sponsor access. Sponsor-console access is derived from personal sponsorship, organization authority, and episode authority—not a global `sponsor` flag.

### Reviewer workflow

1. A human principal signs in and completes a reviewer profile.
2. Identity assertions may record ORCID, affiliation, expertise, and conflict disclosures.
3. Verification events distinguish self-asserted from operator- or institution-verified claims.
4. A solicitation does not itself grant access. Acceptance creates an episode-scoped reviewer authority grant.
5. The grant permits only the assigned actions and resources, with an expiration date.
6. ElementReviews and SynthesisReviews preserve the reviewer principal, represented organization when applicable, assurance at submission, and assignment grant used.
7. Revocation stops future actions without erasing prior authored work.

### Operator workflow

1. Platform operator authority is granted explicitly by another authorized operator or a controlled bootstrap procedure.
2. Operator grants and revocations are append-only identity events.
3. High-risk actions require a recent high-assurance session or step-up:
   - granting operator authority;
   - publishing or superseding a public SynthesisReview;
   - recording payments or payouts;
   - accessing confidential evidence;
   - exporting private audit data.
4. Operators cannot silently grant themselves authority or remove the final active operator.
5. Every administrative decision records actor, authority basis, policy version, time, and outcome.

### Organization workflow

An organization is both:

- a business-domain record used for sponsorship and payment; and
- an identity principal capable of being represented by humans.

An organization is optional for sponsorship. This workflow governs organizational representation when a person chooses to commission or participate on an organization's behalf; it is not a prerequisite for commissioning personally.

The existing `organizations` table should remain the business record. It should link one-to-one to an organization `IdentityPrincipal`. Membership and authority belong in the identity context rather than as columns on `organizations`.

Organization claims should support:

- invited member;
- administrator;
- commissioning representative;
- billing contact;
- authorized signatory;
- reviewer affiliation;
- former or revoked member.

### Audit-episode workflow

`AuditEpisode` is the primary authorization boundary for commissioned work.

Episode participation should be derived from explicit relationships:

- sponsoring principal, which may be a human or organization;
- sponsor actor;
- sponsor representatives when the sponsor is an organization;
- represented organization when applicable;
- assigned reviewers;
- synthesis author;
- responsible operator;
- invited observers;
- public participants.

Public episodes may permit low-assurance participation while commissioned or confidential episodes require explicit grants. Episode facts continue to use the existing C-SQD fact graph; identity authority is referenced through provenance and authorization metadata.

### Provenance workflow

For sensitive or authored actions, provenance should be capable of recording:

- `sponsor_principal_id` for commission and sponsorship events;
- `actor_principal_id`;
- authenticated `account_id`;
- optional `represented_organization_principal_id`;
- authentication method and assurance;
- relevant `authority_grant_id`;
- authorization policy identifier and version;
- decision timestamp;
- existing source-system and source-document information.

For personal sponsorship, `sponsor_principal_id` and `actor_principal_id` are the same. For organization sponsorship, the sponsor is the organization and the actor is the authorized human representative.

Public views should expose a safe projection such as display name, verified affiliation, role in the audit, and relevant disclosure. They must not expose private credentials, raw identity documents, session identifiers, or internal risk signals.

## 7. Data Classification

### Public or publishable

- chosen display name;
- verified professional affiliation, when the person consents;
- ORCID or comparable public identifier;
- audit role;
- authorship and contribution;
- public conflict disclosure;
- sponsor attribution according to the chosen policy:
  - named individual;
  - named organization;
  - generic “independent sponsor”;
  - confidential sponsor.

### Private application data

- email address;
- organization invitations;
- the real sponsor identity when public attribution is generic or confidential;
- sponsor-attribution preference;
- authority grants and revocations;
- reviewer verification notes;
- confidential conflict disclosures;
- access decisions;
- session and credential metadata.

### Highly sensitive evidence

- identity documents;
- raw proofing-provider results;
- recovery secrets;
- passkey material;
- security and risk signals.

Highly sensitive evidence should not be collected until a concrete workflow requires it. When collected, it requires encryption, retention limits, materialization auditing, and operational key management.

## 8. Relationship to the Imported Architecture

Reuse or adapt:

- principal kinds;
- assurance levels;
- identity witnesses and assertions;
- authority scopes;
- revocation and supersession semantics;
- policy evaluation;
- step-up decisions;
- append-only projections;
- encrypted fact envelopes and materialization audits;
- provider-neutral OIDC verification.

Do not import:

- patient, payer, clinical, insurance, caregiver, or HIPAA vocabulary;
- biometric continuity and liveness workflows for the initial product;
- the iOS App Attest proof application;
- its parallel `Fact`, episode, membership, identifier, time, server, and repository implementations;
- Keycloak as a mandatory deployment component.

C-SQD should have one FEN substrate and one API runtime. The identity crate should share C-SQD identifiers, timestamps, database conventions, and error boundaries.

## 9. Incremental Adoption

1. Add identity principals, account links, personal sponsorship, organizations, memberships, and authority grants.
2. Keep magic-link authentication while making production delivery and session controls complete.
3. Derive organization and episode authorization from active grants.
4. Record role grants, revocations, and access decisions as auditable events.
5. Add OIDC and reviewer identity assertions when pilot partners require them.
6. Add passkey or MFA step-up for high-risk operator actions.
7. Add encrypted evidence storage only for workflows that genuinely require sensitive proof.

No phase should require rewriting public audit facts or changing the meaning of existing audit provenance.

## 10. Success Criteria

The architecture is successful when:

- an account can represent a durable human principal;
- a human can sponsor an audit in their own capacity without creating or joining an organization;
- a human can act for an organization only through active authority;
- sponsor and reviewer access can be scoped to an audit episode;
- authority changes are historically auditable;
- commission and authored facts distinguish sponsor, actor, and any represented organization;
- confidential access and high-risk operations enforce explicit policy;
- public provenance is meaningful without exposing private identity data;
- current permissions can be rebuilt deterministically from stored identity events.
