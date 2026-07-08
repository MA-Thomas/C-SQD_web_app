//! Evidence artifacts attached to audit episodes (claim-scoped audits memo).
//!
//! Attachment is epistemically neutral: these rows record that an artifact
//! was attached for inspection, never that it supports the target claim.
//! Bearing is derived per read from warrant facts and element reviews via
//! `csqd_domain::derive_artifact_bearing` — the same pure-function pattern
//! as the evaluation tuple.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use csqd_academic_adapter::{
    AuditEpisodeInvolvementSummary, AuditTargetSummary, EvidenceArtifactSummary,
    InvolvementAuditState, WorkAuditInvolvement, WorkRoleInAudit,
};
use csqd_domain::{
    derive_artifact_bearing, AttachEvidenceArtifactRequest, EpisodeEvidenceArtifact,
    EvidenceArtifactId, EvidenceArtifactStatus, EvidenceRole, Fact, FactPayload, FactStatus,
    NarrativeStatus, Principal, ScopeCondition, SynthesisReview,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::{audit_episodes, RepositoryError};

const LINK_COLUMNS: &str = r#"
            eea.id::text AS id,
            eea.episode_id::text AS episode_id,
            eea.scholarly_object_id::text AS scholarly_object_id,
            eea.role,
            eea.note,
            eea.attached_by,
            eea.attached_at,
            eea.status,
            eea.status_metadata,
            so.title,
            so.authors,
            so.canonical_url,
            EXTRACT(YEAR FROM so.publication_date)::int AS publication_year,
            COALESCE(j.name, 'Unknown source') AS source_name
"#;

pub async fn attach(
    db: &PgPool,
    episode_id: &str,
    request: AttachEvidenceArtifactRequest,
) -> Result<EvidenceArtifactSummary, RepositoryError> {
    ensure_episode_exists(db, episode_id).await?;
    ensure_scholarly_object_exists(db, &request.scholarly_object_id).await?;

    let attached_by = serde_json::to_value(request.attached_by.unwrap_or(Principal::Platform))
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;

    // Re-attaching a previously retracted artifact reactivates the link; the
    // retraction remains visible in the row's status metadata history is not
    // required by the memo, so the link is simply made active again.
    let row = sqlx::query(&format!(
        r#"
        WITH upserted AS (
            INSERT INTO episode_evidence_artifacts (
                episode_id,
                scholarly_object_id,
                role,
                note,
                attached_by,
                status
            )
            VALUES ($1::uuid, $2::uuid, $3, $4, $5::jsonb, 'active')
            ON CONFLICT (episode_id, scholarly_object_id) DO UPDATE SET
                role = EXCLUDED.role,
                note = EXCLUDED.note,
                attached_by = EXCLUDED.attached_by,
                status = 'active',
                status_metadata = '{{}}'::jsonb
            RETURNING *
        )
        SELECT {LINK_COLUMNS}
        FROM upserted eea
        JOIN scholarly_objects so ON so.id = eea.scholarly_object_id
        LEFT JOIN journals j ON j.id = so.journal_id
        "#
    ))
    .bind(episode_id)
    .bind(&request.scholarly_object_id)
    .bind(request.role.as_db_str())
    .bind(request.note.as_deref().map(str::trim))
    .bind(attached_by)
    .fetch_one(db)
    .await?;

    let facts = audit_episodes::list_facts_for_episode(db, episode_id).await?;

    row_to_summary(row, &facts)
}

pub async fn retract(
    db: &PgPool,
    episode_id: &str,
    artifact_id: &str,
    retracted_by: Principal,
) -> Result<(), RepositoryError> {
    let principal = serde_json::to_value(&retracted_by)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let result = sqlx::query(
        r#"
        UPDATE episode_evidence_artifacts
        SET status = 'retracted',
            status_metadata = jsonb_build_object(
                'retracted_by', $3::jsonb,
                'retracted_at', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')
            )
        WHERE id::text = $1
          AND episode_id::text = $2
        "#,
    )
    .bind(artifact_id)
    .bind(episode_id)
    .bind(principal)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound {
            entity: "evidence_artifact",
            id: artifact_id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_summaries_for_episode(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<EvidenceArtifactSummary>, RepositoryError> {
    ensure_episode_exists(db, episode_id).await?;

    let rows = sqlx::query(&format!(
        r#"
        SELECT {LINK_COLUMNS}
        FROM episode_evidence_artifacts eea
        JOIN scholarly_objects so ON so.id = eea.scholarly_object_id
        LEFT JOIN journals j ON j.id = so.journal_id
        WHERE eea.episode_id::text = $1
          AND eea.status = 'active'
        ORDER BY eea.attached_at ASC
        "#
    ))
    .bind(episode_id)
    .fetch_all(db)
    .await?;

    let facts = audit_episodes::list_facts_for_episode(db, episode_id).await?;

    rows.into_iter()
        .map(|row| row_to_summary(row, &facts))
        .collect()
}

/// Confirms an active evidence-artifact link belongs to the episode; used to
/// validate warrant assertions and artifact-targeted element reviews.
pub async fn ensure_active_link(
    db: &PgPool,
    episode_id: &str,
    artifact_id: &str,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM episode_evidence_artifacts
            WHERE id::text = $1
              AND episode_id::text = $2
              AND status = 'active'
        ) AS exists
        "#,
    )
    .bind(artifact_id)
    .bind(episode_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "evidence_artifact",
            id: artifact_id.to_string(),
        })
    }
}

