use actix_web::web::ReqData;
use serde::de::Error;
use serde::Deserialize;
use serde::{Deserializer, Serializer};
use sqlx::PgPool;
use subxt::{
    ext::sp_core::{crypto::SecretStringError, sr25519, Pair},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};
use uuid::Uuid;

use crate::{
    database::{dto::AttestationResponse, querys::get_attestation_request_by_id},
    error::AppError,
    tx::{
        kilt::{self, runtime_types::did::did_details::DidSignature},
        KiltConfig,
    },
    User,
};

pub async fn get_current_block(api: &OnlineClient<KiltConfig>) -> Result<u64, subxt::Error> {
    let block_number = api
        .rpc()
        .block(None)
        .await
        .map_err(|e| format!("Failed to get block number: {e}"))?
        .ok_or("Failed to get block number")?
        .block
        .header
        .number;

    log::info!("Current block for TX: {}", block_number);
    Ok(block_number)
}

pub async fn get_next_tx_counter(
    api: &OnlineClient<KiltConfig>,
    did_address: &AccountId32,
) -> Result<u64, AppError> {
    let did_doc_addr = kilt::storage().did().did(did_address);
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
    signer: &PairSigner<KiltConfig, sr25519::Pair>,
) -> Result<DidSignature, SecretStringError> {
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

/// Serialize and deserialize `0x` prefixed hex strings to and from Vec<u8>.
pub mod prefixed_hex {

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        hex::decode(s.trim_start_matches("0x")).map_err(Error::custom)
    }

    pub fn serialize<S>(value: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(value);
        s.serialize_str(&format!("0x{}", hex))
    }
}

/// Serialize and deserialize `0x` prefixed hex strings to and from Vec<u8>.
pub mod hex_nonce {
    use sodiumoxide::crypto::box_::Nonce;

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Nonce, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(s.trim_start_matches("0x")).map_err(Error::custom)?;
        Nonce::from_slice(&bytes).ok_or_else(|| Error::custom("invalid nonce length"))
    }

    pub fn serialize<S>(value: &Nonce, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(value);
        s.serialize_str(&format!("0x{}", hex))
    }
}
