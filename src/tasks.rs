use axum::{extract::State, routing::post, Json, Router};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::Result;

pub fn router() -> Router<PgPool> {
    Router::new().route("/todo", post(create_task))
}

#[derive(Debug, serde::Deserialize)]
struct NewTask {
    title: String,
    description: String,
}

#[derive(Debug, serde::Serialize)]
struct Task {
    task_id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[tracing::instrument(skip(pool))]
async fn create_task(
    State(pool): State<PgPool>,
    Json(new_task): Json<NewTask>,
) -> Result<Json<Task>> {
    let task = sqlx::query_as!(
        Task,
        r#"
        insert into task (title, description)
        values ($1, $2)
        returning task_id, title, description, completed, created_at, updated_at
        "#,
        new_task.title,
        new_task.description
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(task))
}
