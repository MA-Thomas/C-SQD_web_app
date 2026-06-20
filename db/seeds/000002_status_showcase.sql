-- F7 status-showcase seed.
--
-- Adds audit subjects that exercise every public status label produced by
-- services/api/src/repositories/public_summary.rs::status_label, plus one
-- fully-worked audit carrying the FEN participation/petition/curation/challenge
-- variants introduced in migration 000002.
--
-- Labels covered here (the base 000001 seed already covers "Audit report
-- available"):
--   * Registered for audit       -> subject R (episode, no reviews)
--   * ElementReviews submitted   -> subject E
--   * In synthesis               -> subject S (episode status synthesis_pending)
--   * Challenged                 -> subject C (a submitter_response contests a review)
--   * Superseded                 -> subject Z (only synthesis review is superseded)
--   * Unaudited                  -> subject U (scholarly object with no audit subject)
--
-- Every statement is idempotent so the file is safe to re-run.

-- ── Extra identities ────────────────────────────────────────────
INSERT INTO users (id, email, display_name, role, roles) VALUES
    (
        '00000000-0000-0000-0000-000000000003',
        'reviewer2@csqd.local',
        'Second Reviewer',
        'reviewer',
        ARRAY['member', 'reviewer']
    ),
    (
        '00000000-0000-0000-0000-000000000004',
        'member@csqd.local',
        'Community Member',
        'reader',
        ARRAY['member']
    )
ON CONFLICT (email) DO NOTHING;

INSERT INTO reviewer_profiles (id, user_id, bio, expertise_areas, status) VALUES
    (
        '00000000-0000-0000-0000-000000000103',
        '00000000-0000-0000-0000-000000000003',
        'Second demo reviewer profile (methods and ethics).',
        ARRAY['research_ethics', 'study_design', 'genomics'],
        'active'
    )
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO organizations (id, name, org_type) VALUES
    ('00000000-0000-0000-0000-000000000a02', 'Helix Foundation', 'foundation')
ON CONFLICT DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject R — "Registered for audit": commissioned, no reviews yet.
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance,
    native_display_permitted
) VALUES (
    '11111111-0000-0000-0000-000000000301',
    'preprint',
    '10.0000/csqd.demo.r01',
    'A graph-neural model for predicting CRISPR off-target effects',
    '["D. Genomicist", "E. Engineer"]'::jsonb,
    'Preprint registered for a commissioned audit; no element reviews submitted yet.',
    '00000000-0000-0000-0000-000000000201',
    '2026-02-01', 'CC-BY',
    'https://example.org/articles/demo-object-r01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('11111111-0000-0000-0000-000000000301', 'landing_page', 'https://example.org/articles/demo-object-r01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('11111111-0000-0000-0000-000000000901',
     'A graph-neural model for predicting CRISPR off-target effects',
     'a graph neural model for predicting crispr off target effects',
     '11111111-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('11111111-0000-0000-0000-000000000301', '11111111-0000-0000-0000-000000000901', 'preprint', 0,
     '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO audit_subjects (
    id, domain_instantiation_id, subject_type, title, external_refs,
    registered_by, source_entity_type, source_entity_id, metadata
) VALUES (
    '11111111-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'preprint',
    'A graph-neural model for predicting CRISPR off-target effects',
    '[{"system": "doi", "resource_type": "scholarly_work", "resource_id": "10.0000/csqd.demo.r01", "uri": "https://doi.org/10.0000/csqd.demo.r01"}]'::jsonb,
    '"platform"'::jsonb,
    'scholarly_object',
    '11111111-0000-0000-0000-000000000301',
    '{"source": "academic_publishing_intake"}'::jsonb
) ON CONFLICT (source_entity_type, source_entity_id) DO NOTHING;

INSERT INTO audit_episodes (id, subject_id, domain_instantiation_id, label, status, authored_by, notes) VALUES (
    '11111111-0000-0000-0000-000000000b01',
    '11111111-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Commissioned audit of CRISPR off-target prediction model',
    'active',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}'::jsonb,
    'Just commissioned; reviewer solicitation pending.'
) ON CONFLICT DO NOTHING;

