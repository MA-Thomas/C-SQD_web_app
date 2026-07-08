pub mod access_policy;
pub mod article_access;
pub mod article_retrieval;
pub mod article_versions;
pub mod audit_episodes;
pub mod audit_subjects;
pub mod auth;
pub mod claim_audits;
pub mod doi_retrieval;
pub mod domain_instantiations;
pub mod evidence_artifacts;
pub mod public_summary;
pub mod pubmed_retrieval;
pub mod relations;
pub mod scholarly_objects;
pub mod title_retrieval;
pub mod user_library;
pub mod users;

use csqd_domain::FactPayloadKind;
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub enum RepositoryError {
    Database(sqlx::Error),
    Domain(String),
    NotFound { entity: &'static str, id: String },
    Unauthorized(String),
    Forbidden(String),
}

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

pub(crate) fn enum_json_name<T: serde::Serialize>(
    value: &T,
    label: &'static str,
) -> Result<String, RepositoryError> {
    match serde_json::to_value(value)
        .map_err(|error| RepositoryError::Domain(format!("invalid {label}: {error}")))?
    {
        serde_json::Value::String(value) => Ok(value),
        other => Err(RepositoryError::Domain(format!(
            "{label} should serialize to a string, got {other}"
        ))),
    }
}

/// FEN schema: references between fact variants use `FactId`; "the
/// application layer verifies that the referenced Fact carries the expected
/// payload variant." This is that verification.
pub(crate) async fn expect_payload_kind(
    db: &PgPool,
    fact_id: &str,
    expected: FactPayloadKind,
) -> Result<(), RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT payload_kind
        FROM facts
        WHERE id::text = $1
        "#,
    )
    .bind(fact_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "fact",
        id: fact_id.to_string(),
    })?;
    let actual: String = row.get("payload_kind");

    if actual == expected.as_db_str() {
        Ok(())
    } else {
        Err(RepositoryError::Domain(format!(
            "fact {fact_id} has payload kind {actual}, expected {}",
            expected.as_db_str()
        )))
    }
}
