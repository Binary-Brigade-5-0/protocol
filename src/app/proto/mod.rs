use std::sync::Arc;

use axum::{extract::WebSocketUpgrade, response::Response, routing, Extension, Router};

use crate::server;

async fn socket_handler(
    websocket: WebSocketUpgrade,
    Extension(spawner): Extension<Arc<server::TaskSpawner>>,
) -> Response {
    websocket.on_upgrade(move |ws| spawner.spawn_client(ws))
}

pub fn router(task_spawner: Arc<server::TaskSpawner>) -> Router {
    Router::new()
        .route("/v1", routing::get(socket_handler))
        .layer(Extension(task_spawner))
}
