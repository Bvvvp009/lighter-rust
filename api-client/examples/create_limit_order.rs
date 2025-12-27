use api_client::{LighterClient, CreateOrderRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE LIMIT ORDER EXAMPLE");
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

    // Create a limit order
    println!("📝 Creating limit order...");
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // 0 = BTC-USD or ETH-USD
        client_order_index: 12345, // unique identifier
        base_amount: 1000,         // 0.001 tokens in smallest unit
        price: 349659,             // limit price in cents
        is_ask: false,             // false = buy order
        order_type: 0,             // 0 = LimitOrder
        time_in_force: 1,          // 1 = GoodTillTime
        reduce_only: false,
        trigger_price: 0,
    };

    let response = client.create_order(order).await?;

    println!("✅ Limit order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Order created successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Order submission returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}
