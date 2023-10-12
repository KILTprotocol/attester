use serde::Deserialize;
use subxt::ext::sp_core::crypto::SecretStringError;
use subxt::ext::sp_core::sr25519::Pair;
use subxt::ext::sp_core::Pair as PairTrait;
use subxt::tx::PairSigner;
use subxt::utils::AccountId32;
use subxt::OnlineClient;

use crate::tx::KiltConfig;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    // Seed for attester
    pub attester_did_seed: String,
    // Seed for payer
    pub payer_seed: String,
    // Seed for attestation key
    attester_attestation_seed: String,
    // Websocket address of network
    pub wss_address: String,
    pub host_name: String,
    pub database_url: String,
}

impl Configuration {
    pub fn get_credential_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_attestation_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub fn get_payer_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.payer_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub async fn get_client(&self) -> Result<OnlineClient<KiltConfig>, subxt::Error> {
        Ok(OnlineClient::<KiltConfig>::from_url(&self.wss_address).await?)
    }

    pub fn get_did(&self) -> Result<AccountId32, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_did_seed, None)?.0;
        Ok(pair.public().into())
    }
}

pub fn init() -> Configuration {
    match envy::from_env::<Configuration>() {
        Ok(config) => config,
        Err(error) => panic!("{:#?}", error),
    }
}
