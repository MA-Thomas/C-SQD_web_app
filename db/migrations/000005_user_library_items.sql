CREATE TABLE user_library_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    audit_object_id uuid NOT NULL REFERENCES audit_objects(id) ON DELETE CASCADE,
    added_reason text NOT NULL DEFAULT 'manual' CHECK (added_reason IN ('manual', 'review_created', 'assignment_accepted', 'imported', 'admin_seeded')),
    notes text,
    pinned boolean NOT NULL DEFAULT false,
    archived boolean NOT NULL DEFAULT false,
    added_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, audit_object_id)
);

CREATE INDEX user_library_items_user_idx ON user_library_items(user_id, archived, added_at DESC);
CREATE INDEX user_library_items_audit_object_idx ON user_library_items(audit_object_id);
