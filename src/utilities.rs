use anyhow::Context;
use askama::Template;
use axum::{extract::FromRef, response::Html};
use axum_extra::extract::CookieJar;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use tracing::{debug, warn};

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

#[derive(Debug, Deserialize, Serialize)]
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

    pub fn verify(&self, secret: &HmacKey) -> bool {
        let tag = match hex::decode(&self.tag) {
            Ok(tag) => tag,
            Err(_) => return false,
        };
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(self.message.as_bytes());
        mac.verify_slice(&tag).is_ok()
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub fn get_flash_errors(
    mut jar: CookieJar,
    hmac_key: &HmacKey,
) -> Result<(CookieJar, Option<String>)> {
    let errors = match jar.get("error_flash") {
        Some(err_flash) => {
            debug!(err_flash = ?err_flash.to_string());
            let err_flash: FlashMessage =
                serde_json::from_str(err_flash.value()).context("Error in cookie value")?;
            jar = jar.remove("error_flash");
            if err_flash.verify(hmac_key) {
                Some(err_flash.message().to_string())
            } else {
                warn!("Flash msg with invalid tag");
                None
            }
        }
        None => None,
    };

    Ok((jar, errors))
}
