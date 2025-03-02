use anyhow::Context;
use askama::Template;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use tower_sessions::Session;
use tracing::{debug, instrument};
use axum::{extract::State, http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, Router}, Form, Json};
use serde::Deserialize;

use crate::{error::{Error, ResultExt}, utilities::{render_template, ApiState, FlashMessage, FlashMessageLevel, Result}};

use super::{hash_password, UserSession, templates::*, User};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/register",
            get(register_page).post(register_user).put(update_user),
        )
        .route("/login", get(login_page).post(login_user))
}

#[instrument]
async fn login_page(session: Session) -> Result<Html<String>> {
    let error_flash = match session
        .get(FlashMessage::SESSION_KEY)
        .await
        .context("Session store error")?
    {
        Some(FlashMessage {
            level: FlashMessageLevel::Error,
            msg,
        }) => Some(msg),
        _ => None,
    };

    debug!(flash_errors = ?error_flash);

    render_template(LoginTemplate::new(error_flash))
}

#[instrument]
async fn register_page() -> impl IntoResponse {
    Html(RegisterTemplate.render().unwrap())
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    password: SecretString,
}

#[instrument(skip_all, fields(
    action = "registering a user",
    username = create_user.username,
    email = create_user.email
))]
async fn register_user(
    State(pool): State<PgPool>,
    Form(create_user): Form<CreateUser>,
) -> Result<(StatusCode, Redirect)> {
    let password_hash = hash_password(&create_user.password).await?;

    let _user_id = sqlx::query_scalar!(
        r#"
        insert into users (email, username, password_hash)
        values ($1, $2, $3)
        returning user_id
        "#,
        create_user.email,
        create_user.username,
        password_hash
    )
    .fetch_one(&pool)
    .await
    .map_if_constraint("users_email_key", |_| {
        Error::unprocessable_entity([("email", "email is already taken")])
    })
    .map_if_constraint("users_username_key", |_| {
        Error::unprocessable_entity([("username", "username is already taken")])
    })?;

    Ok((StatusCode::CREATED, Redirect::to("/users/login")))
}


#[derive(Debug, Deserialize)]
struct UpdateUser {
    username: String,
    prev_password: SecretString,
    new_password: SecretString,
}

#[instrument(skip_all)]
async fn update_user(
    State(api_state): State<ApiState>,
    Json(user_update): Json<UpdateUser>,
) -> Result<()> {
    let password_hash = hash_password(&user_update.prev_password).await?;

    let retrieved_record = sqlx::query!(
        r#"
        select * from users
        where username = $1
        "#,
        &user_update.username
    )
    .fetch_optional(&api_state.pool)
    .await?
    .ok_or_else(|| Error::Unauthorized)?;

    if retrieved_record.password_hash != password_hash {
        return Err(Error::Unauthorized);
    }

    let new_password_hash = hash_password(&user_update.new_password).await?;

    let query_result = sqlx::query!(
        r#"
        update users
        set username = $1, password_hash = $2
        where user_id = $3
        "#,
        &user_update.username,
        &new_password_hash,
        &retrieved_record.user_id
    )
    .execute(&api_state.pool)
    .await
    .map_if_constraint("users_username_key", |_| {
        Error::unprocessable_entity([("username", "username is already taken")])
    })?;

    if query_result.rows_affected() != 1 {
        return Err(Error::Other(anyhow::anyhow!(
            "No row was affected in table users"
        )));
    }

    Ok(())
}


#[derive(Debug, Deserialize)]
struct Credentials {
    email: String,
    password: SecretString,
}

#[instrument(skip_all, fields(?session))]
async fn login_user(
    State(state): State<ApiState>,
    session: Session,
    Form(credentials): Form<Credentials>,
) -> impl IntoResponse {
    let password_hash = hash_password(&credentials.password).await?;

    if let Some(user) = sqlx::query_as!(
        User,
        r#"
        select * from users
        where email = $1
        "#,
        &credentials.email
    )
    .fetch_optional(&state.pool)
    .await?
    {
        if user.password_hash.expose_secret().eq(&password_hash) {
            session
                .insert(
                    UserSession::SESSION_KEY,
                    UserSession {
                        user_id: user.user_id,
                        username: user.username,
                    },
                )
                .await
                .context("session error")?;
            return Ok(Redirect::to("/todo"));
        }
    }
    session
        .insert(
            FlashMessage::SESSION_KEY,
            FlashMessage {
                level: FlashMessageLevel::Error,
                msg: String::from("Incorrect Credentials"),
            },
        )
        .await
        .context("session store error")?;
    Err(Error::Unauthorized)
}


