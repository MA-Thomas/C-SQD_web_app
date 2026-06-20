use csqd_academic_adapter::{
    ArticleVersionGroupSummary, ArticleVersionKind, ArticleVersionSummary, AuditWorkStatus,
    ExternalArticleLocationSummary, ExternalArticleLocationType, ProblemAreaRelevance,
    ProblemAreaWorkSummary, ScholarlyObjectDetail, ScholarlyObjectSummary, ScholarlyObjectType,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_summaries(db: &PgPool) -> Result<Vec<ScholarlyObjectSummary>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        {}
        ORDER BY COALESCE(swg.updated_at, so.updated_at) DESC, swv.version_rank ASC NULLS LAST, so.created_at ASC
        LIMIT 50
        "#,
        scholarly_summary_select_sql()
    ))
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
    let sql = format!(
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
        {}
        JOIN matched_objects ON matched_objects.id = so.id
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
        scholarly_summary_select_sql()
    );

    let rows = sqlx::query(&sql)
        .bind(trimmed_query)
        .bind(query_pattern)
        .fetch_all(db)
        .await?;

    rows.into_iter().map(row_to_summary).collect()
}

pub async fn browse_problem_area_works(
    db: &PgPool,
    query: &str,
    cwe_node_id: Option<&str>,
) -> Result<Vec<ProblemAreaWorkSummary>, RepositoryError> {
    let cwe_context = find_cwe_browse_context(db, cwe_node_id).await?;
    let patterns = browse_text_patterns(query, cwe_context.as_ref());
    let cwe_node_id = cwe_context.as_ref().map(|context| context.id.as_str());
    let sql = format!(
        r#"
        {}
        WHERE
            (
                cardinality($1::text[]) = 0
                OR EXISTS (
                    SELECT 1
                    FROM unnest($1::text[]) AS pattern
                    WHERE lower(
                        so.title || ' ' ||
                        COALESCE(so.abstract, '') || ' ' ||
                        so.authors::text || ' ' ||
                        COALESCE(sos.search_text, '') || ' ' ||
                        COALESCE(j.name, '')
                    ) LIKE pattern
                )
            )
            OR (
                $2::text IS NOT NULL
                AND EXISTS (
                    SELECT 1
                    FROM facts f
                    WHERE f.subject_id = aus.id
                      AND f.payload_kind = 'element_review'
                      AND f.status = 'active'
                      AND f.payload->'element_review'->'cwe_criterion'->>'node_id' = $2
                )
            )
        ORDER BY problem_fact_count DESC, COALESCE(swg.updated_at, so.updated_at) DESC
        LIMIT 50
        "#,
        scholarly_summary_select_sql()
    );
    let rows = sqlx::query(&sql)
        .bind(patterns)
        .bind(cwe_node_id)
        .fetch_all(db)
        .await?;

    rows.into_iter()
        .map(|row| {
            let problem_fact_count = row.get("problem_fact_count");
            let relevance = if problem_fact_count > 0 {
                ProblemAreaRelevance::FactActivity
            } else if !query.trim().is_empty() || cwe_context.is_some() {
                ProblemAreaRelevance::TextMatch
            } else {
                ProblemAreaRelevance::RecentDomainWork
            };

            Ok(ProblemAreaWorkSummary {
                scholarly_object: row_to_summary(row)?,
                problem_fact_count,
                relevance,
            })
        })
        .collect()
}

pub async fn find_detail(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<ScholarlyObjectDetail, RepositoryError> {
    let sql = format!(
        r#"
        {}
        WHERE so.id::text = $1
        "#,
        scholarly_detail_select_sql()
    );
    let row = sqlx::query(&sql)
        .bind(scholarly_object_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "scholarly_object",
            id: scholarly_object_id.to_string(),
        })?;

    let versions = list_versions_for_object(db, scholarly_object_id).await?;
    let external_locations = list_external_locations(db, scholarly_object_id).await?;

    row_to_detail(row, versions, external_locations)
}

