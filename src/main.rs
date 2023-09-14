pub mod message;
pub mod server;

use axum::{extract::WebSocketUpgrade, response::Response, routing, Json, Router};
use chrono::{DateTime, Utc};
use server::Server;
use std::{net::SocketAddr, sync::OnceLock, time::SystemTime};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub static START_TIME: OnceLock<SystemTime> = OnceLock::new();

#[tracing::instrument]
async fn checkhealth() -> Json<serde_json::Value> {
    let Some(start_time) = START_TIME.get() else {
        return Json(serde_json::json!({ "status": "failure" }));
    };

    let uptime = SystemTime::now().duration_since(*start_time).unwrap();
    let start_time = DateTime::<Utc>::from(*start_time);

    let json = serde_json::json!({
        "status": "ok",
        "uptime": uptime,
        "started": start_time,
    });

    Json(json)
}

async fn socket_handler(websocket: WebSocketUpgrade) -> Response {
    websocket.on_upgrade(Server::spawn_client)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    START_TIME.get_or_init(SystemTime::now);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(if cfg!(debug_assertions) {
            "trace"
        } else {
            "info"
        }));

    let sock_addr = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .unwrap_or_else(|| Ok(SocketAddr::from(([0u8; 4], 3000))))?;

    let slog_builder = tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_env_filter(env_filter);

    #[cfg(debug_assertions)]
    let slog_builder = slog_builder
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::FULL);

    slog_builder.init();

    let app = Router::new()
        .route("/api/checkhealth", routing::get(checkhealth))
        .route("/proto/v1", routing::get(socket_handler));

    let server = axum::Server::bind(&sock_addr).serve(app.into_make_service());

    tracing::info!("server starting on address: {sock_addr}");

    server.await?;

    Ok(())
}
