use serde::{Serialize, Deserialize};

use cln_rpc::{
    primitives::{ Sha256 },
};

pub const REQUEST_CHANNEL_TAG: &str = "request-channel";

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestChannelResponse {
    pub uri: String,
    pub callback: String,
    pub k1: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenChannelResponse {
    pub mindepth: Option<u32>,
    pub channel_id: Sha256,
    pub outnum: u32,
    pub tx: String,
    pub txid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenChannelRequest {
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub satoshis: u64,
    pub k1: String,
}

#[derive(Serialize, Deserialize)]
pub struct WithdrawRequestChannelResponse {
    callback: String,
    k1: String,
    tag: String,
    default_description: String,
    min_withdrawable: u64,
    max_withdrawable: u64,
}
