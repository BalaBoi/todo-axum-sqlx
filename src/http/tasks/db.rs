use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{instrument, warn};
use uuid::Uuid;

use super::super::{error::Error, utilities::Result};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub task_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub user_id: Uuid,
}

#[instrument]
pub async fn create_new_task(
    pool: &PgPool,
    title: &str,
    description: &str,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        r#"
        insert into task (title, description, user_id)
        values ($1, $2, $3)
        "#,
        title,
        description,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[instrument]
pub async fn delete_task(pool: &PgPool, task_id: Uuid) -> Result<()> {
    let query_result = sqlx::query!(
        r#"
        delete from task
        where task_id = $1
        "#,
        task_id
    )
    .execute(pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    Ok(())
}

#[instrument]
pub async fn get_all_tasks(pool: &PgPool, user_id: Uuid) -> Result<Vec<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        select * from task
        where user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(Error::SQLx)
}

#[instrument]
pub async fn get_task(pool: &PgPool, task_id: Uuid, user_id: Uuid) -> Result<Option<Task>> {
    let task = sqlx::query_as!(
        Task,
        r#"
        select * from task
        where task_id = $1
        "#,
        task_id
    )
    .fetch_optional(pool)
    .await?;

    if task.is_none() {
        return Ok(None);
    } else if task.as_ref().unwrap().user_id != user_id {
        warn!(
            ?task,
            ?user_id,
            "user trying to access todo of another user"
        );
        return Ok(None);
    }

    Ok(task)
}

#[instrument]
pub async fn update_task(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
    title: &str,
    description: &str,
    completed: bool,
) -> Result<()> {
    let query_result = sqlx::query!(
        r#"
        update task
        set title = $1, description = $2, completed = $3
        where task_id = $4 and user_id = $5
        "#,
        title,
        description,
        completed,
        task_id,
        user_id
    )
    .execute(pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(())
}
