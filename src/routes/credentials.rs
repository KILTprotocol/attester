use actix_web::{post, web, HttpResponse, Scope};
use sodiumoxide::crypto::box_;

use crate::{
    database::{
        dto::{
            Credential, EncryptedMessage, Message, MessageBody, RequestAttestationMessageContent,
            RequestTerms, SubmitAttestationMessageBody, SubmitTermsMessageContent,
        },
        querys::{get_attestation_request_by_id, get_session},
    },
    error::AppError,
    AppState,
};

#[post("")]
async fn request_attestation(
    state: web::Data<AppState>,
    encrypted_message: web::Json<EncryptedMessage>,
) -> Result<HttpResponse, AppError> {
    let sender = &encrypted_message.sender_key_uri;
    let others_pubkey =
        crate::kilt::get_encryption_key_from_fulldid_key_uri(sender, &state.chain_client).await?;

    let decrypted_message_bytes = box_::open(
        &encrypted_message.cipher_text,
        &encrypted_message.nonce,
        &others_pubkey,
        &state.encryption_key,
    )
    .map_err(|_| AppError::Challenge("Unable to decrypt".to_string()))?;

    let decrypted_message: Message<RequestAttestationMessageContent> =
        serde_json::from_slice(&decrypted_message_bytes).unwrap();

    let credential = decrypted_message.body.content.credential;

    let content = SubmitAttestationMessageBody {
        c_type_hash: credential.claim.ctype_hash,
        claim_hash: credential.root_hash,
        owner: credential.claim.owner,
        delegation_id: None,
        revoke: false,
    };

    let msg_response = Message {
        body: MessageBody {
            content,
            type_: "submit-attestation".to_string(),
        },
        created_at: 0,
        sender: state.session.key_uri.clone(),
        receiver: sender.clone(),
        message_id: uuid::Uuid::new_v4().to_string(),
        in_reply_to: None,
        references: None,
    };

    let msg_json = serde_json::to_string(&msg_response)?;
    let msg_bytes = msg_json.as_bytes();
    let our_secretkey = state.encryption_key.clone();
    let nonce = box_::gen_nonce();
    let encrypted_msg = box_::seal(msg_bytes, &nonce, &others_pubkey, &our_secretkey);
    let response = EncryptedMessage {
        cipher_text: encrypted_msg,
        nonce,
        sender_key_uri: state.session.key_uri.clone(),
        receiver_key_uri: sender.to_string(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/terms")]
async fn send_terms(
    state: web::Data<AppState>,
    body: web::Json<RequestTerms>,
) -> Result<HttpResponse, AppError> {
    let RequestTerms {
        challenge,
        attestation_id,
    } = body.0;

    let session = get_session(&state.db_executor, challenge).await?;

    if session.encryption_key_uri.is_none() {
        log::error!(
            "Found session without encryption key uri in send terms: {}",
            challenge
        );
        return Ok(HttpResponse::BadRequest().json("Session set up not completed"));
    }

    let sender_key_uri = session.encryption_key_uri.unwrap();
    let others_pubkey = crate::kilt::parse_encryption_key_from_lightdid(&sender_key_uri)?;

    let attestation = get_attestation_request_by_id(&attestation_id, &state.db_executor).await?;

    let ctype_hash = { attestation.ctype_hash };

    let credential: Credential = serde_json::from_value(attestation.credential)?;

    let encryption_key_uri = state.session.key_uri.clone();

    let sender = encryption_key_uri
        .split('#')
        .collect::<Vec<&str>>()
        .first()
        .ok_or_else(|| AppError::Challenge("Invalid Key URI for sender".into()))?
        .to_owned();

    let mut claim = credential.claim;

    claim.ctype_hash = ctype_hash.clone();

    let content = SubmitTermsMessageContent {
        claim,
        legitimations: Some(vec![]),
    };

    let msg = Message {
        body: MessageBody {
            content,
            type_: "submit-terms".to_string(),
        },
        created_at: 0,
        sender: sender.to_string(),
        receiver: sender_key_uri.clone(),
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
        receiver_key_uri: sender_key_uri,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn get_credential_scope() -> Scope {
    web::scope("/api/v1/credential")
        .service(send_terms)
        .service(request_attestation)
}
