use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 MARKET ORDER WITH SLIPPAGE PROTECTION");
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

    // Create a market order with slippage protection
    // This example buys 0.1 ETH but with a maximum slippage of 5%
    // If current price is $4000, max acceptable price is $4200 (5% higher)
    println!("📝 Creating market order with slippage protection...");
    println!("  Market: ETH-USD (index 0)");
    println!("  Amount: 0.1 ETH");
    println!("  Type: Market Buy");
    println!("  Current Price (example): ~$4000");
    println!("  Max Slippage: 5%");
    println!("  Max Price: $4200");
    println!();

    let response = client.create_market_order(
        0,              // market_index (0 = ETH-USD)
        600,            // client_order_index
        1000,           // base_amount (0.1 ETH)
        4200_00,        // avg_execution_price (max price with 5% slippage tolerance)
        false,          // is_ask (false = buy order)
    ).await?;

    println!("✅ Order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Market order created successfully!");
        println!("   Order will execute at best available price up to $4200");
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
