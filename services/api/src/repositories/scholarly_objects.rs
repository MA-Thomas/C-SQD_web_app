use csqd_domain::{
    ArticleVersionGroupSummary, ArticleVersionKind, ArticleVersionSummary,
    ExternalArticleLocationSummary, ExternalArticleLocationType, ReviewStatus,
    ScholarlyObjectDetail, ScholarlyObjectSummary, ScholarlyObjectType,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_summaries(db: &PgPool) -> Result<Vec<ScholarlyObjectSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
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
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        ORDER BY COALESCE(swg.updated_at, so.updated_at) DESC, swv.version_rank ASC NULLS LAST, so.created_at ASC
        LIMIT 50
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_summary).collect()
}

pub async fn search_summaries(
    db: &PgPool,
    query: &str,
) -> Result<Vec<ScholarlyObjectSummary>, RepositoryError> {
    let trimmed_query = query.trim();

    if trimmed_query.is_empty() {
        return list_summaries(db).await;
    }

    let query_pattern = format!("%{}%", trimmed_query.to_ascii_lowercase());
    let rows = sqlx::query(
        r#"
        WITH matched_objects AS (
            SELECT DISTINCT so.id
            FROM scholarly_objects so
            LEFT JOIN journals j ON j.id = so.journal_id
            LEFT JOIN scholarly_object_search sos ON sos.scholarly_object_id = so.id
            LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
            LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
            WHERE
                sos.search_vector @@ plainto_tsquery('english', $1)
                OR lower(so.title) LIKE $2
                OR lower(COALESCE(so.doi, '')) = lower($1)
                OR lower(COALESCE(so.doi, '')) LIKE $2
                OR lower(so.canonical_url) LIKE $2
                OR lower(COALESCE(j.name, '')) LIKE $2
                OR lower(COALESCE(swg.title, '')) LIKE $2
                OR lower(so.authors::text) LIKE $2
                OR lower(COALESCE(so.metadata_provenance->>'pmid', '')) = lower($1)
                OR lower(COALESCE(so.metadata_provenance->>'pmcid', '')) = lower($1)
                OR lower(COALESCE(so.metadata_provenance->>'arxiv_id', '')) = lower($1)
        )
        SELECT
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
        FROM scholarly_objects so
        JOIN matched_objects ON matched_objects.id = so.id
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        ORDER BY
            CASE
                WHEN lower(COALESCE(swg.title, so.title)) = lower($1) THEN 0
                WHEN lower(so.title) LIKE $2 THEN 1
                ELSE 2
            END,
            COALESCE(swg.updated_at, so.updated_at) DESC,
            swv.version_rank ASC NULLS LAST,
            so.created_at ASC
        LIMIT 50
        "#,
    )
    .bind(trimmed_query)
    .bind(query_pattern)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_summary).collect()
}

pub async fn find_detail(
    db: &PgPool,
    object_id: &str,
) -> Result<ScholarlyObjectDetail, RepositoryError> {
    let object_row = sqlx::query(
        r#"
        SELECT
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
            so.doi,
            so.title,
            so.authors,
            so.abstract AS abstract_text,
            COALESCE(j.name, 'Unknown source') AS source_name,
            so.publication_date::text AS publication_date,
            EXTRACT(YEAR FROM so.publication_date)::int AS publication_year,
            so.canonical_url,
            so.license,
            so.native_display_permitted,
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
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        WHERE so.id::text = $1
        "#,
    )
    .bind(object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "scholarly_object",
        id: object_id.to_string(),
    })?;

    let location_rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            location_type,
            url,
            license,
            is_canonical
        FROM external_article_locations
        WHERE scholarly_object_id::text = $1
        ORDER BY is_canonical DESC, created_at ASC
        "#,
    )
    .bind(object_id)
    .fetch_all(db)
    .await?;

    let version_rows = sqlx::query(
        r#"
        SELECT
            so.id::text AS scholarly_object_id,
            so.title,
            swv.version_kind,
            so.doi,
            COALESCE(j.name, 'Unknown source') AS source_name,
            so.canonical_url,
            so.native_display_permitted,
            so.id::text = $1 AS is_current,
            swg.primary_scholarly_object_id = so.id AS is_primary
        FROM scholarly_work_versions current_version
        JOIN scholarly_work_versions swv
            ON swv.work_group_id = current_version.work_group_id
        JOIN scholarly_objects so ON so.id = swv.scholarly_object_id
        LEFT JOIN journals j ON j.id = so.journal_id
        JOIN scholarly_work_groups swg ON swg.id = current_version.work_group_id
        WHERE current_version.scholarly_object_id::text = $1
        ORDER BY swv.version_rank ASC, so.created_at ASC
        "#,
    )
    .bind(object_id)
    .fetch_all(db)
    .await?;

    row_to_detail(object_row, location_rows, version_rows)
}

fn row_to_summary(row: PgRow) -> Result<ScholarlyObjectSummary, RepositoryError> {
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

fn row_to_detail(
    row: PgRow,
    location_rows: Vec<PgRow>,
    version_rows: Vec<PgRow>,
) -> Result<ScholarlyObjectDetail, RepositoryError> {
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
    let versions = version_rows
        .into_iter()
        .map(row_to_version_summary)
        .collect::<Result<Vec<_>, _>>()?;
    let external_locations = location_rows
        .into_iter()
        .map(row_to_external_location)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ScholarlyObjectDetail {
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
        versions,
        doi: row.get("doi"),
        title: row.get("title"),
        authors: jsonb_string_array(authors),
        abstract_text: row.get("abstract_text"),
        source_name: row.get("source_name"),
        publication_date: row.get("publication_date"),
        publication_year: row.get("publication_year"),
        canonical_url: row.get("canonical_url"),
        license: row.get("license"),
        native_display_permitted: row.get("native_display_permitted"),
        review_status: ReviewStatus::try_from(review_status.as_str())
            .map_err(RepositoryError::Domain)?,
        evaluation_fact_count: row.get("evaluation_fact_count"),
        external_locations,
    })
}

fn row_to_version_summary(row: PgRow) -> Result<ArticleVersionSummary, RepositoryError> {
    let version_kind: String = row.get("version_kind");

    Ok(ArticleVersionSummary {
        scholarly_object_id: row.get("scholarly_object_id"),
        title: row.get("title"),
        version_kind: ArticleVersionKind::try_from(version_kind.as_str())
            .map_err(RepositoryError::Domain)?,
        doi: row.get("doi"),
        source_name: row.get("source_name"),
        canonical_url: row.get("canonical_url"),
        native_display_permitted: row.get("native_display_permitted"),
        is_current: row.get("is_current"),
        is_primary: row.get("is_primary"),
    })
}

fn row_to_external_location(row: PgRow) -> Result<ExternalArticleLocationSummary, RepositoryError> {
    let location_type: String = row.get("location_type");

    Ok(ExternalArticleLocationSummary {
        id: row.get("id"),
        location_type: ExternalArticleLocationType::try_from(location_type.as_str())
            .map_err(RepositoryError::Domain)?,
        url: row.get("url"),
        license: row.get("license"),
        is_canonical: row.get("is_canonical"),
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
