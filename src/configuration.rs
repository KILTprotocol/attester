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
/// Represents the configuration settings required for running the attestation service.
pub struct Configuration {
    /// Seed for generating the attester's DID (Decentralized Identifier).
    #[clap(env)]
    attester_did_seed: String,

    /// Seed for the payer, responsible for transaction fees.
    #[clap(env)]
    payer_seed: String,

    /// Seed for generating the attester's attestation key.
    #[clap(env)]
    attester_attestation_seed: String,

    /// Websocket address of the blockchain network.
    #[clap(env)]
    pub wss_address: String,

    /// Hostname for serving the attestation service.
    #[clap(env)]
    pub host_name: String,

    /// URL for the PostgreSQL database.
    #[clap(env)]
    pub database_url: String,

    /// Port for serving the attestation service.
    #[clap(env)]
    pub port: u16,

    /// Secret key for JWT (JSON Web Tokens) generation and validation.
    #[clap(env)]
    pub jwt_secret: String,

    /// Path to the front-end web application.
    #[clap(env)]
    pub front_end_path: String,
}

impl Configuration {
    /// Get the credential signer based on the attester's attestation key seed.
    /// Returns a `Result` containing the `PairSigner` or an error if the seed is invalid.
    pub fn get_credential_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_attestation_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    /// Get the payer signer based on the payer's seed.
    /// Returns a `Result` containing the `PairSigner` or an error if the seed is invalid.
    pub fn get_payer_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.payer_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    /// Get an online client for interacting with the blockchain network based on the provided WebSocket address.
    /// Returns a `Result` containing the `OnlineClient` or an error if the connection fails.
    pub async fn get_client(&self) -> Result<OnlineClient<KiltConfig>, subxt::Error> {
        Ok(OnlineClient::<KiltConfig>::from_url(&self.wss_address).await?)
    }

    /// Get the DID (Decentralized Identifier) for the attester based on the attester's DID seed.
    /// Returns a `Result` containing the `AccountId32` representing the DID or an error if the seed is invalid.
    pub fn get_did(&self) -> Result<AccountId32, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_did_seed, None)?.0;
        Ok(pair.public().into())
    }
}
