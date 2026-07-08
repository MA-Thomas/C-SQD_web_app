-- Claim-scoped audits (CLAIM_SCOPED_AUDITS_MEMO.md).
--
-- The audit object becomes a bounded claim with explicit scope conditions.
-- Papers stay first-class discovery surfaces but are attached to audit
-- episodes as evidence artifacts to be inspected, not votes to be counted.
-- Warrant links — why an artifact is supposed to bear on the target claim —
-- become facts, so they are authored, timestamped, and challengeable.

-- 1. Scoped-claim audit subjects: new subject type plus claim fields.
ALTER TABLE audit_subjects DROP CONSTRAINT IF EXISTS audit_subjects_subject_type_check;
ALTER TABLE audit_subjects ADD CONSTRAINT audit_subjects_subject_type_check CHECK (
    subject_type IN (
        'scoped_claim',
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
);

-- The claim under audit, stated precisely enough that reviewers can ask what
-- would count as support, challenge, limitation, or non-applicability.
ALTER TABLE audit_subjects ADD COLUMN IF NOT EXISTS claim_statement text;

-- Structured scope conditions: [{"label": "population", "value": "adults 40-70"}, ...]
ALTER TABLE audit_subjects
    ADD COLUMN IF NOT EXISTS scope_conditions jsonb NOT NULL DEFAULT '[]'::jsonb;

-- 2. Evidence artifacts: many scholarly objects attached to one episode.
-- Attachment is epistemically neutral ("attached for inspection"); whether an
-- artifact supports, challenges, narrows, or fails to bear on the target
-- claim is an audit finding derived from warrant facts and element reviews,
-- never an intake property.
CREATE TABLE IF NOT EXISTS episode_evidence_artifacts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    episode_id uuid NOT NULL REFERENCES audit_episodes(id) ON DELETE CASCADE,
    scholarly_object_id uuid NOT NULL REFERENCES scholarly_objects(id) ON DELETE CASCADE,
    role text NOT NULL DEFAULT 'evidence' CHECK (role IN ('evidence', 'background')),
    note text,
    attached_by jsonb NOT NULL DEFAULT '"platform"'::jsonb,
    attached_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retracted')),
    status_metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (episode_id, scholarly_object_id)
);

CREATE INDEX IF NOT EXISTS episode_evidence_artifacts_episode_idx
    ON episode_evidence_artifacts(episode_id);
CREATE INDEX IF NOT EXISTS episode_evidence_artifacts_object_idx
    ON episode_evidence_artifacts(scholarly_object_id);

-- 3. Warrant assertions are facts.
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
        'curation_decision',
        'warrant_assertion'
    )
);

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
        'warrant',
        'administrative',
        'other'
    )
);

-- 4. Advertise the claim subject type in the academic domain config.
UPDATE domain_instantiations
SET config = jsonb_set(
    config,
    '{audit_subject_types}',
    (config->'audit_subject_types') || '["scoped_claim"]'::jsonb
)
WHERE domain_type = 'academic_publishing'
  AND NOT (config->'audit_subject_types') ? 'scoped_claim';
