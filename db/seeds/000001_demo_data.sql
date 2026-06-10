INSERT INTO users (id, email, display_name, role) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin@csqd.local', 'C-SQD Admin', 'admin'),
    ('00000000-0000-0000-0000-000000000002', 'reviewer@csqd.local', 'Demo Reviewer', 'reviewer')
ON CONFLICT (email) DO NOTHING;

INSERT INTO reviewer_profiles (id, user_id, bio, expertise_areas, status) VALUES
    (
        '00000000-0000-0000-0000-000000000101',
        '00000000-0000-0000-0000-000000000002',
        'Demo reviewer profile for local commissioned-audit development.',
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
    metadata_provenance,
    native_display_permitted
) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        'article',
        '10.0000/csqd.demo.001',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy',
        '["A. Researcher", "B. Statistician", "C. Oncologist"]'::jsonb,
        'A demo scholarly object used for local C-SQD commissioned audit workflow development.',
        '00000000-0000-0000-0000-000000000201',
        '2026-01-15',
        'CC-BY',
        'https://example.org/articles/demo-object-001',
        '{"source": "demo_seed", "doi": "10.0000/csqd.demo.001"}'::jsonb,
        false
    )
ON CONFLICT (doi) DO UPDATE SET
    title = EXCLUDED.title,
    authors = EXCLUDED.authors,
    abstract = EXCLUDED.abstract,
    journal_id = EXCLUDED.journal_id,
    publication_date = EXCLUDED.publication_date,
    license = EXCLUDED.license,
    canonical_url = EXCLUDED.canonical_url,
    metadata_provenance = EXCLUDED.metadata_provenance,
    native_display_permitted = EXCLUDED.native_display_permitted,
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
    )
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (
    id,
    title,
    normalized_title,
    primary_scholarly_object_id
) VALUES
    (
        '00000000-0000-0000-0000-000000000901',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy',
        'immune selection pressure and tumor clone escape after pd l1 therapy',
        '00000000-0000-0000-0000-000000000301'
    )
ON CONFLICT (normalized_title) DO UPDATE SET
    title = EXCLUDED.title,
    primary_scholarly_object_id = EXCLUDED.primary_scholarly_object_id,
    updated_at = now();

INSERT INTO scholarly_work_versions (
    scholarly_object_id,
    work_group_id,
    version_kind,
    version_rank,
    relationship_basis
) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        '00000000-0000-0000-0000-000000000901',
        'publisher',
        0,
        '{"source": "demo_seed", "basis": "seeded_work"}'::jsonb
    )
ON CONFLICT (scholarly_object_id) DO UPDATE SET
    work_group_id = EXCLUDED.work_group_id,
    version_kind = EXCLUDED.version_kind,
    version_rank = EXCLUDED.version_rank,
    relationship_basis = EXCLUDED.relationship_basis,
    updated_at = now();

INSERT INTO audit_subjects (
    id,
    domain_instantiation_id,
    subject_type,
    title,
    external_refs,
    registered_by,
    source_entity_type,
    source_entity_id,
    metadata
) VALUES
    (
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'research_manuscript',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy',
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
        '"platform"'::jsonb,
        'scholarly_object',
        '00000000-0000-0000-0000-000000000301',
        '{
            "source": "academic_publishing_intake",
            "authors": ["A. Researcher", "B. Statistician", "C. Oncologist"],
            "abstract": "A demo scholarly object used for local C-SQD commissioned audit workflow development.",
            "license": "CC-BY",
            "canonical_url": "https://example.org/articles/demo-object-001"
        }'::jsonb
    )
ON CONFLICT (source_entity_type, source_entity_id) DO UPDATE SET
    subject_type = EXCLUDED.subject_type,
    title = EXCLUDED.title,
    external_refs = EXCLUDED.external_refs,
    metadata = EXCLUDED.metadata,
    updated_at = now();

INSERT INTO organizations (id, name, org_type) VALUES
    (
        '00000000-0000-0000-0000-000000000a01',
        'Northstar Bio Diligence',
        'biotech'
    )
ON CONFLICT DO NOTHING;

INSERT INTO audit_episodes (
    id,
    subject_id,
    domain_instantiation_id,
    label,
    status,
    authored_by,
    notes
) VALUES
    (
        '00000000-0000-0000-0000-000000000b01',
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'Commissioned diligence audit for translational oncology claim',
        'active',
        '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
        'Demo episode for the commissioned-audit operations console.'
    )
ON CONFLICT DO NOTHING;

