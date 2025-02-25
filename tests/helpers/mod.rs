use anyhow::Context;
use axum_test::TestServer;
use secrecy::SecretString;
use sqlx::PgPool;
use todo_web_app::{api_router, ApiState, HmacKey};

pub fn test_app(pool: PgPool) -> anyhow::Result<TestServer> {
    let state = ApiState::new(pool, HmacKey(SecretString::from("test hmac yuito")));
    let app_router = api_router(state);
    Ok(TestServer::new(app_router).context("Could not make test server from app router")?)
}
