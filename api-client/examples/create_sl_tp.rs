use api_client::{LighterClient, CreateOrderRequest};
use std::env;

/// Example: Create Stop Loss and Take Profit Orders
/// 
/// This example demonstrates how to create:
/// - Stop Loss (SL) orders - Market orders that trigger at a specific price
/// - Take Profit (TP) orders - Market orders that trigger at a specific price
/// - Stop Loss Limit orders - Limit orders that trigger at a specific price
/// - Take Profit Limit orders - Limit orders that trigger at a specific price
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🛡️ CREATE STOP LOSS & TAKE PROFIT ORDERS EXAMPLE");
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

    // Order type constants
    // 0 = LimitOrder
    // 1 = MarketOrder
    // 2 = StopLossOrder (Market SL)
    // 3 = StopLossLimitOrder (Limit SL)
    // 4 = TakeProfitOrder (Market TP)
    // 5 = TakeProfitLimitOrder (Limit TP)

    // Example 1: Take Profit Market Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 1: Take Profit Market Order");
    println!("{}", "─".repeat(80));
    let tp_market = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 20001,
        base_amount: 1000,         // 0.1 ETH
        price: 0,                   // Market order - price ignored
        is_ask: false,             // Buy order (to close short position)
        order_type: 4,             // TakeProfitOrder (Market)
        time_in_force: 0,          // ImmediateOrCancel for market orders
        reduce_only: true,         // Only reduce position
        trigger_price: 500000,     // Trigger at $5000
    };

    println!("  Creating Take Profit Market Order:");
    println!("    Trigger Price: ${}", tp_market.trigger_price as f64 / 100.0);
    println!("    Amount: {}", tp_market.base_amount);
    println!("    Type: Market Order");
    let response = client.create_order(tp_market).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("  ✅ Take Profit Market Order created!");
    } else {
        println!("  ⚠️  Failed: {}", response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    // Example 2: Stop Loss Market Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 2: Stop Loss Market Order");
    println!("{}", "─".repeat(80));
    let sl_market = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 20002,
        base_amount: 1000,         // 0.1 ETH
        price: 0,                   // Market order - price ignored
        is_ask: false,             // Buy order (to close short position)
        order_type: 2,             // StopLossOrder (Market)
        time_in_force: 0,          // ImmediateOrCancel for market orders
        reduce_only: true,         // Only reduce position
        trigger_price: 450000,     // Trigger at $4500
    };

    println!("  Creating Stop Loss Market Order:");
    println!("    Trigger Price: ${}", sl_market.trigger_price as f64 / 100.0);
    println!("    Amount: {}", sl_market.base_amount);
    println!("    Type: Market Order");
    let response = client.create_order(sl_market).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("  ✅ Stop Loss Market Order created!");
    } else {
        println!("  ⚠️  Failed: {}", response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    // Example 3: Take Profit Limit Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 3: Take Profit Limit Order");
    println!("{}", "─".repeat(80));
    let tp_limit = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 20003,
        base_amount: 1000,         // 0.1 ETH
        price: 500000,             // Limit price $5000
        is_ask: false,             // Buy order (to close short position)
        order_type: 5,             // TakeProfitLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 500000,     // Trigger at $5000
    };

    println!("  Creating Take Profit Limit Order:");
    println!("    Trigger Price: ${}", tp_limit.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", tp_limit.price as f64 / 100.0);
    println!("    Amount: {}", tp_limit.base_amount);
    println!("    Type: Limit Order");
    let response = client.create_order(tp_limit).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("  ✅ Take Profit Limit Order created!");
    } else {
        println!("  ⚠️  Failed: {}", response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    // Example 4: Stop Loss Limit Order
    println!("{}", "─".repeat(80));
    println!("📝 Example 4: Stop Loss Limit Order");
    println!("{}", "─".repeat(80));
    let sl_limit = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 20004,
        base_amount: 1000,         // 0.1 ETH
        price: 450000,             // Limit price $4500
        is_ask: false,             // Buy order (to close short position)
        order_type: 3,             // StopLossLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 450000,     // Trigger at $4500
    };

    println!("  Creating Stop Loss Limit Order:");
    println!("    Trigger Price: ${}", sl_limit.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", sl_limit.price as f64 / 100.0);
    println!("    Amount: {}", sl_limit.base_amount);
    println!("    Type: Limit Order");
    let response = client.create_order(sl_limit).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("  ✅ Stop Loss Limit Order created!");
    } else {
        println!("  ⚠️  Failed: {}", response["message"].as_str().unwrap_or("Unknown error"));
    }
    println!();

    println!("{}", "═".repeat(80));
    println!("✅ SL/TP Examples Complete");
    println!("{}", "═".repeat(80));
    println!();
    println!("💡 Key Points:");
    println!("  • Stop Loss: Triggers when price moves against you");
    println!("  • Take Profit: Triggers when price moves in your favor");
    println!("  • Market Orders: Execute immediately at trigger");
    println!("  • Limit Orders: Wait for price to reach limit after trigger");
    println!("  • reduce_only: true ensures orders only close positions");

    Ok(())
}

