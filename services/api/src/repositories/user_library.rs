use csqd_domain::{
    ArticleVersionGroupSummary, ArticleVersionKind, LibraryAddedReason, LibraryItemSummary,
    ReviewStatus, ScholarlyObjectSummary, ScholarlyObjectType,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::{audit_objects, RepositoryError};

const DEMO_LIBRARY_USER_ID: &str = "00000000-0000-0000-0000-000000000002";

pub async fn list_items(db: &PgPool) -> Result<Vec<LibraryItemSummary>, RepositoryError> {
    let sql = format!(
        r#"
        {}
        ORDER BY uli.pinned DESC, uli.added_at DESC
        "#,
        library_base_select_sql()
    );
    let rows = sqlx::query(&sql)
        .bind(DEMO_LIBRARY_USER_ID)
        .fetch_all(db)
        .await?;

    rows.into_iter().map(row_to_library_item).collect()
}

pub async fn add_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<LibraryItemSummary, RepositoryError> {
    let audit_object_id =
        audit_objects::ensure_academic_for_scholarly_object(db, scholarly_object_id).await?;

    sqlx::query(
        r#"
        INSERT INTO user_library_items (
            user_id,
            audit_object_id,
            added_reason
        )
        VALUES ($1::uuid, $2::uuid, 'manual')
        ON CONFLICT (user_id, audit_object_id) DO UPDATE SET
            archived = false,
            updated_at = now()
        "#,
    )
    .bind(DEMO_LIBRARY_USER_ID)
    .bind(&audit_object_id)
    .execute(db)
    .await?;

    find_item_by_audit_object(db, &audit_object_id).await
}

async fn find_item_by_audit_object(
    db: &PgPool,
    audit_object_id: &str,
) -> Result<LibraryItemSummary, RepositoryError> {
    let sql = format!(
        r#"
        {}
          AND uli.audit_object_id::text = $2
        "#,
        library_base_select_sql()
    );
    let rows = sqlx::query(&sql)
        .bind(DEMO_LIBRARY_USER_ID)
        .bind(audit_object_id)
        .fetch_all(db)
        .await?;

    rows.into_iter()
        .next()
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "library_item",
            id: audit_object_id.to_string(),
        })
        .and_then(row_to_library_item)
}

fn library_base_select_sql() -> &'static str {
    r#"
        SELECT
            uli.id::text AS library_item_id,
            uli.user_id::text AS library_user_id,
            uli.audit_object_id::text AS audit_object_id,
            uli.added_reason,
            uli.added_at::text AS added_at,
            so.id::text AS id,
            so.object_type,
            swg.id::text AS work_group_id,
            swg.title AS work_group_title,
            swg.primary_scholarly_object_id::text AS primary_scholarly_object_id,
            swv.version_kind,
            (
                SELECT COUNT(*)
                FROM scholarly_work_versions sibling_versions
                WHERE sibling_versions.work_group_id = swg.id
            ) AS version_count,
            so.title,
            so.authors,
            COALESCE(j.name, 'Unknown source') AS source_name,
            EXTRACT(YEAR FROM so.publication_date)::int AS publication_year,
            so.canonical_url,
            so.license,
            CASE
                WHEN EXISTS (
                    SELECT 1 FROM review_episodes re
                    WHERE re.scholarly_object_id = so.id
                      AND re.state = 'published'
                ) THEN 'published'
                WHEN EXISTS (
                    SELECT 1 FROM review_episodes re
                    WHERE re.scholarly_object_id = so.id
                      AND re.state = 'submitted'
                ) THEN 'submitted'
                WHEN EXISTS (
                    SELECT 1 FROM review_assignments ra
                    WHERE ra.scholarly_object_id = so.id
                      AND ra.state IN ('accepted', 'in_progress')
                ) THEN 'in_review'
                WHEN EXISTS (
                    SELECT 1 FROM review_assignments ra
                    WHERE ra.scholarly_object_id = so.id
                      AND ra.state IN ('created', 'offered')
                ) THEN 'assigned'
                ELSE 'not_assigned'
            END AS review_status,
            (
                SELECT COUNT(*) FROM evaluation_facts ef
                WHERE ef.scholarly_object_id = so.id
            ) AS evaluation_fact_count
        FROM user_library_items uli
        JOIN audit_objects ao ON ao.id = uli.audit_object_id
        JOIN scholarly_objects so
            ON ao.source_entity_type = 'scholarly_object'
           AND ao.source_entity_id = so.id
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        WHERE uli.user_id::text = $1
          AND uli.archived = false
        "#
}

fn row_to_library_item(row: PgRow) -> Result<LibraryItemSummary, RepositoryError> {
    let added_reason: String = row.get("added_reason");

    Ok(LibraryItemSummary {
        id: row.get("library_item_id"),
        user_id: row.get("library_user_id"),
        audit_object_id: row.get("audit_object_id"),
        added_reason: LibraryAddedReason::try_from(added_reason.as_str())
            .map_err(RepositoryError::Domain)?,
        added_at: row.get("added_at"),
        scholarly_object: row_to_scholarly_summary(row)?,
    })
}

fn row_to_scholarly_summary(row: PgRow) -> Result<ScholarlyObjectSummary, RepositoryError> {
    let object_type: String = row.get("object_type");
    let review_status: String = row.get("review_status");
    let authors: Value = row.get("authors");
    let version_kind: Option<String> = row.get("version_kind");
    let work_group_id: Option<String> = row.get("work_group_id");
    let work_group = work_group_id.map(|id| ArticleVersionGroupSummary {
        id,
        title: row.get("work_group_title"),
        primary_scholarly_object_id: row.get("primary_scholarly_object_id"),
        version_count: row.get("version_count"),
    });

    Ok(ScholarlyObjectSummary {
        id: row.get("id"),
        object_type: ScholarlyObjectType::try_from(object_type.as_str())
            .map_err(RepositoryError::Domain)?,
        work_group,
        version_kind: version_kind
            .as_deref()
            .map(ArticleVersionKind::try_from)
            .transpose()
            .map_err(RepositoryError::Domain)?
            .unwrap_or(ArticleVersionKind::Unknown),
        title: row.get("title"),
        authors: jsonb_string_array(authors),
        source_name: row.get("source_name"),
        publication_year: row.get("publication_year"),
        canonical_url: row.get("canonical_url"),
        license: row.get("license"),
        review_status: ReviewStatus::try_from(review_status.as_str())
            .map_err(RepositoryError::Domain)?,
        evaluation_fact_count: row.get("evaluation_fact_count"),
    })
}

fn jsonb_string_array(value: Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}
