#![cfg_attr(debug_assertions, allow(unused_must_use, unused_mut, unused_variables))]

pub mod broker;
pub mod client;
pub mod message;
pub mod server;

use server::Server;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let slog_builder = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")));

    #[cfg(debug_assertions)]
    let slog_builder = slog_builder
        .with_file(true)
        .with_env_filter(EnvFilter::new("TRACE"))
        .with_line_number(true);

    slog_builder.init();

    let server = Server::new(SocketAddr::from(([0; 4], 3000))).await?;
    server.await?;
    Ok(())
}
