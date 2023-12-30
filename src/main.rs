pub mod app;
pub mod mailbox;
pub mod message;
pub mod server;
pub mod settings;

use app::Application;
use sqlx::PgPool;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sock_addr = settings::Settings::instance().addr();
    let pg_conn = PgPool::connect(settings::Settings::instance().pg_uri()).await?;
    sqlx::migrate!("./migrations").run(&pg_conn).await?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(cfg!(debug_assertions).then_some("trace").unwrap_or("info"))
    });

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

    let server = Application::new(app::AppExtensions { pg_conn })?;

    tracing::info!("starting server at: {sock_addr}");
    server.bind(*sock_addr).await
}
