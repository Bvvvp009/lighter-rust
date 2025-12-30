use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE MARKET SELL ORDER");
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

    // Create a market sell order for 0.1 ETH at market price (worst acceptable price: $3500)
    println!("📝 Creating market SELL order...");
    println!("  Market: ETH-USD (index 0)");
    println!("  Amount: 0.1 ETH (1000 units)");
    println!("  Type: Market Sell");
    println!("  Min acceptable price: $3500");
    println!();

    let response = client.create_market_order(
        0,              // market_index (0 = ETH-USD)
        101,            // client_order_index (unique identifier)
        1000,           // base_amount (0.1 ETH in smallest unit)
        3500_00,        // avg_execution_price (min price in cents, i.e., $3500)
        true,           // is_ask (true = sell order)
    ).await?;

    println!("✅ Order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Market SELL order created successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Order returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}
