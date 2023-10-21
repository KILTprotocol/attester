use clap::Parser;
use serde::Deserialize;
use subxt::{
    ext::sp_core::{crypto::SecretStringError, sr25519::Pair, Pair as PairTrait},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};

use crate::tx::KiltConfig;

#[derive(Deserialize, Debug, Clone, Parser)]
pub struct Configuration {
    // Seed for attester
    #[clap(env)]
    attester_did_seed: String,
    // Seed for payer
    #[clap(env)]
    payer_seed: String,
    // Seed for attestation key
    #[clap(env)]
    attester_attestation_seed: String,
    // Websocket address of network
    #[clap(env)]
    wss_address: String,
    #[clap(env)]
    pub host_name: String,
    #[clap(env)]
    pub database_url: String,
    #[clap(env)]
    pub port: u16,
    #[clap(env)]
    pub jwt_secret: String,
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
