use api_client::{LighterClient, CreateOrderRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE TWAP ORDER (Time-Weighted Average Price)");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    // Load credentials from environment variables
    // Create a .env file with: BASE_URL, ACCOUNT_INDEX, API_KEY_INDEX, API_PRIVATE_KEY
    let base_url = env::var("BASE_URL")
        .map_err(|_| "BASE_URL environment variable is required. Please set it in your .env file.")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .map_err(|_| "ACCOUNT_INDEX environment variable is required. Please set it in your .env file.")?
        .parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .map_err(|_| "API_KEY_INDEX environment variable is required. Please set it in your .env file.")?
        .parse()?;
    let api_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required. Please set it in your .env file.")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // TWAP order configuration
    let market_index: u16 = 1; // BTC-USD market
    let target_price = 910000; // $910,000 target price (in cents)
    let base_amount = 1000; // 0.01 BTC (in smallest unit) - TWAP will split this over time
    
    // Generate unique client order index
    let client_order_index = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64;

    println!("📝 Creating TWAP Order (Time-Weighted Average Price)");
    println!("  Market Index: {}", market_index);
    println!("  Target Price: ${}", target_price / 100);
    println!("  Base Amount: {} (TWAP will split this over time)", base_amount);
    println!("  Order Type: TWAPOrder (6)");
    println!("  Time In Force: GoodTillTime (1)");
    println!("  Order Expiry: 1 day from now");
    println!();
    println!("💡 TWAP orders split execution over time to reduce market impact");
    println!("   The order will be executed gradually rather than all at once");
    println!();

    let order = CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index,
        base_amount,
        price: target_price,
        is_ask: false,         // Buy order
        order_type: 6,         // TWAPOrder
        time_in_force: 1,      // GoodTillTime
        reduce_only: false,
        trigger_price: 0,
    };

    let response = client.create_order(order).await?;

    println!("✅ TWAP Order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ TWAP order created successfully!");
        println!("\n📊 How TWAP Works:");
        println!("  1. Order is split into smaller chunks over time");
        println!("  2. Each chunk executes at intervals to minimize market impact");
        println!("  3. Execution continues until order is filled or expires");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("\n  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Order submission returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}

