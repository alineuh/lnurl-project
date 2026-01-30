use cln_rpc::{
    model::{requests as creq, responses as cresp},
    ClnRpc,
};
use lnurl_project::*;
use reqwest::Client;
use std::error::Error;

const SERVER_URL: &str = "http://127.0.0.1:3000"; // localhost pour tests// URL de ton serveur local pour test
// const SERVER_URL: &str = "http://IP_DU_PROF:3000"; // Quand tu testes avec le serveur du prof


fn parse_uri(uri: &str) -> Result<(String, String, u16), Box<dyn Error>> {
    // format "pubkey@ip:port"
    let parts: Vec<_> = uri.split('@').collect();
    if parts.len() != 2 {
        return Err("invalid uri format".into());
    }

    let node_id = parts[0].to_string();
    let socket = parts[1];

    let addr_parts: Vec<_> = socket.split(':').collect();
    if addr_parts.len() != 2 {
        return Err("invalid address format".into());
    }

    let host = addr_parts[0].to_string();
    let port: u16 = addr_parts[1].parse()?;

    Ok((node_id, host, port))
}

// ============================================================================
// LUD-02: Channel Request
// ============================================================================

async fn test_channel_request(client: &Client) -> Result<(), Box<dyn Error>> {
    println!("\n🔷 Testing LUD-02: Channel Request");
    println!("=====================================");

    // 1. Get channel request info
    println!("📡 Calling /channel-request ...");
    let req: ChannelRequestResponse = client
        .get(&format!("{}/channel-request", SERVER_URL))
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Received channel request:");
    println!("   tag: {}", req.tag);
    println!("   k1: {}", req.k1);
    println!("   callback: {}", req.callback);
    println!("   uri: {}", req.uri);

    // 2. Parse URI
    let (_node_id, _host, _port) = parse_uri(&req.uri)?;

    // 3. Get our node ID (from Core Lightning)
    let home = std::env::var("HOME").expect("HOME env var not set");
    let rpc_path = format!("{home}/.lightning/testnet4/lightning-rpc");
    let mut cln = ClnRpc::new(&rpc_path).await?;

    let getinfo_req = creq::GetinfoRequest {};
    let info: cresp::GetinfoResponse = cln.call_typed(&getinfo_req).await?;
    let our_pubkey = hex::encode(info.id.serialize());

    println!("📍 Our node pubkey: {}", our_pubkey);

    // 4. Call the callback to open channel
    println!("📡 Calling callback to open channel...");
    let callback_url = format!(
        "{}?k1={}&remoteid={}&private=0",
        req.callback, req.k1, our_pubkey
    );

    let resp: OpenChannelResponse = client
        .get(&callback_url)
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Channel request response: {}", resp.status);
    println!("🎉 Channel request test completed!\n");

    Ok(())
}

// ============================================================================
// LUD-03: Withdraw Request
// ============================================================================

async fn test_withdraw_request(client: &Client) -> Result<(), Box<dyn Error>> {
    println!("\n💰 Testing LUD-03: Withdraw Request");
    println!("====================================");

    // 1. Get withdraw request info
    println!("📡 Calling /withdraw-request ...");
    let req: WithdrawRequestResponse = client
        .get(&format!("{}/withdraw-request", SERVER_URL))
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Received withdraw request:");
    println!("   tag: {}", req.tag);
    println!("   k1: {}", req.k1);
    println!("   callback: {}", req.callback);
    println!("   min: {} msats", req.min_withdrawable);
    println!("   max: {} msats", req.max_withdrawable);

    // 2. Create an invoice (using Core Lightning)
    let home = std::env::var("HOME").expect("HOME env var not set");
    let rpc_path = format!("{home}/.lightning/testnet4/lightning-rpc");
    let mut cln = ClnRpc::new(&rpc_path).await?;

    let amount_msats = 50_000; // 50 sats
    let invoice_req = creq::InvoiceRequest {
        amount_msat: cln_rpc::primitives::AmountOrAny::Amount(
            cln_rpc::primitives::Amount::from_msat(amount_msats)
        ),
        description: "LNURL withdraw test".to_string(),
        label: format!("lnurl-withdraw-{}", chrono::Utc::now().timestamp()),
        expiry: Some(3600),
        fallbacks: None,
        preimage: None,
        cltv: None,
        deschashonly: None,
        exposeprivatechannels: None,
    };

    let invoice_resp: cresp::InvoiceResponse = cln.call_typed(&invoice_req).await?;
    let bolt11 = invoice_resp.bolt11;

    println!("📄 Generated invoice: {}...", &bolt11[..50]);

    // 3. Call the callback with our invoice
    println!("📡 Calling callback to withdraw...");
    let callback_url = format!("{}?k1={}&pr={}", req.callback, req.k1, bolt11);

    let resp: WithdrawResponse = client
        .get(&callback_url)
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Withdraw response: {}", resp.status);
    println!("🎉 Withdraw request test completed!\n");

    Ok(())
}

