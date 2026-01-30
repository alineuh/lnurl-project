use reqwest::Client;
use server_lightning::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Call the channel request endpoint
    println!("Calling /channel_request ...");

    let req: RequestChannelResponse = client
        .get("http://192.168.27.67:3000/channel_request")
        .send()
        .await?
        .json()
        .await?;

    println!("Received: {req:#?}");

    // Extract remote node information
    let (node_id, host, port) = parse_uri(&req.uri)?;

    println!("Opening channel with remote node:");
    println!("  node_id = {node_id}");
    println!("  host    = {host}:{port}");

    // Build request
    let open_req = OpenChannelRequest {
        node_id,
        host,
        port,
        satoshis: 50_000,
        k1: req.k1,
    };

    let resp: OpenChannelResponse = client
        .post("http://192.168.27.67:3000/open_channel")
        .json(&open_req)
        .send()
        .await?
        .json()
        .await?;

    println!("Channel opened:");
    println!("{resp:#?}");

    Ok(())
}

fn parse_uri(uri: &str) -> Result<(String, String, u16), Box<dyn Error>> {
    // format "pubkey@ip:port"
    let parts: Vec<_> = uri.split('@').collect();
    if parts.len() != 2 {
        return Err("invalid uri".into());
    }

    let node_id = parts[0].to_string();
    let socket = parts[1];

    let addr_parts: Vec<_> = socket.split(':').collect();
    if addr_parts.len() != 2 {
        return Err("invalid address".into());
    }

    let host = addr_parts[0].to_string();
    let port: u16 = addr_parts[1].parse()?;

    Ok((node_id, host, port))
}
