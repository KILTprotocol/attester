use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::SecretKey;
use subxt::{
    ext::sp_core::{crypto::SecretStringError, sr25519::Pair, Pair as PairTrait},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};

use crate::tx::KiltConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub port: u16,
    pub kilt_endpoint: String,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub session_key: String,
    pub key_uri: String,
    pub nacl_public_key: String,
    pub nacl_secret_key: String,
    pub session_ttl: i64,
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
    pub fn get_credential_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_attestation_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub fn get_payer_signer(&self) -> Result<PairSigner<KiltConfig, Pair>, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.payer_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }

    pub async fn get_client(&self) -> Result<OnlineClient<KiltConfig>, subxt::Error> {
        Ok(OnlineClient::<KiltConfig>::from_url(&self.kilt_endpoint).await?)
    }

    pub fn get_did(&self) -> Result<AccountId32, SecretStringError> {
        let pair = Pair::from_string_with_seed(&self.attester_did_seed, None)?.0;
        Ok(pair.public().into())
    }

    pub fn get_nacl_secret_key(&self) -> SecretKey {
        let raw_key = hex::decode(self.session.nacl_secret_key.trim_start_matches("0x")).unwrap();
        SecretKey::from_slice(&raw_key)
            .ok_or(Err::<(), i32>(0))
            .unwrap()
    }
}
