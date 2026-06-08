CREATE TABLE domain_instantiations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_type text NOT NULL CHECK (domain_type IN ('academic_publishing', 'clinical_trial_review', 'ai_auditing', 'policy_review', 'custom')),
    name text NOT NULL,
    config jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    governed_by jsonb NOT NULL DEFAULT '"platform"'::jsonb
);

CREATE TABLE cwe_nodes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id) ON DELETE CASCADE,
    parent_id uuid REFERENCES cwe_nodes(id),
    label text NOT NULL,
    description text NOT NULL,
    source text NOT NULL CHECK (source IN ('base_taxonomy', 'community_extension', 'verified_tag')),
    source_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_objects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id),
    object_type text NOT NULL,
    title text NOT NULL,
    submitted_by uuid REFERENCES users(id),
    submitted_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revised', 'withdrawn')),
    status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    submission_tier text NOT NULL DEFAULT 'tier0' CHECK (submission_tier IN ('tier0', 'tier1', 'tier2', 'tier3_plus')),
    external_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    source_entity_type text,
    source_entity_id uuid,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_entity_type, source_entity_id)
);

CREATE TABLE audit_object_relations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    target uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    relation_type text NOT NULL CHECK (relation_type IN ('supersedes', 'revises', 'split_from', 'merged_into', 'related_to')),
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE review_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    audit_object_id uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    payload_kind text NOT NULL CHECK (payload_kind IN ('element_review', 'synthesis_review', 'submitter_response', 'bounty_posting', 'bounty_submission', 'bounty_adjudication')),
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded', 'retracted')),
    status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    provenance jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE review_event_memberships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    review_event_id uuid NOT NULL REFERENCES review_events(id) ON DELETE CASCADE,
    audit_object_id uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    role text NOT NULL CHECK (role IN ('element_review', 'synthesis_review', 'submitter_response', 'bounty_posting', 'bounty_submission', 'bounty_adjudication')),
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retracted')),
    UNIQUE (review_event_id, audit_object_id, role)
);

CREATE TABLE er_solicitations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    audit_object_id uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    cwe_node_id uuid NOT NULL REFERENCES cwe_nodes(id),
    issued_to uuid NOT NULL REFERENCES users(id),
    payment_scheme jsonb NOT NULL DEFAULT '{}'::jsonb,
    issued_at timestamptz NOT NULL DEFAULT now(),
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id)
);

CREATE TABLE solicitation_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    solicitation_id uuid NOT NULL REFERENCES er_solicitations(id) ON DELETE CASCADE,
    event_type text NOT NULL CHECK (event_type IN ('issued', 'accepted', 'declined', 'expired', 'completed', 'penalty_flagged')),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    principal jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    note text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE synthesis_sections (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    review_event_id uuid NOT NULL REFERENCES review_events(id) ON DELETE CASCADE,
    section_type text NOT NULL CHECK (section_type IN ('summary', 'methodological_assessment', 'ethical_assessment', 'evidence_integration', 'recommendations', 'open_questions')),
    content text NOT NULL,
    referenced_reviews uuid[] NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE synthesis_review_relations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source uuid NOT NULL REFERENCES review_events(id) ON DELETE CASCADE,
    target uuid NOT NULL REFERENCES review_events(id) ON DELETE CASCADE,
    relation_type text NOT NULL CHECK (relation_type IN ('supersedes', 'contests', 'related_to')),
    contestation_scope text CHECK (contestation_scope IN ('partial', 'full')),
    rationale text,
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE challenges (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_type text NOT NULL CHECK (challenge_type IN ('direct', 'petition')),
    target_type text NOT NULL CHECK (target_type IN ('element_review', 'synthesis_review')),
    target_review_event_id uuid NOT NULL REFERENCES review_events(id) ON DELETE CASCADE,
    challenger_review_event_id uuid REFERENCES review_events(id) ON DELETE SET NULL,
    initiated_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    initiated_at timestamptz NOT NULL DEFAULT now(),
    election_date timestamptz NOT NULL,
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'withdrawn')),
    winner_review_event_id uuid REFERENCES review_events(id) ON DELETE SET NULL,
    decided_at timestamptz,
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id)
);

CREATE INDEX audit_objects_domain_idx ON audit_objects(domain_instantiation_id);
CREATE INDEX audit_objects_source_idx ON audit_objects(source_entity_type, source_entity_id);
CREATE INDEX review_events_object_idx ON review_events(audit_object_id);
CREATE INDEX review_events_domain_idx ON review_events(domain_instantiation_id);
CREATE INDEX review_event_memberships_object_idx ON review_event_memberships(audit_object_id);
CREATE INDEX er_solicitations_object_idx ON er_solicitations(audit_object_id);
CREATE INDEX solicitation_events_solicitation_idx ON solicitation_events(solicitation_id, occurred_at DESC);

