use axum::{
    extract::{Path, State},
    middleware::from_fn,
    response::{Html, Redirect},
    routing::{delete, get, post},
    Extension, Form, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::http::users::{auth_middleware, UserSessionData};

use super::super::{
    error::Error,
    utilities::{render_template, ApiState, Result},
};

use super::{db, templates::*};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/", post(create_task).get(tasks_page))
        .route("/{task_id}", delete(delete_task).post(update_task))
        .route("/{task_id}/edit", get(edit_task_page))
        .route("/new", get(new_todo_page))
        .route_layer(from_fn(auth_middleware))
}

#[instrument(skip_all)]
pub async fn new_todo_page() -> Result<Html<String>> {
    render_template(NewTodoTemplate)
}

#[derive(Debug, Deserialize)]
pub struct NewTask {
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
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
    Extension(user_session): Extension<UserSessionData>,
    Form(new_task): Form<NewTask>,
) -> Result<Redirect> {
    db::create_new_task(
        &pool,
        &new_task.title,
        &new_task.description,
        user_session.user_id(),
    )
    .await?;

    Ok(Redirect::to("/todo"))
}

#[instrument(skip_all, fields(action = "deleting a task", %task_id))]
pub async fn delete_task(State(pool): State<PgPool>, Path(task_id): Path<Uuid>) -> Result<()> {
    db::delete_task(&pool, task_id).await
}

#[instrument(skip_all, fields(action = "displaying tasks page"))]
pub async fn tasks_page(
    State(pool): State<PgPool>,
    Extension(user_session): Extension<UserSessionData>,
) -> Result<Html<String>> {
    let tasks = db::get_all_tasks(&pool, user_session.user_id()).await?;

    render_template(TodosTemplate {
        todos: tasks,
        username: user_session.username(),
    })
}

#[instrument(skip_all, fields(action = "displaying edit task page", %task_id))]
pub async fn edit_task_page(
    State(pool): State<PgPool>,
    Path(task_id): Path<Uuid>,
    Extension(user_session): Extension<UserSessionData>,
) -> Result<Html<String>> {
    let task = db::get_task(&pool, task_id, user_session.user_id()).await?;

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
    Extension(user_session): Extension<UserSessionData>,
    Form(update_task): Form<UpdateTask>,
) -> Result<Redirect> {
    db::update_task(
        &pool,
        task_id,
        user_session.user_id(),
        &update_task.title,
        &update_task.description,
        update_task.completed,
    )
    .await?;

    Ok(Redirect::to("/todo"))
}
