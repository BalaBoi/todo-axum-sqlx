use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    routing::{get, Router},
    Form,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use tracing::{debug, instrument};

use crate::http::users::verify_password;

use super::super::{
    error::Error,
    utilities::{render_template, ApiState, FlashMessageLevel, FlashMessages, Result},
};

use super::{db, hash_password, session::SessionExt, templates::*};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/register", get(register_page).post(register_user))
        .route("/login", get(login_page).post(login_user))
        .route("/logout", get(logout_user))
}

#[instrument(skip_all)]
async fn login_page(mut flash_msgs: FlashMessages) -> Result<Html<String>> {
    let error_flash = flash_msgs
        .get_msgs()
        .await?
        .into_iter()
        .find_map(|fm| if fm.level == FlashMessageLevel::Error {
            Some(fm.msg)
        } else {
            None
        });

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
) -> Result<Redirect> {
    let password_hash = hash_password(&create_user.password).await?;
    db::insert_user(
        &pool,
        &create_user.email,
        &create_user.username,
        &password_hash,
    )
    .await?;

    Ok(Redirect::to("/users/login"))
}

#[derive(Debug, Deserialize)]
struct Credentials {
    email: String,
    password: SecretString,
}

#[instrument(skip_all)]
async fn login_user(
    State(pool): State<PgPool>,
    session: Session,
    mut flash_msgs: FlashMessages,
    Form(credentials): Form<Credentials>,
) -> impl IntoResponse {
    debug!(user_password = ?credentials.password.expose_secret());

    if let Some(user) = db::get_user_by_email(&pool, &credentials.email).await? {
        debug!("user in db");
        if verify_password(&credentials.password, &user.password_hash).await? {
            debug!("user authorized");
            session.create_user_session(&user).await?;
            return Ok(Redirect::to("/todo"));
        }
    }
    debug!("user unauthorized");
    flash_msgs
        .set_msg(FlashMessageLevel::Error, "Incorrect Credentials")
        .await?;
    Err(Error::Unauthorized)
}

#[instrument(skip_all)]
async fn logout_user(session: Session) -> Result<Redirect> {
    session.delete().await?;
    session.cycle_id().await?;
    Ok(Redirect::to("/"))
}
