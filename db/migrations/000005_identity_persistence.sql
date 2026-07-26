-- Durable identity, sponsorship, authority, and authorization provenance.
--
-- This is intentionally additive. `users.roles` remains the production
-- compatibility mechanism until the API authorization cutover is complete.
-- Every backfill insert is idempotent so this file can also be exercised
-- directly by migration tests.

CREATE TABLE IF NOT EXISTS identity_principals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    kind text NOT NULL CHECK (kind IN ('human', 'organization', 'system_agent', 'device')),
    status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'disputed', 'superseded', 'deactivated')),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL,
    created_by jsonb NOT NULL,
    record jsonb NOT NULL,
    source_system text,
    source_entity_type text,
    source_entity_id uuid,
    CHECK (
        (source_system IS NULL AND source_entity_type IS NULL AND source_entity_id IS NULL)
        OR
        (source_system IS NOT NULL AND source_entity_type IS NOT NULL AND source_entity_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS identity_principals_source_idx
    ON identity_principals (source_system, source_entity_type, source_entity_id)
    WHERE source_system IS NOT NULL;

CREATE TABLE IF NOT EXISTS account_principal_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    principal_id uuid NOT NULL REFERENCES identity_principals(id),
    status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'disputed', 'superseded', 'deactivated')),
    established_by jsonb NOT NULL,
    established_at timestamptz NOT NULL,
    record jsonb NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_principal_links_one_active_account_idx
    ON account_principal_links (account_id)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS account_principal_links_principal_idx
    ON account_principal_links (principal_id);

CREATE TABLE IF NOT EXISTS organization_principal_links (
    organization_id uuid PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    principal_id uuid NOT NULL UNIQUE REFERENCES identity_principals(id),
    established_by jsonb NOT NULL,
    established_at timestamptz NOT NULL,
    record jsonb NOT NULL
);

CREATE TABLE IF NOT EXISTS authentication_identities (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'revoked', 'superseded')),
    identity_kind text NOT NULL
        CHECK (identity_kind IN ('magic_link_email', 'oidc_subject', 'passkey')),
    identity_key text NOT NULL,
    established_at timestamptz NOT NULL,
    record jsonb NOT NULL,
    UNIQUE (identity_kind, identity_key)
);

CREATE TABLE IF NOT EXISTS identity_assertions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    assertion_kind text NOT NULL CHECK (btrim(assertion_kind) <> ''),
    assurance text NOT NULL CHECK (assurance IN ('low', 'medium', 'high', 'very_high')),
    status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'disputed', 'superseded', 'revoked')),
    asserted_by jsonb NOT NULL,
    asserted_at timestamptz NOT NULL,
    valid_from timestamptz,
    valid_until timestamptz,
    evidence_refs jsonb NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(evidence_refs) = 'array'),
    record jsonb NOT NULL,
    CHECK (valid_until IS NULL OR (valid_from IS NOT NULL AND valid_until > valid_from))
);

CREATE INDEX IF NOT EXISTS identity_assertions_subject_idx
    ON identity_assertions (subject_principal_id, status);

CREATE TABLE IF NOT EXISTS organization_memberships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    member_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    organization_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    organization_id uuid NOT NULL REFERENCES organizations(id),
    role text NOT NULL CHECK (btrim(role) <> ''),
    assurance text NOT NULL CHECK (assurance IN ('low', 'medium', 'high', 'very_high')),
    status text NOT NULL CHECK (status IN ('invited', 'active', 'revoked', 'expired', 'superseded')),
    valid_from timestamptz,
    valid_until timestamptz,
    asserted_by jsonb NOT NULL,
    asserted_at timestamptz NOT NULL,
    record jsonb NOT NULL,
    CHECK (member_principal_id <> organization_principal_id),
    CHECK (valid_until IS NULL OR (valid_from IS NOT NULL AND valid_until > valid_from))
);

CREATE INDEX IF NOT EXISTS organization_memberships_member_idx
    ON organization_memberships (member_principal_id, status);

