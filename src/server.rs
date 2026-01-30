use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use cln_rpc::{
    model::{requests as creq, responses as cresp},
    primitives::{Amount, AmountOrAll, PublicKey},
    ClnRpc,
};
use rand::Rng;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::info;

use lnurl_project::*;

const PUBLIC_KEY: &str = "029249978ef61cf264d2cf57589c96780bdd86266fdc065d6b54c48d2c9ea3ad40";
const IP_PORT: &str = "89.87.30.156:9735"; 
const SERVER_URL: &str = "http://89.87.30.156:3000"; //server URL



#[derive(Clone)]
struct AppState {
    client_rpc: Arc<Mutex<ClnRpc>>,
    k1_cache: Arc<Mutex<HashMap<String, K1Data>>>,
}

#[derive(Clone, Debug)]
struct K1Data {
    challenge: String,
    used: bool,
}


fn generate_k1() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    hex::encode(random_bytes)
}


/// GET /channel-request
/// Retourne les infos pour qu'un client puisse demander l'ouverture d'un channel
async fn channel_request(State(state): State<AppState>) -> (StatusCode, Json<ChannelRequestResponse>) {
    let k1 = generate_k1();
    
    // Store k1 in cache
    {
        let mut cache = state.k1_cache.lock().await;
        cache.insert(k1.clone(), K1Data {
            challenge: k1.clone(),
            used: false,
        });
    }

    let response = ChannelRequestResponse {
        tag: CHANNEL_REQUEST_TAG.to_string(),
        k1: k1.clone(),
        callback: format!("{}/channel-callback", SERVER_URL),
        uri: format!("{}@{}", PUBLIC_KEY, IP_PORT),
    };

    info!("Channel request generated with k1: {}", k1);
    (StatusCode::OK, Json(response))
}

