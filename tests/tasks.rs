use helpers::test_app;
use serde_json::json;
use sqlx::PgPool;

mod helpers;

#[sqlx::test]
async fn creating_a_task(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let response = app
        .post("/todo")
        .form(&json!({
            "title": "Hello",
            "description": "This is the description",
        }))
        .await;

    response.assert_status_see_other();
    response.assert_header("Location", "/todo");
}
