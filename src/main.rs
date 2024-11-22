use todo_web_app::config;

#[tokio::main]
async fn main() {
    let app_settings = config::get_config();
    println!("{:?}", app_settings);
    let pool = app_settings
        .postgres
        .get_pool()
        .await
        .expect("Should be able to get a connection from the database");
}
