use api_client::{LighterClient, CreateOrderRequest};
use std::env;

/// Example: Create a Spot Limit Order
/// 
/// Spot trading allows you to trade assets directly (buy/sell) without leverage.
/// This example demonstrates how to create a limit order on a spot market.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🪙 CREATE SPOT LIMIT ORDER EXAMPLE");
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

    // Create a spot limit order
    // Note: Spot markets typically use different market indices than perpetual markets
    // Check the API documentation for the correct spot market index
    println!("📝 Creating spot limit order...");
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // Spot market index (check API docs for correct index)
        client_order_index: 12345, // unique identifier
        base_amount: 1000,         // Amount in smallest unit (e.g., wei for ETH)
        price: 349659,             // Limit price in smallest unit (e.g., micro-USDC)
        is_ask: false,             // false = buy order, true = sell order
        order_type: 0,             // 0 = LimitOrder
        time_in_force: 1,          // 1 = GoodTillTime (GTT)
        reduce_only: false,        // Spot orders typically don't use reduce_only
        trigger_price: 0,          // Not used for regular limit orders
    };

    println!("  Order Details:");
    println!("    Market Index: {}", order.order_book_index);
    println!("    Side: {}", if order.is_ask { "Sell" } else { "Buy" });
    println!("    Amount: {}", order.base_amount);
    println!("    Price: {}", order.price);
    println!("    Type: Limit Order");
    println!("    Time in Force: Good Till Time");
    println!();

    let response = client.create_order(order).await?;

    println!("✅ Spot limit order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Spot order created successfully!");
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

