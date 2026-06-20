-- FEN alignment: variant detail columns, new fact payload kinds and
-- membership roles, unsolicited synthesis reviews, identity (roles, reviewer
-- tags), and magic-link session auth.

-- 1. Variant labels for Other/Custom enum variants (FEN carries the label).
ALTER TABLE domain_instantiations ADD COLUMN IF NOT EXISTS domain_type_detail text;
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS org_type_detail text;
ALTER TABLE audit_subjects ADD COLUMN IF NOT EXISTS subject_type_detail text;
ALTER TABLE cwe_nodes ADD COLUMN IF NOT EXISTS community_id uuid;

-- 2. New fact payload kinds (participation, petitions, curation).
ALTER TABLE facts DROP CONSTRAINT IF EXISTS facts_payload_kind_check;
ALTER TABLE facts ADD CONSTRAINT facts_payload_kind_check CHECK (
    payload_kind IN (
        'audit_commission',
        'element_review',
        'er_solicitation',
        'solicitation_event',
        'submitter_response',
        'episode_participation',
        'feature_petition',
        'cwe_petition',
        'curation_decision'
    )
);

-- 3. New episode membership roles.
ALTER TABLE episode_memberships DROP CONSTRAINT IF EXISTS episode_memberships_role_check;
ALTER TABLE episode_memberships ADD CONSTRAINT episode_memberships_role_check CHECK (
    role IN (
        'commission',
        'element_review',
        'solicitation',
        'solicitation_lifecycle',
        'response',
        'participation',
        'petition',
        'curation',
        'administrative',
        'other'
    )
);

-- 4. Unsolicited synthesis reviews are marked by the type system (memo).
ALTER TABLE episode_synthesis_reviews
    ADD COLUMN IF NOT EXISTS unsolicited boolean NOT NULL DEFAULT false;

-- 5. Identity: application roles, user status, reviewer tags + extensions.
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS roles text[] NOT NULL DEFAULT '{member}';
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS status text NOT NULL DEFAULT 'active';
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_status_check;
ALTER TABLE users ADD CONSTRAINT users_status_check CHECK (
    status IN ('active', 'suspended', 'deactivated')
);
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

-- Map legacy coarse roles into the roles array once.
UPDATE users
SET roles = CASE role
    WHEN 'admin' THEN ARRAY['member', 'operator']
    WHEN 'reviewer' THEN ARRAY['member', 'reviewer']
    WHEN 'funder' THEN ARRAY['member', 'sponsor']
    ELSE ARRAY['member']
END
WHERE roles = '{member}';

ALTER TABLE reviewer_profiles DROP CONSTRAINT IF EXISTS reviewer_profiles_status_check;
ALTER TABLE reviewer_profiles ADD CONSTRAINT reviewer_profiles_status_check CHECK (
    status IN ('candidate', 'grace_period', 'active', 'suspended')
);
ALTER TABLE reviewer_profiles
    ADD COLUMN IF NOT EXISTS domain_extensions jsonb NOT NULL DEFAULT '[]'::jsonb;

CREATE TABLE IF NOT EXISTS reviewer_tags (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label text NOT NULL,
    scope text NOT NULL DEFAULT 'global' CHECK (scope IN ('global', 'domain')),
    domain_instantiation_id uuid REFERENCES domain_instantiations(id),
    verified boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, label, scope, domain_instantiation_id)
);

-- 6. Magic-link authentication and cookie sessions.
CREATE TABLE IF NOT EXISTS auth_magic_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL,
    token_hash text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz
);

CREATE INDEX IF NOT EXISTS auth_magic_links_email_idx ON auth_magic_links (email);

CREATE TABLE IF NOT EXISTS auth_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz
);

CREATE INDEX IF NOT EXISTS auth_sessions_user_idx ON auth_sessions (user_id);
