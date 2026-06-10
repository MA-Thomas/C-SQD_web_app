pub mod access_policy;
pub mod article_access;
pub mod article_retrieval;
pub mod article_versions;
pub mod audit_episodes;
pub mod audit_subjects;
pub mod doi_retrieval;
pub mod domain_instantiations;
pub mod pubmed_retrieval;
pub mod scholarly_objects;
pub mod title_retrieval;
pub mod user_library;

#[derive(Debug)]
pub enum RepositoryError {
    Database(sqlx::Error),
    Domain(String),
    NotFound { entity: &'static str, id: String },
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
