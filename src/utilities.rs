use askama::Template;
use axum::{extract::FromRef, response::Html};
use secrecy::SecretString;
use sqlx::PgPool;

use crate::error::Error;

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
