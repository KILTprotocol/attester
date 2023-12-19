use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, box_::Nonce};
use uuid::Uuid;

use super::{
    utils::{hex_nonce, prefixed_hex},
    Claim, Credential,
};

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
    pub claim: Claim,
    pub legitimations: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestAttestationMessageContent {
    pub credential: Credential,
    pub quote: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub encryption_key_uri: Option<String>,
    #[serde(alias = "challenge")]
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTerms {
    pub challenge: Uuid,
    pub attestation_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAttestationMessageBody {
    pub claim_hash: String,
    pub c_type_hash: String,
    pub owner: String,
    pub delegation_id: Option<String>,
    pub revoke: bool,
}
