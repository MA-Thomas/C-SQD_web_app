pub mod access_policy;
pub mod article_access;
pub mod article_retrieval;
pub mod article_versions;
pub mod audit_objects;
pub mod doi_retrieval;
pub mod domain_instantiations;
pub mod pubmed_retrieval;
pub mod review_assignments;
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