/// Every audit episode a scholarly work participates in — as the subject of
/// a single-paper audit or as attached evidence in a claim-scoped one. The
/// paper page stays a first-class discovery surface without being the
/// audit's epistemic target.
pub async fn list_involvements_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<Vec<WorkAuditInvolvement>, RepositoryError> {
    let subject_rows = sqlx::query(
        r#"
        SELECT
            ae.id::text AS episode_id,
            ae.label AS episode_label,
            ae.status AS episode_status,
            aus.id::text AS subject_id,
            aus.subject_type,
            aus.title AS subject_title,
            aus.claim_statement,
            aus.scope_conditions
        FROM audit_episodes ae
        JOIN audit_subjects aus ON aus.id = ae.subject_id
        WHERE aus.source_entity_type = 'scholarly_object'
          AND aus.source_entity_id::text = $1
        ORDER BY ae.authored_at DESC
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_all(db)
    .await?;

    let evidence_rows = sqlx::query(
        r#"
        SELECT
            ae.id::text AS episode_id,
            ae.label AS episode_label,
            ae.status AS episode_status,
            aus.id::text AS subject_id,
            aus.subject_type,
            aus.title AS subject_title,
            aus.claim_statement,
            aus.scope_conditions,
            eea.id::text AS artifact_id,
            eea.role
        FROM episode_evidence_artifacts eea
        JOIN audit_episodes ae ON ae.id = eea.episode_id
        JOIN audit_subjects aus ON aus.id = ae.subject_id
        WHERE eea.scholarly_object_id::text = $1
          AND eea.status = 'active'
        ORDER BY eea.attached_at DESC
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_all(db)
    .await?;

    let mut involvements = Vec::with_capacity(subject_rows.len() + evidence_rows.len());
    let mut episode_contexts = HashMap::new();

    for row in subject_rows {
        let episode_id: String = row.get("episode_id");
        let episode_status: String = row.get("episode_status");
        let audit_target = row_to_audit_target(&row)?;
        let episode = row_to_involvement_episode(&row);
        let episode_context =
            cached_episode_context(db, &episode_id, &episode_status, &mut episode_contexts).await?;

        involvements.push(WorkAuditInvolvement {
            episode,
            audit_target,
            work_role: WorkRoleInAudit::DirectSubject,
            audit_state: episode_context.audit_state,
        });
    }

    for row in evidence_rows {
        let episode_id: String = row.get("episode_id");
        let episode_status: String = row.get("episode_status");
        let artifact_id: String = row.get("artifact_id");
        let role: String = row.get("role");
        let audit_target = row_to_audit_target(&row)?;
        let episode = row_to_involvement_episode(&row);
        let episode_context =
            cached_episode_context(db, &episode_id, &episode_status, &mut episode_contexts).await?;
        let artifact_id = EvidenceArtifactId::new(artifact_id);
        let bearing = derive_artifact_bearing(&artifact_id, &episode_context.facts);
        let (warrant_count, review_count) =
            artifact_fact_counts(&artifact_id, &episode_context.facts);
        let work_role =
            match EvidenceRole::try_from(role.as_str()).map_err(RepositoryError::Domain)? {
                EvidenceRole::Evidence => WorkRoleInAudit::Evidence {
                    artifact_id: artifact_id.as_str().to_string(),
                    bearing: bearing.clone(),
                    warrant_count,
                    review_count,
                },
                EvidenceRole::Background => WorkRoleInAudit::Background {
                    artifact_id: artifact_id.as_str().to_string(),
                    bearing,
                    warrant_count,
                    review_count,
                },
            };

        involvements.push(WorkAuditInvolvement {
            episode,
            audit_target,
            work_role,
            audit_state: episode_context.audit_state,
        });
    }

    Ok(involvements)
}

pub async fn audit_state_for_episode(
    db: &PgPool,
    episode_id: &str,
    episode_status: &str,
) -> Result<InvolvementAuditState, RepositoryError> {
    Ok(build_episode_context(db, episode_id, episode_status)
        .await?
        .audit_state)
}

#[derive(Clone)]
struct InvolvementEpisodeContext {
    facts: Vec<Fact>,
    audit_state: InvolvementAuditState,
}

async fn cached_episode_context(
    db: &PgPool,
    episode_id: &str,
    episode_status: &str,
    cache: &mut HashMap<String, InvolvementEpisodeContext>,
) -> Result<InvolvementEpisodeContext, RepositoryError> {
    if let Some(context) = cache.get(episode_id) {
        return Ok(context.clone());
    }

    let context = build_episode_context(db, episode_id, episode_status).await?;
    cache.insert(episode_id.to_string(), context.clone());

    Ok(context)
}

async fn build_episode_context(
    db: &PgPool,
    episode_id: &str,
    episode_status: &str,
) -> Result<InvolvementEpisodeContext, RepositoryError> {
    let facts = audit_episodes::list_facts_for_episode(db, episode_id).await?;
    let tuple = audit_episodes::compute_eval_tuple(
        db,
        episode_id,
        audit_episodes::EvalTupleQuery::default(),
    )
    .await?;
    let all_reports = audit_episodes::list_synthesis_reviews(db, episode_id).await?;
    let reports = active_synthesis_reviews(all_reports.clone());
    let element_review_count = facts
        .iter()
        .filter(|fact| matches!(fact.status, FactStatus::Active))
        .filter(|fact| matches!(fact.payload, FactPayload::ElementReview { .. }))
        .count() as i64;
    let challenge_count = challenge_count_for_episode(db, episode_id).await?;
    let synthesis_review_count = reports.len() as i64;
    let status_label = involvement_status_label(
        episode_status,
        element_review_count,
        synthesis_review_count,
        challenge_count,
        &all_reports,
    );

    Ok(InvolvementEpisodeContext {
        facts,
        audit_state: InvolvementAuditState {
            status_label,
            tuple: Some(tuple),
            latest_synthesis: reports.first().cloned(),
            element_review_count,
            synthesis_review_count,
            challenge_count,
        },
    })
}

fn active_synthesis_reviews(mut reports: Vec<SynthesisReview>) -> Vec<SynthesisReview> {
    reports.sort_by(|left, right| right.authored_at.cmp(&left.authored_at));

    reports
        .into_iter()
        .filter(|review| {
            matches!(
                review.status,
                NarrativeStatus::Current | NarrativeStatus::Draft
            )
        })
        .collect()
}

fn involvement_status_label(
    episode_status: &str,
    element_review_count: i64,
    synthesis_review_count: i64,
    challenge_count: i64,
    all_reports: &[SynthesisReview],
) -> String {
    if challenge_count > 0 {
        return "Challenged".to_string();
    }

    if synthesis_review_count > 0 {
        return "Audit report available".to_string();
    }

    if !all_reports.is_empty() {
        return "Superseded".to_string();
    }

    if episode_status == "synthesis_pending" {
        return "In synthesis".to_string();
    }

    if element_review_count > 0 {
        return "ElementReviews submitted".to_string();
    }

    "Registered for audit".to_string()
}

async fn challenge_count_for_episode(
    db: &PgPool,
    episode_id: &str,
) -> Result<i64, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            (
                SELECT COUNT(*)
                FROM episode_synthesis_review_relations rel
                JOIN episode_synthesis_reviews esr
                  ON esr.id = rel.target OR esr.id = rel.source
                WHERE esr.episode_id::text = $1
                  AND rel.relation_type = 'contests'
            )
            +
            (
                SELECT COUNT(*)
                FROM facts f
                JOIN episode_memberships em ON em.fact_id = f.id
                WHERE em.episode_id::text = $1
                  AND em.status = 'active'
                  AND f.status = 'active'
                  AND f.payload_kind = 'submitter_response'
                  AND f.payload->'submitter_response'->>'response_type' = 'contests'
            ) AS challenge_count
        "#,
    )
    .bind(episode_id)
    .fetch_one(db)
    .await?;

    Ok(row.get("challenge_count"))
}

