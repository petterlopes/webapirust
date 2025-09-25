use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};
use utoipa::ToSchema;

pub type AppResult<T> = Result<T, AppError>;

// Catálogo de erros da aplicação. Cada variante mapeia para um status HTTP e é logada de forma estruturada.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("conflict detected: {0}")]
    Conflict(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("database error: {0}")]
    Database(sqlx::Error),
    #[error("unexpected error: {0}")]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        match (&self, status) {
            (AppError::Validation(detail), _) => {
                warn!(status = %status, detail = detail.as_str(), "validation error")
            }
            (AppError::NotFound(detail), _) => {
                warn!(status = %status, detail = detail.as_str(), "resource not found")
            }
            (AppError::Conflict(detail), _) => {
                warn!(status = %status, detail = detail.as_str(), "conflict detected")
            }
            (AppError::Unauthorized(detail), _) => {
                warn!(status = %status, detail = detail.as_str(), "unauthorized request")
            }
            (AppError::Forbidden(detail), _) => {
                warn!(status = %status, detail = detail.as_str(), "forbidden request")
            }
            (AppError::Database(err), _) => {
                error!(status = %status, error = %err, "database error")
            }
            (AppError::Unexpected(err), _) => {
                error!(status = %status, error = %err, "unexpected error")
            }
        }

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match &error {
            sqlx::Error::RowNotFound => {
                return AppError::NotFound("resource not found".to_string());
            }
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code().as_deref() {
                    if code == "23505" {
                        let message = db_err.message().to_string();
                        return AppError::Conflict(message);
                    }
                }
            }
            _ => {}
        }

        AppError::Database(error)
    }
}
