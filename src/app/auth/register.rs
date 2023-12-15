use std::time::SystemTime;

use axum::{Extension, Json};
use jsonwebtoken::{EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use super::{Claims, UserInfo};
use crate::app::error::AppError;

#[cfg_attr(debug_assertions, axum_macros::debug_handler)]
pub(super) async fn register_user(
    Extension(pg_pool): Extension<PgPool>,
    Json(user): Json<UserInfo>,
) -> Result<String, AppError> {
    let UserInfo { username, password } = user;
    let ins_query = r"INSERT INTO users (name, password) VALUES ($1, $2) RETURNING userid";

    let mut txn = pg_pool.begin().await?;
    let (userid,): (Uuid,) = sqlx::query_as(ins_query)
        .bind(username)
        .bind(password)
        .fetch_one(txn.as_mut())
        .await?;

    txn.commit().await?;

    let claims = Claims {
        userid,
        logged: SystemTime::now().into(),
    };

    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(super::PTPDS_SECRET_ENCODE.as_bytes()),
    )?;

    Ok(jwt)
}
