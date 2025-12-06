use api_client::{LighterClient, CreateOrderRequest};
use std::env;

/// Example: Create a Spot Market Order
/// 
/// Spot market orders are executed immediately at the best available price.
/// This example demonstrates how to create a market order on a spot market.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🪙 CREATE SPOT MARKET ORDER EXAMPLE");
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

    // Create a spot market order
    // Market orders execute immediately at the best available price
    println!("📝 Creating spot market order...");
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // Spot market index (check API docs for correct index)
        client_order_index: 12346, // unique identifier
        base_amount: 1000,         // Amount in smallest unit (e.g., wei for ETH)
        price: 0,                   // Market orders: price is ignored (set to 0)
        is_ask: false,             // false = buy order, true = sell order
        order_type: 1,               // 1 = MarketOrder
        time_in_force: 0,           // 0 = ImmediateOrCancel (IOC) for market orders
        reduce_only: false,         // Spot orders typically don't use reduce_only
        trigger_price: 0,          // Not used for market orders
    };

    println!("  Order Details:");
    println!("    Market Index: {}", order.order_book_index);
    println!("    Side: {}", if order.is_ask { "Sell" } else { "Buy" });
    println!("    Amount: {}", order.base_amount);
    println!("    Type: Market Order");
    println!("    Time in Force: Immediate or Cancel (IOC)");
    println!("    Note: Market orders execute at best available price");
    println!();

    let response = client.create_order(order).await?;

    println!("✅ Spot market order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Spot market order created successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("  Transaction Hash: {}", tx_hash);
        }
        if let Some(avg_price) = response["avg_execution_price"].as_i64() {
            println!("  Average Execution Price: {}", avg_price);
        }
    } else {
        println!("\n⚠️  Order submission returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}

