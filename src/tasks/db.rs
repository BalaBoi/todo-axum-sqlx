use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

use crate::{error::Error, utilities::Result};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub task_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[instrument]
pub async fn create_new_task(pool: &PgPool, title: &str, description: &str) -> Result<()> {
    sqlx::query!(
        r#"
        insert into task (title, description)
        values ($1, $2)
        "#,
        title,
        description
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
pub async fn get_all_tasks(pool: &PgPool) -> Result<Vec<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        select * from task
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| Error::SQLx(e))
}

#[instrument]
pub async fn get_task(pool: &PgPool, task_id: Uuid) -> Result<Option<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        select * from task
        where task_id = $1
        "#,
        task_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| Error::SQLx(e))
}

#[instrument]
pub async fn update_task(
    pool: &PgPool,
    task_id: Uuid,
    title: &str,
    description: &str,
    completed: bool,
) -> Result<()> {
    let query_result = sqlx::query!(
        r#"
        update task
        set title = $2, description = $3, completed = $4
        where task_id = $1
        "#,
        task_id,
        title,
        description,
        completed
    )
    .execute(pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(())
}
