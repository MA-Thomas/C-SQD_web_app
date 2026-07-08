use csqd_academic_adapter::{
    AuditEpisodeInvolvementSummary, AuditTargetSummary, ClaimAuditIndexEntry, ClaimAuditRole,
    ClaimAuditScholarlyObjectSummary,
};
use csqd_domain::ScopeCondition;
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::{evidence_artifacts, RepositoryError};

pub async fn list_index(db: &PgPool) -> Result<Vec<ClaimAuditIndexEntry>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            aus.id::text AS subject_id,
            aus.subject_type,
            aus.title AS subject_title,
            aus.claim_statement,
            aus.scope_conditions,
            ae.id::text AS episode_id,
            ae.label AS episode_label,
            ae.status AS episode_status,
            so.id::text AS scholarly_object_id,
            so.object_type,
            so.title AS scholarly_object_title,
            so.authors AS scholarly_object_authors,
            COALESCE(j.name, 'Unknown source') AS source_name,
            EXTRACT(YEAR FROM so.publication_date)::int AS publication_year,
            so.canonical_url,
            COALESCE(evidence_counts.evidence_artifact_count, 0) AS evidence_artifact_count
        FROM audit_subjects aus
        JOIN LATERAL (
            SELECT ae.*
            FROM audit_episodes ae
            WHERE ae.subject_id = aus.id
            ORDER BY ae.authored_at DESC
            LIMIT 1
        ) ae ON true
        LEFT JOIN scholarly_objects so
          ON aus.source_entity_type = 'scholarly_object'
         AND aus.source_entity_id = so.id
        LEFT JOIN journals j ON j.id = so.journal_id
        LEFT JOIN LATERAL (
            SELECT COUNT(*) AS evidence_artifact_count
            FROM episode_evidence_artifacts eea
            WHERE eea.episode_id = ae.id
              AND eea.status = 'active'
        ) evidence_counts ON true
        WHERE aus.subject_type = 'scoped_claim'
           OR aus.source_entity_type = 'scholarly_object'
        ORDER BY ae.authored_at DESC, aus.updated_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(db)
    .await?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        entries.push(row_to_claim_audit_index_entry(db, row).await?);
    }

    Ok(entries)
}

async fn row_to_claim_audit_index_entry(
    db: &PgPool,
    row: PgRow,
) -> Result<ClaimAuditIndexEntry, RepositoryError> {
    let subject_type: String = row.get("subject_type");
    let episode_id: String = row.get("episode_id");
    let episode_status: String = row.get("episode_status");
    let scope_conditions: Value = row.get("scope_conditions");
    let scholarly_object_id: Option<String> = row.get("scholarly_object_id");
    let scholarly_object_authors: Option<Value> = row.get("scholarly_object_authors");
    let audit_state =
        evidence_artifacts::audit_state_for_episode(db, &episode_id, &episode_status).await?;

    Ok(ClaimAuditIndexEntry {
        subject: AuditTargetSummary {
            subject_id: row.get("subject_id"),
            subject_type: subject_type.clone(),
            title: row.get("subject_title"),
            claim_statement: row.get("claim_statement"),
            scope_conditions: serde_json::from_value::<Vec<ScopeCondition>>(scope_conditions)
                .map_err(|error| {
                    RepositoryError::Domain(format!("invalid scope conditions: {error}"))
                })?,
        },
        claim_role: if subject_type == "scoped_claim" {
            ClaimAuditRole::ExplicitScopedClaim
        } else {
            ClaimAuditRole::WorkAsClaim
        },
        primary_episode: AuditEpisodeInvolvementSummary {
            id: episode_id,
            label: row.get("episode_label"),
            status: episode_status,
        },
        audit_state,
        scholarly_object: scholarly_object_id.map(|id| ClaimAuditScholarlyObjectSummary {
            id,
            object_type: row.get("object_type"),
            title: row.get("scholarly_object_title"),
            authors: scholarly_object_authors
                .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
                .unwrap_or_default(),
            source_name: row.get("source_name"),
            publication_year: row.get("publication_year"),
            canonical_url: row.get("canonical_url"),
        }),
        evidence_artifact_count: row.get("evidence_artifact_count"),
    })
}
