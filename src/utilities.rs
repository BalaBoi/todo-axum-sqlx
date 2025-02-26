use askama::Template;
use axum::{extract::FromRef, response::Html};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
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

#[derive(Debug)]
pub struct FlashMessage {
    message: String,
    tag: String,
}

impl FlashMessage {
    pub fn new(msg: &str, secret: &HmacKey) -> Self {
        //according to hmac docs, this can't fail
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(msg.as_bytes());
        let tag = format!("{:x}", mac.finalize().into_bytes()); //encoding it in hex format
        Self {
            message: msg.to_owned(),
            tag,
        }
    }

    pub fn query_string(&self, key: &str) -> String {
        format!("{}={}&tag={}", key, urlencoding::encode(&self.message), self.tag)
    }

    pub fn verify(msg: &str, tag: &str, secret: &HmacKey) -> bool {
        let tag = match hex::decode(tag) {
            Ok(tag) => tag,
            Err(_) => return false,
        };
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(msg.as_bytes());
        mac.verify_slice(&tag).is_ok()
    }
}
