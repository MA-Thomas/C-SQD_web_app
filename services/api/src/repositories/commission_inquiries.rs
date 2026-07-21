//! Commission inquiries: stage one of the two-stage commission path.
//!
//! Real sponsors do not self-serve a four-figure commitment through a web
//! form. Stage one is a short public inquiry — who you are, what you want
//! audited, the decision context, a budget band. Stage two happens after a
//! scoping conversation, when an operator (or the signed-in sponsor)
//! finalizes scope and funding through the full commission flow.
//!
//! Inquiries are pre-graph: they join the audit record only when converted
//! into a real commission.

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

/// Cheap brake on inquiry spam: max inquiries per contact email per day.
const MAX_INQUIRIES_PER_EMAIL_PER_DAY: i64 = 5;

pub const BUDGET_BANDS: &[&str] = &[
    "under_5k",
    "5k_to_15k",
    "15k_to_50k",
    "over_50k",
    "undisclosed",
];

pub const INQUIRY_STATUSES: &[&str] = &["new", "in_conversation", "converted", "declined"];

const ORGANIZATION_TYPES: &[&str] = &[
    "biotech",
    "venture_capital",
    "foundation",
    "university",
    "journal",
    "regulator",
    "other",
];

#[derive(Debug, Clone, Serialize)]
pub struct CommissionInquiry {
    pub id: String,
    pub contact_name: String,
    pub contact_email: String,
    pub organization_name: Option<String>,
    pub organization_type: String,
    pub subject_description: String,
    pub decision_context: Option<String>,
    pub budget_band: String,
    pub status: String,
    pub converted_episode_id: Option<String>,
    pub operator_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommissionInquiryRequest {
    pub contact_name: String,
    pub contact_email: String,
    #[serde(default)]
    pub organization_name: Option<String>,
    #[serde(default)]
    pub organization_type: Option<String>,
    pub subject_description: String,
    #[serde(default)]
    pub decision_context: Option<String>,
    #[serde(default)]
    pub budget_band: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCommissionInquiryRequest {
    pub status: String,
    #[serde(default)]
    pub operator_note: Option<String>,
    #[serde(default)]
    pub converted_episode_id: Option<String>,
}

const INQUIRY_COLUMNS: &str = r#"
    id::text AS id,
    contact_name,
    contact_email,
    organization_name,
    organization_type,
    subject_description,
    decision_context,
    budget_band,
    status,
    converted_episode_id::text AS converted_episode_id,
    operator_note,
    created_at
"#;

pub async fn create(
    db: &PgPool,
    request: CreateCommissionInquiryRequest,
) -> Result<CommissionInquiry, RepositoryError> {
    let contact_name = request.contact_name.trim();
    let contact_email = request.contact_email.trim().to_lowercase();
    let subject_description = request.subject_description.trim();

    if contact_name.is_empty() {
        return Err(RepositoryError::Domain(
            "contact name cannot be empty".to_string(),
        ));
    }

    if !contact_email.contains('@') || contact_email.len() < 5 {
        return Err(RepositoryError::Domain(
            "a valid contact email is required".to_string(),
        ));
    }

    if subject_description.len() < 20 {
        return Err(RepositoryError::Domain(
            "describe what you want audited in at least a sentence".to_string(),
        ));
    }

    let organization_type = request
        .organization_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("other");

    if !ORGANIZATION_TYPES.contains(&organization_type) {
        return Err(RepositoryError::Domain(format!(
            "unknown organization type: {organization_type}"
        )));
    }

    let budget_band = request
        .budget_band
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("undisclosed");

    if !BUDGET_BANDS.contains(&budget_band) {
        return Err(RepositoryError::Domain(format!(
            "unknown budget band: {budget_band}"
        )));
    }

    let recent: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM commission_inquiries
        WHERE contact_email = $1
          AND created_at > now() - interval '1 day'
        "#,
    )
    .bind(&contact_email)
    .fetch_one(db)
    .await?;

    if recent >= MAX_INQUIRIES_PER_EMAIL_PER_DAY {
        return Err(RepositoryError::RateLimited(
            "too many inquiries from this address today; we will reply to the ones you sent"
                .to_string(),
        ));
    }

    let row = sqlx::query(&format!(
        r#"
        INSERT INTO commission_inquiries (
            contact_name,
            contact_email,
            organization_name,
            organization_type,
            subject_description,
            decision_context,
            budget_band
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING {INQUIRY_COLUMNS}
        "#
    ))
    .bind(contact_name)
    .bind(&contact_email)
    .bind(
        request
            .organization_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(organization_type)
    .bind(subject_description)
    .bind(
        request
            .decision_context
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(budget_band)
    .fetch_one(db)
    .await?;

    row_to_inquiry(row)
}

pub async fn list(db: &PgPool) -> Result<Vec<CommissionInquiry>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        SELECT {INQUIRY_COLUMNS}
        FROM commission_inquiries
        ORDER BY
            CASE status
                WHEN 'new' THEN 0
                WHEN 'in_conversation' THEN 1
                WHEN 'converted' THEN 2
                ELSE 3
            END,
            created_at DESC
        LIMIT 200
        "#
    ))
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_inquiry).collect()
}

pub async fn update_status(
    db: &PgPool,
    inquiry_id: &str,
    request: UpdateCommissionInquiryRequest,
) -> Result<CommissionInquiry, RepositoryError> {
    let status = request.status.trim();

    if !INQUIRY_STATUSES.contains(&status) {
        return Err(RepositoryError::Domain(format!(
            "unknown inquiry status: {status}"
        )));
    }

    let converted_episode_id = request
        .converted_episode_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if status == "converted" && converted_episode_id.is_none() {
        return Err(RepositoryError::Domain(
            "a converted inquiry needs the episode it converted into".to_string(),
        ));
    }

    let row = sqlx::query(&format!(
        r#"
        UPDATE commission_inquiries
        SET
            status = $2,
            operator_note = COALESCE($3, operator_note),
            converted_episode_id = COALESCE($4::uuid, converted_episode_id),
            updated_at = now()
        WHERE id::text = $1
        RETURNING {INQUIRY_COLUMNS}
        "#
    ))
    .bind(inquiry_id)
    .bind(status)
    .bind(
        request
            .operator_note
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(converted_episode_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "commission_inquiry",
        id: inquiry_id.to_string(),
    })?;

    row_to_inquiry(row)
}

fn row_to_inquiry(row: PgRow) -> Result<CommissionInquiry, RepositoryError> {
    Ok(CommissionInquiry {
        id: row.get("id"),
        contact_name: row.get("contact_name"),
        contact_email: row.get("contact_email"),
        organization_name: row.get("organization_name"),
        organization_type: row.get("organization_type"),
        subject_description: row.get("subject_description"),
        decision_context: row.get("decision_context"),
        budget_band: row.get("budget_band"),
        status: row.get("status"),
        converted_episode_id: row.get("converted_episode_id"),
        operator_note: row.get("operator_note"),
        created_at: row.get("created_at"),
    })
}
