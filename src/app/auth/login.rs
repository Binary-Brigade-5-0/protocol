use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Extension, Json};
use jsonwebtoken::{EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::app::error::AppError;

use super::{Claims, UserInfo, PTPDS_SECRET_ENCODE};

#[tracing::instrument]
#[cfg_attr(debug_assertions, axum_macros::debug_handler)]
pub(super) async fn login_user(
    Extension(pg_pool): Extension<PgPool>,
    Json(user): Json<UserInfo>,
) -> Result<String, AppError> {
    let UserInfo { username, password } = user;
    let logged = SystemTime::now();
    let ftc_query = r"SELECT userid FROM users WHERE name=$1 and password=$2";

    let (userid,): (Uuid,) = sqlx::query_as(ftc_query)
        .bind(username)
        .bind(&password)
        .fetch_one(&pg_pool)
        .await
        .map_err(|_| AppError::InvalidCredentials)?;

    let claims = Claims {
        userid,
        logged: logged.into(),
        exp: (logged.duration_since(UNIX_EPOCH).unwrap() + super::EXPIRY_TIME_HOURS).as_secs(),
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(PTPDS_SECRET_ENCODE),
    )?;

    Ok(token)
}