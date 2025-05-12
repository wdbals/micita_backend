use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound(String),
    #[error("Conflict")]
    Conflict(String),
    #[error("Unauthorized")]
    Unauthorized(String),
    #[error("Internal server error")]
    InternalServerError(String),
    #[error("Validation error")]
    ValidationError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::NotFound(message) => HttpResponse::NotFound().json(message),
            ApiError::Conflict(message) => HttpResponse::Conflict().json(message),
            ApiError::Unauthorized(message) => HttpResponse::Unauthorized().json(message),
            ApiError::InternalServerError(message) => {
                HttpResponse::InternalServerError().json(message)
            }
            ApiError::ValidationError(message) => HttpResponse::BadRequest().json(message),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".into()),
            _ => ApiError::InternalServerError(error.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(error.to_string())
    }
}