CREATE TABLE IF NOT EXISTS authority_grants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    represented_organization_principal_id uuid REFERENCES identity_principals(id),
    authority_kind text NOT NULL CHECK (
        authority_kind IN (
            'platform_operator',
            'organization_administrator',
            'organization_representative',
            'sponsor_representative',
            'episode_sponsor',
            'episode_reviewer',
            'synthesis_author',
            'episode_operator',
            'observer'
        )
    ),
    scope jsonb NOT NULL,
    permitted_actions jsonb NOT NULL
        CHECK (jsonb_typeof(permitted_actions) = 'array' AND jsonb_array_length(permitted_actions) > 0),
    issued_by_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    issued_at timestamptz NOT NULL,
    valid_from timestamptz,
    valid_until timestamptz,
    evidence_refs jsonb NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(evidence_refs) = 'array'),
    record jsonb NOT NULL,
    CHECK (valid_until IS NULL OR (valid_from IS NOT NULL AND valid_until > valid_from))
);

CREATE INDEX IF NOT EXISTS authority_grants_actor_idx
    ON authority_grants (actor_principal_id, issued_at);

CREATE TABLE IF NOT EXISTS authority_revocations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    grant_id uuid NOT NULL UNIQUE REFERENCES authority_grants(id),
    revoked_by_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    revoked_at timestamptz NOT NULL,
    reason text NOT NULL CHECK (btrim(reason) <> ''),
    record jsonb NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_episode_sponsorships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    sponsor_kind text NOT NULL CHECK (sponsor_kind IN ('individual', 'organization')),
    sponsor_principal_id uuid NOT NULL REFERENCES identity_principals(id),
    actor_principal_id uuid REFERENCES identity_principals(id),
    represented_organization_principal_id uuid REFERENCES identity_principals(id),
    authority_grant_id uuid REFERENCES authority_grants(id),
    visibility text NOT NULL CHECK (visibility IN ('named', 'generic', 'confidential')),
    created_at timestamptz NOT NULL,
    legacy_backfill_status text NOT NULL DEFAULT 'complete'
        CHECK (legacy_backfill_status IN ('complete', 'actor_attribution_required')),
    record jsonb,
    UNIQUE (episode_id, sponsor_principal_id),
    CHECK (
        (
            legacy_backfill_status = 'complete'
            AND actor_principal_id IS NOT NULL
            AND record IS NOT NULL
            AND (
                (
                    sponsor_kind = 'individual'
                    AND sponsor_principal_id = actor_principal_id
                    AND represented_organization_principal_id IS NULL
                    AND authority_grant_id IS NULL
                )
                OR
                (
                    sponsor_kind = 'organization'
                    AND sponsor_principal_id = represented_organization_principal_id
                    AND authority_grant_id IS NOT NULL
                    AND sponsor_principal_id <> actor_principal_id
                )
            )
        )
        OR
        (
            legacy_backfill_status = 'actor_attribution_required'
            AND sponsor_kind = 'organization'
            AND actor_principal_id IS NULL
            AND represented_organization_principal_id = sponsor_principal_id
            AND authority_grant_id IS NULL
            AND record IS NULL
        )
    )
);

CREATE INDEX IF NOT EXISTS audit_episode_sponsorships_sponsor_idx
    ON audit_episode_sponsorships (sponsor_principal_id);

CREATE TABLE IF NOT EXISTS identity_access_decisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES users(id),
    actor_reference jsonb NOT NULL
        CHECK (
            CASE jsonb_typeof(actor_reference)
                WHEN 'object' THEN
                    (
                        actor_reference = jsonb_build_object(
                            'known',
                            actor_reference -> 'known'
                        )
                        AND jsonb_typeof(actor_reference -> 'known') = 'string'
                    )
                    OR (
                        actor_reference = jsonb_build_object(
                            'unresolved',
                            actor_reference -> 'unresolved'
                        )
                        AND jsonb_typeof(actor_reference -> 'unresolved') = 'string'
                    )
                ELSE false
            END
        ),
    representation_reference jsonb NOT NULL
        CHECK (
            CASE jsonb_typeof(representation_reference)
                WHEN 'string' THEN representation_reference = '"none"'::jsonb
                WHEN 'object' THEN
                    (
                        representation_reference = jsonb_build_object(
                            'known',
                            representation_reference -> 'known'
                        )
                        AND jsonb_typeof(
                            representation_reference -> 'known'
                        ) = 'string'
                    )
                    OR (
                        representation_reference = jsonb_build_object(
                            'unresolved',
                            representation_reference -> 'unresolved'
                        )
                        AND jsonb_typeof(
                            representation_reference -> 'unresolved'
                        ) = 'string'
                    )
                ELSE false
            END
        ),
    action text NOT NULL CHECK (btrim(action) <> ''),
    scope jsonb NOT NULL,
    outcome text NOT NULL
        CHECK (outcome IN ('allowed', 'denied', 'step_up_required', 'manual_review_required')),
    policy_id text NOT NULL CHECK (btrim(policy_id) <> ''),
    reason_codes jsonb NOT NULL CHECK (jsonb_typeof(reason_codes) = 'array'),
    evaluated_at timestamptz NOT NULL,
    record jsonb NOT NULL
);

