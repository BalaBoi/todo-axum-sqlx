use todo_web_app::{logging::init_tracing_subscriber, serve_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();
    serve_app().await
}
