//! Episode relations and synthesis review relations (the challenge layer).
//!
//! Challenges preserve the challenged artifact: a contestation adds a
//! provenance-bearing relation record; superseding flips status and links the
//! replacement. Nothing is deleted.

use csqd_domain::{
    CreateEpisodeRelationRequest, CreateSynthesisReviewRelationRequest, EpisodeRelation,
    EpisodeRelationType, NarrativeRelationType, Principal, SynthesisContestationInfo,
    SynthesisContestationScope, SynthesisReviewRelation,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_episode_relations(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<EpisodeRelation>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            source_episode_id::text AS source_episode_id,
            target_episode_id::text AS target_episode_id,
            relation_type,
            asserted_by,
            asserted_at
        FROM episode_relations
        WHERE source_episode_id::text = $1
           OR target_episode_id::text = $1
        ORDER BY asserted_at ASC
        "#,
    )
    .bind(episode_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_episode_relation).collect()
}

pub async fn create_episode_relation(
    db: &PgPool,
    source_episode_id: &str,
    request: CreateEpisodeRelationRequest,
) -> Result<EpisodeRelation, RepositoryError> {
    ensure_episode_exists(db, source_episode_id).await?;
    ensure_episode_exists(db, request.target_episode_id.as_str()).await?;

    if source_episode_id == request.target_episode_id.as_str() {
        return Err(RepositoryError::Domain(
            "an episode cannot relate to itself".to_string(),
        ));
    }

    let asserted_by = request.asserted_by.unwrap_or(Principal::Platform);
    let asserted_by_value = serde_json::to_value(&asserted_by)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let row = sqlx::query(
        r#"
        INSERT INTO episode_relations (
            source_episode_id,
            target_episode_id,
            relation_type,
            asserted_by
        )
        VALUES ($1::uuid, $2::uuid, $3, $4::jsonb)
        RETURNING
            id::text AS id,
            source_episode_id::text AS source_episode_id,
            target_episode_id::text AS target_episode_id,
            relation_type,
            asserted_by,
            asserted_at
        "#,
    )
    .bind(source_episode_id)
    .bind(request.target_episode_id.as_str())
    .bind(request.relation_type.as_db_str())
    .bind(&asserted_by_value)
    .fetch_one(db)
    .await?;

    row_to_episode_relation(row)
}

pub async fn list_synthesis_relations_for_review(
    db: &PgPool,
    review_id: &str,
) -> Result<Vec<SynthesisReviewRelation>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            source::text AS source,
            target::text AS target,
            relation_type,
            contestation_scope,
            rationale,
            asserted_by,
            asserted_at
        FROM episode_synthesis_review_relations
        WHERE source::text = $1
           OR target::text = $1
        ORDER BY asserted_at ASC
        "#,
    )
    .bind(review_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_synthesis_relation).collect()
}

