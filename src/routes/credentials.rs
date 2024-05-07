use actix_session::Session;
use actix_web::{post, web, HttpResponse, Scope};
use sodiumoxide::crypto::box_;
use sp_core::H256;
use subxt::OnlineClient;
use uuid::Uuid;

use crate::{
    database::{
        dto::{
            Credential, EncryptedMessage, Message, MessageBody, RequestAttestationMessageContent,
            SubmitTermsMessageContent,
        },
        querys::{approve_attestation_request, get_attestation_request_by_id},
    },
    error::AppError,
    kilt::KiltConfig,
    AppState,
};

#[post("/terms/{attestation_id}")]
async fn send_terms(
    state: web::Data<AppState>,
    param: web::Path<Uuid>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let attestation_id = param.into_inner();

    let sender_key_uri = session
        .get::<String>("encryption_key_uri")
        .map_err(|_| AppError::Challenge("Session not set"))?
        .ok_or(AppError::Challenge("Session not set"))?;

    let others_pubkey = crate::kilt::parse_encryption_key_from_lightdid(&sender_key_uri)?;

    let attestation = get_attestation_request_by_id(&attestation_id, &state.db_executor).await?;

    let ctype_hash = { attestation.ctype_hash };

    let credential: Credential = serde_json::from_value(attestation.credential)?;

    let encryption_key_uri = state.session_encryption_public_key_uri.clone();

    let sender = encryption_key_uri
        .split('#')
        .collect::<Vec<&str>>()
        .first()
        .ok_or_else(|| AppError::Attestation("Invalid Key URI for sender"))?
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
    let our_secretkey = state.secret_key.clone();
    let nonce = box_::gen_nonce();
    let encrypted_msg = box_::seal(msg_bytes, &nonce, &others_pubkey, &our_secretkey);
    let response = EncryptedMessage {
        cipher_text: encrypted_msg,
        nonce,
        sender_key_uri: state.session_encryption_public_key_uri.clone(),
        receiver_key_uri: sender_key_uri,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/{attestation_id}")]
async fn request_attestation(
    state: web::Data<AppState>,
    encrypted_message: web::Json<EncryptedMessage>,
    param: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let attestation_id = param.into_inner();

    let attestation = get_attestation_request_by_id(&attestation_id, &state.db_executor).await?;

    if attestation.approved_at.is_none() {
        Err(actix_web::error::ErrorBadRequest(
            "Attestation is not approved",
        ))?
    }

    let chain_client = OnlineClient::<KiltConfig>::from_url(&state.endpoint).await?;

    let others_pubkey = crate::kilt::get_encryption_key_from_fulldid_key_uri(
        &encrypted_message.sender_key_uri,
        &chain_client,
    )
    .await?;

    let decrypted_message_bytes = box_::open(
        &encrypted_message.cipher_text,
        &encrypted_message.nonce,
        &others_pubkey,
        &state.secret_key,
    )
    .map_err(|_| AppError::Attestation("Unable to decrypt"))?;

    let decrypted_message: Message<RequestAttestationMessageContent> =
        serde_json::from_slice(&decrypted_message_bytes).unwrap();

    let credential = decrypted_message.body.content.credential;

    let ctype_hash = hex::decode(credential.claim.ctype_hash.trim_start_matches("0x").trim())?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 || ctype_hash.len() != 32 {
        Err(actix_web::error::ErrorBadRequest(
            "Claim hash or ctype hash have a wrong format",
        ))?
    }

    let payer = state.payer.clone();
    let did = state.attester_did.clone();
    let chain_client = OnlineClient::<KiltConfig>::from_url(&state.endpoint).await?;
    let signer = state.signer.clone();

    crate::kilt::create_claim(
        H256::from_slice(&claim_hash),
        H256::from_slice(&ctype_hash),
        &did,
        &chain_client,
        &payer,
        &signer,
    )
    .await?;

    let mut db_tx = state.db_executor.begin().await?;
    approve_attestation_request(&attestation_id, &mut db_tx).await?;
    db_tx.commit().await?;

    Ok(HttpResponse::Ok().json("ok"))
}

pub fn get_credential_scope() -> Scope {
    web::scope("/api/v1/credential")
        .service(send_terms)
        .service(request_attestation)
}
