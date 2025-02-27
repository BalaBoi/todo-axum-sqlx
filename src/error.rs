use std::{borrow::Cow, collections::HashMap};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use sqlx::error::DatabaseError;

use crate::utilities::FlashMessage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the databse")]
    SQLx(#[from] sqlx::Error),
    #[error("an internal server error occurred")]
    Other(#[from] anyhow::Error),
    #[error("entity not found")]
    NotFound,
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },
    #[error("error in authentication")]
    Unauthorized(FlashMessage),
    #[error("error in displaying page")]
    Template(#[from] askama::Error),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { errors: _ } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Other(_) | Self::SQLx(_) | Self::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::SEE_OTHER,
        }
    }

    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut map = HashMap::new();
        for (entity, issue) in errors {
            map.entry(entity.into())
                .or_insert_with(Vec::new)
                .push(issue.into());
        }
        Self::UnprocessableEntity { errors: map }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match &self {
            Self::SQLx(error) => tracing::error!("SQLx error: {:?}", error),
            Self::UnprocessableEntity { errors } => {
                tracing::trace!("Errors in the reguest: {:?}", errors)
            }
            Self::Other(error) => tracing::error!("Generic error: {:?}", error),
            Self::Unauthorized(flash_msg) => {
                tracing::trace!("Authentication failed");
                let jar = CookieJar::new().add(Cookie::new(
                    "error_flash",
                    serde_json::to_string(flash_msg).unwrap(),
                ));
                return (jar, Redirect::to("/users/login")).into_response();
            }
            Self::Template(error) => tracing::error!("Template rendering error: {:?}", error),
            _ => {}
        };
        (self.status_code(), self.to_string()).into_response()
    }
}

///Convenience trait for being able to easily convert constraint based DatabaseErrors from sqlx to some other error
pub trait ResultExt<T> {
    fn map_if_constraint<F>(self, constraint: &str, map_err: F) -> Result<T, Error>
    where
        F: FnOnce(Box<dyn DatabaseError>) -> Error;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn map_if_constraint<F>(self, constraint: &str, map_err: F) -> Result<T, Error>
    where
        F: FnOnce(Box<dyn DatabaseError>) -> Error,
    {
        self.map_err(|err| match err.into() {
            Error::SQLx(sqlx::Error::Database(db_err))
                if db_err.constraint() == Some(constraint) =>
            {
                map_err(db_err)
            }
            other => other,
        })
    }
}
