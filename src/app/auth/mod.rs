use std::time::Duration;

use axum::{routing, Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

mod delete;
mod login;
mod register;

pub mod middleware;

pub const PTPDS_SECRET_ENCODE: &[u8; 1024] =
    include_bytes!(concat!(env!("OUT_DIR"), "/jwt-key.out"));
pub const PTPDS_SECRET_DECODE: &[u8; 1024] =
    include_bytes!(concat!(env!("OUT_DIR"), "/jwt-key.out"));

// expiry time for the jwt, defaulted to a week
pub const EXPIRY_TIME_HOURS: Duration = Duration::from_secs(168 * 60 * 60);

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(sqlx::FromRow)]
pub struct UserInfo {
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub(super) struct Claims {
    userid: Uuid,
    logged: DateTime<Utc>,
    exp: u64,
}

pub fn router(db: PgPool) -> Router {
    tracing::info!(?PTPDS_SECRET_DECODE, ?PTPDS_SECRET_ENCODE);

    Router::new()
        .route("/login", routing::get(login::login_user))
        .route("/register", routing::post(register::register_user))
        .route("/delete", routing::delete(delete::delete_user))
        .layer(Extension(db))
}
