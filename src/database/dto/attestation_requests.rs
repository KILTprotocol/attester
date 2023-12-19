use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: HashMap<String, String>,
    claim_hashes: Vec<String>,
    delegation_id: Option<String>,
    legitimations: Option<Vec<Credential>>,
    pub root_hash: String,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Debug)]
#[sqlx(type_name = "tx_states")]
pub enum TxState {
    Succeeded,
    Failed,
    Pending,
    InFlight,
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
