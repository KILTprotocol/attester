use base58::FromBase58;
use parity_scale_codec::Decode;
use serde_with::{serde_as, Bytes};
use sodiumoxide::crypto::box_;
use subxt::{
    blocks::{BlockBody, ExtrinsicDetails},
    config::polkadot::PolkadotExtrinsicParams,
    config::Config,
    ext::{
        codec::Encode,
        sp_core, sp_runtime,
        sp_runtime::traits::{IdentifyAccount, Verify},
    },
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};

use crate::{
    error::AppError,
    utils::{calculate_signature, get_current_block, get_next_tx_counter},
};

use self::kilt::runtime_types;
use self::kilt::runtime_types::did::did_details::DidAuthorizedCallOperation;

#[cfg(feature = "spiritnet")]
#[subxt::subxt(runtime_metadata_path = "./metadata_spiritnet_11110.scale")]
pub mod kilt {}

#[cfg(feature = "spiritnet")]
pub type ProxyType = kilt::runtime_types::spiritnet_runtime::ProxyType;
#[cfg(feature = "spiritnet")]
pub type RuntimeCall = kilt::runtime_types::spiritnet_runtime::RuntimeCall;
#[cfg(feature = "spiritnet")]
pub type RuntimeEvent = kilt::runtime_types::spiritnet_runtime::RuntimeEvent;

#[cfg(not(feature = "spiritnet"))]
#[subxt::subxt(runtime_metadata_path = "metadata_peregrine_11110..scale")]
pub mod kilt {}

#[cfg(not(feature = "spiritnet"))]
pub type ProxyType = kilt::runtime_types::peregrine_runtime::ProxyType;

#[cfg(not(feature = "spiritnet"))]
pub type RuntimeCall = kilt::runtime_types::peregrine_runtime::RuntimeCall;

#[cfg(not(feature = "spiritnet"))]
pub type RuntimeEvent = kilt::runtime_types::peregrine_runtime::RuntimeEvent;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KiltConfig;
impl Config for KiltConfig {
    type Hash = sp_core::H256;
    type Hasher = <subxt::config::SubstrateConfig as Config>::Hasher;
    type AccountId = <<Self::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = sp_runtime::MultiAddress<Self::AccountId, ()>;
    type Header = subxt::config::substrate::SubstrateHeader<u64, Self::Hasher>;
    type Signature = sp_runtime::MultiSignature;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}

async fn get_ctype_extrinsic(
    block_content: &BlockBody<KiltConfig, OnlineClient<KiltConfig>>,
    id: sp_core::H256,
) -> Result<ExtrinsicDetails<KiltConfig, OnlineClient<KiltConfig>>, AppError> {
    let filtered_iterator = block_content
        .extrinsics()
        .iter()
        .filter(|details| details.is_ok())
        .map(|details| details.unwrap());

    for ext in filtered_iterator {
        if let Some(ctype_created_event) = extract_ctype_created_event(&ext).await? {
            let (_, ctype_id) = ctype_created_event;
            if ctype_id == id {
                return Ok(ext);
            }
        }
    }
    Err(AppError::Subxt(subxt::Error::Other(
        "Did not find extrinsic".to_string(),
    )))
}

async fn extract_ctype_created_event(
    ext: &ExtrinsicDetails<KiltConfig, OnlineClient<KiltConfig>>,
) -> Result<Option<(AccountId32, sp_core::H256)>, AppError> {
    for extrinsic_details in ext.events().await?.iter() {
        let details = extrinsic_details?.to_owned();
        let name = details.event_metadata().pallet.name();
        if name.to_ascii_lowercase() == "ctype" {
            let mut byte_values = details.field_bytes();
            return <(AccountId32, sp_core::H256) as Decode>::decode(&mut byte_values)
                .map_err(|_| {
                    AppError::Subxt(subxt::Error::Other(
                        "Failed to decode create ctype event.".to_string(),
                    ))
                })
                .map(Some);
        }
    }
    Ok(None)
}

