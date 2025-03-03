use anyhow::anyhow;
use askama::Template;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::Html,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_sessions::Session;

use super::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, FromRef)]
pub struct ApiState {
    pub pool: PgPool,
    pub hmac_key: HmacKey,
}

impl ApiState {
    pub fn new(pool: PgPool, hmac_key: HmacKey) -> Self {
        Self { pool, hmac_key }
    }
}

#[derive(Debug, Clone)]
pub struct HmacKey(pub SecretString);

pub fn render_template<T>(template: T) -> Result<Html<String>>
where
    T: Template,
{
    Ok(Html(template.render()?))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlashMessage {
    pub level: FlashMessageLevel,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FlashMessageLevel {
    Error,
    Success,
}

pub struct FlashMessages {
    session: Session,
    msgs: Vec<FlashMessage>,
}

impl FlashMessages {
    const SESSION_KEY: &'static str = "flash_messages";

    async fn update_session(&self) -> Result<()> {
        self.session
            .insert(Self::SESSION_KEY, self.msgs.clone())
            .await?;

        Ok(())
    }

    pub async fn set_msg(&mut self, level: FlashMessageLevel, msg: &str) -> Result<()> {
        self.msgs.push(FlashMessage {
            level,
            msg: msg.to_string(),
        });
        self.update_session().await
    }

    pub async fn get_msgs(&mut self) -> Result<Vec<FlashMessage>> {
        let out = std::mem::take(&mut self.msgs);
        self.update_session().await?;
        Ok(out)
    }
}

impl<S> FromRequestParts<S> for FlashMessages
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Other(anyhow!("Session manager layer seems to not be present")))?;

        let flash_msgs_vec = session.get(Self::SESSION_KEY).await?.unwrap_or_default();

        let flash_msgs = Self {
            session,
            msgs: flash_msgs_vec,
        };

        flash_msgs.update_session().await?;

        Ok(flash_msgs)
    }
}
