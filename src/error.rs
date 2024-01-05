use actix_web::{http::StatusCode, HttpResponse};
use serde_json::error::Category;
use subxt::ext::sp_core::crypto::SecretStringError;
use thiserror::Error;

/// The `AppError` enum represents various error types that can occur within the application.
/// It is used to handle and report different error conditions with user-friendly error messages.
#[derive(Debug, Error)]
pub enum AppError {
    /// Represents a database-related error with an associated `sqlx::Error`.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Represents a blockchain-related error with an associated `subxt::Error`.
    #[error("Blockchain error: {0}")]
    Subxt(#[from] subxt::Error),

    /// Represents a server-related error with an associated `actix_web::Error`.
    #[error("Server error: {0}")]
    ActixWeb(#[from] actix_web::Error),

    /// Represents a signature-related error with an associated `SecretStringError`.
    #[error("Signature error: {0}")]
    Secret(#[from] SecretStringError),

    /// Represents a JSON-related error with an associated `serde_json::Error`.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Represents a hexadecimal encoding/decoding error with an associated `hex::FromHexError`.
    #[error("Hex error: {0}")]
    Hex(#[from] hex::FromHexError),
}

/// The `AppError` is designed to be thread-safe. It ensures there are no data races or similar issues.
unsafe impl Send for AppError {}
unsafe impl Sync for AppError {}

/// The `AppError` implements the `actix_web::error::ResponseError` trait, allowing it to provide customized error responses.
impl actix_web::error::ResponseError for AppError {
    /// Generates an HTTP response body with the error message.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    /// Determines the HTTP status code based on the specific error type.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::Hex(hex::FromHexError::InvalidHexCharacter { .. })
            | AppError::Hex(hex::FromHexError::InvalidStringLength) => StatusCode::BAD_REQUEST,

            AppError::Json(e) => match e.classify() {
                Category::Data | Category::Syntax => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
