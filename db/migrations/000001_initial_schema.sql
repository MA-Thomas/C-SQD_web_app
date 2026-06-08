CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL UNIQUE,
    display_name text NOT NULL,
    role text NOT NULL CHECK (role IN ('reader', 'reviewer', 'admin', 'funder')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE reviewer_profiles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL UNIQUE REFERENCES users(id),
    bio text,
    expertise_areas text[] NOT NULL DEFAULT '{}',
    status text NOT NULL DEFAULT 'candidate' CHECK (status IN ('candidate', 'active', 'suspended')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE journals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    publisher text,
    issn text,
    source_classification text NOT NULL DEFAULT 'curated',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE scholarly_objects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    object_type text NOT NULL CHECK (object_type IN ('article', 'preprint', 'dataset', 'software', 'protocol', 'report')),
    doi text UNIQUE,
    title text NOT NULL,
    authors jsonb NOT NULL DEFAULT '[]'::jsonb,
    abstract text,
    journal_id uuid REFERENCES journals(id),
    publication_date date,
    license text,
    canonical_url text NOT NULL,
    metadata_provenance jsonb NOT NULL DEFAULT '{}'::jsonb,
    native_display_permitted boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE external_article_locations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    location_type text NOT NULL CHECK (location_type IN ('publisher', 'landing_page', 'full_text', 'pdf', 'repository')),
    url text NOT NULL,
    license text,
    is_canonical boolean NOT NULL DEFAULT false,
    provenance jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE review_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    reviewer_profile_id uuid NOT NULL REFERENCES reviewer_profiles(id),
    assignment_type text NOT NULL CHECK (assignment_type IN ('element_review', 'synthesis_review', 'error_claim_review')),
    compensation_status text NOT NULL DEFAULT 'unpaid' CHECK (compensation_status IN ('unpaid', 'eligible', 'approved', 'paid')),
    state text NOT NULL DEFAULT 'created' CHECK (state IN ('created', 'offered', 'accepted', 'declined', 'in_progress', 'submitted', 'quality_control', 'published', 'canceled')),
    due_at timestamptz,
    conflict_disclosure text,
    created_by uuid REFERENCES users(id),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE review_episodes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    assignment_id uuid REFERENCES review_assignments(id),
    reviewer_profile_id uuid REFERENCES reviewer_profiles(id),
    episode_type text NOT NULL CHECK (episode_type IN ('element_review', 'synthesis_review', 'error_claim', 'author_response', 'adjudication', 'challenge', 'quality_control')),
    state text NOT NULL DEFAULT 'draft' CHECK (state IN ('draft', 'submitted', 'quality_control', 'published', 'rejected', 'withdrawn')),
    visibility text NOT NULL DEFAULT 'private' CHECK (visibility IN ('private', 'platform', 'public')),
    title text NOT NULL,
    summary text,
    submitted_at timestamptz,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE evaluation_facts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    review_episode_id uuid REFERENCES review_episodes(id),
    fact_type text NOT NULL CHECK (fact_type IN ('claim_support', 'methodological_adequacy', 'statistical_adequacy', 'data_availability', 'code_availability', 'interpretation_strength', 'reproducibility_concern', 'error_claim_validation')),
    polarity text NOT NULL DEFAULT 'concern' CHECK (polarity IN ('support', 'concern', 'neutral')),
    severity integer CHECK (severity BETWEEN 0 AND 5),
    statement text NOT NULL,
    evidence text,
    target_locator jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE synthesis_reviews (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    review_episode_id uuid NOT NULL UNIQUE REFERENCES review_episodes(id),
    contribution_summary text NOT NULL,
    major_strengths text NOT NULL,
    major_weaknesses text NOT NULL,
    reliability_concerns text NOT NULL,
    overall_evaluation text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE error_claims (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    submitted_by uuid REFERENCES users(id),
    review_episode_id uuid REFERENCES review_episodes(id),
    alleged_error text NOT NULL,
    affected_part text,
    evidence text NOT NULL,
    proposed_severity integer CHECK (proposed_severity BETWEEN 0 AND 5),
    state text NOT NULL DEFAULT 'submitted' CHECK (state IN ('submitted', 'triage', 'accepted_for_evaluation', 'rejected', 'validated', 'invalidated', 'partially_supported')),
    disclosure text NOT NULL DEFAULT 'private' CHECK (disclosure IN ('private', 'limited', 'public')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE bounties (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id),
    error_claim_id uuid REFERENCES error_claims(id),
    title text NOT NULL,
    category text NOT NULL,
    reward_amount_cents integer NOT NULL CHECK (reward_amount_cents >= 0),
    currency text NOT NULL DEFAULT 'USD',
    state text NOT NULL DEFAULT 'draft' CHECK (state IN ('draft', 'funded', 'triage', 'in_evaluation', 'validated', 'invalidated', 'paid', 'canceled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE funding_sources (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type text NOT NULL CHECK (source_type IN ('author_fee', 'institutional_contract', 'bounty_sponsor', 'conference_organizer', 'manual_admin_allocation')),
    label text NOT NULL,
    owner_user_id uuid REFERENCES users(id),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE payment_obligations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    obligation_type text NOT NULL CHECK (obligation_type IN ('collect_review_fee', 'pay_review', 'fund_bounty', 'pay_bounty', 'refund')),
    funding_source_id uuid REFERENCES funding_sources(id),
    owed_by_user_id uuid REFERENCES users(id),
    owed_to_user_id uuid REFERENCES users(id),
    related_assignment_id uuid REFERENCES review_assignments(id),
    related_bounty_id uuid REFERENCES bounties(id),
    amount_cents integer NOT NULL CHECK (amount_cents >= 0),
    currency text NOT NULL DEFAULT 'USD',
    state text NOT NULL DEFAULT 'created' CHECK (state IN ('created', 'approved', 'pending', 'satisfied', 'failed', 'canceled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE payment_attempts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_obligation_id uuid NOT NULL REFERENCES payment_obligations(id),
    provider text NOT NULL DEFAULT 'manual',
    direction text NOT NULL CHECK (direction IN ('collection', 'payout', 'refund')),
    state text NOT NULL DEFAULT 'created' CHECK (state IN ('created', 'pending', 'succeeded', 'failed', 'canceled')),
    provider_reference jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE payment_provider_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_attempt_id uuid REFERENCES payment_attempts(id),
    provider text NOT NULL DEFAULT 'manual',
    event_type text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    received_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE ledger_entries (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_obligation_id uuid REFERENCES payment_obligations(id),
    payment_attempt_id uuid REFERENCES payment_attempts(id),
    entry_type text NOT NULL CHECK (entry_type IN ('obligation_created', 'funds_collected', 'payout_approved', 'payout_sent', 'refund_sent', 'adjustment')),
    amount_cents integer NOT NULL,
    currency text NOT NULL DEFAULT 'USD',
    memo text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id uuid REFERENCES users(id),
    event_type text NOT NULL,
    entity_type text NOT NULL,
    entity_id uuid,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE tags (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    label text NOT NULL UNIQUE,
    tag_type text NOT NULL DEFAULT 'user' CHECK (tag_type IN ('user', 'verified', 'system')),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE taggings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tag_id uuid NOT NULL REFERENCES tags(id),
    entity_type text NOT NULL,
    entity_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tag_id, entity_type, entity_id)
);

CREATE TABLE scholarly_object_search (
    scholarly_object_id uuid PRIMARY KEY REFERENCES scholarly_objects(id),
    search_text text NOT NULL,
    search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', search_text)) STORED,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX scholarly_object_search_vector_idx ON scholarly_object_search USING gin(search_vector);
CREATE INDEX evaluation_facts_object_idx ON evaluation_facts(scholarly_object_id);
CREATE INDEX review_episodes_object_idx ON review_episodes(scholarly_object_id);
CREATE INDEX review_assignments_reviewer_idx ON review_assignments(reviewer_profile_id);

