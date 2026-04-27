use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
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
        let (status, code, message) = match self {
            CommonError::Deny(err) => {
                // This error is caused by bad user input so don't log it
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", err)
            }
            CommonError::NotFound(err) => {
                // Because `TraceLayer` wraps each request in a span that contains the request
                // method, uri, etc we don't need to include those details here
                // tracing::error!(%err, "error");

                // Don't expose any details about the error to the client
                (StatusCode::NOT_FOUND, "NOT_FOUND", err)
            }
            CommonError::InvalidInput(err) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", err),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Something went wrong".to_owned(),
            ),
        };

        (
            status,
            Json(CommonResult::<String>::failed_with_code(code, &message)),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {}
