use anyhow::Context;
use askama::Template;
use axum::{
    http::{HeaderName, Request},
    response::IntoResponse,
    routing::get,
    Router,
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{request_id::MakeRequestUuid, trace::TraceLayer, ServiceBuilderExt};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{info, info_span, instrument, trace};
use utilities::{render_template, ApiState, HmacKey};

use crate::config::Settings;

mod error;
mod tasks;
mod users;
pub mod utilities;

const REQUEST_ID_HEADER: &'static str = "todo-request-id";

pub async fn serve_app(
    config: Settings,
    pool: PgPool,
    listener: TcpListener,
) -> anyhow::Result<()> {
    trace!("constructing ApiState");
    let state = ApiState::new(pool, HmacKey(config.application.hmac_key.clone()));

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
    let req_id_header = HeaderName::from_static(REQUEST_ID_HEADER);
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
    Router::new()
        .route("/", get(home_page))
        .nest("/todo", tasks::router())
        .nest("/users", users::router())
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .set_request_id(req_id_header.clone(), MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                        let req_id = req
                            .headers()
                            .get(REQUEST_ID_HEADER)
                            .expect("request id header should be set by the trace layer")
                            .to_str()
                            .unwrap();

                        info_span!(
                            "request",
                            method = %req.method(),
                            uri = %req.uri(),
                            request_id = %req_id
                        )
                    }),
                )
                .propagate_request_id(req_id_header)
                .layer(session_layer),
        )
}
