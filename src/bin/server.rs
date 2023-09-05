use etron::server::Server;

use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .map(|s| s.parse::<SocketAddr>())
        .unwrap_or(Ok(SocketAddr::from(([0u8; 4], 3000))))?;

    let slog_builder = tracing_subscriber::fmt()
        .with_file(cfg!(debug_assertions))
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")));

    #[cfg(debug_assertions)]
    let slog_builder = slog_builder
        .with_file(true)
        .with_env_filter(EnvFilter::new("trace"))
        .with_line_number(true);

    slog_builder.init();

    let server = Server::new(addr).await?;
    server.await?;

    Ok(())
}