// ============================================================================
// LUD-04: LNURL-auth
// ============================================================================

async fn test_lnurl_auth(client: &Client) -> Result<(), Box<dyn Error>> {
    println!("\n🔐 Testing LUD-04: LNURL-auth");
    println!("==============================");

    // 1. Get auth challenge
    println!("📡 Calling /auth-challenge ...");
    let challenge: AuthChallengeResponse = client
        .get(&format!("{}/auth-challenge", SERVER_URL))
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Received auth challenge:");
    println!("   tag: {}", challenge.tag);
    println!("   k1: {}", challenge.k1);
    println!("   action: {:?}", challenge.action);

    // 2. Sign the challenge using Core Lightning
    let home = std::env::var("HOME").expect("HOME env var not set");
    let rpc_path = format!("{home}/.lightning/testnet4/lightning-rpc");
    let mut cln = ClnRpc::new(&rpc_path).await?;

    // Sign with signmessage
    let sign_req = creq::SignmessageRequest {
        message: challenge.k1.clone(),
    };

    let sign_resp: cresp::SignmessageResponse = cln.call_typed(&sign_req).await?;
    let signature = sign_resp.zbase;

    // Get our node pubkey
    let getinfo_req = creq::GetinfoRequest {};
    let info: cresp::GetinfoResponse = cln.call_typed(&getinfo_req).await?;
    let pubkey = hex::encode(info.id.serialize());

    println!("📝 Signature: {}...", &signature[..50]);
    println!("🔑 Pubkey: {}", pubkey);

    // 3. Call auth-response with signature
    println!("📡 Calling /auth-response ...");
    let auth_url = format!(
        "{}/auth-response?k1={}&sig={}&key={}",
        SERVER_URL, challenge.k1, signature, pubkey
    );

    let resp: AuthResponse = client
        .get(&auth_url)
        .send()
        .await?
        .json()
        .await?;

    println!("✅ Auth response: {}", resp.status);
    println!("   event: {:?}", resp.event);
    println!("🎉 LNURL-auth test completed!\n");

    Ok(())
}

// ============================================================================
// MAIN - Menu interactif
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    println!("\n⚡ LNURL Client - Test Suite");
    println!("============================\n");
    println!("What would you like to test?");
    println!("1. LUD-02: Channel Request");
    println!("2. LUD-03: Withdraw Request");
    println!("3. LUD-04: LNURL-auth");
    println!("4. Run all tests");
    println!("0. Exit");

    loop {
        print!("\nEnter your choice (0-4): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                if let Err(e) = test_channel_request(&client).await {
                    eprintln!("❌ Error: {}", e);
                }
            }
            "2" => {
                if let Err(e) = test_withdraw_request(&client).await {
                    eprintln!("❌ Error: {}", e);
                }
            }
            "3" => {
                if let Err(e) = test_lnurl_auth(&client).await {
                    eprintln!("❌ Error: {}", e);
                }
            }
            "4" => {
                println!("\n🚀 Running all tests...\n");
                if let Err(e) = test_channel_request(&client).await {
                    eprintln!("❌ Channel request error: {}", e);
                }
                if let Err(e) = test_withdraw_request(&client).await {
                    eprintln!("❌ Withdraw request error: {}", e);
                }
                if let Err(e) = test_lnurl_auth(&client).await {
                    eprintln!("❌ LNURL-auth error: {}", e);
                }
                println!("✅ All tests completed!");
            }
            "0" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid choice, please try again");
            }
        }
    }

    Ok(())
}
