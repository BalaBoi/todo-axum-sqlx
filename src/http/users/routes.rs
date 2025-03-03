use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, Router},
    Form,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use tracing::{debug, instrument};

use super::super::{
    error::Error,
    utilities::{render_template, ApiState, FlashMessageLevel, FlashMessages, Result},
};

use super::{db, hash_password, templates::*, UserSession};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/register", get(register_page).post(register_user))
        .route("/login", get(login_page).post(login_user))
}

#[instrument(skip_all)]
async fn login_page(mut flash_msgs: FlashMessages) -> Result<Html<String>> {
    let error_flash = flash_msgs
        .get_msgs()
        .await?
        .iter()
        .filter_map(|f| {
            if f.level == FlashMessageLevel::Error {
                Some(f.msg.clone())
            } else {
                None
            }
        })
        .collect::<String>();

    debug!(flash_errors = ?error_flash);

    render_template(LoginTemplate::new(error_flash))
}

#[instrument]
async fn register_page() -> impl IntoResponse {
    render_template(RegisterTemplate)
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

    db::insert_user(
        &pool,
        &create_user.email,
        &create_user.username,
        &password_hash,
    )
    .await?;

    Ok((StatusCode::CREATED, Redirect::to("/users/login")))
}

#[derive(Debug, Deserialize)]
struct Credentials {
    email: String,
    password: SecretString,
}

#[instrument(skip_all, fields(?session))]
async fn login_user(
    State(pool): State<PgPool>,
    session: Session,
    mut flash_msgs: FlashMessages,
    Form(credentials): Form<Credentials>,
) -> impl IntoResponse {
    let password_hash = hash_password(&credentials.password).await?;

    if let Some(user) = db::get_user_by_email(&pool, &credentials.email).await? {
        if user.password_hash.expose_secret().eq(&password_hash) {
            session
                .insert(
                    UserSession::SESSION_KEY,
                    UserSession {
                        user_id: user.user_id,
                        username: user.username,
                    },
                )
                .await?;
            return Ok(Redirect::to("/todo"));
        }
    }
    flash_msgs
        .set_msg(FlashMessageLevel::Error, "Incorrect Credentials")
        .await?;
    Err(Error::Unauthorized)
}