fn scholarly_summary_select_sql() -> &'static str {
    r#"
        SELECT
            so.id::text AS id,
            aus.id::text AS audit_subject_id,
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
            COALESCE(episode_rollup.audit_status, 'not_commissioned') AS audit_status,
            COALESCE(episode_rollup.audit_episode_count, 0) AS audit_episode_count,
            COALESCE(episode_rollup.fact_count, 0) AS fact_count,
            COALESCE(episode_rollup.element_review_fact_count, 0) AS element_review_fact_count,
            COALESCE(episode_rollup.synthesis_review_count, 0) AS synthesis_review_count,
            COALESCE(episode_rollup.element_review_fact_count, 0) AS problem_fact_count
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        LEFT JOIN audit_subjects aus
            ON aus.source_entity_type = 'scholarly_object'
           AND aus.source_entity_id = so.id
        LEFT JOIN scholarly_object_search sos ON sos.scholarly_object_id = so.id
        LEFT JOIN LATERAL (
            SELECT
                CASE
                    WHEN bool_or(ae.status = 'delivered') THEN 'delivered'
                    WHEN bool_or(ae.status = 'closed') THEN 'closed'
                    WHEN bool_or(ae.status = 'synthesis_pending') THEN 'synthesis_pending'
                    WHEN COUNT(*) FILTER (WHERE f.payload_kind = 'element_review' AND f.status = 'active') > 0 THEN 'in_progress'
                    WHEN COUNT(ae.id) > 0 THEN 'commissioned'
                    ELSE 'not_commissioned'
                END AS audit_status,
                COUNT(DISTINCT ae.id) AS audit_episode_count,
                COUNT(DISTINCT f.id) FILTER (WHERE f.status = 'active') AS fact_count,
                COUNT(DISTINCT f.id) FILTER (
                    WHERE f.payload_kind = 'element_review'
                      AND f.status = 'active'
                ) AS element_review_fact_count,
                COUNT(DISTINCT esr.id) FILTER (
                    WHERE esr.status IN ('draft', 'current')
                ) AS synthesis_review_count
            FROM audit_subjects rollup_subject
            LEFT JOIN audit_episodes ae ON ae.subject_id = rollup_subject.id
            LEFT JOIN facts f ON f.subject_id = rollup_subject.id
            LEFT JOIN episode_synthesis_reviews esr ON esr.episode_id = ae.id
            WHERE rollup_subject.id = aus.id
        ) episode_rollup ON true
        "#
}

fn scholarly_detail_select_sql() -> &'static str {
    r#"
        SELECT
            so.id::text AS id,
            aus.id::text AS audit_subject_id,
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
            COALESCE(episode_rollup.audit_status, 'not_commissioned') AS audit_status,
            COALESCE(episode_rollup.audit_episode_count, 0) AS audit_episode_count,
            COALESCE(episode_rollup.fact_count, 0) AS fact_count,
            COALESCE(episode_rollup.element_review_fact_count, 0) AS element_review_fact_count,
            COALESCE(episode_rollup.synthesis_review_count, 0) AS synthesis_review_count
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN scholarly_work_versions swv ON swv.scholarly_object_id = so.id
        LEFT JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        LEFT JOIN audit_subjects aus
            ON aus.source_entity_type = 'scholarly_object'
           AND aus.source_entity_id = so.id
        LEFT JOIN LATERAL (
            SELECT
                CASE
                    WHEN bool_or(ae.status = 'delivered') THEN 'delivered'
                    WHEN bool_or(ae.status = 'closed') THEN 'closed'
                    WHEN bool_or(ae.status = 'synthesis_pending') THEN 'synthesis_pending'
                    WHEN COUNT(*) FILTER (WHERE f.payload_kind = 'element_review' AND f.status = 'active') > 0 THEN 'in_progress'
                    WHEN COUNT(ae.id) > 0 THEN 'commissioned'
                    ELSE 'not_commissioned'
                END AS audit_status,
                COUNT(DISTINCT ae.id) AS audit_episode_count,
                COUNT(DISTINCT f.id) FILTER (WHERE f.status = 'active') AS fact_count,
                COUNT(DISTINCT f.id) FILTER (
                    WHERE f.payload_kind = 'element_review'
                      AND f.status = 'active'
                ) AS element_review_fact_count,
                COUNT(DISTINCT esr.id) FILTER (
                    WHERE esr.status IN ('draft', 'current')
                ) AS synthesis_review_count
            FROM audit_subjects rollup_subject
            LEFT JOIN audit_episodes ae ON ae.subject_id = rollup_subject.id
            LEFT JOIN facts f ON f.subject_id = rollup_subject.id
            LEFT JOIN episode_synthesis_reviews esr ON esr.episode_id = ae.id
            WHERE rollup_subject.id = aus.id
        ) episode_rollup ON true
        "#
}

