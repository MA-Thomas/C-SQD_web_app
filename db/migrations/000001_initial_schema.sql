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
    user_id uuid NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    bio text,
    expertise_areas text[] NOT NULL DEFAULT '{}',
    status text NOT NULL DEFAULT 'candidate' CHECK (status IN ('candidate', 'active', 'suspended')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE domain_instantiations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_type text NOT NULL CHECK (
        domain_type IN (
            'academic_publishing',
            'clinical_trial_review',
            'ai_auditing',
            'policy_review',
            'custom'
        )
    ),
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
    source text NOT NULL CHECK (source IN ('base_taxonomy', 'community_extension')),
    source_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE organizations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    org_type text NOT NULL CHECK (
        org_type IN (
            'biotech',
            'venture_capital',
            'foundation',
            'university',
            'journal',
            'regulator',
            'other'
        )
    ),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_subjects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id),
    subject_type text NOT NULL CHECK (
        subject_type IN (
            'research_manuscript',
            'preprint',
            'dataset',
            'code_repository',
            'clinical_trial_protocol',
            'ai_model_evaluation',
            'benchmark',
            'policy_document',
            'grant_proposal',
            'technical_report',
            'other'
        )
    ),
    title text,
    external_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    registered_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    registered_at timestamptz NOT NULL DEFAULT now(),
    source_entity_type text,
    source_entity_id uuid,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_entity_type, source_entity_id)
);

CREATE TABLE facts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id uuid NOT NULL REFERENCES audit_subjects(id) ON DELETE CASCADE,
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    payload_kind text NOT NULL CHECK (
        payload_kind IN (
            'audit_commission',
            'element_review',
            'er_solicitation',
            'solicitation_event',
            'submitter_response'
        )
    ),
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded', 'retracted')),
    status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    provenance jsonb NOT NULL DEFAULT '{}'::jsonb,
    external_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    source_entity_type text,
    source_entity_id uuid,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_entity_type, source_entity_id)
);

CREATE TABLE audit_episodes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id uuid NOT NULL REFERENCES audit_subjects(id) ON DELETE CASCADE,
    domain_instantiation_id uuid NOT NULL REFERENCES domain_instantiations(id),
    label text NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (
        status IN ('active', 'synthesis_pending', 'delivered', 'closed')
    ),
    authored_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    authored_at timestamptz NOT NULL DEFAULT now(),
    notes text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE episode_memberships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    fact_id uuid NOT NULL REFERENCES facts(id) ON DELETE CASCADE,
    episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    role text NOT NULL CHECK (
        role IN (
            'commission',
            'element_review',
            'solicitation',
            'solicitation_lifecycle',
            'response',
            'administrative',
            'other'
        )
    ),
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retracted')),
    status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (fact_id, episode_id, role)
);