async fn channel_callback(
    State(state): State<AppState>,
    Query(params): Query<OpenChannelRequest>,
) -> Result<Json<OpenChannelResponse>, StatusCode> {
    info!("Channel callback received: k1={}, remoteid={}", params.k1, params.remote_id);

    // Verify k1
    {
        let mut cache = state.k1_cache.lock().await;
        match cache.get_mut(&params.k1) {
            Some(data) if !data.used => {
                data.used = true;
            }
            Some(_) => {
                info!("k1 already used");
                return Err(StatusCode::BAD_REQUEST);
            }
            None => {
                info!("k1 not found in cache");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Parse node_id
    let node_id = PublicKey::from_str(&params.remote_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Open channel via Core Lightning
    let req = creq::FundchannelRequest {
        id: node_id,
        amount: AmountOrAll::Amount(Amount::from_sat(100_000)), // 100k sats par défaut
        feerate: None,
        announce: Some(params.private != "1"), // private=1 means private channel
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

    let _resp: cresp::FundchannelResponse = {
        let mut guard = state.client_rpc.lock().await;
        guard
            .call_typed(&req)
            .await
            .map_err(|e| {
                info!("Failed to fund channel: {:?}", e);
                StatusCode::BAD_GATEWAY
            })?
    };

    info!("Channel opened successfully!");
    Ok(Json(OpenChannelResponse {
        status: "OK".to_string(),
    }))
}

/// GET /withdraw-request
/// Retourne les infos pour qu'un client puisse demander un withdraw
async fn withdraw_request(State(state): State<AppState>) -> (StatusCode, Json<WithdrawRequestResponse>) {
    let k1 = generate_k1();
    
    // Store k1 in cache
    {
        let mut cache = state.k1_cache.lock().await;
        cache.insert(k1.clone(), K1Data {
            challenge: k1.clone(),
            used: false,
        });
    }

    let response = WithdrawRequestResponse {
        tag: WITHDRAW_REQUEST_TAG.to_string(),
        callback: format!("{}/withdraw-callback", SERVER_URL),
        k1: k1.clone(),
        default_description: "LNURL withdraw".to_string(),
        min_withdrawable: 1_000, // 1 sat minimum (in millisats)
        max_withdrawable: 1_000_000, // 1000 sats maximum (in millisats)
    };

    info!("Withdraw request generated with k1: {}", k1);
    (StatusCode::OK, Json(response))
}

/// GET /withdraw-callback?k1=...&pr=...
/// Callback appelé par le client pour effectivement effectuer le withdraw
async fn withdraw_callback(
    State(state): State<AppState>,
    Query(params): Query<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, StatusCode> {
    info!("Withdraw callback received: k1={}", params.k1);

    // Verify k1
    {
        let mut cache = state.k1_cache.lock().await;
        match cache.get_mut(&params.k1) {
            Some(data) if !data.used => {
                data.used = true;
            }
            Some(_) => {
                info!("k1 already used");
                return Err(StatusCode::BAD_REQUEST);
            }
            None => {
                info!("k1 not found in cache");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Pay the invoice via Core Lightning
    let req = creq::PayRequest {
        bolt11: params.pr.clone(),
        amount_msat: None,
        label: None,
        riskfactor: None,
        maxfeepercent: None,
        retry_for: None,
        maxdelay: None,
        exemptfee: None,
        localinvreqid: None,
        exclude: None,
        maxfee: None,
        description: None,
        partial_msat: None,
    };

    let _resp: cresp::PayResponse = {
        let mut guard = state.client_rpc.lock().await;
        guard
            .call_typed(&req)
            .await
            .map_err(|e| {
                info!("Failed to pay invoice: {:?}", e);
                StatusCode::BAD_GATEWAY
            })?
    };

    info!("Withdraw successful!");
    Ok(Json(WithdrawResponse {
        status: "OK".to_string(),
    }))
}

// ============================================================================
// LUD-04: LNURL-auth Handlers
// ============================================================================

/// GET /auth-challenge
/// Retourne un challenge k1 pour l'authentification
async fn auth_challenge(State(state): State<AppState>) -> (StatusCode, Json<AuthChallengeResponse>) {
    let k1 = generate_k1();
    
    // Store k1 in cache
    {
        let mut cache = state.k1_cache.lock().await;
        cache.insert(k1.clone(), K1Data {
            challenge: k1.clone(),
            used: false,
        });
    }

    let response = AuthChallengeResponse {
        tag: AUTH_TAG.to_string(),
        k1: k1.clone(),
        action: Some("login".to_string()),
    };

    info!("Auth challenge generated with k1: {}", k1);
    (StatusCode::OK, Json(response))
}

/// GET /auth-response?k1=...&sig=...&key=...
/// Vérifie la signature et authentifie l'utilisateur
async fn auth_response(
    State(state): State<AppState>,
    Query(params): Query<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Auth response received: k1={}, key={}", params.k1, params.key);

    // Verify k1 exists and is not used
    {
        let mut cache = state.k1_cache.lock().await;
        match cache.get_mut(&params.k1) {
            Some(data) if !data.used => {
                data.used = true;
            }
            Some(_) => {
                info!("k1 already used");
                return Err(StatusCode::BAD_REQUEST);
            }
            None => {
                info!("k1 not found in cache");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Parse the public key
    let pubkey = PublicKey::from_str(&params.key)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify signature using Core Lightning's checkmessage
    let req = creq::CheckmessageRequest {
        message: params.k1.clone(),
        zbase: params.sig.clone(),
        pubkey: Some(pubkey),
    };

    let resp: cresp::CheckmessageResponse = {
        let mut guard = state.client_rpc.lock().await;
        guard
            .call_typed(&req)
            .await
            .map_err(|e| {
                info!("Failed to verify signature: {:?}", e);
                StatusCode::UNAUTHORIZED
            })?
    };

    if !resp.verified {
        info!("Signature verification failed");
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("Auth successful for key: {}", params.key);
    
    Ok(Json(AuthResponse {
        status: "OK".to_string(),
        event: Some("LOGGEDIN".to_string()),
    }))
}
 

//main 

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to Core Lightning
    let home = std::env::var("HOME").expect("HOME env var not set");
    let rpc_path = format!("{home}/.lightning/testnet4/lightning-rpc");

    let client = ClnRpc::new(&rpc_path).await;
    if let Err(e) = &client {
        eprintln!("ERROR connecting to Core Lightning: {e}");
        eprintln!("Make sure lightningd is running on testnet4!");
        std::process::exit(1);
    }

    let shared_state = AppState {
        client_rpc: Arc::new(Mutex::new(client.unwrap())),
        k1_cache: Arc::new(Mutex::new(HashMap::new())),
    };

    // Build router
    let app = Router::new()
        // LUD-02: Channel Request
        .route("/channel-request", get(channel_request))
        .route("/channel-callback", get(channel_callback))
        // LUD-03: Withdraw Request
        .route("/withdraw-request", get(withdraw_request))
        .route("/withdraw-callback", get(withdraw_callback))
        // LUD-04: LNURL-auth
        .route("/auth-challenge", get(auth_challenge))
        .route("/auth-response", get(auth_response))
        .with_state(shared_state);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    info!("🚀 Server running on http://0.0.0.0:3000");
    info!("📡 Endpoints:");
    info!("  - GET  /channel-request");
    info!("  - GET  /channel-callback");
    info!("  - GET  /withdraw-request");
    info!("  - GET  /withdraw-callback");
    info!("  - GET  /auth-challenge");
    info!("  - GET  /auth-response");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
