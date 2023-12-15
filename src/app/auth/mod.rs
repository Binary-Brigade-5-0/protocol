use std::time::Duration;

use axum::{routing, Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

mod delete;
mod login;
mod register;

pub const PTPDS_SECRET_ENCODE: &str = "ptpds-secret";
pub const PTPDS_SECRET_DECODE: &str = "ptpds-secret";

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
}

pub fn router(db: PgPool) -> Router {
    Router::new()
        .route("/login", routing::get(login::login_user))
        .route("/register", routing::post(register::register_user))
        .route("/delete", routing::delete(|| async { "delete" }))
        .layer(Extension(db))
}