CREATE TABLE episode_relations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    target_episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    relation_type text NOT NULL CHECK (
        relation_type IN ('supersedes', 'split_from', 'merged_into', 'related_to', 'part_of')
    ),
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE episode_synthesis_reviews (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    submitted_by uuid NOT NULL REFERENCES users(id),
    authored_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'current', 'superseded')),
    summary text NOT NULL,
    featured boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE episode_synthesis_sections (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id uuid NOT NULL REFERENCES episode_synthesis_reviews(id) ON DELETE CASCADE,
    section_type text NOT NULL CHECK (
        section_type IN (
            'summary',
            'methodological_assessment',
            'ethical_assessment',
            'evidence_integration',
            'recommendations',
            'open_questions'
        )
    ),
    content text NOT NULL,
    referenced_facts uuid[] NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE episode_synthesis_review_relations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source uuid NOT NULL REFERENCES episode_synthesis_reviews(id) ON DELETE CASCADE,
    target uuid NOT NULL REFERENCES episode_synthesis_reviews(id) ON DELETE CASCADE,
    relation_type text NOT NULL CHECK (relation_type IN ('supersedes', 'contests', 'related_to')),
    contestation_scope text CHECK (contestation_scope IN ('partial', 'full')),
    rationale text,
    asserted_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    asserted_at timestamptz NOT NULL DEFAULT now()
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
    object_type text NOT NULL CHECK (
        object_type IN ('article', 'preprint', 'dataset', 'software', 'protocol', 'report')
    ),
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
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id) ON DELETE CASCADE,
    location_type text NOT NULL CHECK (location_type IN ('publisher', 'landing_page', 'full_text', 'pdf', 'repository')),
    url text NOT NULL,
    license text,
    is_canonical boolean NOT NULL DEFAULT false,
    provenance jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE scholarly_work_groups (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    title text NOT NULL,
    normalized_title text NOT NULL UNIQUE,
    primary_scholarly_object_id uuid REFERENCES scholarly_objects(id),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE scholarly_work_versions (
    scholarly_object_id uuid PRIMARY KEY REFERENCES scholarly_objects(id) ON DELETE CASCADE,
    work_group_id uuid NOT NULL REFERENCES scholarly_work_groups(id) ON DELETE CASCADE,
    version_kind text NOT NULL DEFAULT 'unknown' CHECK (version_kind IN ('publisher', 'preprint', 'repository', 'unknown')),
    version_rank integer NOT NULL DEFAULT 99,
    relationship_basis jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE scholarly_object_search (
    scholarly_object_id uuid PRIMARY KEY REFERENCES scholarly_objects(id) ON DELETE CASCADE,
    search_text text NOT NULL,
    search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', search_text)) STORED,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE user_library_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject_id uuid NOT NULL REFERENCES audit_subjects(id) ON DELETE CASCADE,
    added_reason text NOT NULL DEFAULT 'manual' CHECK (
        added_reason IN ('manual', 'commissioned', 'imported', 'admin_seeded')
    ),
    notes text,
    pinned boolean NOT NULL DEFAULT false,
    archived boolean NOT NULL DEFAULT false,
    added_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, subject_id)
);

CREATE INDEX cwe_nodes_domain_idx ON cwe_nodes(domain_instantiation_id);
CREATE INDEX audit_subjects_domain_idx ON audit_subjects(domain_instantiation_id);
CREATE INDEX audit_subjects_source_idx ON audit_subjects(source_entity_type, source_entity_id);
CREATE INDEX facts_subject_idx ON facts(subject_id, occurred_at DESC);
CREATE INDEX facts_domain_idx ON facts(domain_instantiation_id);
CREATE INDEX facts_payload_kind_idx ON facts(payload_kind);
CREATE INDEX audit_episodes_subject_idx ON audit_episodes(subject_id, authored_at DESC);
CREATE INDEX episode_memberships_episode_idx ON episode_memberships(episode_id);
CREATE INDEX episode_memberships_fact_idx ON episode_memberships(fact_id);
CREATE INDEX episode_synthesis_reviews_episode_idx ON episode_synthesis_reviews(episode_id);
CREATE INDEX external_article_locations_object_idx ON external_article_locations(scholarly_object_id);
CREATE INDEX scholarly_object_search_vector_idx ON scholarly_object_search USING gin(search_vector);
CREATE INDEX scholarly_work_versions_group_idx ON scholarly_work_versions(work_group_id);
CREATE INDEX user_library_items_user_idx ON user_library_items(user_id, archived, added_at DESC);
CREATE INDEX user_library_items_subject_idx ON user_library_items(subject_id);

INSERT INTO domain_instantiations (
    id,
    domain_type,
    name,
    config,
    governed_by
) VALUES (
    '00000000-0000-0000-0000-000000000501',
    'academic_publishing',
    'Academic Publishing Commissioned Audits',
    '{
        "phase_config": null,
        "eval_tuple_config": {
            "stakes_operationalization": "scientific_significance",
            "uptake_operationalization": "citation_impact",
            "l_weight_params": {
                "solicited_review_multiplier": 1.5,
                "expertise_weight_fn": "academic_tag_endorsement_weight_v1"
            }
        },
        "audit_subject_types": ["research_manuscript", "preprint", "dataset", "code_repository", "clinical_trial_protocol", "technical_report"]
    }'::jsonb,
    '"platform"'::jsonb
) ON CONFLICT (id) DO NOTHING;

INSERT INTO cwe_nodes (
    id,
    domain_instantiation_id,
    parent_id,
    label,
    description,
    source,
    source_metadata
) VALUES
    (
        '00000000-0000-0000-0000-000000000601',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Methodological adequacy',
        'The audit subject uses methods suitable for its claims, data, and inferential setting.',
        'base_taxonomy',
        '{"browse_keywords": ["method", "methods", "methodology", "study design", "protocol", "reproducibility", "replication"]}'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000602',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Statistical adequacy',
        'The audit subject uses statistical methods and uncertainty claims appropriate to the evidence.',
        'base_taxonomy',
        '{"browse_keywords": ["statistics", "statistical", "biostatistics", "uncertainty", "power", "confidence interval", "p value"]}'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000603',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Data and code availability',
        'The audit subject makes supporting data, code, materials, or protocols available at a level appropriate to its claims.',
        'base_taxonomy',
        '{"browse_keywords": ["data", "code", "materials", "availability", "open data", "repository", "reproducibility"]}'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000604',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Interpretation strength',
        'The audit subject states conclusions with a strength justified by the underlying evidence.',
        'base_taxonomy',
        '{"browse_keywords": ["interpretation", "conclusion", "claim strength", "causal claim", "inference", "evidence strength"]}'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000605',
        '00000000-0000-0000-0000-000000000501',
        NULL,
        'Ethical concern',
        'The audit subject raises or mishandles an ethical issue relevant to the domain.',
        'base_taxonomy',
        '{"browse_keywords": ["ethics", "ethical", "consent", "privacy", "risk", "harm", "equity"]}'::jsonb
    )
ON CONFLICT (id) DO NOTHING;
