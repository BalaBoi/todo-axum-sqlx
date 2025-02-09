use todo_web_app::serve_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    serve_app().await
}
