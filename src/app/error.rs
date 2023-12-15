use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error)]
#[derive(Debug)]
pub enum AppError {
    #[error("Postgresql error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Generic error: {0}")]
    BaseError(String),
    #[error("JWT Error: {0}")]
    JWTError(#[from] jsonwebtoken::errors::Error),

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
            Self::JWTError(err) => err.to_string(),

            Self::InvalidCredentials => {
                return (StatusCode::UNAUTHORIZED, format!("{self}")).into_response()
            },
        };

        (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
    }
}