CREATE TABLE IF NOT EXISTS identity_access_decision_actor_principals (
    decision_id uuid PRIMARY KEY
        REFERENCES identity_access_decisions(id) ON DELETE CASCADE,
    principal_id uuid NOT NULL REFERENCES identity_principals(id)
);

CREATE INDEX IF NOT EXISTS identity_access_decision_actor_principals_principal_idx
    ON identity_access_decision_actor_principals (principal_id);

CREATE TABLE IF NOT EXISTS identity_access_decision_organization_principals (
    decision_id uuid PRIMARY KEY
        REFERENCES identity_access_decisions(id) ON DELETE CASCADE,
    principal_id uuid NOT NULL REFERENCES identity_principals(id)
);

CREATE INDEX IF NOT EXISTS identity_access_decision_organization_principals_principal_idx
    ON identity_access_decision_organization_principals (principal_id);

CREATE SEQUENCE IF NOT EXISTS identity_event_append_sequence_seq AS bigint;

CREATE TABLE IF NOT EXISTS identity_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    append_sequence bigint NOT NULL DEFAULT nextval('identity_event_append_sequence_seq') UNIQUE
        CHECK (append_sequence > 0),
    recorded_at timestamptz NOT NULL,
    recorded_by jsonb NOT NULL,
    payload jsonb NOT NULL,
    source_key text UNIQUE
);

CREATE INDEX IF NOT EXISTS identity_events_recorded_at_idx
    ON identity_events (recorded_at, append_sequence);

-- One durable platform principal issues compatibility grants. It is distinct
-- from the `Principal::Platform` provenance marker because authority grants
-- intentionally require an identity-principal issuer.
INSERT INTO identity_principals (
    id,
    kind,
    status,
    display_name,
    created_at,
    created_by,
    record,
    source_system,
    source_entity_type,
    source_entity_id
)
SELECT
    gen.id,
    'system_agent',
    'active',
    'C-SQD platform authority service',
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object(
        'id', gen.id::text,
        'kind', 'system_agent',
        'status', 'active',
        'display_name', 'C-SQD platform authority service',
        'created_at', transaction_timestamp(),
        'created_by', 'platform'
    ),
    'csqd',
    'platform_authority_service',
    '00000000-0000-0000-0000-000000000000'::uuid
FROM (SELECT gen_random_uuid() AS id) AS gen
ON CONFLICT (source_system, source_entity_type, source_entity_id)
    WHERE source_system IS NOT NULL
DO NOTHING;

-- Existing accounts become durable human principals. Account status is
-- represented on the principal but the legacy role array remains untouched.
INSERT INTO identity_principals (
    id,
    kind,
    status,
    display_name,
    created_at,
    created_by,
    record,
    source_system,
    source_entity_type,
    source_entity_id
)
SELECT
    gen.id,
    'human',
    CASE u.status
        WHEN 'suspended' THEN 'disputed'
        WHEN 'deactivated' THEN 'deactivated'
        ELSE 'active'
    END,
    u.display_name,
    u.created_at,
    '"platform"'::jsonb,
    jsonb_build_object(
        'id', gen.id::text,
        'kind', 'human',
        'status',
            CASE u.status
                WHEN 'suspended' THEN 'disputed'
                WHEN 'deactivated' THEN 'deactivated'
                ELSE 'active'
            END,
        'display_name', u.display_name,
        'created_at', u.created_at,
        'created_by', 'platform'
    ),
    'csqd_legacy_backfill',
    'user',
    u.id
