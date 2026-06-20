use csqd_academic_adapter::{ArticleVersionGroupSummary, ArticleVersionKind};
use serde_json::json;
use sqlx::{PgPool, Row};

use super::RepositoryError;

pub async fn ensure_version_group(
    db: &PgPool,
    scholarly_object_id: &str,
    title: &str,
    version_kind: ArticleVersionKind,
    source: &str,
    source_identifier: &str,
) -> Result<ArticleVersionGroupSummary, RepositoryError> {
    let normalized_title = normalize_title(title);

    if normalized_title.is_empty() {
        return Err(RepositoryError::Domain(
            "article version group requires a non-empty title".to_string(),
        ));
    }

    let version_kind_value = version_kind.as_str();
    let relationship_basis = json!({
        "basis": "normalized_title",
        "source": source,
        "source_identifier": source_identifier,
    });

    let row = sqlx::query(
        r#"
        WITH group_row AS (
            INSERT INTO scholarly_work_groups (
                title,
                normalized_title,
                primary_scholarly_object_id
            )
            VALUES ($2, $3, $1::uuid)
            ON CONFLICT (normalized_title) DO UPDATE SET
                title = CASE
                    WHEN $4 = 'publisher' THEN EXCLUDED.title
                    ELSE scholarly_work_groups.title
                END,
                primary_scholarly_object_id = CASE
                    WHEN $4 = 'publisher' THEN $1::uuid
                    WHEN scholarly_work_groups.primary_scholarly_object_id IS NULL THEN $1::uuid
                    ELSE scholarly_work_groups.primary_scholarly_object_id
                END,
                updated_at = now()
            RETURNING id
        )
        INSERT INTO scholarly_work_versions (
            scholarly_object_id,
            work_group_id,
            version_kind,
            version_rank,
            relationship_basis
        )
        SELECT
            $1::uuid,
            group_row.id,
            $4,
            $5,
            $6
        FROM group_row
        ON CONFLICT (scholarly_object_id) DO UPDATE SET
            work_group_id = EXCLUDED.work_group_id,
            version_kind = EXCLUDED.version_kind,
            version_rank = EXCLUDED.version_rank,
            relationship_basis = EXCLUDED.relationship_basis,
            updated_at = now()
        RETURNING work_group_id::text AS work_group_id
        "#,
    )
    .bind(scholarly_object_id)
    .bind(title)
    .bind(normalized_title)
    .bind(version_kind_value)
    .bind(version_rank(&version_kind))
    .bind(relationship_basis)
    .fetch_one(db)
    .await?;

    let work_group_id: String = row.get("work_group_id");
    find_version_group(db, &work_group_id).await
}

async fn find_version_group(
    db: &PgPool,
    work_group_id: &str,
) -> Result<ArticleVersionGroupSummary, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            scholarly_work_groups.id::text AS id,
            scholarly_work_groups.title,
            scholarly_work_groups.primary_scholarly_object_id::text AS primary_scholarly_object_id,
            (
                SELECT COUNT(*)
                FROM scholarly_work_versions
                WHERE scholarly_work_versions.work_group_id = scholarly_work_groups.id
            ) AS version_count
        FROM scholarly_work_groups
        WHERE scholarly_work_groups.id::text = $1
        "#,
    )
    .bind(work_group_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "scholarly_work_group",
        id: work_group_id.to_string(),
    })?;

    Ok(ArticleVersionGroupSummary {
        id: row.get("id"),
        title: row.get("title"),
        primary_scholarly_object_id: row.get("primary_scholarly_object_id"),
        version_count: row.get("version_count"),
    })
}

fn version_rank(version_kind: &ArticleVersionKind) -> i32 {
    match version_kind {
        ArticleVersionKind::Publisher => 0,
        ArticleVersionKind::Preprint => 10,
        ArticleVersionKind::Repository => 20,
        ArticleVersionKind::Unknown => 99,
    }
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize_title;

    #[test]
    fn normalizes_title_for_version_grouping() {
        assert_eq!(
            normalize_title("Rapid Assessment of T-Cell Receptor Specificity!"),
            "rapid assessment of t cell receptor specificity"
        );
    }
}
