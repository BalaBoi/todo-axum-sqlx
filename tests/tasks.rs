use helpers::test_app;
use serde_json::json;
use sqlx::PgPool;

mod helpers;

#[sqlx::test]
async fn creating_a_task(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let task: serde_json::Value = app
        .post("/todo")
        .json(&json!({
            "title": "Hello",
            "description": "This is the description",
        }))
        .await
        .json();

    assert_eq!(task.get("title").unwrap(), "Hello");
    assert_eq!(task.get("description").unwrap(), "This is the description");
}
