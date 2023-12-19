use parity_scale_codec::Encode;
use subxt::{ext::sp_core, tx::PairSigner, utils::AccountId32, OnlineClient};

use crate::kilt::{
    runtime,
    utils::{calculate_signature, get_current_block, get_next_tx_counter},
    KiltConfig, RuntimeCall,
};

use runtime::runtime_types;
use runtime::runtime_types::did::did_details::DidAuthorizedCallOperation;

pub async fn create_claim(
    claim_hash: sp_core::H256,
    ctype_hash: sp_core::H256,
    did_address: &AccountId32,
    chain_client: &OnlineClient<KiltConfig>,
    payer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
    signer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
) -> Result<Vec<u8>, subxt::Error> {
    let tx_counter = get_next_tx_counter(&chain_client, &did_address).await?;
    let block_number = get_current_block(&chain_client).await?;

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

    let signature = calculate_signature(&did_call.encode(), signer);
    let final_tx = runtime::tx().did().submit_did_call(did_call, signature);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, payer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let created_event = events.find_first::<runtime::attestation::events::AttestationCreated>()?;

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
            Err(subxt::Error::Other("Created Event not found".to_string()))
        }
    }
}

pub async fn revoke_claim(
    claim_hash: sp_core::H256,
    did_address: &AccountId32,
    chain_client: &OnlineClient<KiltConfig>,
    payer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
    signer: &PairSigner<KiltConfig, sp_core::sr25519::Pair>,
) -> Result<Vec<u8>, subxt::Error> {
    let tx_counter = get_next_tx_counter(&chain_client, &did_address).await?;
    let block_number = get_current_block(&chain_client).await?;

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

    let signature = calculate_signature(&did_call.encode(), &signer);
    let final_tx = runtime::tx().did().submit_did_call(did_call, signature);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, payer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let revoke_event = events.find_first::<runtime::attestation::events::AttestationRevoked>()?;

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
            Err(subxt::Error::Other("Created Event not found".to_string()))
        }
    }
}
