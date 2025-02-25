use anyhow::Context;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, trace};
use utilities::{ApiState, HmacKey};

mod config;
mod error;
pub mod logging;
mod tasks;
mod users;
pub mod utilities;

pub async fn serve_app() -> anyhow::Result<()> {
    trace!("getting config");
    let config = config::get_config();

    trace!("constructing ApiState");
    let state = ApiState::new(
        config.postgres.get_pool().await?,
        HmacKey(config.application.hmac_key.clone()),
    );

    trace!("making api_router");
    let app = api_router(state);
    trace!("binding a TcpListener to the config ip addr");
    let listener = TcpListener::bind(config.application.ip_addr()).await?;

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
    Html(HomeTemplate.render().unwrap())
}

pub fn api_router(state: ApiState) -> Router {
    Router::new()
        .route("/", get(home_page))
        .nest("/todo", tasks::router())
        .nest("/users", users::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
