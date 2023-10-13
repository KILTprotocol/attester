use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use uuid::Uuid;

pub type Hash = String;

#[derive(Serialize, Deserialize)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(Serialize, Deserialize)]
pub struct Credential {
    pub claim: Claim,
    #[serde(rename = "claimNonceMap")]
    claim_nonce_map: serde_json::Value,
    #[serde(rename = "claimHashes")]
    claim_hashes: Vec<Hash>,
    #[serde(rename = "delegationId")]
    delegation_id: Option<String>,
    legitimations: Vec<Credential>,
    #[serde(rename = "rootHash")]
    pub root_hash: Hash,
}

#[derive(Serialize, Deserialize)]
pub enum TxState {
    InFlight,
    InBlock,
    Succeded,
    Failed,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct AttestationResponse {
    pub id: Uuid,
    pub approved: bool,
    pub revoked: bool,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub approved_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub ctype_hash: String,
    pub credential: serde_json::Value,
    pub claimer: String,
    pub tx_state: TxState,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AttestationRequest {
    pub ctype_hash: String,
    pub claim: serde_json::Value,
    pub claimer: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateAttestation {
    pub claim: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub offset: Option<[u32; 2]>,
    pub sort: Option<[String; 2]>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Query {
    pub offset: Option<String>,
    pub sort: Option<String>,
}

impl From<Query> for Pagination {
    fn from(value: Query) -> Self {
        Pagination {
            offset: match &value.offset {
                Some(offset) => serde_json::from_str(offset).ok(),
                _ => None,
            },
            sort: match &value.sort {
                Some(sort) => serde_json::from_str(sort).ok(),
                _ => None,
            },
        }
    }
}
