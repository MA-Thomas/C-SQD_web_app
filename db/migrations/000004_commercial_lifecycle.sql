-- Commercial lifecycle: money-as-facts and the two-stage commission intake.
--
-- 1. New fact payload kinds. Invoices, sponsor payments, and reviewer
--    payouts are administrative facts on the audit record — immutable,
--    provenance-bearing, and excluded from the evaluation tuple. An
--    episode counts as "funded" when an active payment_received fact
--    exists for its commission (a derived view, not a status column).
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
        'warrant_assertion',
        'invoice_issued',
        'payment_received',
        'reviewer_payout'
    )
);

-- 2. Commission inquiries: the public stage-one of the two-stage
--    commission path. A stranger describes what they want audited and how
--    to reach them; an operator scopes it into a real commission (stage
--    two) after a conversation. Inquiries are pre-graph: they become part
--    of the audit record only when converted into a commission.
CREATE TABLE IF NOT EXISTS commission_inquiries (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    contact_name text NOT NULL,
    contact_email text NOT NULL,
    organization_name text,
    organization_type text NOT NULL DEFAULT 'other' CHECK (
        organization_type IN (
            'biotech',
            'venture_capital',
            'foundation',
            'university',
            'journal',
            'regulator',
            'other'
        )
    ),
    -- What they want audited, in their own words.
    subject_description text NOT NULL,
    decision_context text,
    budget_band text NOT NULL DEFAULT 'undisclosed' CHECK (
        budget_band IN (
            'under_5k',
            '5k_to_15k',
            '15k_to_50k',
            'over_50k',
            'undisclosed'
        )
    ),
    status text NOT NULL DEFAULT 'new' CHECK (
        status IN ('new', 'in_conversation', 'converted', 'declined')
    ),
    -- Set when an operator converts the inquiry into a commissioned episode.
    converted_episode_id uuid REFERENCES audit_episodes(id),
    operator_note text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS commission_inquiries_status_idx
    ON commission_inquiries (status, created_at DESC);
