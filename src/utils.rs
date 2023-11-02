use actix_web::web::ReqData;
use sqlx::PgPool;
use subxt::ext::sp_core::{crypto::SecretStringError, Pair};
use uuid::Uuid;

use crate::{
    configuration::Configuration,
    database::{dto::AttestationResponse, querys::get_attestation_request_by_id},
    error::AppError,
    tx::kilt::{self, runtime_types::did::did_details::DidSignature},
    User,
};

pub async fn get_current_block(config: &Configuration) -> Result<u64, subxt::Error> {
    let api = config.get_client().await?;
    let block_number = api
        .rpc()
        .block(None)
        .await
        .map_err(|e| format!("Failed to get block number: {e}"))?
        .ok_or("Failed to get block number")?
        .block
        .header
        .number;
    Ok(block_number)
}

pub async fn get_next_tx_counter(config: &Configuration) -> Result<u64, AppError> {
    let api = config.get_client().await?;
    let did_doc_addr = kilt::storage().did().did(&config.get_did()?);
    let tx_counter = api
        .storage()
        .at_latest()
        .await?
        .fetch(&did_doc_addr)
        .await?
        .map(|doc| doc.last_tx_counter + 1)
        .unwrap_or(1u64);
    Ok(tx_counter)
}

pub fn calculate_signature(
    call: &[u8],
    config: Configuration,
) -> Result<DidSignature, SecretStringError> {
    let signer = config.get_credential_signer()?;
    let signed_data = signer.signer().sign(call);
    Ok(DidSignature::Sr25519(
        kilt::runtime_types::sp_core::sr25519::Signature(signed_data.into()),
    ))
}

pub fn is_user_allowed_to_see_data(
    user: ReqData<User>,
    attestatations: &Vec<AttestationResponse>,
) -> Result<(), actix_web::Error> {
    let user_ids = attestatations
        .iter()
        .map(|a| &a.claimer)
        .all(|claimer| claimer == &user.id);

    if user_ids || user.is_admin {
        Ok(())
    } else {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allow to see data",
        ))
    }
}

pub async fn is_user_allowed_to_update_data(
    user: ReqData<User>,
    attestation_id: &Uuid,
    db_executor: &PgPool,
) -> Result<(), actix_web::Error> {
    let attestation = get_attestation_request_by_id(attestation_id, db_executor).await?;

    if attestation.claimer == user.id || user.is_admin {
        Ok(())
    } else {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allow to see data",
        ))
    }
}

pub fn is_user_admin(user: ReqData<User>) -> Result<(), actix_web::Error> {
    if !user.is_admin {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allow to see data",
        ))
    } else {
        Ok(())
    }
}
