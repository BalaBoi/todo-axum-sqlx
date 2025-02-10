use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the databse")]
    SQLx(#[from] sqlx::Error),
    #[error("an internal server error occurred")]
    Other(#[from] anyhow::Error),
    #[error("entity not found")]
    NotFound,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Other(_) | Self::SQLx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match &self {
            Self::SQLx(error) => tracing::error!("SQLx error: {:?}", error),
            Self::Other(error) => tracing::error!("Generic error: {:?}", error),
            _ => {}
        };
        (self.status_code(), self.to_string()).into_response()
    }
}
