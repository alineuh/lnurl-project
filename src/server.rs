use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use cln_rpc::{
    ClnRpc,
    primitives::{Amount, AmountOrAll, PublicKey},
    model::{responses as cresp, requests as creq},
};

use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use server_lightning::RequestChannelResponse;
use server_lightning::OpenChannelRequest;
use server_lightning::OpenChannelResponse;
use server_lightning::WithdrawRequestChannelResponse;


/* static TPCCLIENT: OnceLock<Arc<Mutex<ClnRpc>>> = OnceLock::new();
 */
const REQUESTCHANELTAG: &str = "request-channel";
const PUBLIC_KEY: &str = "021f9b61b38536de1476d37ae75f037717b3aa4223081c2ee9eda51edd14147c16";
const IP_PORT: &str = "192.168.27.67:9735";
const CALLBACK_IP: &str = "192.168.27.67:3000";

const CALLBACK_IP2: &str = "192.168.27.67:3001";

#[derive(Clone)]
struct AppState {
    client_rpc: Arc<Mutex<ClnRpc>>,
}


// --------------------  HANDLERS --------------------
async fn channel_request() -> (StatusCode, Json<RequestChannelResponse>) {
    let response = RequestChannelResponse {
        uri: format!("{}@{}", PUBLIC_KEY, IP_PORT),
        callback: format!("{}", CALLBACK_IP),
        k1: "help".to_string(),
        tag: REQUESTCHANELTAG.to_string(),
    };

    (StatusCode::OK, Json(response))
}

// Open channel from remote node to local one
// Communicate the remote server side to lightning network server side
async fn open_channel(
    State(state): State<AppState>,
    Json(body): Json<OpenChannelRequest>,
) -> Result<Json<OpenChannelResponse>, StatusCode> {
    let _remote_uri = format!("{}@{}:{}", body.node_id, body.host, body.port);

    // Parse node_id safely into a PublicKey
    let node_id_pub_key =
        PublicKey::from_str(&body.node_id).map_err(|_| StatusCode::BAD_REQUEST)?; // 400 if invalid pubkey

    let req = creq::FundchannelRequest {
        id: node_id_pub_key,
        amount: AmountOrAll::Amount(Amount::from_sat(body.satoshis)),
        feerate: None,
        announce: None, // default to true if None
        channel_type: None,
        minconf: None,
        utxos: None,
        push_msat: None,
        close_to: None,
        request_amt: None,
        compact_lease: None,
        reserve: None,
        mindepth: None,
    };

    let resp: cresp::FundchannelResponse = {
        // lock the mutex and map a poisoned lock to a 500 error
        let mut guard = state.client_rpc.lock().await;

        guard
            .call_typed(&req)
            .await
            .map_err(|_e| StatusCode::BAD_GATEWAY)? // same here
    };

    // Map CLN response -> your API response
    let api_resp = OpenChannelResponse {
        mindepth: resp.mindepth,
        channel_id: resp.channel_id,
        outnum: resp.outnum,
        tx: resp.tx,
        txid: resp.txid,
    };

    Ok(Json(api_resp))
}

/* async fn withdraw_request() -> (StatusCode, Json<WithdrawRequestChannelResponse>) {
    let crr = WithdrawRequestChannelResponse {
        callback: format!("{}", CALLBACK_IP2),
        k1: "idk2".to_string(),
        tag: REQUESTCHANELTAG.to_string(),
        default_description: "Default withdraw configuration".to_string(),
        min_withdrawable: 1000,
        max_withdrawable: 1000000,
    };

    (StatusCode::OK, Json(crr))
}
 */
/* async fn withdraw() -> () {
    todo!("Complete")
} */
// --------------------  MAIN --------------------
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let home = std::env::var("HOME").expect("HOME env var not set");
    let rpc_path = format!("{home}/.lightning/testnet4/lightning-rpc");

    let client = ClnRpc::new(&rpc_path).await;
    if let Err(e) = &client {
        eprintln!("ERROR DEFINING THE CLIENT: {e}");
        std::process::exit(1);
    }

    /*     let shared_state = Arc::new(AppState {
        client_rpc: client.unwrap()
    }); */

    let shared_state = AppState {
        client_rpc: Arc::new(Mutex::new(client.unwrap())), // Todo: handle the result
    };

    // build our application with a route
    let app = Router::new()
        .route("/channel_request", get(channel_request)) // could be run in different place than open-channel
        .route("/open_channel", post(open_channel)) // Return a status code instead
        .with_state(shared_state);
        /* .route("/withdraw_request", get(withdraw_request))
        .route("/withdraw", get(withdraw)); */

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
