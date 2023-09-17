pub mod message;
pub mod server;

use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
    time::SystemTime,
};

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing, Json, Router,
};
use chrono::{DateTime, Utc};
use server::Server;
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

async fn socket_handler(
    websocket: WebSocketUpgrade,
    State(channels): State<Arc<server::TaskSpawner>>,
) -> Response {
    websocket.on_upgrade(move |ws| channels.spawn_client(ws))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    START_TIME.get_or_init(SystemTime::now);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(
        cfg!(debug_assertions).then_some("trace").unwrap_or("info"),
    ));

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

    let mut mesg_server = Server::new();

    let Some(spawner) = mesg_server.task_spawner() else {
        anyhow::bail!("could not retreive task spawner because it was already removed");
    };

    let app = Router::new()
        .route("/api/checkhealth", routing::get(checkhealth))
        .route("/proto/v1", routing::get(socket_handler))
        .with_state(spawner);

    let server = axum::Server::bind(&sock_addr).serve(app.into_make_service());

    tracing::info!("server starting on address: {sock_addr}");

    let m_fut = tokio::spawn(mesg_server.spawn_server());
    let s_fut = tokio::spawn(server);

    let (result, _) = tokio::join!(m_fut, s_fut,);
    Ok(result?)
}
