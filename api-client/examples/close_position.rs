use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CLOSE PERPETUAL POSITION");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Close a position by creating a market order with reduce_only flag
    // This example closes an ETH position
    println!("📝 Closing ETH position...");
    println!("  Market: ETH-USD (index 0)");
    println!("  Type: Market order");
    println!("  Effect: Closes entire position");
    println!();
    println!("⚠️  Note: Use with caution - this will close your position immediately at market price");
    println!();

    // Create a market close order - the side should be opposite of current position
    // This example assumes a LONG position, so we send a SELL order
    let response = client.create_market_order(
        0,              // order_book_index (0 = ETH-USD)
        400,            // client_order_index
        0,              // base_amount (0 means entire position)
        3500_00,        // avg_execution_price (acceptable slippage)
        true,           // is_ask (true = sell, closes long position)
    ).await?;

    println!("✅ Close order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Position closed successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Close order returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}

