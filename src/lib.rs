use anyhow::Context;
use askama::Template;
use axum::{
    extract::FromRef,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use error::Error;
use secrecy::SecretString;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, trace};

mod config;
mod error;
pub mod logging;
pub mod tasks;
mod users;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, FromRef)]
pub struct ApiState {
    pool: PgPool,
    hmac_key: HmacKey,
}

impl ApiState {
    pub fn new(pool: PgPool, hmac_key: HmacKey) -> Self {
        Self { pool, hmac_key }
    }
}

#[derive(Debug, Clone)]
pub struct HmacKey(pub SecretString);

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
