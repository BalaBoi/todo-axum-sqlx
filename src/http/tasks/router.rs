use axum::{
    middleware::from_fn, routing::{delete, get, post}, Router
};

use crate::http::users::auth;

use super::super::utilities::ApiState;

use super::routes::*;

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/", post(create_task).get(tasks_page))
        .route("/{task_id}", delete(delete_task).post(update_task))
        .route("/{task_id}/edit", get(edit_task_page))
        .route("/new", get(new_todo_page))
        .route_layer(from_fn(auth))
}
