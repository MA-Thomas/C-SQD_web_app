//! Public audit summaries: the single source the public frontend reads.
//!
//! Replaces the previous client-side fan-out (per-work episodes -> facts ->
//! per-episode tuple + syntheses) with one API call. Status labels are
//! computed server-side so Discover, Public Audits, the homepage, and subject
//! pages agree by construction.

use csqd_domain::{AuditEpisode, EvalTuple, Fact, FactPayload, FactStatus, SynthesisReview};
use serde::Serialize;
use sqlx::{PgPool, Row};

use super::{audit_episodes, relations, RepositoryError};

#[derive(Debug, Clone, Serialize)]
pub struct PublicSubjectSummary {
    pub scholarly_object_id: Option<String>,
    pub audit_subject_id: Option<String>,
    pub status_label: String,
    pub tuple: Option<PublicTupleSummary>,
    pub crwe_reviewed_node_ids: Vec<String>,
    pub element_review_count: i64,
    pub synthesis_review_count: i64,
    pub challenge_count: i64,
    pub latest_report: Option<SynthesisReview>,
    pub episodes: Vec<AuditEpisode>,
}

/// Friendly-labeled aggregate of per-episode evaluation tuples
/// (sum N/M/L/U, max S — matching the original frontend aggregation).
#[derive(Debug, Clone, Serialize)]
pub struct PublicTupleSummary {
    pub problems: f64,
    pub ethical_concerns: f64,
    pub stakes: f64,
    pub scrutiny_depth: f64,
    pub uptake: f64,
}

pub async fn summary_for_audit_subject(
    db: &PgPool,
    audit_subject_id: &str,
) -> Result<PublicSubjectSummary, RepositoryError> {
    build_summary(db, Some(audit_subject_id.to_string()), None).await
}

pub async fn summary_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<PublicSubjectSummary, RepositoryError> {
    let audit_subject_id = audit_subject_id_for_scholarly_object(db, scholarly_object_id).await?;

    build_summary(db, audit_subject_id, Some(scholarly_object_id.to_string())).await
}

pub async fn summaries_for_scholarly_objects(
    db: &PgPool,
    scholarly_object_ids: &[String],
) -> Result<Vec<PublicSubjectSummary>, RepositoryError> {
    let mut summaries = Vec::with_capacity(scholarly_object_ids.len());

    for object_id in scholarly_object_ids {
        summaries.push(summary_for_scholarly_object(db, object_id).await?);
    }

    Ok(summaries)
}

/// Batch summaries by audit-subject id, preserving input order. Lets Discover,
/// the home page, and Public Audits render from a single API call instead of a
/// per-subject fan-out. (Each summary is currently assembled with its own
/// queries; a single aggregating SQL pass is a later optimization.)
pub async fn summaries_for_audit_subjects(
    db: &PgPool,
    audit_subject_ids: &[String],
) -> Result<Vec<PublicSubjectSummary>, RepositoryError> {
    let mut summaries = Vec::with_capacity(audit_subject_ids.len());

    for subject_id in audit_subject_ids {
        summaries.push(summary_for_audit_subject(db, subject_id).await?);
    }

    Ok(summaries)
}

async fn audit_subject_id_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM audit_subjects
        WHERE source_entity_type = 'scholarly_object'
          AND source_entity_id::text = $1
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|row| row.get("id")))
}

