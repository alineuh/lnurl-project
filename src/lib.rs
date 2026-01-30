use serde::{Deserialize, Serialize};
use cln_rpc::primitives::Sha256;

// ============================================================================
// LUD-02: Channel Request
// ============================================================================

pub const CHANNEL_REQUEST_TAG: &str = "channelRequest";

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelRequestResponse {
    pub tag: String,
    pub k1: String,
    pub callback: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenChannelRequest {
    pub k1: String,
    #[serde(rename = "remoteid")]
    pub remote_id: String,
    pub private: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenChannelResponse {
    pub status: String,
}

// Pour le callback interne (pas dans la spec)
#[derive(Serialize, Deserialize, Debug)]
pub struct InternalOpenChannelRequest {
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub satoshis: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalOpenChannelResponse {
    pub mindepth: Option<u32>,
    pub channel_id: Sha256,
    pub outnum: u32,
    pub tx: String,
    pub txid: String,
}

// ============================================================================
// LUD-03: Withdraw Request
// ============================================================================

pub const WITHDRAW_REQUEST_TAG: &str = "withdrawRequest";

#[derive(Serialize, Deserialize, Debug)]
pub struct WithdrawRequestResponse {
    pub tag: String,
    pub callback: String,
    pub k1: String,
    #[serde(rename = "defaultDescription")]
    pub default_description: String,
    #[serde(rename = "minWithdrawable")]
    pub min_withdrawable: u64, // in millisatoshis
    #[serde(rename = "maxWithdrawable")]
    pub max_withdrawable: u64, // in millisatoshis
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WithdrawRequest {
    pub k1: String,
    pub pr: String, // BOLT11 invoice
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WithdrawResponse {
    pub status: String,
}

// ============================================================================
// LUD-04: LNURL-auth
// ============================================================================

pub const AUTH_TAG: &str = "login";

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthChallengeResponse {
    pub tag: String,
    pub k1: String,
    pub action: Option<String>, // "register" | "login" | "link" | "auth"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    pub k1: String,
    pub sig: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub status: String,
    pub event: Option<String>, // "REGISTERED" | "LOGGEDIN" | "LINKED" | "AUTHED"
}