fn row_to_audit_target(row: &PgRow) -> Result<AuditTargetSummary, RepositoryError> {
    let scope_conditions: Value = row.get("scope_conditions");

    Ok(AuditTargetSummary {
        subject_id: row.get("subject_id"),
        subject_type: row.get("subject_type"),
        title: row.get("subject_title"),
        claim_statement: row.get("claim_statement"),
        scope_conditions: serde_json::from_value::<Vec<ScopeCondition>>(scope_conditions).map_err(
            |error| RepositoryError::Domain(format!("invalid scope conditions: {error}")),
        )?,
    })
}

fn row_to_involvement_episode(row: &PgRow) -> AuditEpisodeInvolvementSummary {
    AuditEpisodeInvolvementSummary {
        id: row.get("episode_id"),
        label: row.get("episode_label"),
        status: row.get("episode_status"),
    }
}

fn row_to_summary(row: PgRow, facts: &[Fact]) -> Result<EvidenceArtifactSummary, RepositoryError> {
    let artifact = row_to_link(&row)?;
    let authors: Value = row.get("authors");
    let bearing = derive_artifact_bearing(&artifact.id, facts);
    let (warrant_count, review_count) = artifact_fact_counts(&artifact.id, facts);

    Ok(EvidenceArtifactSummary {
        scholarly_object_id: artifact.scholarly_object_id.clone(),
        title: row.get("title"),
        authors: serde_json::from_value::<Vec<String>>(authors).unwrap_or_default(),
        source_name: row.get("source_name"),
        publication_year: row.get("publication_year"),
        canonical_url: row.get("canonical_url"),
        bearing,
        warrant_count,
        review_count,
        artifact,
    })
}