FROM users AS u
CROSS JOIN LATERAL (
    SELECT gen_random_uuid() AS id
    WHERE u.id IS NOT NULL
) AS gen
WHERE NOT EXISTS (
    SELECT 1
    FROM account_principal_links AS existing
    WHERE existing.account_id = u.id
)
ON CONFLICT (source_system, source_entity_type, source_entity_id)
    WHERE source_system IS NOT NULL
DO NOTHING;

INSERT INTO account_principal_links (
    id,
    account_id,
    principal_id,
    status,
    established_by,
    established_at,
    record
)
SELECT
    gen.id,
    u.id,
    p.id,
    'active',
    '"platform"'::jsonb,
    u.created_at,
    jsonb_build_object(
        'id', gen.id::text,
        'account_id', u.id::text,
        'principal_id', p.id::text,
        'status', 'active',
        'established_by', 'platform',
        'established_at', u.created_at
    )
FROM users AS u
JOIN identity_principals AS p
  ON p.source_system = 'csqd_legacy_backfill'
 AND p.source_entity_type = 'user'
 AND p.source_entity_id = u.id
CROSS JOIN LATERAL (
    SELECT gen_random_uuid() AS id
    WHERE u.id IS NOT NULL
) AS gen
WHERE NOT EXISTS (
    SELECT 1
    FROM account_principal_links AS existing
    WHERE existing.account_id = u.id
);

-- Every organization business record gets exactly one organization principal.
INSERT INTO identity_principals (
    id,
    kind,
    status,
    display_name,
    created_at,
    created_by,
    record,
    source_system,
    source_entity_type,
    source_entity_id
)
SELECT
    gen.id,
    'organization',
    'active',
    o.name,
    o.created_at,
    '"platform"'::jsonb,
    jsonb_build_object(
        'id', gen.id::text,
        'kind', 'organization',
        'status', 'active',
        'display_name', o.name,
        'created_at', o.created_at,
        'created_by', 'platform'
    ),
    'csqd_legacy_backfill',
    'organization',
    o.id
FROM organizations AS o
CROSS JOIN LATERAL (
    SELECT gen_random_uuid() AS id
    WHERE o.id IS NOT NULL
) AS gen
WHERE NOT EXISTS (
    SELECT 1
    FROM organization_principal_links AS existing
    WHERE existing.organization_id = o.id
)
ON CONFLICT (source_system, source_entity_type, source_entity_id)
    WHERE source_system IS NOT NULL
DO NOTHING;

INSERT INTO organization_principal_links (
    organization_id,
    principal_id,
    established_by,
    established_at,
    record
)
SELECT
    o.id,
    p.id,
    '"platform"'::jsonb,
    o.created_at,
    jsonb_build_object(
        'organization_id', o.id::text,
        'principal_id', p.id::text,
        'established_by', 'platform',
        'established_at', o.created_at
    )
FROM organizations AS o
JOIN identity_principals AS p
  ON p.source_system = 'csqd_legacy_backfill'
 AND p.source_entity_type = 'organization'
 AND p.source_entity_id = o.id
ON CONFLICT (organization_id) DO NOTHING;

-- Reviewer is eligibility evidence, not episode authority.
INSERT INTO identity_assertions (
    id,
    subject_principal_id,
    assertion_kind,
    assurance,
    status,
    asserted_by,
    asserted_at,
    evidence_refs,
    record
)
SELECT
    gen.id,
    p.id,
    'reviewer_expertise',
    'low',
    'active',
    '"platform"'::jsonb,
    transaction_timestamp(),
    jsonb_build_array('legacy users.roles reviewer eligibility'),
    jsonb_build_object(
        'id', gen.id::text,
        'subject_principal_id', p.id::text,
        'kind', jsonb_build_object(
            'reviewer_expertise',
            jsonb_build_object('label', 'Legacy reviewer role eligibility')
        ),
        'assurance', 'low',
        'status', 'active',
        'asserted_by', 'platform',
        'asserted_at', transaction_timestamp(),
        'validity', NULL,
        'evidence_refs', jsonb_build_array('legacy users.roles reviewer eligibility')
    )
