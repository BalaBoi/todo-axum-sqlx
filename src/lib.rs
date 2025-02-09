use anyhow::Context;
use axum::Router;
use error::Error;
use logging::init_tracing_subscriber;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub mod config;
pub mod error;
pub mod logging;
pub mod tasks;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn serve_app() -> anyhow::Result<()> {
    init_tracing_subscriber();

    let config = config::get_config();

    let app = api_router(config.postgres.get_pool().await?);
    let listener = TcpListener::bind(config.application.ip_addr()).await?;

    axum::serve(listener, app)
        .await
        .context("Error running HTTP server")
}

fn api_router(pg: PgPool) -> Router {
    Router::new()
        .merge(tasks::router())
        .layer(TraceLayer::new_for_http())
        .with_state(pg)
}
