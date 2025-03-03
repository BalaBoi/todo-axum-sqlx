use axum::{extract::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::debug;
use uuid::Uuid;

use crate::http::{error::Error, utilities::Result};

use super::db::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionData {
    user_id: Uuid,
    username: String,
}

impl UserSessionData {
    const SESSION_KEY: &'static str = "user_session";

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

pub trait SessionExt {
    async fn create_user_session(&self, user: &User) -> Result<()>;
}

impl SessionExt for Session {
    async fn create_user_session(&self, user: &User) -> Result<()> {
        debug!("inserting user session into session store");
        self.insert(
            UserSessionData::SESSION_KEY,
            UserSessionData {
                user_id: user.user_id,
                username: user.username.clone(),
            },
        )
        .await?;
        self.cycle_id().await?;
        Ok(())
    }
}

pub async fn auth_middleware(session: Session, mut req: Request, next: Next) -> Result<Response> {
    match session
        .get::<UserSessionData>(UserSessionData::SESSION_KEY)
        .await?
    {
        Some(user_session_data) => {
            req.extensions_mut().insert(user_session_data);
            let response = next.run(req).await;
            Ok(response)
        }
        None => Err(Error::Unauthorized),
    }
}
