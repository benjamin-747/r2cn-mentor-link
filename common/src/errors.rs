use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::model::CommonResult;


#[derive(Debug, Error)]
pub enum CommonError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Authentication failed: {0}")]
    Deny(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
}

impl IntoResponse for CommonError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CommonError::Deny(err) => {
                // This error is caused by bad user input so don't log it
                (StatusCode::UNAUTHORIZED, err)
            }
            CommonError::NotFound(err) => {
                // Because `TraceLayer` wraps each request in a span that contains the request
                // method, uri, etc we don't need to include those details here
                // tracing::error!(%err, "error");

                // Don't expose any details about the error to the client
                (StatusCode::NOT_FOUND, err)
            }
            CommonError::InvalidInput(err) => (StatusCode::BAD_REQUEST, err),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_owned(),
            ),
        };

        (status, Json(CommonResult::<String>::failed(&message))).into_response()
    }
}

#[cfg(test)]
mod tests {}