FROM users AS u
JOIN identity_principals AS p
  ON p.source_system = 'csqd_legacy_backfill'
 AND p.source_entity_type = 'user'
 AND p.source_entity_id = u.id
CROSS JOIN LATERAL (
    SELECT gen_random_uuid() AS id
    WHERE p.id IS NOT NULL
) AS gen
WHERE 'reviewer' = ANY(u.roles)
  AND NOT EXISTS (
      SELECT 1
      FROM identity_assertions AS a
      WHERE a.subject_principal_id = p.id
        AND a.assertion_kind = 'reviewer_expertise'
        AND a.record->'kind'->'reviewer_expertise'->>'label'
            = 'Legacy reviewer role eligibility'
  );

-- Legacy operator authority is operationally equivalent. Legacy sponsor
-- authority is deliberately platform-scoped compatibility evidence and does
-- not authorize representation of any organization.
INSERT INTO authority_grants (
    id,
    actor_principal_id,
    represented_organization_principal_id,
    authority_kind,
    scope,
    permitted_actions,
    issued_by_principal_id,
    issued_at,
    evidence_refs,
    record
)
SELECT
    gen.id,
    p.id,
    NULL,
    CASE role.role_name
        WHEN 'operator' THEN 'platform_operator'
        ELSE 'sponsor_representative'
    END,
    '"platform"'::jsonb,
    CASE role.role_name
        WHEN 'operator' THEN jsonb_build_array(
            'register_public_audit_subject',
            'commission_audit',
            'manage_organization_members',
            'view_sponsored_audit',
            'accept_review_assignment',
            'submit_element_review',
            'submit_synthesis_review',
            'view_confidential_evidence',
            'publish_synthesis_review',
            'record_invoice',
            'record_payment',
            'record_reviewer_payout',
            'manage_accounts',
            'grant_authority',
            'revoke_authority',
            'export_private_audit'
        )
        ELSE jsonb_build_array('commission_audit')
    END,
    platform.id,
    transaction_timestamp(),
    jsonb_build_array('legacy users.roles ' || role.role_name),
    jsonb_build_object(
        'id', gen.id::text,
        'actor_principal_id', p.id::text,
        'represented_organization_principal_id', NULL,
        'kind',
            CASE role.role_name
                WHEN 'operator' THEN 'platform_operator'
                ELSE 'sponsor_representative'
            END,
        'scope', 'platform',
        'permitted_actions',
            CASE role.role_name
                WHEN 'operator' THEN jsonb_build_array(
                    'register_public_audit_subject',
                    'commission_audit',
                    'manage_organization_members',
                    'view_sponsored_audit',
                    'accept_review_assignment',
                    'submit_element_review',
                    'submit_synthesis_review',
                    'view_confidential_evidence',
                    'publish_synthesis_review',
                    'record_invoice',
                    'record_payment',
                    'record_reviewer_payout',
                    'manage_accounts',
                    'grant_authority',
                    'revoke_authority',
                    'export_private_audit'
                )
                ELSE jsonb_build_array('commission_audit')
            END,
        'issued_by_principal_id', platform.id::text,
        'issued_at', transaction_timestamp(),
        'validity', NULL,
        'evidence_refs', jsonb_build_array('legacy users.roles ' || role.role_name)
    )
FROM users AS u
CROSS JOIN LATERAL unnest(u.roles) AS role(role_name)
JOIN identity_principals AS p
  ON p.source_system = 'csqd_legacy_backfill'
 AND p.source_entity_type = 'user'
 AND p.source_entity_id = u.id
JOIN identity_principals AS platform
  ON platform.source_system = 'csqd'
 AND platform.source_entity_type = 'platform_authority_service'
 AND platform.source_entity_id = '00000000-0000-0000-0000-000000000000'::uuid
CROSS JOIN LATERAL (
    SELECT gen_random_uuid() AS id
    WHERE p.id IS NOT NULL AND role.role_name IS NOT NULL
) AS gen
WHERE role.role_name IN ('operator', 'sponsor')
  AND NOT EXISTS (
      SELECT 1
      FROM authority_grants AS existing
      WHERE existing.actor_principal_id = p.id
        AND existing.evidence_refs @> jsonb_build_array(
            'legacy users.roles ' || role.role_name
        )
  );

