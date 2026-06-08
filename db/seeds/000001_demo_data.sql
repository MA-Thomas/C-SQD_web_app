INSERT INTO users (id, email, display_name, role) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin@csqd.local', 'C-SQD Admin', 'admin'),
    ('00000000-0000-0000-0000-000000000002', 'reviewer@csqd.local', 'Demo Reviewer', 'reviewer')
ON CONFLICT (email) DO NOTHING;

INSERT INTO reviewer_profiles (id, user_id, bio, expertise_areas, status) VALUES
    (
        '00000000-0000-0000-0000-000000000101',
        '00000000-0000-0000-0000-000000000002',
        'Demo reviewer profile for local development.',
        ARRAY['biostatistics', 'reproducibility', 'oncology'],
        'active'
    )
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO journals (id, name, publisher, source_classification) VALUES
    (
        '00000000-0000-0000-0000-000000000201',
        'Open Biomedical Review Corpus',
        'C-SQD Demo Source',
        'curated'
    )
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_objects (
    id,
    object_type,
    doi,
    title,
    authors,
    abstract,
    journal_id,
    publication_date,
    license,
    canonical_url,
    native_display_permitted
) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        'article',
        '10.0000/csqd.demo.001',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy',
        '["A. Researcher", "B. Statistician", "C. Oncologist"]'::jsonb,
        'A demo scholarly object used for local C-SQD review workflow development.',
        '00000000-0000-0000-0000-000000000201',
        '2026-01-15',
        'CC-BY',
        'https://example.org/articles/demo-object-001',
        false
    )
ON CONFLICT (doi) DO NOTHING;

INSERT INTO audit_objects (
    id,
    domain_instantiation_id,
    object_type,
    title,
    submitted_by,
    submitted_at,
    status,
    submission_tier,
    external_refs,
    source_entity_type,
    source_entity_id,
    metadata
) VALUES
    (
        '00000000-0000-0000-0000-000000000701',
        '00000000-0000-0000-0000-000000000501',
        'article',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy',
        '00000000-0000-0000-0000-000000000001',
        now(),
        'active',
        'tier0',
        '[
            {
                "system": "doi",
                "resource_type": "scholarly_work",
                "resource_id": "10.0000/csqd.demo.001",
                "uri": "https://doi.org/10.0000/csqd.demo.001"
            },
            {
                "system": "url",
                "resource_type": "canonical_url",
                "resource_id": "https://example.org/articles/demo-object-001",
                "uri": "https://example.org/articles/demo-object-001"
            }
        ]'::jsonb,
        'scholarly_object',
        '00000000-0000-0000-0000-000000000301',
        '{
            "source": "academic_publishing_adapter",
            "authors": ["A. Researcher", "B. Statistician", "C. Oncologist"],
            "abstract": "A demo scholarly object used for local C-SQD review workflow development.",
            "license": "CC-BY",
            "canonical_url": "https://example.org/articles/demo-object-001"
        }'::jsonb
    )
ON CONFLICT (source_entity_type, source_entity_id) DO UPDATE SET
    object_type = EXCLUDED.object_type,
    title = EXCLUDED.title,
    submitted_by = EXCLUDED.submitted_by,
    external_refs = EXCLUDED.external_refs,
    metadata = EXCLUDED.metadata,
    updated_at = now();

INSERT INTO external_article_locations (
    scholarly_object_id,
    location_type,
    url,
    license,
    is_canonical
) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        'landing_page',
        'https://example.org/articles/demo-object-001',
        'CC-BY',
        true
    );

INSERT INTO review_assignments (
    id,
    scholarly_object_id,
    reviewer_profile_id,
    assignment_type,
    compensation_status,
    state,
    due_at,
    created_by
) VALUES
    (
        '00000000-0000-0000-0000-000000000401',
        '00000000-0000-0000-0000-000000000301',
        '00000000-0000-0000-0000-000000000101',
        'element_review',
        'eligible',
        'accepted',
        now() + interval '14 days',
        '00000000-0000-0000-0000-000000000001'
    )
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy oncology biostatistics reproducibility'
    )
ON CONFLICT (scholarly_object_id) DO UPDATE SET
    search_text = EXCLUDED.search_text,
    updated_at = now();
