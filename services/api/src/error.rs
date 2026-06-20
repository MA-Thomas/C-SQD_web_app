use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::repositories::RepositoryError;

#[derive(Debug)]
pub enum ApiError {
    Database(sqlx::Error),
    Domain(String),
    NotFound { entity: &'static str, id: String },
    Unauthorized(String),
    Forbidden(String),
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<RepositoryError> for ApiError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Database(error) => Self::Database(error),
            RepositoryError::Domain(message) => Self::Domain(message),
            RepositoryError::NotFound { entity, id } => Self::NotFound { entity, id },
            RepositoryError::Unauthorized(message) => Self::Unauthorized(message),
            RepositoryError::Forbidden(message) => Self::Forbidden(message),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Database(error) => {
                tracing::error!(%error, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database operation failed".to_string(),
                )
            }
            Self::Domain(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Self::NotFound { entity, id } => {
                (StatusCode::NOT_FOUND, format!("{entity} not found: {id}"))
            }
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
