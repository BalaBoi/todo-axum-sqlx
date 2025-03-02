use todo_web_app::{get_config, init_tracing_subscriber, serve_app};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();
    let config = get_config();
    let pool = config
        .postgres
        .get_pool()
        .await
        .expect("should be able to get database connection pool");
    let ip_addr = config.application.ip_addr();
    let listener = TcpListener::bind(ip_addr)
        .await
        .expect("should be able to bind to the addr");
    serve_app(config, pool, listener).await
}