async fn build_summary(
    db: &PgPool,
    audit_subject_id: Option<String>,
    scholarly_object_id: Option<String>,
) -> Result<PublicSubjectSummary, RepositoryError> {
    let Some(subject_id) = audit_subject_id else {
        return Ok(empty_summary(scholarly_object_id));
    };

    let episodes = audit_episodes::list_for_subject(db, &subject_id).await?;
    let facts = audit_episodes::list_facts_for_subject(db, &subject_id).await?;
    let mut tuples: Vec<EvalTuple> = Vec::with_capacity(episodes.len());
    let mut reports: Vec<SynthesisReview> = Vec::new();

    for episode in &episodes {
        let tuple = audit_episodes::compute_eval_tuple(
            db,
            episode.id.as_str(),
            audit_episodes::EvalTupleQuery::default(),
        )
        .await?;
        tuples.push(tuple);

        let mut reviews = audit_episodes::list_synthesis_reviews(db, episode.id.as_str()).await?;
        reports.append(&mut reviews);
    }

    reports.sort_by(|left, right| right.authored_at.cmp(&left.authored_at));

    let active_reports: Vec<&SynthesisReview> = reports
        .iter()
        .filter(|review| {
            matches!(
                review.status,
                csqd_domain::NarrativeStatus::Current | csqd_domain::NarrativeStatus::Draft
            )
        })
        .collect();
    let element_reviews: Vec<&Fact> = facts
        .iter()
        .filter(|fact| matches!(fact.status, FactStatus::Active))
        .filter(|fact| matches!(fact.payload, FactPayload::ElementReview { .. }))
        .collect();
    let mut crwe_reviewed_node_ids: Vec<String> = element_reviews
        .iter()
        .filter_map(|fact| match &fact.payload {
            FactPayload::ElementReview { cwe_criterion, .. } => {
                Some(cwe_criterion.node_id.as_str().to_string())
            }
            _ => None,
        })
        .collect();
    crwe_reviewed_node_ids.sort();
    crwe_reviewed_node_ids.dedup();

    let challenge_count = relations::challenge_counts_for_subject(db, &subject_id).await?;
    let synthesis_review_count = active_reports.len() as i64;
    let element_review_count = element_reviews.len() as i64;
    let status_label = status_label(
        &episodes,
        element_review_count,
        synthesis_review_count,
        challenge_count,
        &reports,
    );

    Ok(PublicSubjectSummary {
        scholarly_object_id,
        audit_subject_id: Some(subject_id),
        status_label,
        tuple: aggregate_tuples(&tuples),
        crwe_reviewed_node_ids,
        element_review_count,
        synthesis_review_count,
        challenge_count,
        latest_report: active_reports.first().map(|review| (*review).clone()),
        episodes,
    })
}

fn empty_summary(scholarly_object_id: Option<String>) -> PublicSubjectSummary {
    PublicSubjectSummary {
        scholarly_object_id,
        audit_subject_id: None,
        status_label: "Unaudited".to_string(),
        tuple: None,
        crwe_reviewed_node_ids: Vec::new(),
        element_review_count: 0,
        synthesis_review_count: 0,
        challenge_count: 0,
        latest_report: None,
        episodes: Vec::new(),
    }
}

/// Memo status labels: Unaudited, ElementReviews submitted, In synthesis,
/// Audit report available, Challenged, Superseded.
fn status_label(
    episodes: &[AuditEpisode],
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

    // Reports exist but none are current/draft: the audit record was
    // superseded without replacement.
    if !all_reports.is_empty() {
        return "Superseded".to_string();
    }

    let any_synthesis_pending = episodes
        .iter()
        .any(|episode| matches!(episode.status, csqd_domain::EpisodeStatus::SynthesisPending));

    if any_synthesis_pending {
        return "In synthesis".to_string();
    }

    if element_review_count > 0 {
        return "ElementReviews submitted".to_string();
    }

    if episodes.is_empty() {
        "Unaudited".to_string()
    } else {
        "Registered for audit".to_string()
    }
}

fn aggregate_tuples(tuples: &[EvalTuple]) -> Option<PublicTupleSummary> {
    if tuples.is_empty() {
        return None;
    }

    Some(tuples.iter().fold(
        PublicTupleSummary {
            problems: 0.0,
            ethical_concerns: 0.0,
            stakes: 0.0,
            scrutiny_depth: 0.0,
            uptake: 0.0,
        },
        |summary, tuple| PublicTupleSummary {
            problems: summary.problems + tuple.n,
            ethical_concerns: summary.ethical_concerns + tuple.m,
            stakes: summary.stakes.max(tuple.s),
            scrutiny_depth: summary.scrutiny_depth + tuple.l,
            uptake: summary.uptake + tuple.u,
        },
    ))
}
