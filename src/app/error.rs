use std::time::SystemTimeError;

use axum::{http::StatusCode, response::IntoResponse};
use jsonwebtoken::errors::ErrorKind;

#[derive(thiserror::Error)]
#[derive(Debug)]
pub enum AppError {
    #[error("Postgresql error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Generic error: {0}")]
    BaseError(String),
    #[error("JWT Error: {0}")]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error("invlalid time stamp: {0}")]
    TimeError(#[from] SystemTimeError),

    #[error("invalid username or password")]
    InvalidCredentials,
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::BaseError(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let err = match self {
            Self::BaseError(err) => err,
            Self::SqlError(err) => err.to_string(),
            Self::TimeError(err) => err.to_string(),

            Self::InvalidCredentials => {
                return (StatusCode::UNAUTHORIZED, format!("{self}")).into_response()
            },

            Self::JWTError(err) => {
                if let ErrorKind::ExpiredSignature = err.kind() {
                    return (StatusCode::UNAUTHORIZED, format!("{err}")).into_response();
                } else {
                    err.to_string()
                }
            },
        };

        (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
    }
}
