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
        .route("/", post(create_task).get(get_tasks))
        .route("/{task_id}", delete(delete_task).get(get_task).put(update_task))
}

#[derive(Debug, serde::Deserialize)]
struct NewTask {
    title: String,
    description: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    task_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    completed: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateTask {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
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
    let query_result = sqlx::query!(
        r#"
        delete from task
        where task_id = $1
        "#,
        task_id
    )
    .execute(&pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

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

#[tracing::instrument(skip(pool))]
async fn update_task(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
    Json(update_task): Json<UpdateTask>,
) -> Result<Json<Task>> {
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

    let mut task = task.unwrap();

    if update_task.title.is_some() {
        task.title = update_task.title.unwrap();
    }

    if update_task.description.is_some() {
        task.description = update_task.description;
    }

    if update_task.completed.is_some() {
        task.completed = update_task.completed.unwrap();
    }

    let task = sqlx::query_as!(
        Task,
        r#"
        insert into task (title, description)
        values ($1, $2)
        returning task_id, title, description, completed, created_at, updated_at
        "#,
        task.title,
        task.description.unwrap_or_else(|| "".into())
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(task))
}
