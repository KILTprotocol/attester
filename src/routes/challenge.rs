use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Scope};
use rand::Rng;
use sodiumoxide::crypto::box_;

use crate::{
    database::dto::{ChallengeData, ChallengeResponse},
    error::AppError,
    AppState,
};

#[get("")]
async fn challenge_handler(
    state: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let app_name = state.app_name.clone();

    let mut rng = rand::thread_rng();
    let challenge: [u8; 32] = rng.gen();

    let encryption_key_uri = state.session_encryption_public_key_uri.clone();
    let challenge_data = ChallengeData {
        challenge: challenge.to_vec(),
        app_name,
        encryption_key_uri,
    };

    session
        .insert("challenge", challenge_data.clone())
        .map_err(|_| AppError::Challenge("Could not insert encryption key"))?;

    Ok(HttpResponse::Ok().json(challenge_data))
}

#[post("")]
async fn challenge_response_handler(
    state: web::Data<AppState>,
    challenge_response: web::Json<ChallengeResponse>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let challenge = session
        .get::<ChallengeData>("challenge")
        .map_err(|_| AppError::Challenge("Session not set"))?
        .ok_or(AppError::Challenge("Session not set"))?
        .challenge;

    let encryption_key_uri = &challenge_response.encryption_key_uri;
    let others_pubkey = crate::kilt::parse_encryption_key_from_lightdid(encryption_key_uri)?;

    let decrypted_challenge = box_::open(
        &challenge_response.encrypted_challenge,
        &challenge_response.nonce,
        &others_pubkey,
        &state.secret_key,
    )
    .map_err(|_| AppError::Challenge("Unable to decrypt"))?;

    if decrypted_challenge != challenge {
        return Err(AppError::Challenge("Challenge do not match"));
    }

    session
        .insert("encryption_key_uri", encryption_key_uri)
        .map_err(|_| AppError::Challenge("Could not insert encryption key"))?;

    Ok(HttpResponse::Ok().json("Ok"))
}

pub fn get_challenge_scope() -> Scope {
    web::scope("/api/v1/challenge")
        .service(challenge_handler)
        .service(challenge_response_handler)
}