-- Legacy episodes identify the organization sponsor but not the authenticated
-- human actor. Preserve the sponsor relationship without inventing actor or
-- grant evidence; Session 6 can resolve these explicit compatibility rows.
INSERT INTO audit_episode_sponsorships (
    episode_id,
    sponsor_kind,
    sponsor_principal_id,
    actor_principal_id,
    represented_organization_principal_id,
    authority_grant_id,
    visibility,
    created_at,
    legacy_backfill_status,
    record
)
SELECT
    ae.id,
    'organization',
    opl.principal_id,
    NULL,
    opl.principal_id,
    NULL,
    CASE
        WHEN COALESCE(
            (
                SELECT (f.payload->>'confidential')::boolean
                FROM episode_memberships AS em
                JOIN facts AS f ON f.id = em.fact_id
                WHERE em.episode_id = ae.id
                  AND em.role = 'commission'
                  AND em.status = 'active'
                  AND f.payload_kind = 'audit_commission'
                ORDER BY f.created_at
                LIMIT 1
            ),
            false
        ) THEN 'confidential'
        ELSE 'named'
    END,
    ae.authored_at,
    'actor_attribution_required',
    NULL
FROM audit_episodes AS ae
JOIN organizations AS o
  ON ae.authored_by->'organization'->>'organization_id' = o.id::text
JOIN organization_principal_links AS opl
  ON opl.organization_id = o.id
ON CONFLICT (episode_id, sponsor_principal_id) DO NOTHING;

-- Backfilled ledger events use migration recording time. Their embedded
-- effective timestamps retain the legacy creation times.
INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object(
        'event_type', 'principal_created',
        'principal',
        p.record || jsonb_build_object('status', 'active')
    ),
    'legacy:principal:' || p.id::text || ':created'
FROM identity_principals AS p
WHERE p.source_system IN ('csqd', 'csqd_legacy_backfill')
ON CONFLICT (source_key) DO NOTHING;

INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object(
        'event_type', 'principal_status_changed',
        'principal_id', p.id::text,
        'status',
            CASE p.status
                WHEN 'disputed' THEN to_jsonb('disputed'::text)
                WHEN 'deactivated' THEN to_jsonb('deactivated'::text)
                ELSE to_jsonb(p.status)
            END
    ),
    'legacy:principal:' || p.id::text || ':status'
FROM identity_principals AS p
WHERE p.source_system IN ('csqd', 'csqd_legacy_backfill')
  AND p.status <> 'active'
ON CONFLICT (source_key) DO NOTHING;

INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object('event_type', 'account_principal_linked', 'link', l.record),
    'legacy:account-link:' || l.id::text
FROM account_principal_links AS l
JOIN identity_principals AS p
  ON p.id = l.principal_id
 AND p.source_system = 'csqd_legacy_backfill'
ON CONFLICT (source_key) DO NOTHING;

INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object('event_type', 'organization_principal_linked', 'link', l.record),
    'legacy:organization-link:' || l.organization_id::text
FROM organization_principal_links AS l
JOIN identity_principals AS p
  ON p.id = l.principal_id
 AND p.source_system = 'csqd_legacy_backfill'
ON CONFLICT (source_key) DO NOTHING;

INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object('event_type', 'assertion_recorded', 'assertion', a.record),
    'legacy:assertion:' || a.id::text
FROM identity_assertions AS a
WHERE a.evidence_refs @> '["legacy users.roles reviewer eligibility"]'::jsonb
ON CONFLICT (source_key) DO NOTHING;

INSERT INTO identity_events (recorded_at, recorded_by, payload, source_key)
SELECT
    transaction_timestamp(),
    '"platform"'::jsonb,
    jsonb_build_object('event_type', 'authority_granted', 'grant', g.record),
    'legacy:authority-grant:' || g.id::text
FROM authority_grants AS g
WHERE g.evidence_refs @> '["legacy users.roles operator"]'::jsonb
   OR g.evidence_refs @> '["legacy users.roles sponsor"]'::jsonb
ON CONFLICT (source_key) DO NOTHING;
