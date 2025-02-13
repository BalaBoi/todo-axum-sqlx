use anyhow::Context;
use axum::Router;
use error::Error;
use logging::init_tracing_subscriber;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

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

pub fn api_router(pg: PgPool) -> Router {
    Router::new()
        .nest("/todo", tasks::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(pg)
}
