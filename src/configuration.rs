use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::SecretKey;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as PairTrait},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};

use crate::kilt::KiltConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub port: u16,
    pub endpoint: String,
    pub session: SessionConfig,
    #[serde(rename = "wellKnownDid")]
    pub well_known_did_config: WellKnownDidConfig,
    attester_did_seed: String,
    attester_attestation_seed: String,
    pub database_url: String,
    pub front_end_path: String,
    pub jwt_secret: String,
    pub payer_seed: String,
    pub app_name: String,
    pub auth_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub session_key: String,
    pub key_uri: String,
    pub nacl_public_key: String,
    pub nacl_secret_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WellKnownDidConfig {
    pub did: String,
    pub key_uri: String,
    pub origin: String,
    pub seed: String,
}

impl Configuration {
    pub fn get_credential_signer(&self) -> anyhow::Result<PairSigner<KiltConfig, Pair>> {
        let pair = Pair::from_string_with_seed(&self.attester_attestation_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub fn get_payer_signer(&self) -> anyhow::Result<PairSigner<KiltConfig, Pair>> {
        let pair = Pair::from_string_with_seed(&self.payer_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub async fn get_client(&self) -> anyhow::Result<OnlineClient<KiltConfig>> {
        Ok(OnlineClient::<KiltConfig>::from_url(&self.endpoint).await?)
    }

    pub fn get_did(&self) -> anyhow::Result<AccountId32> {
        let pair = Pair::from_string_with_seed(&self.attester_did_seed, None)?.0;
        Ok(pair.public().into())
    }

    pub fn get_nacl_secret_key(&self) -> anyhow::Result<SecretKey> {
        let raw_key = hex::decode(self.session.nacl_secret_key.trim_start_matches("0x"))?;
        SecretKey::from_slice(&raw_key).ok_or(anyhow::anyhow!("Generating secret key failed"))
    }
}
