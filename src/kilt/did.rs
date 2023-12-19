use base58::FromBase58;
use serde_with::{serde_as, Bytes};
use sodiumoxide::crypto::box_;
use std::str::FromStr;
use subxt::{ext::sp_core, OnlineClient};

use crate::{
    error::AppError,
    kilt::{runtime, KiltConfig},
};

use runtime::runtime_types::did::did_details::{DidDetails, DidEncryptionKey, DidPublicKey};

pub async fn get_did_doc(
    did: &str,
    cli: &OnlineClient<KiltConfig>,
) -> Result<DidDetails, AppError> {
    let did = subxt::utils::AccountId32::from_str(did.trim_start_matches("did:kilt:"))
        .map_err(|_| AppError::Did("Invalid DID"))?;
    let did_doc_key = runtime::storage().did().did(&did);
    let details = cli
        .storage()
        .at_latest()
        .await?
        .fetch(&did_doc_key)
        .await?
        .ok_or(AppError::Did("DID not found"))?;

    Ok(details)
}

#[serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDidKeyDetails {
    #[serde_as(as = "Bytes")]
    #[serde(rename = "publicKey")]
    public_key: Vec<u8>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDidDetails {
    e: LightDidKeyDetails,
}

pub fn parse_encryption_key_from_lightdid(did: &str) -> Result<box_::PublicKey, AppError> {
    // example did:kilt:light:00${authAddress}:${details}#encryption
    let mut parts = did.split('#');
    let first = parts
        .next()
        .ok_or(AppError::LightDid("malformed".to_string()))?;
    let mut parts = first.split(':').skip(4);
    let details = parts
        .next()
        .ok_or(AppError::LightDid("malformed".to_string()))?;

    let mut chars = details.chars();
    chars
        .next()
        .ok_or(AppError::LightDid("malformed".to_string()))?;
    let bs: Vec<u8> = FromBase58::from_base58(chars.as_str())
        .map_err(|_| AppError::LightDid("malformed base58".to_string()))?;

    let details: LightDidDetails =
        serde_cbor::from_slice(&bs[1..]).map_err(|e| AppError::LightDid(e.to_string()))?;
    box_::PublicKey::from_slice(&details.e.public_key)
        .ok_or(AppError::LightDid("Not a valid public key".to_string()))
}

fn parse_key_uri(key_uri: &str) -> Result<(&str, sp_core::H256), AppError> {
    let key_uri_parts: Vec<&str> = key_uri.split('#').collect();
    if key_uri_parts.len() != 2 {
        return Err(AppError::Did("Invalid sender key URI"));
    }
    let did = key_uri_parts[0];
    let key_id = key_uri_parts[1];
    let kid_bs: [u8; 32] = hex::decode(key_id.trim_start_matches("0x"))
        .map_err(|_| AppError::Did("key ID isn't valid hex"))?
        .try_into()
        .map_err(|_| AppError::Did("key ID is expected to have 32 bytes"))?;
    let kid = sp_core::H256::from(kid_bs);

    Ok((did, kid))
}

pub async fn get_encryption_key_from_fulldid_key_uri(
    key_uri: &str,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<box_::PublicKey, AppError> {
    let (did, kid) = parse_key_uri(key_uri)?;
    let doc = get_did_doc(did, chain_client).await?;

    let (_, details) = doc
        .public_keys
        .0
        .iter()
        .find(|&(k, _v)| *k == kid)
        .ok_or(AppError::Did("Could not get sender public key"))?;
    let pk = if let DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(pk)) = details.key {
        pk
    } else {
        return Err(AppError::Did("Invalid sender public key"));
    };
    box_::PublicKey::from_slice(&pk).ok_or(AppError::Did("Invalid sender public key"))
}
