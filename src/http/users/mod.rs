use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier
};
use secrecy::{ExposeSecret, SecretString};

mod db;
mod routes;
mod session;
mod templates;

pub use routes::router;
pub use session::auth;

use super::{error::Error, utilities::Result};

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

async fn verify_password(password: &SecretString, password_hash: &SecretString) -> Result<bool> {
    let argon_password_hash = PasswordHash::new(password_hash.expose_secret())
        .context("failed to parse password in phc format")
        .map_err(Error::Other)?;

    match Argon2::default()
        .verify_password(
            password.expose_secret().as_bytes(),
            &argon_password_hash
        )
    {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }

}
