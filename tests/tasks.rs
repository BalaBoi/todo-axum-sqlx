use helpers::test_app;
use serde_json::json;
use sqlx::PgPool;
use todo_web_app::tasks::Task;

mod helpers;

#[sqlx::test]
async fn creating_a_task(pool: PgPool) {
    let app = test_app(pool).unwrap();
    let task: Task = app
        .post("/todo")
        .json(&json!({
            "title": "Hello",
            "description": "This is the description",
        }))
        .await
        .json();

    assert_eq!(task.title, "Hello");
    assert_eq!(task.description, Some("This is the description".into()));
}