async fn list_versions_for_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<Vec<ArticleVersionSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        WITH current_version AS (
            SELECT work_group_id
            FROM scholarly_work_versions
            WHERE scholarly_object_id::text = $1
        )
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
        FROM current_version
        JOIN scholarly_work_versions swv ON swv.work_group_id = current_version.work_group_id
        JOIN scholarly_objects so ON so.id = swv.scholarly_object_id
        JOIN scholarly_work_groups swg ON swg.id = swv.work_group_id
        LEFT JOIN journals j ON j.id = so.journal_id
        ORDER BY swv.version_rank ASC, so.created_at ASC
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
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
        })
        .collect()
}

async fn list_external_locations(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<Vec<ExternalArticleLocationSummary>, RepositoryError> {
    let rows = sqlx::query(
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
    .bind(scholarly_object_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_external_location).collect()
}

fn row_to_summary(row: PgRow) -> Result<ScholarlyObjectSummary, RepositoryError> {
    let object_type: String = row.get("object_type");
    let audit_status: String = row.get("audit_status");
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
        audit_subject_id: row.get("audit_subject_id"),
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
        audit_status: AuditWorkStatus::try_from(audit_status.as_str())
            .map_err(RepositoryError::Domain)?,
        audit_episode_count: row.get("audit_episode_count"),
        fact_count: row.get("fact_count"),
        element_review_fact_count: row.get("element_review_fact_count"),
        synthesis_review_count: row.get("synthesis_review_count"),
    })
}

fn row_to_detail(
    row: PgRow,
    versions: Vec<ArticleVersionSummary>,
    external_locations: Vec<ExternalArticleLocationSummary>,
) -> Result<ScholarlyObjectDetail, RepositoryError> {
    let object_type: String = row.get("object_type");
    let audit_status: String = row.get("audit_status");
    let authors: Value = row.get("authors");
    let version_kind: Option<String> = row.get("version_kind");
    let work_group_id: Option<String> = row.get("work_group_id");
    let work_group = work_group_id.map(|id| ArticleVersionGroupSummary {
        id,
        title: row.get("work_group_title"),
        primary_scholarly_object_id: row.get("primary_scholarly_object_id"),
        version_count: row.get("version_count"),
    });

    Ok(ScholarlyObjectDetail {
        id: row.get("id"),
        audit_subject_id: row.get("audit_subject_id"),
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
        audit_status: AuditWorkStatus::try_from(audit_status.as_str())
            .map_err(RepositoryError::Domain)?,
        audit_episode_count: row.get("audit_episode_count"),
        fact_count: row.get("fact_count"),
        element_review_fact_count: row.get("element_review_fact_count"),
        synthesis_review_count: row.get("synthesis_review_count"),
        external_locations,
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

struct CweBrowseContext {
    id: String,
    label: String,
    description: String,
    keywords: Vec<String>,
}

async fn find_cwe_browse_context(
    db: &PgPool,
    cwe_node_id: Option<&str>,
) -> Result<Option<CweBrowseContext>, RepositoryError> {
    let Some(cwe_node_id) = cwe_node_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            label,
            description,
            source_metadata
        FROM cwe_nodes
        WHERE id::text = $1
        "#,
    )
    .bind(cwe_node_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "cwe_node",
        id: cwe_node_id.to_string(),
    })?;

    let source_metadata: Value = row.get("source_metadata");
    let keywords = source_metadata
        .get("browse_keywords")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();

    Ok(Some(CweBrowseContext {
        id: row.get("id"),
        label: row.get("label"),
        description: row.get("description"),
        keywords,
    }))
}

fn browse_text_patterns(query: &str, cwe_context: Option<&CweBrowseContext>) -> Vec<String> {
    let mut terms = Vec::new();
    let query = query.trim();

    if !query.is_empty() {
        terms.push(query.to_string());
    }

    if let Some(context) = cwe_context {
        terms.push(context.label.clone());
        terms.push(context.description.clone());
        terms.extend(context.keywords.iter().cloned());
    }

    terms
        .into_iter()
        .flat_map(|term| {
            term.split(|character: char| !character.is_alphanumeric())
                .filter(|part| part.len() >= 4)
                .map(|part| format!("%{}%", part.to_ascii_lowercase()))
                .collect::<Vec<_>>()
        })
        .collect()
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