/// Creates a synthesis review relation. For `Supersedes`, also marks the
/// target review superseded (the historical record is preserved, status only).
pub async fn create_synthesis_relation(
    db: &PgPool,
    source_review_id: &str,
    request: CreateSynthesisReviewRelationRequest,
) -> Result<SynthesisReviewRelation, RepositoryError> {
    ensure_synthesis_review_exists(db, source_review_id).await?;
    ensure_synthesis_review_exists(db, request.target.as_str()).await?;

    if source_review_id == request.target.as_str() {
        return Err(RepositoryError::Domain(
            "a synthesis review cannot relate to itself".to_string(),
        ));
    }

    let asserted_by = request.asserted_by.unwrap_or(Principal::Platform);
    let asserted_by_value = serde_json::to_value(&asserted_by)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let (scope, rationale) = match &request.relation_type {
        NarrativeRelationType::Contests(info) => {
            let scope = match info.scope {
                SynthesisContestationScope::Partial => "partial",
                SynthesisContestationScope::Full => "full",
            };

            (Some(scope), info.rationale.clone())
        }
        _ => (None, None),
    };

    let mut tx = db.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO episode_synthesis_review_relations (
            source,
            target,
            relation_type,
            contestation_scope,
            rationale,
            asserted_by
        )
        VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6::jsonb)
        RETURNING
            id::text AS id,
            source::text AS source,
            target::text AS target,
            relation_type,
            contestation_scope,
            rationale,
            asserted_by,
            asserted_at
        "#,
    )
    .bind(source_review_id)
    .bind(request.target.as_str())
    .bind(request.relation_type.as_db_str())
    .bind(scope)
    .bind(rationale.as_deref())
    .bind(&asserted_by_value)
    .fetch_one(&mut *tx)
    .await?;

    if matches!(request.relation_type, NarrativeRelationType::Supersedes) {
        sqlx::query(
            r#"
            UPDATE episode_synthesis_reviews
            SET status = 'superseded', updated_at = now()
            WHERE id::text = $1
            "#,
        )
        .bind(request.target.as_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    row_to_synthesis_relation(row)
}

/// Counts contestations touching any synthesis review of the subject's
/// episodes plus contesting submitter responses on the subject's facts.
pub async fn challenge_counts_for_subject(
    db: &PgPool,
    audit_subject_id: &str,
) -> Result<i64, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            (
                SELECT COUNT(*)
                FROM episode_synthesis_review_relations rel
                JOIN episode_synthesis_reviews esr
                  ON esr.id = rel.target OR esr.id = rel.source
                JOIN audit_episodes ae ON ae.id = esr.episode_id
                WHERE ae.subject_id::text = $1
                  AND rel.relation_type = 'contests'
            )
            +
            (
                SELECT COUNT(*)
                FROM facts f
                WHERE f.subject_id::text = $1
                  AND f.status = 'active'
                  AND f.payload_kind = 'submitter_response'
                  AND f.payload->'submitter_response'->>'response_type' = 'contests'
            ) AS challenge_count
        "#,
    )
    .bind(audit_subject_id)
    .fetch_one(db)
    .await?;

    Ok(row.get("challenge_count"))
}

async fn ensure_episode_exists(db: &PgPool, episode_id: &str) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (SELECT 1 FROM audit_episodes WHERE id::text = $1) AS exists
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

async fn ensure_synthesis_review_exists(
    db: &PgPool,
    review_id: &str,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM episode_synthesis_reviews WHERE id::text = $1
        ) AS exists
        "#,
    )
    .bind(review_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "synthesis_review",
            id: review_id.to_string(),
        })
    }
}

fn row_to_episode_relation(row: PgRow) -> Result<EpisodeRelation, RepositoryError> {
    let relation_type: String = row.get("relation_type");
    let asserted_by: Value = row.get("asserted_by");

    Ok(EpisodeRelation {
        id: row.get("id"),
        source_episode_id: row.get("source_episode_id"),
        target_episode_id: row.get("target_episode_id"),
        relation_type: EpisodeRelationType::try_from(relation_type.as_str())
            .map_err(RepositoryError::Domain)?,
        asserted_by: serde_json::from_value::<Principal>(asserted_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        asserted_at: row.get("asserted_at"),
    })
}

fn row_to_synthesis_relation(row: PgRow) -> Result<SynthesisReviewRelation, RepositoryError> {
    let relation_type: String = row.get("relation_type");
    let contestation_scope: Option<String> = row.get("contestation_scope");
    let rationale: Option<String> = row.get("rationale");
    let asserted_by: Value = row.get("asserted_by");
    let relation_type = match relation_type.as_str() {
        "supersedes" => NarrativeRelationType::Supersedes,
        "related_to" => NarrativeRelationType::RelatedTo,
        "contests" => NarrativeRelationType::Contests(SynthesisContestationInfo {
            scope: match contestation_scope.as_deref() {
                Some("full") => SynthesisContestationScope::Full,
                _ => SynthesisContestationScope::Partial,
            },
            rationale,
        }),
        other => {
            return Err(RepositoryError::Domain(format!(
                "unknown synthesis relation type: {other}"
            )))
        }
    };

    Ok(SynthesisReviewRelation {
        id: row.get("id"),
        source: row.get("source"),
        target: row.get("target"),
        relation_type,
        asserted_by: serde_json::from_value::<Principal>(asserted_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        asserted_at: row.get("asserted_at"),
    })
}