INSERT INTO facts (id, subject_id, domain_instantiation_id, occurred_at, payload_kind, payload, status, provenance) VALUES (
    '11111111-0000-0000-0000-000000000c01',
    '11111111-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    '2026-02-02T00:00:00Z',
    'audit_commission',
    '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000601"}], "funding": {"amount": 4000, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
    'active',
    '{"source_system": "demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}}'::jsonb
) ON CONFLICT (id) DO NOTHING;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES (
    '11111111-0000-0000-0000-000000000c01',
    '11111111-0000-0000-0000-000000000b01',
    'commission',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}'::jsonb,
    'active'
) ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('11111111-0000-0000-0000-000000000301',
     'graph neural model CRISPR off-target prediction genomics machine learning')
ON CONFLICT (scholarly_object_id) DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject E — "ElementReviews submitted": reviews exist, no synthesis.
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance, native_display_permitted
) VALUES (
    '22222222-0000-0000-0000-000000000301',
    'article',
    '10.0000/csqd.demo.e01',
    'Single-cell atlas of resistance in relapsed leukemia',
    '["F. Hematologist", "G. Bioinformatician"]'::jsonb,
    'Element reviews submitted; synthesis not yet started.',
    '00000000-0000-0000-0000-000000000201',
    '2026-01-20', 'CC-BY',
    'https://example.org/articles/demo-object-e01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('22222222-0000-0000-0000-000000000301', 'landing_page', 'https://example.org/articles/demo-object-e01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('22222222-0000-0000-0000-000000000901',
     'Single-cell atlas of resistance in relapsed leukemia',
     'single cell atlas of resistance in relapsed leukemia',
     '22222222-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('22222222-0000-0000-0000-000000000301', '22222222-0000-0000-0000-000000000901', 'publisher', 0, '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO audit_subjects (
    id, domain_instantiation_id, subject_type, title, external_refs,
    registered_by, source_entity_type, source_entity_id, metadata
) VALUES (
    '22222222-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'research_manuscript',
    'Single-cell atlas of resistance in relapsed leukemia',
    '[{"system": "doi", "resource_type": "scholarly_work", "resource_id": "10.0000/csqd.demo.e01", "uri": "https://doi.org/10.0000/csqd.demo.e01"}]'::jsonb,
    '"platform"'::jsonb,
    'scholarly_object',
    '22222222-0000-0000-0000-000000000301',
    '{"source": "academic_publishing_intake"}'::jsonb
) ON CONFLICT (source_entity_type, source_entity_id) DO NOTHING;

INSERT INTO audit_episodes (id, subject_id, domain_instantiation_id, label, status, authored_by, notes) VALUES (
    '22222222-0000-0000-0000-000000000b01',
    '22222222-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Diligence audit of relapsed-leukemia resistance atlas',
    'active',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
    'Element reviews in; synthesis author not yet assigned.'
) ON CONFLICT DO NOTHING;

INSERT INTO facts (id, subject_id, domain_instantiation_id, occurred_at, payload_kind, payload, status, provenance) VALUES
    (
        '22222222-0000-0000-0000-000000000c01',
        '22222222-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-21T00:00:00Z',
        'audit_commission',
        '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000603"}], "funding": {"amount": 6000, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}}'::jsonb
    ),
    (
        '22222222-0000-0000-0000-000000000c02',
        '22222222-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-25T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, "submitted_by": "00000000-0000-0000-0000-000000000003", "solicitation": null, "finding": "non_ethical_problem", "severity": "major", "confidence": "high", "limitations": "Reviewer could not access raw counts.", "recommendations": "Report effect sizes with confidence intervals, not only p-values.", "content": "Multiple-testing correction is not described; the resistance-signature claim may not survive FDR control.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}}'::jsonb
    ),
    (
        '22222222-0000-0000-0000-000000000c03',
        '22222222-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-26T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000603"}, "submitted_by": "00000000-0000-0000-0000-000000000002", "solicitation": null, "finding": "no_problems", "severity": null, "confidence": "moderate", "limitations": null, "recommendations": "None; data and code are appropriately deposited.", "content": "Processed data and analysis code are available in a public repository with a clear license.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}}'::jsonb
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES
    ('22222222-0000-0000-0000-000000000c01', '22222222-0000-0000-0000-000000000b01', 'commission', '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb, 'active'),
    ('22222222-0000-0000-0000-000000000c02', '22222222-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}'::jsonb, 'active'),
    ('22222222-0000-0000-0000-000000000c03', '22222222-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb, 'active')
ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('22222222-0000-0000-0000-000000000301',
     'single cell atlas resistance relapsed leukemia oncology genomics biostatistics')
ON CONFLICT (scholarly_object_id) DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject S — "In synthesis": episode in synthesis_pending state.
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance, native_display_permitted
) VALUES (
    '33333333-0000-0000-0000-000000000301',
    'article',
    '10.0000/csqd.demo.s01',
    'Causal inference for observational vaccine effectiveness',
    '["H. Epidemiologist"]'::jsonb,
    'Reviews complete; the synthesis author is integrating findings.',
    '00000000-0000-0000-0000-000000000201',
    '2026-01-10', 'CC-BY',
    'https://example.org/articles/demo-object-s01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('33333333-0000-0000-0000-000000000301', 'landing_page', 'https://example.org/articles/demo-object-s01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('33333333-0000-0000-0000-000000000901',
     'Causal inference for observational vaccine effectiveness',
     'causal inference for observational vaccine effectiveness',
     '33333333-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('33333333-0000-0000-0000-000000000301', '33333333-0000-0000-0000-000000000901', 'publisher', 0, '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO audit_subjects (
    id, domain_instantiation_id, subject_type, title, external_refs,
    registered_by, source_entity_type, source_entity_id, metadata
) VALUES (
    '33333333-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'research_manuscript',
    'Causal inference for observational vaccine effectiveness',
    '[{"system": "doi", "resource_type": "scholarly_work", "resource_id": "10.0000/csqd.demo.s01", "uri": "https://doi.org/10.0000/csqd.demo.s01"}]'::jsonb,
    '"platform"'::jsonb,
    'scholarly_object',
    '33333333-0000-0000-0000-000000000301',
    '{"source": "academic_publishing_intake"}'::jsonb
) ON CONFLICT (source_entity_type, source_entity_id) DO NOTHING;

INSERT INTO audit_episodes (id, subject_id, domain_instantiation_id, label, status, authored_by, notes) VALUES (
    '33333333-0000-0000-0000-000000000b01',
    '33333333-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Audit of observational vaccine-effectiveness causal claims',
    'synthesis_pending',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}'::jsonb,
    'All solicited reviews returned; synthesis in progress.'
) ON CONFLICT DO NOTHING;

INSERT INTO facts (id, subject_id, domain_instantiation_id, occurred_at, payload_kind, payload, status, provenance) VALUES
    (
        '33333333-0000-0000-0000-000000000c01',
        '33333333-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-11T00:00:00Z',
        'audit_commission',
        '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000604"}], "funding": {"amount": 5000, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}}'::jsonb
    ),
    (
        '33333333-0000-0000-0000-000000000c02',
        '33333333-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-14T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000604"}, "submitted_by": "00000000-0000-0000-0000-000000000003", "solicitation": null, "finding": "non_ethical_problem", "severity": "moderate", "confidence": "high", "limitations": null, "recommendations": "Soften causal language or add a sensitivity analysis for unmeasured confounding.", "content": "The design supports association; the abstract states an unqualified causal effect.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}}'::jsonb
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES
    ('33333333-0000-0000-0000-000000000c01', '33333333-0000-0000-0000-000000000b01', 'commission', '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a02"}}'::jsonb, 'active'),
    ('33333333-0000-0000-0000-000000000c02', '33333333-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}'::jsonb, 'active')
ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('33333333-0000-0000-0000-000000000301',
     'causal inference observational vaccine effectiveness epidemiology confounding')
ON CONFLICT (scholarly_object_id) DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject C — "Challenged": fully-worked audit. Carries solicitation
-- lifecycle, solicited + unsolicited element reviews, an ethical-problem
-- finding, public participation, feature + CWE petitions, an operator
-- curation decision, a current synthesis report, and a submitter response
-- that contests a review (which drives the Challenged label).
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance, native_display_permitted
) VALUES (
    '44444444-0000-0000-0000-000000000301',
    'article',
    '10.0000/csqd.demo.c01',
    'Federated learning on hospital records without explicit consent',
    '["I. Informatician", "J. Clinician"]'::jsonb,
    'A fully-worked commissioned audit with an active challenge thread.',
    '00000000-0000-0000-0000-000000000201',
    '2025-12-15', 'CC-BY',
    'https://example.org/articles/demo-object-c01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('44444444-0000-0000-0000-000000000301', 'landing_page', 'https://example.org/articles/demo-object-c01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('44444444-0000-0000-0000-000000000901',
     'Federated learning on hospital records without explicit consent',
     'federated learning on hospital records without explicit consent',
     '44444444-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('44444444-0000-0000-0000-000000000301', '44444444-0000-0000-0000-000000000901', 'publisher', 0, '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO audit_subjects (
    id, domain_instantiation_id, subject_type, title, external_refs,
    registered_by, source_entity_type, source_entity_id, metadata
) VALUES (
    '44444444-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'research_manuscript',
    'Federated learning on hospital records without explicit consent',
    '[{"system": "doi", "resource_type": "scholarly_work", "resource_id": "10.0000/csqd.demo.c01", "uri": "https://doi.org/10.0000/csqd.demo.c01"}]'::jsonb,
    '"platform"'::jsonb,
    'scholarly_object',
    '44444444-0000-0000-0000-000000000301',
    '{"source": "academic_publishing_intake"}'::jsonb
) ON CONFLICT (source_entity_type, source_entity_id) DO NOTHING;

INSERT INTO audit_episodes (id, subject_id, domain_instantiation_id, label, status, authored_by, notes) VALUES (
    '44444444-0000-0000-0000-000000000b01',
    '44444444-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Commissioned audit of consent and statistics in federated EHR study',
    'active',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
    'Public episode; solicited and unsolicited participation, with an open challenge.'
) ON CONFLICT DO NOTHING;

INSERT INTO facts (id, subject_id, domain_instantiation_id, occurred_at, payload_kind, payload, status, provenance) VALUES
    (
        '44444444-0000-0000-0000-000000000c01',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-16T00:00:00Z',
        'audit_commission',
        '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000605"}], "funding": {"amount": 9000, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c02',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-17T00:00:00Z',
        'er_solicitation',
        '{"er_solicitation": {"issued_to": "00000000-0000-0000-0000-000000000002", "cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, "commission": "44444444-0000-0000-0000-000000000c01", "payment_scheme": {"amount": {"amount": 600, "currency": "USD"}, "currency": "USD", "condition": "on_submission"}}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": "platform"}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c03',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-20T00:00:00Z',
        'solicitation_event',
        '{"solicitation_event": {"solicitation": "44444444-0000-0000-0000-000000000c02", "event_type": "completed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}, "note": "Statistical adequacy review submitted."}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c04',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-20T01:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, "submitted_by": "00000000-0000-0000-0000-000000000002", "solicitation": "44444444-0000-0000-0000-000000000c02", "finding": "non_ethical_problem", "severity": "major", "confidence": "high", "limitations": "Aggregated metrics only.", "recommendations": "Provide per-site calibration and a fairness breakdown.", "content": "Reported accuracy aggregates across sites and hides degraded performance at smaller hospitals.", "featured": true}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c05',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-21T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000605"}, "submitted_by": "00000000-0000-0000-0000-000000000003", "solicitation": null, "finding": "ethical_problem", "severity": "critical", "confidence": "high", "limitations": null, "recommendations": "Document IRB approval and the legal basis for consent waiver before any reuse.", "content": "The study trains on identifiable hospital records without explicit consent and does not describe an IRB waiver.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c06',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-22T00:00:00Z',
        'episode_participation',
        '{"episode_participation": {"episode": "44444444-0000-0000-0000-000000000b01", "participant": "00000000-0000-0000-0000-000000000004", "action": "join", "note": "Joining the public episode to add a methods review."}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c07',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-23T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000601"}, "submitted_by": "00000000-0000-0000-0000-000000000004", "solicitation": null, "finding": "no_problems", "severity": null, "confidence": "moderate", "limitations": "Reviewed the described protocol only.", "recommendations": "None.", "content": "The federated training protocol itself is methodologically sound and clearly described.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c08',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-24T00:00:00Z',
        'feature_petition',
        '{"feature_petition": {"element_review": "44444444-0000-0000-0000-000000000c05", "petitioner": "00000000-0000-0000-0000-000000000004", "rationale": "The consent finding is the most decision-relevant result and should be featured."}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c09',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-24T01:00:00Z',
        'cwe_petition',
        '{"cwe_petition": {"kind": "applicability", "cwe_node": "00000000-0000-0000-0000-000000000603", "proposed_label": null, "petitioner": "00000000-0000-0000-0000-000000000003", "rationale": "Data and code availability should be in scope: training data cannot be shared here."}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c0a',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-12-26T00:00:00Z',
        'curation_decision',
        '{"curation_decision": {"target": {"element_review": {"fact_id": "44444444-0000-0000-0000-000000000c05"}}, "decision": "feature", "decided_by": {"user": {"user_id": "00000000-0000-0000-0000-000000000001"}}, "rationale": "Granting the feature petition; the ethical finding is central.", "petitions": ["44444444-0000-0000-0000-000000000c08"]}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000001"}}}'::jsonb
    ),
    (
        '44444444-0000-0000-0000-000000000c0b',
        '44444444-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-01-05T00:00:00Z',
        'submitter_response',
        '{"submitter_response": {"responding_to": ["44444444-0000-0000-0000-000000000c04"], "response_type": "contests", "content": "The authors contest the statistical-adequacy finding: per-site calibration is reported in Supplementary Table 4, which the reviewer did not cite.", "revision_ref": null}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}}'::jsonb
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES
    ('44444444-0000-0000-0000-000000000c01', '44444444-0000-0000-0000-000000000b01', 'commission', '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c02', '44444444-0000-0000-0000-000000000b01', 'solicitation', '"platform"'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c03', '44444444-0000-0000-0000-000000000b01', 'solicitation_lifecycle', '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c04', '44444444-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c05', '44444444-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c06', '44444444-0000-0000-0000-000000000b01', 'participation', '{"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c07', '44444444-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c08', '44444444-0000-0000-0000-000000000b01', 'petition', '{"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c09', '44444444-0000-0000-0000-000000000b01', 'petition', '{"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c0a', '44444444-0000-0000-0000-000000000b01', 'curation', '{"user": {"user_id": "00000000-0000-0000-0000-000000000001"}}'::jsonb, 'active'),
    ('44444444-0000-0000-0000-000000000c0b', '44444444-0000-0000-0000-000000000b01', 'response', '{"user": {"user_id": "00000000-0000-0000-0000-000000000004"}}'::jsonb, 'active')
ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO episode_synthesis_reviews (id, episode_id, submitted_by, status, summary, featured) VALUES (
    '44444444-0000-0000-0000-000000000d01',
    '44444444-0000-0000-0000-000000000b01',
    '00000000-0000-0000-0000-000000000003',
    'current',
    'The federated model is methodologically sound but the audit surfaces a critical consent/ethics gap and a statistical-reporting problem. The sponsor should treat the consent issue as blocking pending IRB documentation.',
    true
) ON CONFLICT DO NOTHING;

INSERT INTO episode_synthesis_sections (id, review_id, section_type, content, referenced_facts) VALUES
    (
        '44444444-0000-0000-0000-000000000d02',
        '44444444-0000-0000-0000-000000000d01',
        'ethical_assessment',
        'The most serious finding is the absence of documented consent or an IRB waiver for training on identifiable records.',
        ARRAY['44444444-0000-0000-0000-000000000c05'::uuid]
    ),
    (
        '44444444-0000-0000-0000-000000000d03',
        '44444444-0000-0000-0000-000000000d01',
        'evidence_integration',
        'The statistical-adequacy review flags hidden per-site degradation; the authors contest this, citing a supplementary table. The contest is preserved as part of the record.',
        ARRAY['44444444-0000-0000-0000-000000000c04'::uuid, '44444444-0000-0000-0000-000000000c0b'::uuid]
    )
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('44444444-0000-0000-0000-000000000301',
     'federated learning hospital records consent ethics privacy machine learning fairness')
ON CONFLICT (scholarly_object_id) DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject Z — "Superseded": the only synthesis review is superseded
-- and has no current replacement.
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance, native_display_permitted
) VALUES (
    '55555555-0000-0000-0000-000000000301',
    'preprint',
    '10.0000/csqd.demo.z01',
    'Early withdrawn report on a tumor microbiome signature',
    '["K. Microbiologist"]'::jsonb,
    'The original audit report was superseded after the preprint was substantially revised.',
    '00000000-0000-0000-0000-000000000201',
    '2025-11-01', 'CC-BY',
    'https://example.org/articles/demo-object-z01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('55555555-0000-0000-0000-000000000301', 'landing_page', 'https://example.org/articles/demo-object-z01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('55555555-0000-0000-0000-000000000901',
     'Early withdrawn report on a tumor microbiome signature',
     'early withdrawn report on a tumor microbiome signature',
     '55555555-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('55555555-0000-0000-0000-000000000301', '55555555-0000-0000-0000-000000000901', 'preprint', 0, '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO audit_subjects (
    id, domain_instantiation_id, subject_type, title, external_refs,
    registered_by, source_entity_type, source_entity_id, metadata
) VALUES (
    '55555555-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'preprint',
    'Early withdrawn report on a tumor microbiome signature',
    '[{"system": "doi", "resource_type": "scholarly_work", "resource_id": "10.0000/csqd.demo.z01", "uri": "https://doi.org/10.0000/csqd.demo.z01"}]'::jsonb,
    '"platform"'::jsonb,
    'scholarly_object',
    '55555555-0000-0000-0000-000000000301',
    '{"source": "academic_publishing_intake"}'::jsonb
) ON CONFLICT (source_entity_type, source_entity_id) DO NOTHING;

INSERT INTO audit_episodes (id, subject_id, domain_instantiation_id, label, status, authored_by, notes) VALUES (
    '55555555-0000-0000-0000-000000000b01',
    '55555555-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Audit of tumor microbiome signature (superseded)',
    'closed',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
    'Report superseded after the preprint was revised; no replacement synthesis yet.'
) ON CONFLICT DO NOTHING;

INSERT INTO facts (id, subject_id, domain_instantiation_id, occurred_at, payload_kind, payload, status, provenance) VALUES
    (
        '55555555-0000-0000-0000-000000000c01',
        '55555555-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-11-02T00:00:00Z',
        'audit_commission',
        '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000604"}], "funding": {"amount": 3500, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}}'::jsonb
    ),
    (
        '55555555-0000-0000-0000-000000000c02',
        '55555555-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2025-11-05T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000604"}, "submitted_by": "00000000-0000-0000-0000-000000000002", "solicitation": null, "finding": "non_ethical_problem", "severity": "moderate", "confidence": "moderate", "limitations": "Assessed the original preprint version.", "recommendations": "Re-audit against the revised version.", "content": "Conclusions in the original version overreached; the revision addresses several points, so this assessment no longer applies.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}}'::jsonb
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES
    ('55555555-0000-0000-0000-000000000c01', '55555555-0000-0000-0000-000000000b01', 'commission', '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb, 'active'),
    ('55555555-0000-0000-0000-000000000c02', '55555555-0000-0000-0000-000000000b01', 'element_review', '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb, 'active')
ON CONFLICT (fact_id, episode_id, role) DO NOTHING;

INSERT INTO episode_synthesis_reviews (id, episode_id, submitted_by, status, summary, featured) VALUES (
    '55555555-0000-0000-0000-000000000d01',
    '55555555-0000-0000-0000-000000000b01',
    '00000000-0000-0000-0000-000000000002',
    'superseded',
    'Original audit report. Superseded after the authors revised the preprint; retained for provenance and not treated as the current assessment.',
    false
) ON CONFLICT DO NOTHING;

INSERT INTO episode_synthesis_sections (id, review_id, section_type, content, referenced_facts) VALUES (
    '55555555-0000-0000-0000-000000000d02',
    '55555555-0000-0000-0000-000000000d01',
    'recommendations',
    'Commission a fresh audit episode against the revised version before relying on this conclusion.',
    ARRAY['55555555-0000-0000-0000-000000000c02'::uuid]
) ON CONFLICT DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('55555555-0000-0000-0000-000000000301',
     'tumor microbiome signature oncology superseded withdrawn preprint')
ON CONFLICT (scholarly_object_id) DO NOTHING;

-- ════════════════════════════════════════════════════════════════
-- Subject U — "Unaudited": a discoverable scholarly object with no
-- audit subject registered against it.
-- ════════════════════════════════════════════════════════════════
INSERT INTO scholarly_objects (
    id, object_type, doi, title, authors, abstract, journal_id,
    publication_date, license, canonical_url, metadata_provenance, native_display_permitted
) VALUES (
    '66666666-0000-0000-0000-000000000301',
    'dataset',
    '10.0000/csqd.demo.u01',
    'Benchmark dataset of annotated histopathology slides',
    '["L. Pathologist"]'::jsonb,
    'Discoverable scholarly object with no audit commissioned yet.',
    '00000000-0000-0000-0000-000000000201',
    '2026-03-01', 'CC-BY',
    'https://example.org/articles/demo-object-u01',
    '{"source": "demo_seed"}'::jsonb,
    false
) ON CONFLICT (doi) DO NOTHING;

INSERT INTO external_article_locations (scholarly_object_id, location_type, url, license, is_canonical) VALUES
    ('66666666-0000-0000-0000-000000000301', 'repository', 'https://example.org/articles/demo-object-u01', 'CC-BY', true)
ON CONFLICT DO NOTHING;

INSERT INTO scholarly_work_groups (id, title, normalized_title, primary_scholarly_object_id) VALUES
    ('66666666-0000-0000-0000-000000000901',
     'Benchmark dataset of annotated histopathology slides',
     'benchmark dataset of annotated histopathology slides',
     '66666666-0000-0000-0000-000000000301')
ON CONFLICT (normalized_title) DO NOTHING;

INSERT INTO scholarly_work_versions (scholarly_object_id, work_group_id, version_kind, version_rank, relationship_basis) VALUES
    ('66666666-0000-0000-0000-000000000301', '66666666-0000-0000-0000-000000000901', 'repository', 0, '{"source": "demo_seed"}'::jsonb)
ON CONFLICT (scholarly_object_id) DO NOTHING;

INSERT INTO scholarly_object_search (scholarly_object_id, search_text) VALUES
    ('66666666-0000-0000-0000-000000000301',
     'benchmark dataset annotated histopathology slides pathology unaudited')
ON CONFLICT (scholarly_object_id) DO NOTHING;
