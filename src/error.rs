use actix_web::{http::StatusCode, HttpResponse};
use serde_json::error::Category;
use subxt::ext::sp_core::crypto::SecretStringError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Blockchain error: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("Server error: {0}")]
    ActixWeb(#[from] actix_web::Error),
    #[error("Signature error: {0}")]
    Secret(#[from] SecretStringError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Hex error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("Challenge error: {0}")]
    Challenge(String),
    #[error("Light DID error: {0}")]
    LightDid(String),
}

// Is thread safe. No data races or similar can happen.
unsafe impl Send for AppError {}
unsafe impl Sync for AppError {}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self.to_string());
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::Hex(hex::FromHexError::InvalidHexCharacter { .. }) => StatusCode::BAD_REQUEST,
            AppError::Hex(hex::FromHexError::InvalidStringLength) => StatusCode::BAD_REQUEST,
            AppError::Json(e) => match e.classify() {
                Category::Data => StatusCode::BAD_REQUEST,
                Category::Syntax => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