fn row_to_link(row: &PgRow) -> Result<EpisodeEvidenceArtifact, RepositoryError> {
    let role: String = row.get("role");
    let status: String = row.get("status");
    let status_metadata: Value = row.get("status_metadata");
    let attached_by: Value = row.get("attached_by");
    let attached_at: DateTime<Utc> = row.get("attached_at");
    let link_status = match status.as_str() {
        "active" => EvidenceArtifactStatus::Active,
        "retracted" => EvidenceArtifactStatus::Retracted {
            retracted_by: match status_metadata.get("retracted_by") {
                Some(value) => {
                    serde_json::from_value::<Principal>(value.clone()).map_err(|error| {
                        RepositoryError::Domain(format!("invalid principal: {error}"))
                    })?
                }
                None => Principal::Platform,
            },
            retracted_at: status_metadata
                .get("retracted_at")
                .and_then(Value::as_str)
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .map(|value| value.with_timezone(&Utc))
                .unwrap_or(attached_at),
        },
        other => {
            return Err(RepositoryError::Domain(format!(
                "unknown evidence artifact status: {other}"
            )))
        }
    };

    Ok(EpisodeEvidenceArtifact {
        id: row.get("id"),
        episode_id: row.get("episode_id"),
        scholarly_object_id: row.get("scholarly_object_id"),
        role: EvidenceRole::try_from(role.as_str()).map_err(RepositoryError::Domain)?,
        note: row.get("note"),
        attached_by: serde_json::from_value::<Principal>(attached_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        attached_at,
        status: link_status,
    })
}

/// Active warrant assertions through the artifact, and active element
/// reviews targeting the artifact or one of its warrants.
fn artifact_fact_counts(artifact_id: &EvidenceArtifactId, facts: &[Fact]) -> (i64, i64) {
    let active = |fact: &&Fact| matches!(fact.status, FactStatus::Active);
    let warrant_ids: Vec<&str> = facts
        .iter()
        .filter(active)
        .filter_map(|fact| match &fact.payload {
            FactPayload::WarrantAssertion {
                evidence_artifact: Some(link),
                ..
            } if link == artifact_id.as_str() => Some(fact.id.as_str()),
            _ => None,
        })
        .collect();
    let review_count = facts
        .iter()
        .filter(active)
        .filter(|fact| match &fact.payload {
            FactPayload::ElementReview {
                evidence_artifact,
                warrant,
                ..
            } => {
                evidence_artifact
                    .as_deref()
                    .is_some_and(|link| link == artifact_id.as_str())
                    || warrant
                        .as_ref()
                        .is_some_and(|id| warrant_ids.contains(&id.as_str()))
            }
            _ => false,
        })
        .count() as i64;

    (warrant_ids.len() as i64, review_count)
}

async fn ensure_episode_exists(db: &PgPool, episode_id: &str) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM audit_episodes WHERE id::text = $1
        ) AS exists
        "#,
    )
    .bind(episode_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "audit_episode",
            id: episode_id.to_string(),
        })
    }
}

async fn ensure_scholarly_object_exists(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM scholarly_objects WHERE id::text = $1
        ) AS exists
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "scholarly_object",
            id: scholarly_object_id.to_string(),
        })
    }
}
