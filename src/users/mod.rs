use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod db;
mod routes;
mod templates;

pub use routes::router;

use crate::utilities::Result;

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

#[derive(Debug, Serialize, Deserialize)]
struct UserSession {
    user_id: Uuid,
    username: String,
}

impl UserSession {
    const SESSION_KEY: &str = "user_session";
}
