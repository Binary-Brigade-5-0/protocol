use std::time::SystemTime;

use axum::{http::HeaderMap, Extension};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use sqlx::PgPool;

use crate::app::error::AppError;

use super::Claims;

#[tracing::instrument]
pub(super) async fn validation_mw(
    headers: HeaderMap,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<Claims, AppError> {
    let token = headers.get("x-auth-token").ok_or(AppError::BaseError(
        "no auth token found please login".into(),
    ))?;

    let claims = jsonwebtoken::decode::<Claims>(
        token.to_str().map_err(|e| e.to_string())?,
        &DecodingKey::from_secret(super::PTPDS_SECRET_DECODE),
        &Validation::new(Algorithm::HS256),
    )?;

    let Claims { userid, logged, .. } = &claims.claims;
    SystemTime::now().duration_since((*logged).into())?;

    sqlx::query(r"SELECT FROM users WHERE userid=$1")
        .bind(userid)
        .fetch_one(&pg_pool)
        .await?;

    Ok(claims.claims)
}

#[tracing::instrument]
#[cfg_attr(debug_assertions, axum_macros::debug_handler)]
pub(super) async fn validate_user(
    headers: HeaderMap,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<(), AppError> {
    validation_mw(headers, Extension(pg_pool)).await?;
    Ok(())
}
