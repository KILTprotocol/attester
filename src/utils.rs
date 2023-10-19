use subxt::ext::sp_core::{crypto::SecretStringError, Pair};

use crate::{
    configuration::Configuration,
    error::AppError,
    tx::kilt::{self, runtime_types::did::did_details::DidSignature},
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