INSERT INTO facts (
    id,
    subject_id,
    domain_instantiation_id,
    payload_kind,
    payload,
    status,
    provenance
) VALUES
    (
        '00000000-0000-0000-0000-000000000c01',
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'audit_commission',
        '{
            "audit_commission": {
                "commissioned_by": {
                    "organization": {
                        "organization_id": "00000000-0000-0000-0000-000000000a01"
                    }
                },
                "scope": [
                    {
                        "domain": "00000000-0000-0000-0000-000000000501",
                        "node_id": "00000000-0000-0000-0000-000000000601"
                    },
                    {
                        "domain": "00000000-0000-0000-0000-000000000501",
                        "node_id": "00000000-0000-0000-0000-000000000602"
                    }
                ],
                "funding": {
                    "amount": 7500,
                    "currency": "USD"
                },
                "deadline": null,
                "confidential": false
            }
        }'::jsonb,
        'active',
        '{
            "source_system": "demo_seed",
            "source_document": null,
            "imported_at": "2026-01-15T00:00:00Z",
            "principal": {
                "organization": {
                    "organization_id": "00000000-0000-0000-0000-000000000a01"
                }
            }
        }'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000c03',
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'er_solicitation',
        '{
            "er_solicitation": {
                "issued_to": "00000000-0000-0000-0000-000000000002",
                "cwe_criterion": {
                    "domain": "00000000-0000-0000-0000-000000000501",
                    "node_id": "00000000-0000-0000-0000-000000000602"
                },
                "commission": "00000000-0000-0000-0000-000000000c01",
                "payment_scheme": {
                    "amount": {
                        "amount": 500,
                        "currency": "USD"
                    },
                    "currency": "USD",
                    "condition": "on_submission"
                }
            }
        }'::jsonb,
        'active',
        '{
            "source_system": "demo_seed",
            "source_document": null,
            "imported_at": "2026-01-15T00:00:00Z",
            "principal": "platform"
        }'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000c04',
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'solicitation_event',
        '{
            "solicitation_event": {
                "solicitation": "00000000-0000-0000-0000-000000000c03",
                "event_type": "completed",
                "principal": {
                    "user": {
                        "user_id": "00000000-0000-0000-0000-000000000002"
                    }
                },
                "note": "Demo reviewer completed the assigned statistical adequacy review."
            }
        }'::jsonb,
        'active',
        '{
            "source_system": "demo_seed",
            "source_document": null,
            "imported_at": "2026-01-16T00:00:00Z",
            "principal": {
                "user": {
                    "user_id": "00000000-0000-0000-0000-000000000002"
                }
            }
        }'::jsonb
    ),
    (
        '00000000-0000-0000-0000-000000000c02',
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        'element_review',
        '{
            "element_review": {
                "cwe_criterion": {
                    "domain": "00000000-0000-0000-0000-000000000501",
                    "node_id": "00000000-0000-0000-0000-000000000602"
                },
                "submitted_by": "00000000-0000-0000-0000-000000000002",
                "solicitation": "00000000-0000-0000-0000-000000000c03",
                "finding": "inconclusive",
                "severity": "moderate",
                "confidence": "moderate",
                "limitations": "Seeded demo review; not a real assessment.",
                "recommendations": "Request protocol-level detail on statistical assumptions and robustness checks.",
                "content": "The causal and statistical claims require closer scrutiny before the sponsor treats the result as diligence-grade evidence.",
                "featured": true
            }
        }'::jsonb,
        'active',
        '{
            "source_system": "demo_seed",
            "source_document": null,
            "imported_at": "2026-01-15T00:00:00Z",
            "principal": {
                "user": {
                    "user_id": "00000000-0000-0000-0000-000000000002"
                }
            }
        }'::jsonb
    )
ON CONFLICT (id) DO UPDATE SET
    payload = EXCLUDED.payload,
    status = EXCLUDED.status,
    provenance = EXCLUDED.provenance;

INSERT INTO episode_memberships (
    fact_id,
    episode_id,
    role,
    asserted_by,
    status
) VALUES
    (
        '00000000-0000-0000-0000-000000000c01',
        '00000000-0000-0000-0000-000000000b01',
        'commission',
        '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
        'active'
    ),
    (
        '00000000-0000-0000-0000-000000000c03',
        '00000000-0000-0000-0000-000000000b01',
        'solicitation',
        '"platform"'::jsonb,
        'active'
    ),
    (
        '00000000-0000-0000-0000-000000000c04',
        '00000000-0000-0000-0000-000000000b01',
        'solicitation_lifecycle',
        '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb,
        'active'
    ),
    (
        '00000000-0000-0000-0000-000000000c02',
        '00000000-0000-0000-0000-000000000b01',
        'element_review',
        '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb,
        'active'
    )
ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO episode_synthesis_reviews (
    id,
    episode_id,
    submitted_by,
    status,
    summary,
    featured
) VALUES (
    '00000000-0000-0000-0000-000000000d01',
    '00000000-0000-0000-0000-000000000b01',
    '00000000-0000-0000-0000-000000000002',
    'current',
    'The commissioned audit remains open, but the initial statistical adequacy review indicates that the sponsor should not treat the paper as diligence-grade evidence without additional robustness detail.',
    true
)
ON CONFLICT DO NOTHING;

INSERT INTO episode_synthesis_sections (
    id,
    review_id,
    section_type,
    content,
    referenced_facts
) VALUES (
    '00000000-0000-0000-0000-000000000d02',
    '00000000-0000-0000-0000-000000000d01',
    'evidence_integration',
    'The solicited element review is inconclusive rather than adverse, but its limitation and recommendation fields identify the next diligence step: request protocol-level statistical assumptions and robustness checks.',
    ARRAY['00000000-0000-0000-0000-000000000c02'::uuid]
)
ON CONFLICT DO NOTHING;

INSERT INTO user_library_items (
    user_id,
    subject_id,
    added_reason
) VALUES
    (
        '00000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000801',
        'commissioned'
    )
ON CONFLICT (user_id, subject_id) DO UPDATE SET
    added_reason = 'commissioned',
    archived = false,
    updated_at = now();

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    (
        '00000000-0000-0000-0000-000000000301',
        'Immune selection pressure and tumor clone escape after PD-L1 therapy oncology biostatistics reproducibility'
    )
ON CONFLICT (scholarly_object_id) DO UPDATE SET
    search_text = EXCLUDED.search_text,
    updated_at = now();
