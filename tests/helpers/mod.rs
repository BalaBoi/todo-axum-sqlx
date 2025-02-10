use anyhow::Context;
use axum_test::TestServer;
use sqlx::PgPool;
use todo_web_app::api_router;

pub fn test_app(pool: PgPool) -> anyhow::Result<TestServer> {
    let app_router = api_router(pool);
    Ok(TestServer::new(app_router).context("Could not make test server from app router")?)
}
