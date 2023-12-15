use actix_web::{get, post, web, HttpResponse, Scope};
use sodiumoxide::crypto::box_;
use uuid::Uuid;

use crate::{
    database::{
        dto::{
            ChallengeData, ChallengeResponse, Credential, EncryptedMessage, Message, MessageBody,
            SubmitTermsMessageContent,
        },
        querys::{generate_new_session, get_attestation_request_by_id, remove_session},
    },
    error::AppError,
    tx::get_ctype_by_id,
    AppState,
};

#[post("")]
async fn challenge_response_handler(
    state: web::Data<AppState>,
    challenge_response: web::Json<ChallengeResponse>,
) -> Result<HttpResponse, AppError> {
    let others_pubkey =
        crate::tx::parse_encryption_key_from_lightdid(&challenge_response.encryption_key_uri)?;

    let decrypted_challenge = box_::open(
        &challenge_response.encrypted_challenge,
        &challenge_response.nonce,
        &others_pubkey,
        &state.encryption_key,
    )
    .map_err(|_| AppError::Challenge("Unable to decrypt".to_string()))?;

    let session_id = Uuid::from_slice(&decrypted_challenge)
        .map_err(|_| AppError::Challenge("Challenge has a wrong format".to_string()))?;

    let attestation =
        get_attestation_request_by_id(&challenge_response.attestation_id, &state.db_executor)
            .await?;

    let ctype_hash = hex::decode(attestation.ctype_hash.trim_start_matches("0x").trim())?;
    get_ctype_by_id(sp_core::H256::from_slice(&ctype_hash), &state.api).await?;

    let credential: Credential = serde_json::from_value(attestation.credential)?;

    let encryption_key_uri = state.session.key_uri.clone();

    let sender = encryption_key_uri
        .split('#')
        .collect::<Vec<&str>>()
        .first()
        .ok_or_else(|| AppError::Challenge("Invalid Key URI for sender".into()))?
        .to_owned();

    let content = SubmitTermsMessageContent {
        claim: credential.claim,
        quote: None,
        delegation_id: None,
        legitimations: None,
        c_types: "sa".to_string(),
    };

    let msg = Message {
        body: MessageBody {
            type_: "submit-terms".to_string(),
            content: vec![content],
        },
        created_at: 0,
        sender: sender.to_string(),
        receiver: challenge_response.encryption_key_uri.clone(),
        message_id: uuid::Uuid::new_v4().to_string(),
        in_reply_to: None,
        references: None,
    };

    let msg_json = serde_json::to_string(&msg).unwrap();
    let msg_bytes = msg_json.as_bytes();
    let our_secretkey = state.encryption_key.clone();
    let nonce = box_::gen_nonce();
    let encrypted_msg = box_::seal(msg_bytes, &nonce, &others_pubkey, &our_secretkey);
    let response = EncryptedMessage {
        cipher_text: encrypted_msg,
        nonce,
        sender_key_uri: state.session.key_uri.clone(),
        receiver_key_uri: challenge_response.encryption_key_uri.clone(),
    };

    let pool = &state.db_executor;

    if remove_session(pool, session_id).await? {
        Ok(HttpResponse::Ok().json(response))
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
