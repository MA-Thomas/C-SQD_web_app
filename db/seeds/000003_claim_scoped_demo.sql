-- Claim-scoped audit demo.
--
-- Gives the local public UI one concrete case where a scholarly work is
-- evidence for a scoped claim, rather than the audit object itself.

INSERT INTO audit_subjects (
    id,
    domain_instantiation_id,
    subject_type,
    title,
    claim_statement,
    scope_conditions,
    external_refs,
    registered_by,
    metadata
) VALUES (
    '77777777-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'scoped_claim',
    'PD-L1 tumor clone escape claim',
    'PD-L1 therapy creates immune selection pressure that enables tumor clone escape in relapsed solid tumors.',
    '[
        {"label": "population", "value": "Adults with relapsed solid tumors treated with PD-L1 therapy"},
        {"label": "intervention", "value": "PD-L1 immune checkpoint therapy"},
        {"label": "outcome", "value": "Emergence of resistant tumor clone populations"},
        {"label": "timeframe", "value": "During or after the first documented relapse window"}
    ]'::jsonb,
    '[]'::jsonb,
    '"platform"'::jsonb,
    '{"source": "claim_scoped_demo_seed"}'::jsonb
) ON CONFLICT (id) DO UPDATE SET
    subject_type = EXCLUDED.subject_type,
    title = EXCLUDED.title,
    claim_statement = EXCLUDED.claim_statement,
    scope_conditions = EXCLUDED.scope_conditions,
    metadata = EXCLUDED.metadata,
    updated_at = now();

INSERT INTO audit_episodes (
    id,
    subject_id,
    domain_instantiation_id,
    label,
    status,
    authored_by,
    notes
) VALUES (
    '77777777-0000-0000-0000-000000000b01',
    '77777777-0000-0000-0000-000000000801',
    '00000000-0000-0000-0000-000000000501',
    'Claim-scoped audit of PD-L1 clone escape warrant',
    'active',
    '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
    'Demo claim audit: the attached paper is evidence for the claim, not the audit object.'
) ON CONFLICT DO NOTHING;

INSERT INTO episode_evidence_artifacts (
    id,
    episode_id,
    scholarly_object_id,
    role,
    note,
    attached_by,
    status
) VALUES (
    '77777777-0000-0000-0000-000000000e01',
    '77777777-0000-0000-0000-000000000b01',
    '00000000-0000-0000-0000-000000000301',
    'evidence',
    'Attached as evidence for the clone-escape claim; attachment does not imply support.',
    '"platform"'::jsonb,
    'active'
) ON CONFLICT (episode_id, scholarly_object_id) DO UPDATE SET
    role = EXCLUDED.role,
    note = EXCLUDED.note,
    attached_by = EXCLUDED.attached_by,
    status = 'active',
    status_metadata = '{}'::jsonb;

INSERT INTO facts (
    id,
    subject_id,
    domain_instantiation_id,
    occurred_at,
    payload_kind,
    payload,
    status,
    provenance
) VALUES
    (
        '77777777-0000-0000-0000-000000000c01',
        '77777777-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-03-10T00:00:00Z',
        'audit_commission',
        '{"audit_commission": {"commissioned_by": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}, "scope": [{"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000601"}, {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}], "funding": {"amount": 8500, "currency": "USD"}, "deadline": null, "confidential": false}}'::jsonb,
        'active',
        '{"source_system": "claim_scoped_demo_seed", "principal": {"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}}'::jsonb
    ),
    (
        '77777777-0000-0000-0000-000000000c02',
        '77777777-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-03-11T00:00:00Z',
        'warrant_assertion',
        '{"warrant_assertion": {"asserted_by": "00000000-0000-0000-0000-000000000002", "evidence_artifact": "77777777-0000-0000-0000-000000000e01", "artifact_claim": "The paper reports expansion of resistant tumor clones after PD-L1 therapy.", "inference_type": "mechanistic", "assumptions": "Observed clonal expansion is therapy-related rather than explained by baseline heterogeneity or sampling drift.", "rationale": "If the artifact claim survives review, it may bear on the target claim about immune selection pressure."}}'::jsonb,
        'active',
        '{"source_system": "claim_scoped_demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}}'::jsonb
    ),
    (
        '77777777-0000-0000-0000-000000000c03',
        '77777777-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000501',
        '2026-03-12T00:00:00Z',
        'element_review',
        '{"element_review": {"cwe_criterion": {"domain": "00000000-0000-0000-0000-000000000501", "node_id": "00000000-0000-0000-0000-000000000602"}, "submitted_by": "00000000-0000-0000-0000-000000000003", "solicitation": null, "finding": "inconclusive", "severity": null, "confidence": "moderate", "limitations": "The review inspects the warrant link, not the full paper.", "recommendations": "Audit whether clonal expansion persists after controlling for baseline clone prevalence.", "evidence_artifact": "77777777-0000-0000-0000-000000000e01", "warrant": "77777777-0000-0000-0000-000000000c02", "content": "The paper is relevant evidence, but the warrant from observed clone expansion to therapy-induced immune selection remains underdetermined without stronger baseline controls.", "featured": false}}'::jsonb,
        'active',
        '{"source_system": "claim_scoped_demo_seed", "principal": {"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}}'::jsonb
    )
ON CONFLICT (id) DO UPDATE SET
    payload = EXCLUDED.payload,
    status = EXCLUDED.status,
    provenance = EXCLUDED.provenance;

INSERT INTO episode_memberships (fact_id, episode_id, role, asserted_by, status) VALUES
    (
        '77777777-0000-0000-0000-000000000c01',
        '77777777-0000-0000-0000-000000000b01',
        'commission',
        '{"organization": {"organization_id": "00000000-0000-0000-0000-000000000a01"}}'::jsonb,
        'active'
    ),
    (
        '77777777-0000-0000-0000-000000000c02',
        '77777777-0000-0000-0000-000000000b01',
        'warrant',
        '{"user": {"user_id": "00000000-0000-0000-0000-000000000002"}}'::jsonb,
        'active'
    ),
    (
        '77777777-0000-0000-0000-000000000c03',
        '77777777-0000-0000-0000-000000000b01',
        'element_review',
        '{"user": {"user_id": "00000000-0000-0000-0000-000000000003"}}'::jsonb,
        'active'
    )
ON CONFLICT (fact_id, episode_id, role) DO UPDATE SET
    asserted_by = EXCLUDED.asserted_by,
    status = EXCLUDED.status;

INSERT INTO episode_synthesis_reviews (
    id,
    episode_id,
    submitted_by,
    status,
    summary,
    featured,
    unsolicited
) VALUES (
    '77777777-0000-0000-0000-000000000d01',
    '77777777-0000-0000-0000-000000000b01',
    '00000000-0000-0000-0000-000000000002',
    'current',
    'Initial claim-scoped synthesis: the attached PD-L1 paper is relevant evidence, but the warrant from clonal expansion to therapy-driven immune selection remains only partly audited.',
    true,
    false
) ON CONFLICT DO NOTHING;

INSERT INTO episode_synthesis_sections (
    id,
    review_id,
    section_type,
    content,
    referenced_facts
) VALUES (
    '77777777-0000-0000-0000-000000000d02',
    '77777777-0000-0000-0000-000000000d01',
    'evidence_integration',
    'The audit target is the scoped clone-escape claim. The PD-L1 paper is attached as evidence and contributes only through the warrant assertion and its ElementReview.',
    ARRAY[
        '77777777-0000-0000-0000-000000000c02'::uuid,
        '77777777-0000-0000-0000-000000000c03'::uuid
    ]
) ON CONFLICT DO NOTHING;
