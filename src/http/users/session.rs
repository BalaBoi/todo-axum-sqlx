use anyhow::anyhow;
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::debug;
use uuid::Uuid;

use crate::http::{utilities::Result, error::Error};

use super::db::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionData {
    user_id: Uuid,
    username: String,
}

pub struct UserSession {
    session: Session,
    data: UserSessionData, 
}

impl UserSession {
    const SESSION_KEY: &'static str = "user_session";

    async fn update_session(&self) -> Result<()> {
        self.session
            .insert(Self::SESSION_KEY, self.data.clone())
            .await?;

        Ok(())
    }

    pub fn user_id(&self) -> Uuid {
        self.data.user_id
    }

    pub fn username(&self) -> &str {
        &self.data.username
    }
}

impl<S> FromRequestParts<S> for UserSession
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Other(anyhow!("Session middleware seems to not be set")))?;
        
        let user_session = session.get(UserSession::SESSION_KEY)
            .await?
            .ok_or_else(|| {
                debug!("unauthorized in usersession extractor");
                Error::Unauthorized
            })?;
        
        Ok(Self {
            session,
            data: user_session
        })
    }
}

pub trait SessionExt {
    async fn create_user_session(&self, user: &User) -> Result<()>;
}

impl SessionExt for Session {
    async fn create_user_session(&self, user: &User) -> Result<()> {
        debug!("inserting user session into session store");
        self.insert(UserSession::SESSION_KEY, UserSessionData {
            user_id: user.user_id,
            username: user.username.clone()
        })
        .await?;
        Ok(())
    }
}
