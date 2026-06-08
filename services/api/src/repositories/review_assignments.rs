use csqd_domain::{
    CompensationStatus, ReviewAssignmentState, ReviewAssignmentSummary, ReviewAssignmentType,
};
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_summaries(db: &PgPool) -> Result<Vec<ReviewAssignmentSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ra.id::text AS id,
            ra.scholarly_object_id::text AS scholarly_object_id,
            so.title AS scholarly_object_title,
            so.canonical_url AS scholarly_object_canonical_url,
            u.display_name AS reviewer_display_name,
            ra.assignment_type,
            ra.compensation_status,
            ra.state,
            ra.due_at::text AS due_at
        FROM review_assignments ra
        JOIN scholarly_objects so ON so.id = ra.scholarly_object_id
        JOIN reviewer_profiles rp ON rp.id = ra.reviewer_profile_id
        JOIN users u ON u.id = rp.user_id
        ORDER BY ra.due_at ASC NULLS LAST, ra.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_summary).collect()
}

fn row_to_summary(row: PgRow) -> Result<ReviewAssignmentSummary, RepositoryError> {
    let assignment_type: String = row.get("assignment_type");
    let compensation_status: String = row.get("compensation_status");
    let state: String = row.get("state");

    Ok(ReviewAssignmentSummary {
        id: row.get("id"),
        scholarly_object_id: row.get("scholarly_object_id"),
        scholarly_object_title: row.get("scholarly_object_title"),
        scholarly_object_canonical_url: row.get("scholarly_object_canonical_url"),
        reviewer_display_name: row.get("reviewer_display_name"),
        assignment_type: ReviewAssignmentType::try_from(assignment_type.as_str())
            .map_err(RepositoryError::Domain)?,
        compensation_status: CompensationStatus::try_from(compensation_status.as_str())
            .map_err(RepositoryError::Domain)?,
        state: ReviewAssignmentState::try_from(state.as_str()).map_err(RepositoryError::Domain)?,
        due_at: row.get("due_at"),
    })
}
