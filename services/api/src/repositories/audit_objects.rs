use csqd_domain::{
    AuditObjectDetail, AuditObjectRelationSummary, AuditObjectRelationType, AuditObjectStatus,
    AuditObjectSummary, DomainType, ExternalRef, Principal, SubmissionTier,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_summaries(db: &PgPool) -> Result<Vec<AuditObjectSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ao.id::text AS id,
            ao.domain_instantiation_id::text AS domain_instantiation_id,
            di.domain_type,
            di.name AS domain_name,
            ao.object_type,
            ao.title,
            ao.status,
            ao.submission_tier,
            ao.submitted_by::text AS submitted_by,
            ao.submitted_at::text AS submitted_at,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS review_event_count,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.role = 'element_review'
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS active_element_review_count,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.role = 'synthesis_review'
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS active_synthesis_review_count
        FROM audit_objects ao
        JOIN domain_instantiations di ON di.id = ao.domain_instantiation_id
        ORDER BY ao.updated_at DESC, ao.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_summary).collect()
}

pub async fn ensure_academic_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<String, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO audit_objects (
            domain_instantiation_id,
            object_type,
            title,
            submitted_at,
            status,
            submission_tier,
            external_refs,
            source_entity_type,
            source_entity_id,
            metadata
        )
        SELECT
            '00000000-0000-0000-0000-000000000501',
            scholarly_objects.object_type,
            scholarly_objects.title,
            scholarly_objects.created_at,
            'active',
            'tier0',
            CASE
                WHEN scholarly_objects.doi IS NULL THEN jsonb_build_array(
                    jsonb_build_object(
                        'system', 'url',
                        'resource_type', 'canonical_url',
                        'resource_id', scholarly_objects.canonical_url,
                        'uri', scholarly_objects.canonical_url
                    )
                )
                ELSE jsonb_build_array(
                    jsonb_build_object(
                        'system', 'doi',
                        'resource_type', 'scholarly_work',
                        'resource_id', scholarly_objects.doi,
                        'uri', 'https://doi.org/' || scholarly_objects.doi
                    ),
                    jsonb_build_object(
                        'system', 'url',
                        'resource_type', 'canonical_url',
                        'resource_id', scholarly_objects.canonical_url,
                        'uri', scholarly_objects.canonical_url
                    )
                )
            END,
            'scholarly_object',
            scholarly_objects.id,
            jsonb_build_object(
                'source', 'academic_publishing_adapter',
                'authors', scholarly_objects.authors,
                'abstract', scholarly_objects.abstract,
                'license', scholarly_objects.license,
                'canonical_url', scholarly_objects.canonical_url,
                'metadata_provenance', scholarly_objects.metadata_provenance
            )
        FROM scholarly_objects
        WHERE scholarly_objects.id::text = $1
        ON CONFLICT (source_entity_type, source_entity_id) DO UPDATE SET
            object_type = EXCLUDED.object_type,
            title = EXCLUDED.title,
            external_refs = EXCLUDED.external_refs,
            metadata = EXCLUDED.metadata,
            updated_at = now()
        RETURNING id::text AS id
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "scholarly_object",
        id: scholarly_object_id.to_string(),
    })?;

    Ok(row.get("id"))
}

pub async fn find_detail(
    db: &PgPool,
    audit_object_id: &str,
) -> Result<AuditObjectDetail, RepositoryError> {
    let object_row = sqlx::query(
        r#"
        SELECT
            ao.id::text AS id,
            ao.domain_instantiation_id::text AS domain_instantiation_id,
            di.domain_type,
            di.name AS domain_name,
            ao.object_type,
            ao.title,
            ao.status,
            ao.submission_tier,
            ao.submitted_by::text AS submitted_by,
            ao.submitted_at::text AS submitted_at,
            ao.external_refs,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS review_event_count,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.role = 'element_review'
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS active_element_review_count,
            (
                SELECT COUNT(*)
                FROM review_event_memberships rem
                JOIN review_events re ON re.id = rem.review_event_id
                WHERE rem.audit_object_id = ao.id
                  AND rem.role = 'synthesis_review'
                  AND rem.status = 'active'
                  AND re.status = 'active'
            ) AS active_synthesis_review_count
        FROM audit_objects ao
        JOIN domain_instantiations di ON di.id = ao.domain_instantiation_id
        WHERE ao.id::text = $1
        "#,
    )
    .bind(audit_object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_object",
        id: audit_object_id.to_string(),
    })?;

    let relation_rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            source::text AS source,
            target::text AS target,
            relation_type,
            asserted_by,
            asserted_at::text AS asserted_at
        FROM audit_object_relations
        WHERE source::text = $1 OR target::text = $1
        ORDER BY asserted_at DESC
        "#,
    )
    .bind(audit_object_id)
    .fetch_all(db)
    .await?;

    row_to_detail(object_row, relation_rows)
}

fn row_to_summary(row: PgRow) -> Result<AuditObjectSummary, RepositoryError> {
    let domain_type: String = row.get("domain_type");
    let status: String = row.get("status");
    let submission_tier: String = row.get("submission_tier");

    Ok(AuditObjectSummary {
        id: row.get("id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        domain_type: DomainType::try_from(domain_type.as_str()).map_err(RepositoryError::Domain)?,
        domain_name: row.get("domain_name"),
        object_type: row.get("object_type"),
        title: row.get("title"),
        status: AuditObjectStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        submission_tier: SubmissionTier::try_from(submission_tier.as_str())
            .map_err(RepositoryError::Domain)?,
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        review_event_count: row.get("review_event_count"),
        active_element_review_count: row.get("active_element_review_count"),
        active_synthesis_review_count: row.get("active_synthesis_review_count"),
    })
}

fn row_to_detail(
    row: PgRow,
    relation_rows: Vec<PgRow>,
) -> Result<AuditObjectDetail, RepositoryError> {
    let domain_type: String = row.get("domain_type");
    let status: String = row.get("status");
    let submission_tier: String = row.get("submission_tier");
    let external_refs: Value = row.get("external_refs");
    let relations = relation_rows
        .into_iter()
        .map(row_to_relation)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AuditObjectDetail {
        id: row.get("id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        domain_type: DomainType::try_from(domain_type.as_str()).map_err(RepositoryError::Domain)?,
        domain_name: row.get("domain_name"),
        object_type: row.get("object_type"),
        title: row.get("title"),
        status: AuditObjectStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        submission_tier: SubmissionTier::try_from(submission_tier.as_str())
            .map_err(RepositoryError::Domain)?,
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        external_refs: serde_json::from_value::<Vec<ExternalRef>>(external_refs)
            .map_err(|error| RepositoryError::Domain(format!("invalid external refs: {error}")))?,
        relations,
        review_event_count: row.get("review_event_count"),
        active_element_review_count: row.get("active_element_review_count"),
        active_synthesis_review_count: row.get("active_synthesis_review_count"),
    })
}

fn row_to_relation(row: PgRow) -> Result<AuditObjectRelationSummary, RepositoryError> {
    let relation_type: String = row.get("relation_type");
    let asserted_by: Value = row.get("asserted_by");

    Ok(AuditObjectRelationSummary {
        id: row.get("id"),
        source: row.get("source"),
        target: row.get("target"),
        relation_type: AuditObjectRelationType::try_from(relation_type.as_str())
            .map_err(RepositoryError::Domain)?,
        asserted_by: serde_json::from_value::<Principal>(asserted_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        asserted_at: row.get("asserted_at"),
    })
}
