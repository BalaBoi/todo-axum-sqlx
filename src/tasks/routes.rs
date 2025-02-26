use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    Form,
};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    error::Error,
    utilities::{render_template, Result},
};

use super::{db, templates::*};

#[instrument(skip_all)]
pub async fn new_todo_page() -> Result<Html<String>> {
    render_template(NewTodoTemplate)
}

#[derive(Debug, serde::Deserialize)]
pub struct NewTask {
    title: String,
    description: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateTask {
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
pub async fn create_task(
    State(pool): State<PgPool>,
    Form(new_task): Form<NewTask>,
) -> Result<Redirect> {
    db::create_new_task(&pool, &new_task.title, &new_task.description).await?;

    Ok(Redirect::to("/todo"))
}

#[instrument(skip_all, fields(action = "deleting a task", %task_id))]
pub async fn delete_task(State(pool): State<PgPool>, Path(task_id): Path<Uuid>) -> Result<()> {
    db::delete_task(&pool, task_id).await
}

#[instrument(skip_all, fields(action = "displaying tasks page"))]
pub async fn tasks_page(State(pool): State<PgPool>) -> Result<Html<String>> {
    let tasks = db::get_all_tasks(&pool).await?;

    render_template(TodosTemplate { todos: tasks })
}

#[instrument(skip_all, fields(action = "displaying edit task page", %task_id))]
pub async fn edit_task_page(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Html<String>> {
    let task = db::get_task(&pool, task_id).await?;

    if task.is_none() {
        return Err(Error::NotFound);
    }

    render_template(EditTodoTemplate {
        todo: task.unwrap(),
    })
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
pub async fn update_task(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
    Form(update_task): Form<UpdateTask>,
) -> Result<Redirect> {
    db::update_task(
        &pool,
        task_id,
        &update_task.title,
        &update_task.description,
        update_task.completed,
    )
    .await?;

    Ok(Redirect::to("/todo"))
}
