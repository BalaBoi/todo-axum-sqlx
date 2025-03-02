use secrecy::SecretString;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{Error, ResultExt};
use crate::utilities::Result;

pub async fn insert_user(
    pool: &PgPool,
    email: &str,
    username: &str,
    password_hash: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        insert into users (email, username, password_hash)
        values ($1, $2, $3)
        "#,
        email,
        username,
        password_hash
    )
    .execute(pool)
    .await
    .map_if_constraint("users_email_key", |_| {
        Error::unprocessable_entity([("email", "email is already taken")])
    })
    .map_if_constraint("users_username_key", |_| {
        Error::unprocessable_entity([("username", "username is already taken")])
    })?;

    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: SecretString,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        select * from users
        where email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}
