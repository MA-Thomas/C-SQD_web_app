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

CREATE INDEX scholarly_work_versions_group_idx ON scholarly_work_versions(work_group_id);

ALTER TABLE review_assignments
    ADD COLUMN work_group_id uuid REFERENCES scholarly_work_groups(id),
    ADD COLUMN review_target_scope text NOT NULL DEFAULT 'specific_version'
        CHECK (review_target_scope IN ('work_group', 'specific_version', 'work_and_version'));

ALTER TABLE review_episodes
    ADD COLUMN work_group_id uuid REFERENCES scholarly_work_groups(id),
    ADD COLUMN review_target_scope text NOT NULL DEFAULT 'specific_version'
        CHECK (review_target_scope IN ('work_group', 'specific_version', 'work_and_version'));

ALTER TABLE evaluation_facts
    ADD COLUMN work_group_id uuid REFERENCES scholarly_work_groups(id),
    ADD COLUMN review_target_scope text NOT NULL DEFAULT 'specific_version'
        CHECK (review_target_scope IN ('work_group', 'specific_version', 'work_and_version'));

WITH normalized_objects AS (
    SELECT
        id,
        title,
        object_type,
        trim(regexp_replace(lower(regexp_replace(title, '[^[:alnum:]]+', ' ', 'g')), '[[:space:]]+', ' ', 'g')) AS normalized_title,
        created_at
    FROM scholarly_objects
),
group_source AS (
    SELECT DISTINCT ON (normalized_title)
        id,
        title,
        normalized_title
    FROM normalized_objects
    WHERE normalized_title <> ''
    ORDER BY normalized_title, created_at ASC
)
INSERT INTO scholarly_work_groups (title, normalized_title, primary_scholarly_object_id)
SELECT title, normalized_title, id
FROM group_source
ON CONFLICT (normalized_title) DO NOTHING;

WITH normalized_objects AS (
    SELECT
        id,
        object_type,
        trim(regexp_replace(lower(regexp_replace(title, '[^[:alnum:]]+', ' ', 'g')), '[[:space:]]+', ' ', 'g')) AS normalized_title
    FROM scholarly_objects
)
INSERT INTO scholarly_work_versions (
    scholarly_object_id,
    work_group_id,
    version_kind,
    version_rank,
    relationship_basis
)
SELECT
    normalized_objects.id,
    scholarly_work_groups.id,
    CASE
        WHEN normalized_objects.object_type = 'preprint' THEN 'preprint'
        WHEN normalized_objects.object_type = 'article' THEN 'publisher'
        ELSE 'unknown'
    END,
    CASE
        WHEN normalized_objects.object_type = 'article' THEN 0
        WHEN normalized_objects.object_type = 'preprint' THEN 10
        ELSE 99
    END,
    jsonb_build_object('source', 'migration', 'basis', 'normalized_title')
FROM normalized_objects
JOIN scholarly_work_groups
    ON scholarly_work_groups.normalized_title = normalized_objects.normalized_title
ON CONFLICT (scholarly_object_id) DO NOTHING;

UPDATE review_assignments
SET work_group_id = scholarly_work_versions.work_group_id
FROM scholarly_work_versions
WHERE scholarly_work_versions.scholarly_object_id = review_assignments.scholarly_object_id;

UPDATE review_episodes
SET work_group_id = scholarly_work_versions.work_group_id
FROM scholarly_work_versions
WHERE scholarly_work_versions.scholarly_object_id = review_episodes.scholarly_object_id;

UPDATE evaluation_facts
SET work_group_id = scholarly_work_versions.work_group_id
FROM scholarly_work_versions
WHERE scholarly_work_versions.scholarly_object_id = evaluation_facts.scholarly_object_id;