pub async fn get_ctype_by_id(
    id: sp_core::H256,
    api: &OnlineClient<KiltConfig>,
) -> Result<(), AppError> {
    let ctype_id = kilt::storage().ctype().ctypes(&id);

    let ctype_entry = api
        .storage()
        .at_latest()
        .await?
        .fetch(&ctype_id)
        .await?
        .ok_or(AppError::Subxt(subxt::Error::Other(
            "Ctype does not exist".to_string(),
        )))?;

    let created_at = ctype_entry.created_at;

    let block_hash =
        api.rpc()
            .block_hash(Some(created_at.into()))
            .await?
            .ok_or(AppError::Subxt(subxt::Error::Other(
                "Ctype does not exist".to_string(),
            )))?;

    let block_content = api.blocks().at(block_hash).await?.body().await?;

    let ext_details = get_ctype_extrinsic(&block_content, id).await?;

    Ok(())
}

pub async fn create_claim(
    claim_hash: sp_core::H256,
    ctype_hash: sp_core::H256,
    did_address: &AccountId32,
    api: &OnlineClient<KiltConfig>,
    payer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
    signer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
) -> Result<Vec<u8>, AppError> {
    let tx_counter = get_next_tx_counter(&api, &did_address).await?;
    let block_number = get_current_block(&api).await?;

    let call = RuntimeCall::Attestation(runtime_types::attestation::pallet::Call::add {
        claim_hash,
        ctype_hash,
        authorization: None,
    });

    let did_call = DidAuthorizedCallOperation {
        did: did_address.to_owned(),
        tx_counter,
        block_number,
        call,
        submitter: payer.account_id().to_owned().into(),
    };

    let encoded_call = did_call.encode();

    let signature = calculate_signature(&did_call.encode(), signer)?;
    let final_tx = kilt::tx().did().submit_did_call(did_call, signature);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, payer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let created_event = events.find_first::<kilt::attestation::events::AttestationCreated>()?;

    match created_event {
        Some(_) => {
            log::info!("Attestation with root hash {:?} created", claim_hash);
            Ok(encoded_call)
        }
        _ => {
            log::info!(
                "Attestation with root hash {:?} could not be created. Create Event not found",
                claim_hash
            );
            Err(AppError::Subxt(subxt::Error::Other(
                "Created Event not found".to_string(),
            )))
        }
    }
}

pub async fn revoke_claim(
    claim_hash: sp_core::H256,
    did_address: &AccountId32,
    api: &OnlineClient<KiltConfig>,
    payer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
    signer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
) -> Result<Vec<u8>, AppError> {
    let tx_counter = get_next_tx_counter(&api, &did_address).await?;
    let block_number = get_current_block(&api).await?;

    let payer_account = payer.account_id();

    let did_call = DidAuthorizedCallOperation {
        did: did_address.to_owned(),
        tx_counter,
        block_number,
        call: RuntimeCall::Attestation(runtime_types::attestation::pallet::Call::revoke {
            claim_hash,
            authorization: None,
        }),
        submitter: payer_account.clone().into(),
    };

    let encoded_call = did_call.encode();

    let signature = calculate_signature(&did_call.encode(), &signer)?;
    let final_tx = kilt::tx().did().submit_did_call(did_call, signature);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, payer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let revoke_event = events.find_first::<kilt::attestation::events::AttestationRevoked>()?;

    match revoke_event {
        Some(_) => {
            log::info!("Attestation with root hash {:?} revoked", claim_hash);
            Ok(encoded_call)
        }
        _ => {
            log::info!(
                "Attestation with root hash {:?} could not be revoked. Revoke Event not found",
                claim_hash
            );
            Err(AppError::Subxt(subxt::Error::Other(
                "Created Event not found".to_string(),
            )))
        }
    }
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
