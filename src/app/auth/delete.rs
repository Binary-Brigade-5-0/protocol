use axum::{http::HeaderMap, Extension};
use sqlx::PgPool;

use crate::app::error::AppError;

use super::{middleware::validation_mw, Claims};

#[tracing::instrument]
#[cfg_attr(debug_assertions, axum_macros::debug_handler)]
pub async fn delete_user(
    headers: HeaderMap,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<(), AppError> {
    let claims = validation_mw(headers.clone(), Extension(pg_pool.clone())).await?;
    let Claims { userid, .. } = claims;

    let mut txn = pg_pool.begin().await?;

    sqlx::query(r"DELETE FROM users WHERE userid=$1")
        .bind(userid)
        .execute(txn.as_mut())
        .await?;

    txn.commit().await?;
    Ok(())
}
