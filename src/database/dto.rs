use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, box_::Nonce};
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use uuid::Uuid;

use crate::utils::{hex_nonce, prefixed_hex};

#[derive(Serialize, Deserialize, FromRow, Clone, PartialEq, Debug)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(Serialize, Deserialize, FromRow, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: serde_json::Value,
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

#[derive(serde::Deserialize, Clone)]
pub struct Pagination {
    pub offset: Option<[u32; 2]>,
    pub sort: Option<[String; 2]>,
    pub filter: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Query {
    pub range: Option<String>,
    pub sort: Option<String>,
    pub filter: Option<String>,
}

impl From<Query> for Pagination {
    fn from(value: Query) -> Self {
        Pagination {
            offset: value
                .range
                .and_then(|offset| serde_json::from_str::<[u32; 2]>(&offset).ok()),

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeData {
    #[serde(rename = "dAppName")]
    pub app_name: String,
    #[serde(rename = "dAppEncryptionKeyUri")]
    pub encryption_key_uri: String,
    pub challenge: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeResponse {
    pub encryption_key_uri: String,
    #[serde(with = "prefixed_hex")]
    pub encrypted_challenge: Vec<u8>,
    #[serde(with = "hex_nonce")]
    pub nonce: box_::Nonce,
    pub attestation_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageBody<T> {
    #[serde(rename = "type")]
    pub type_: String,
    pub content: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub body: MessageBody<T>,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub sender: String,
    pub receiver: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    pub references: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    #[serde(rename = "ciphertext")]
    #[serde(with = "prefixed_hex")]
    pub cipher_text: Vec<u8>,
    #[serde(with = "hex_nonce")]
    pub nonce: Nonce,
    #[serde(rename = "receiverKeyUri")]
    pub receiver_key_uri: String,
    #[serde(rename = "senderKeyUri")]
    pub sender_key_uri: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitTermsMessageContent {
    #[serde(rename = "cTypes")]
    pub c_types: String,
    pub claim: Claim,
    pub quote: Option<String>,
    #[serde(rename = "delegationId")]
    pub delegation_id: Option<String>,
    pub legitimations: Option<String>,
}
