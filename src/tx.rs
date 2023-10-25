use subxt::ext::codec::Encode;
use subxt::ext::sp_runtime::traits::{IdentifyAccount, Verify};
use subxt::ext::{sp_core, sp_runtime};
use subxt::{config::polkadot::PolkadotExtrinsicParams, config::Config};

use crate::configuration::Configuration;
use crate::error::AppError;
use crate::utils::{calculate_signature, get_current_block, get_next_tx_counter};

use self::kilt::runtime_types;
use self::kilt::runtime_types::did::did_details::DidAuthorizedCallOperation;

#[cfg(spiritnet)]
#[subxt::subxt(runtime_metadata_path = "./metadata_spiritnet_11110.scale")]
pub mod kilt {}

#[cfg(spiritnet)]
pub type ProxyType = kilt::runtime_types::spiritnet_runtime::ProxyType;
#[cfg(spiritnet)]
pub type RuntimeCall = kilt::runtime_types::spiritnet_runtime::RuntimeCall;
#[cfg(spiritnet)]
pub type RuntimeEvent = kilt::runtime_types::spiritnet_runtime::RuntimeEvent;

#[cfg(not(spiritnet))]
#[subxt::subxt(runtime_metadata_path = "metadata_peregrine_11110..scale")]
pub mod kilt {}

#[cfg(not(spiritnet))]
pub type ProxyType = kilt::runtime_types::peregrine_runtime::ProxyType;
#[cfg(not(spiritnet))]
pub type RuntimeCall = kilt::runtime_types::peregrine_runtime::RuntimeCall;
#[cfg(not(spiritnet))]
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

pub async fn create_claim(
    claim_hash: sp_core::H256,
    ctype_hash: sp_core::H256,
    config: Configuration,
) -> Result<Vec<u8>, AppError> {
    let api = config.get_client().await?;
    let payer = config.get_payer_signer()?;
    let payer_account = payer.account_id();
    let tx_counter = get_next_tx_counter(&config).await?;
    let block_number = get_current_block(&config).await?;

    let call = RuntimeCall::Attestation(runtime_types::attestation::pallet::Call::add {
        claim_hash,
        ctype_hash,
        authorization: None,
    });

    let did_call = DidAuthorizedCallOperation {
        did: config.get_did()?,
        tx_counter,
        block_number,
        call,
        submitter: payer_account.clone().into(),
    };

    let encoded_call = did_call.encode();

    let signature = calculate_signature(&did_call.encode(), config)?;
    let final_tx = kilt::tx().did().submit_did_call(did_call, signature);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, &payer)
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
    config: Configuration,
) -> Result<Vec<u8>, AppError> {
    let tx_counter = get_next_tx_counter(&config).await?;
    let block_number = get_current_block(&config).await?;
    let payer = config.get_payer_signer()?;
    let payer_account = payer.account_id();
    let api = config.get_client().await?;

    let did_call = DidAuthorizedCallOperation {
        did: config.get_did()?,
        tx_counter,
        block_number,
        call: RuntimeCall::Attestation(runtime_types::attestation::pallet::Call::revoke {
            claim_hash,
            authorization: None,
        }),
        submitter: payer_account.clone().into(),
    };

    let encoded_call = did_call.encode();

    let signature = calculate_signature(&did_call.encode(), config)?;
    let final_tx = kilt::tx().did().submit_did_call(did_call, signature);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, &payer)
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
