use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Form, Json, Router,
};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{debug, warn};

use crate::{
    error::{Error, ResultExt},
    utilities::{render_template, ApiState, FlashMessage, Result, HmacKey},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/register",
            get(register_page).post(register_user).put(update_user),
        )
        .route("/login", get(login_page).post(login_user))
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[tracing::instrument]
async fn register_page() -> impl IntoResponse {
    Html(RegisterTemplate.render().unwrap())
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    errors: Option<String>,
}

#[axum::debug_handler]
#[tracing::instrument]
async fn login_page(State(hmac_key): State<HmacKey>, mut jar: CookieJar) -> Result<(CookieJar, Html<String>)> {
    let errors = match jar.get("error_flash") {
        Some(err_flash) => {
            debug!(err_flash = ?err_flash.to_string());
            let err_flash: FlashMessage = serde_json::from_str(err_flash.value()).context("Error in cookie value")?;
            jar = jar.remove("error_flash");
            if err_flash.verify(&hmac_key) {
                Some(err_flash.message().to_string())
            } else {
                warn!("Flash msg with invalid tag");
                None
            }
        },
        None => None,
    };

    debug!(flash_errors = ?errors);

    Ok((jar, render_template(LoginTemplate { errors })?))
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    password: SecretString,
}

#[tracing::instrument(skip_all, fields(
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

async fn hash_password(password: &SecretString) -> Result<String> {
    let current_span = tracing::Span::current();
    let password = password.clone();
    tokio::task::spawn_blocking(move || -> Result<String> {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut OsRng);
            Ok(Argon2::default()
                .hash_password(password.expose_secret().as_bytes(), &salt)
                .context("Failed to generate password hash")?
                .to_string())
        })
    })
    .await
    .context("panic in spawned blocking thread for hashing")?
}

#[derive(Debug, Deserialize)]
struct UpdateUser {
    username: String,
    prev_password: SecretString,
    new_password: SecretString,
}

#[tracing::instrument(skip_all)]
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
    .ok_or_else(|| Error::Unauthorized(FlashMessage::new("Incorrect Credentials", &api_state.hmac_key)))?;

    if retrieved_record.password_hash != password_hash {
        return Err(Error::Unauthorized(FlashMessage::new("Incorrect Credentials", &api_state.hmac_key)));
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

#[tracing::instrument(skip_all)]
async fn login_user(
    State(state): State<ApiState>,
    Form(credentials): Form<Credentials>,
) -> impl IntoResponse {
    let password_hash = hash_password(&credentials.password).await?;

    let retrieved_record = sqlx::query!(
        r#"
        select * from users
        where email = $1
        "#,
        &credentials.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| Error::Unauthorized(FlashMessage::new("Incorrect Credentials", &state.hmac_key)))?;

    if password_hash == retrieved_record.password_hash {
        Ok(Redirect::to("/todo")) //Figure out how to do sessions here
    } else {
        Err(Error::Unauthorized(FlashMessage::new("Incorrect Credentials", &state.hmac_key)))
    }
}
