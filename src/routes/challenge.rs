use actix_web::{get, post, web, HttpResponse, Scope};
use sodiumoxide::crypto::box_;
use uuid::Uuid;

use crate::{
    database::{
        dto::{ChallengeData, ChallengeResponse},
        querys::{generate_new_session, remove_session},
    },
    error::AppError,
    AppState,
};

#[post("")]
async fn challenge_response_handler(
    app_state: web::Data<AppState>,
    challenge_response: web::Json<ChallengeResponse>,
) -> Result<HttpResponse, AppError> {
    log::info!("POST challenge handler");

    let others_pubkey = crate::tx::parse_encryption_key_from_lightdid(
        challenge_response.encryption_key_uri.as_str(),
    )?;

    let decrypted_challenge = box_::open(
        &challenge_response.encrypted_challenge,
        &challenge_response.nonce,
        &others_pubkey,
        &app_state.encryption_key,
    )
    .map_err(|_| AppError::Challenge("Unable to decrypt".to_string()))?;

    let session_id = Uuid::from_slice(&decrypted_challenge)
        .map_err(|_| AppError::Challenge("Challenge has a wrong format".to_string()))?;

    let pool = &app_state.db_executor;

    if remove_session(pool, session_id).await? {
        Ok(HttpResponse::Ok().body("Challenge accepted"))
    } else {
        Err(AppError::Challenge("Invalid Challenge".to_string()))
    }
}

#[get("")]
async fn challenge_handler(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let app_name = state.app_name.clone();
    let challenge = generate_new_session(&state.db_executor).await?;

    let encryption_key_uri = state.session.key_uri.clone();
    let challenge_data = ChallengeData {
        challenge: challenge.as_bytes().to_vec(),
        app_name,
        encryption_key_uri,
    };
    Ok(HttpResponse::Ok().json(challenge_data))
}

pub fn get_challenge_scope() -> Scope {
    web::scope("/api/v1/challenge")
        .service(challenge_handler)
        .service(challenge_response_handler)
}
