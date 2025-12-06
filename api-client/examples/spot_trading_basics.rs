use api_client::{LighterClient, CreateOrderRequest};
use std::env;

/// Example: Spot Trading Basics
/// 
/// This example demonstrates various spot trading operations:
/// - Creating spot limit orders (buy and sell)
/// - Creating spot market orders
/// - Understanding spot market differences from perpetual markets
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🪙 SPOT TRADING BASICS EXAMPLE");
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

    println!("📚 Spot Trading Overview:");
    println!("  • Spot markets allow direct asset trading (buy/sell)");
    println!("  • No leverage - you trade with actual assets");
    println!("  • Market orders execute immediately at best price");
    println!("  • Limit orders wait for price to reach your limit");
    println!();

    // Example 1: Spot Limit Buy Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 1: Spot Limit Buy Order");
    println!("{}", "─".repeat(80));
    let buy_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // Spot market index
        client_order_index: 20001,
        base_amount: 1000,         // Buy 1000 units
        price: 350000,             // Buy at or below this price
        is_ask: false,             // Buy order
        order_type: 0,             // Limit order
        time_in_force: 1,          // Good Till Time
        reduce_only: false,
        trigger_price: 0,
    };

    println!("  Creating buy order: {} units at price {}", buy_order.base_amount, buy_order.price);
    let buy_response = client.create_order(buy_order).await?;
    let buy_code = buy_response["code"].as_i64().unwrap_or_default();
    if buy_code == 200 {
        println!("  ✅ Buy order created successfully!");
    } else {
        println!("  ⚠️  Buy order failed: {}", buy_response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    // Example 2: Spot Limit Sell Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 2: Spot Limit Sell Order");
    println!("{}", "─".repeat(80));
    let sell_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // Spot market index
        client_order_index: 20002,
        base_amount: 500,          // Sell 500 units
        price: 360000,             // Sell at or above this price
        is_ask: true,              // Sell order
        order_type: 0,             // Limit order
        time_in_force: 1,          // Good Till Time
        reduce_only: false,
        trigger_price: 0,
    };

    println!("  Creating sell order: {} units at price {}", sell_order.base_amount, sell_order.price);
    let sell_response = client.create_order(sell_order).await?;
    let sell_code = sell_response["code"].as_i64().unwrap_or_default();
    if sell_code == 200 {
        println!("  ✅ Sell order created successfully!");
    } else {
        println!("  ⚠️  Sell order failed: {}", sell_response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    // Example 3: Spot Market Buy Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 3: Spot Market Buy Order");
    println!("{}", "─".repeat(80));
    let market_buy = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 20003,
        base_amount: 2000,         // Buy 2000 units
        price: 0,                   // Market order - price ignored
        is_ask: false,             // Buy
        order_type: 1,             // Market order
        time_in_force: 0,          // Immediate or Cancel
        reduce_only: false,
        trigger_price: 0,
    };

    println!("  Creating market buy order: {} units (executes at best available price)", market_buy.base_amount);
    let market_buy_response = client.create_order(market_buy).await?;
    let market_buy_code = market_buy_response["code"].as_i64().unwrap_or_default();
    if market_buy_code == 200 {
        println!("  ✅ Market buy order created successfully!");
        if let Some(avg_price) = market_buy_response["avg_execution_price"].as_i64() {
            println!("  Average execution price: {}", avg_price);
        }
    } else {
        println!("  ⚠️  Market buy order failed: {}", market_buy_response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    println!("{}", "═".repeat(80));
    println!("✅ Spot Trading Examples Complete");
    println!("{}", "═".repeat(80));
    println!();
    println!("💡 Key Differences: Spot vs Perpetual");
    println!("  • Spot: Direct asset exchange, no leverage");
    println!("  • Perpetual: Leveraged positions, funding rates");
    println!("  • Spot: reduce_only typically not used");
    println!("  • Both: Use same order types (limit, market)");
    println!("  • Both: Use same time in force options");

    Ok(())
}

