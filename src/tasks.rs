use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, post},
    Json, Router,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{error::Error, Result};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/todo", post(create_task).get(get_tasks))
        .route("/todo/{task_id}", delete(delete_task).get(get_task))
}

#[derive(Debug, serde::Deserialize)]
struct NewTask {
    title: String,
    description: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
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
) -> Result<(StatusCode, Json<Task>)> {
    tracing::info!("Hello");
    let task = sqlx::query_as!(
        Task,
        r#"
        insert into task (title, description)
        values ($1, $2)
        returning task_id, title, description, completed, created_at, updated_at
        "#,
        new_task.title,
        new_task.description.unwrap_or_else(|| "".into())
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

#[tracing::instrument(skip(pool))]
async fn delete_task(State(pool): State<PgPool>, Path(task_id): Path<Uuid>) -> Result<()> {
    sqlx::query!(
        r#"
        delete from task
        where task_id = $1
        "#,
        task_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_tasks(State(pool): State<PgPool>) -> Result<Json<Vec<Task>>> {
    let tasks = sqlx::query_as!(
        Task,
        r#"
        select * from task
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(tasks))
}

#[tracing::instrument(skip(pool))]
async fn get_task(State(pool): State<PgPool>, Path(task_id): Path<Uuid>) -> Result<Json<Task>> {
    let task = sqlx::query_as!(
        Task,
        r#"
        select * from task
        where task_id = $1
        "#,
        task_id
    )
    .fetch_optional(&pool)
    .await?;

    if task.is_none() {
        return Err(Error::NotFound);
    }

    Ok(Json(task.unwrap()))
}
