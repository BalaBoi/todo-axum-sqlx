use anyhow::Context;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    routing::{delete, get, post},
    Form, Json, Router,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

use crate::{error::Error, ApiState, Result};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/", post(create_task).get(tasks_page))
        .route(
            "/{task_id}",
            delete(delete_task).get(get_task).post(update_task),
        )
        .route("/{task_id}/edit", get(edit_task_page))
        .route("/new", get(new_todo_page))
}

#[derive(Template)]
#[template(path = "new_todo.html")]
struct NewTodoTemplate;

#[instrument(skip_all)]
async fn new_todo_page() -> Result<Html<String>> {
    Ok(Html(
        NewTodoTemplate
            .render()
            .context("Error in NewTodoTemplate")?,
    ))
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
    title: String,
    description: String,
    #[serde(default)]
    completed: bool,
}

#[instrument(
    skip_all,
    fields(
        action = "creating a task",
        %new_task.title,
        ?new_task.description
    )
)]
async fn create_task(
    State(pool): State<PgPool>,
    Form(new_task): Form<NewTask>,
) -> Result<Redirect> {
    sqlx::query!(
        r#"
        insert into task (title, description)
        values ($1, $2)
        "#,
        new_task.title,
        new_task.description.unwrap_or_else(|| "".into())
    )
    .execute(&pool)
    .await?;

    Ok(Redirect::to("/todo"))
}

#[instrument(skip_all, fields(action = "deleting a task", %task_id))]
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

#[derive(Template)]
#[template(path = "todos.html")]
struct TodosTemplate {
    todos: Vec<Task>,
}

#[instrument(skip_all, fields(action = "displaying tasks page"))]
async fn tasks_page(State(pool): State<PgPool>) -> Result<Html<String>, Error> {
    let tasks = sqlx::query_as!(
        Task,
        r#"
        select * from task
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Html(
        TodosTemplate { todos: tasks }
            .render()
            .context("Error in TodosTemplate")?,
    ))
}

#[instrument(skip_all, fields(action = "get a task", %task_id))]
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

#[derive(Template)]
#[template(path = "edit_todo.html")]
struct EditTodoTemplate {
    todo: Task,
}

#[instrument(skip_all, fields(action = "displaying edit task page", %task_id))]
async fn edit_task_page(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Html<String>> {
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

    Ok(Html(
        EditTodoTemplate {
            todo: task.unwrap(),
        }
        .render()
        .context("Error in EditTodoTemplate")?,
    ))
}

#[instrument(
    skip_all,
    fields(
        action = "Updating a task",
        %task_id,
        %update_task.title,
        %update_task.description,
        %update_task.completed,
))]
async fn update_task(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
    Form(update_task) :Form<UpdateTask>,
) -> Result<Redirect> {
    let query_result = sqlx::query!(
        r#"
        update task
        set title = $2, description = $3, completed = $4
        where task_id = $1
        "#,
        &task_id,
        &update_task.title,
        &update_task.description,
        update_task.completed
    )
    .execute(&pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(Redirect::to("/todo"))
}
