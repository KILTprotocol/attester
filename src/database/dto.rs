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
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: serde_json::Value,
    claim_hashes: Vec<Hash>,
    delegation_id: Option<String>,
    legitimations: Vec<Credential>,
    pub root_hash: Hash,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "tx_states")]
pub enum TxState {
    InFlight,
    InBlock,
    Succeeded,
    Failed,
    Pending,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
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
    pub tx_state: Option<TxState>,
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
    pub filter: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Query {
    pub offset: Option<String>,
    pub sort: Option<String>,
    pub filter: Option<String>,
}

impl From<Query> for Pagination {
    fn from(value: Query) -> Self {
        Pagination {
            offset: value
                .offset
                .and_then(|offset| serde_json::from_str(&offset).ok()),

            sort: value.sort.and_then(|sort| serde_json::from_str(&sort).ok()),
            filter: value
                .filter
                .and_then(|filter| serde_json::from_str(&filter).ok()),
        }
    }
}

#[derive(Serialize)]
pub struct AttestationCreatedOverTime {
    pub date: Option<NaiveDateTime>,
    pub total_attestations_created: Option<i64>,
}

#[derive(Serialize)]
pub struct AttestationKPIs {
    pub attestations_created_over_time: Vec<AttestationCreatedOverTime>,
    pub attestations_not_approved: i64,
    pub attestations_revoked: i64,
    pub total_claimers: i64,
}
