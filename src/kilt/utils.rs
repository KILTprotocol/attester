use subxt::{
    ext::sp_core::{sr25519, Pair},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};

use crate::kilt::{
    runtime::{self, runtime_types::did::did_details::DidSignature},
    KiltConfig,
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
) -> Result<u64, subxt::Error> {
    let did_doc_addr = runtime::storage().did().did(did_address);
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
) -> DidSignature {
    let signed_data = signer.signer().sign(call);
    DidSignature::Sr25519(runtime::runtime_types::sp_core::sr25519::Signature(
        signed_data.into(),
    ))
}
