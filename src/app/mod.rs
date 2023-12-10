use std::net::SocketAddr;
use std::time::SystemTime;

use axum::{routing, Json, Router};
use chrono::{DateTime, Utc};
use tokio::net::TcpListener;

use crate::{server, settings::Settings};

pub mod auth;
pub mod proto;

pub struct Application {
    router: Router,
    server: server::Server,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let router = Router::new();
        let mut server = server::Server::new();

        let task_spawner = server.task_spawner().unwrap();
        let router = router
            .nest("/proto", proto::router(task_spawner))
            .route("/checkhealth", routing::get(checkhealth));

        Ok(Self { router, server })
    }

    pub async fn bind(self, sock_addr: SocketAddr) -> anyhow::Result<()> {
        let Self { router, server } = self;

        let tcplistener = TcpListener::bind(sock_addr).await?;
        let appl_server = axum::serve(tcplistener, router);

        let mesg_server_task = tokio::spawn(server.spawn_server());
        #[allow(clippy::redundant_async_block)]
        let appl_server_task = tokio::spawn(async { appl_server.await });

        let (mesg_server_result, appl_server_result) =
            tokio::join!(mesg_server_task, appl_server_task);

        mesg_server_result?;
        appl_server_result??;

        Ok(())
    }
}

async fn checkhealth() -> Json<serde_json::Value> {
    let start_time = Settings::instance().start_time();
    let uptime = SystemTime::now().duration_since(start_time).unwrap();
    let start_time = DateTime::<Utc>::from(start_time);

    let json = serde_json::json!({
        "uptime": uptime,
        "started": start_time,
    });

    Json(json)
}