INSERT INTO domain_instantiations (
    id,
    domain_type,
    name,
    config,
    governed_by
) VALUES (
    '00000000-0000-0000-0000-000000000501',
    'academic_publishing',
    'Academic Publishing Demo',
    '{
        "phase_config": {
            "public_review_duration_seconds": 2592000,
            "response_rounds_permitted": 2,
            "synthesis_significance_threshold": 0.65,
            "anonymity_rules": {
                "blind_submitter": true,
                "blind_reviewer": true,
                "reviewer_reidentification_delay_seconds": 2592000
            }
        },
        "eval_tuple_config": {
            "stakes_operationalization": "scientific_significance",
            "uptake_operationalization": "citation_impact",
            "l_weight_params": {
                "solicited_review_multiplier": 1.5,
                "bounty_triggered_multiplier": 2.0,
                "expertise_weight_fn": "academic_tag_endorsement_weight_v1"
            }
        },
        "audit_object_types": ["article", "preprint", "dataset", "software", "protocol", "report"],
        "reviewer_concurrency": {
            "max_active_element_reviews": 5,
            "max_active_synthesis_reviews": 2
        }
    }'::jsonb,
    '"platform"'::jsonb
) ON CONFLICT (id) DO NOTHING;

INSERT INTO cwe_nodes (id, domain_instantiation_id, parent_id, label, description, source) VALUES
    (
        '00000000-0000-0000-0000-000000000601',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Methodological adequacy',
        'The audit object uses methods suitable for its claims, data, and inferential setting.',
        'base_taxonomy'
    ),
    (
        '00000000-0000-0000-0000-000000000602',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Statistical adequacy',
        'The audit object uses statistical methods and uncertainty claims appropriate to the evidence.',
        'base_taxonomy'
    ),
    (
        '00000000-0000-0000-0000-000000000603',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Data and code availability',
        'The audit object makes supporting data, code, materials, or protocols available at a level appropriate to its claims.',
        'base_taxonomy'
    ),
    (
        '00000000-0000-0000-0000-000000000604',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Interpretation strength',
        'The audit object states conclusions with a strength justified by the underlying evidence.',
        'base_taxonomy'
    ),
    (
        '00000000-0000-0000-0000-000000000605',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Ethical concern',
        'The audit object raises or mishandles an ethical issue relevant to the domain.',
        'base_taxonomy'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO audit_objects (
    domain_instantiation_id,
    object_type,
    title,
    submitted_at,
    status,
    submission_tier,
    external_refs,
    source_entity_type,
    source_entity_id,
    metadata
)
SELECT
    '00000000-0000-0000-0000-000000000501',
    scholarly_objects.object_type,
    scholarly_objects.title,
    scholarly_objects.created_at,
    'active',
    'tier0',
    CASE
        WHEN scholarly_objects.doi IS NULL THEN jsonb_build_array(
            jsonb_build_object(
                'system', 'url',
                'resource_type', 'canonical_url',
                'resource_id', scholarly_objects.canonical_url,
                'uri', scholarly_objects.canonical_url
            )
        )
        ELSE jsonb_build_array(
            jsonb_build_object(
                'system', 'doi',
                'resource_type', 'scholarly_work',
                'resource_id', scholarly_objects.doi,
                'uri', 'https://doi.org/' || scholarly_objects.doi
            ),
            jsonb_build_object(
                'system', 'url',
                'resource_type', 'canonical_url',
                'resource_id', scholarly_objects.canonical_url,
                'uri', scholarly_objects.canonical_url
            )
        )
    END,
    'scholarly_object',
    scholarly_objects.id,
    jsonb_build_object(
        'source', 'academic_publishing_adapter',
        'authors', scholarly_objects.authors,
        'abstract', scholarly_objects.abstract,
        'license', scholarly_objects.license,
        'canonical_url', scholarly_objects.canonical_url,
        'metadata_provenance', scholarly_objects.metadata_provenance
    )
FROM scholarly_objects
ON CONFLICT (source_entity_type, source_entity_id) DO UPDATE SET
    object_type = EXCLUDED.object_type,
    title = EXCLUDED.title,
    external_refs = EXCLUDED.external_refs,
    metadata = EXCLUDED.metadata,
    updated_at = now();
