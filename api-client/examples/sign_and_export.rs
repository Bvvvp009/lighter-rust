use api_client::LighterClient;
use dotenv::dotenv;
use std::env;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Get a fresh nonce
    let nonce = client.get_nonce().await?;
    println!("Fresh nonce from API: {}\n", nonce);

    // Create a market order and sign it (without sending)
    let order = api_client::CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 99999,
        base_amount: 1000,
        price: 350000,
        is_ask: false,
        order_type: 1,    // MarketOrder
        time_in_force: 0, // ImmediateOrCancel
        reduce_only: false,
        trigger_price: 0,
    };

    // Sign the order
    let signed_tx = client.sign_create_order_with_nonce(order, Some(nonce)).await?;

    println!("═══════════════════════════════════════════════════════════");
    println!("Signed Transaction (ready to send or debug):");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("{}", serde_json::to_string_pretty(&signed_tx)?);

    println!("\n═══════════════════════════════════════════════════════════");
    println!("Copy-paste below into cURL or manual verification:");
    println!("═══════════════════════════════════════════════════════════\n");

    let tx_json = serde_json::to_string(&signed_tx)?;
    println!("curl -X POST \\\n  -F 'tx_type=14' \\\n  -F 'tx_info={}' \\\n  https://testnet.zklighter.elliot.ai/api/v1/sendTx", 
        tx_json.replace("\"", "\\\"")
    );

    println!("\n═══════════════════════════════════════════════════════════");
    println!("Key fields for debugging:");
    println!("═══════════════════════════════════════════════════════════");
    println!("  Nonce:        {}", signed_tx["Nonce"].as_i64().unwrap_or(0));
    println!("  ExpiredAt:    {}", signed_tx["ExpiredAt"].as_i64().unwrap_or(0));
    println!("  Signature:    {}", signed_tx["Sig"].as_str().unwrap_or(""));
    println!("  AccountIndex: {}", signed_tx["AccountIndex"].as_i64().unwrap_or(0));
    println!("  ApiKeyIndex:  {}", signed_tx["ApiKeyIndex"].as_u64().unwrap_or(0));

    Ok(())
}
