use anyhow::Context;
use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{info, instrument, trace};
use utilities::{render_template, ApiState, HmacKey};

use crate::config::Settings;

mod error;
mod tasks;
mod users;
pub mod utilities;

pub async fn serve_app(config: Settings, pool: PgPool, listener: TcpListener) -> anyhow::Result<()> {
    trace!("constructing ApiState");
    let state = ApiState::new(
        pool,
        HmacKey(config.application.hmac_key.clone()),
    );

    trace!("making api_router");
    let app = api_router(state);

    info!("serving app");
    axum::serve(listener, app)
        .await
        .context("Error running HTTP server")
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[instrument(fields(action = "serving the home page"))]
async fn home_page() -> impl IntoResponse {
    render_template(HomeTemplate)
}

pub fn api_router(state: ApiState) -> Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
    Router::new()
        .route("/", get(home_page))
        .nest("/todo", tasks::router())
        .nest("/users", users::router())
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(session_layer),
        )
}
