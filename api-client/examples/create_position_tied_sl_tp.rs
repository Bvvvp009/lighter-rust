use api_client::{LighterClient, CreateOrderRequest, CreateGroupedOrdersRequest};
use std::env;

/// Example: Create Position-Tied Stop Loss and Take Profit Orders
/// 
/// This example demonstrates creating SL/TP orders that are tied to your position:
/// - BaseAmount=0 means the orders will match your entire position size
/// - Orders automatically grow/shrink as you accumulate more position
/// - Orders are canceled when the position sign changes
/// 
/// This is useful for protecting existing positions without knowing the exact size.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔗 CREATE POSITION-TIED SL/TP ORDERS EXAMPLE");
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

    // Grouping Type: ONE_CANCELS_THE_OTHER = 2
    // This means: If one SL/TP executes, the other is canceled

    // Create Take Profit Limit order
    // BaseAmount=0 means it will match your entire position size
    let take_profit_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 40001,
        base_amount: 0,            // 0 = match entire position size
        price: 300000,             // Limit price $3000
        is_ask: false,             // Buy order (to close short position)
        order_type: 5,             // TakeProfitLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 300000,     // Trigger at $3000
    };

    // Create Stop Loss Limit order
    // BaseAmount=0 means it will match your entire position size
    let stop_loss_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 40002,
        base_amount: 0,            // 0 = match entire position size
        price: 500000,             // Limit price $5000
        is_ask: false,             // Buy order (to close short position)
        order_type: 3,             // StopLossLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 500000,     // Trigger at $5000
    };

    println!("📝 Creating Position-Tied SL/TP Orders:");
    println!("  Grouping Type: ONE_CANCELS_THE_OTHER");
    println!();
    println!("  Order 1: Take Profit Limit Order");
    println!("    Amount: {} (matches entire position)", take_profit_order.base_amount);
    println!("    Trigger Price: ${}", take_profit_order.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", take_profit_order.price as f64 / 100.0);
    println!("    Purpose: Close position at profit target");
    println!();
    println!("  Order 2: Stop Loss Limit Order");
    println!("    Amount: {} (matches entire position)", stop_loss_order.base_amount);
    println!("    Trigger Price: ${}", stop_loss_order.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", stop_loss_order.price as f64 / 100.0);
    println!("    Purpose: Close position at loss limit");
    println!();

    let grouped_request = CreateGroupedOrdersRequest {
        grouping_type: 2, // ONE_CANCELS_THE_OTHER
        orders: vec![take_profit_order, stop_loss_order],
    };

    let response = client.create_grouped_orders(grouped_request).await?;

    println!("✅ Position-tied SL/TP orders submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Position-tied SL/TP orders created successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Order submission returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    println!();
    println!("{}", "═".repeat(80));
    println!("💡 How Position-Tied Orders Work:");
    println!("{}", "═".repeat(80));
    println!("  • BaseAmount=0 means orders match your entire position");
    println!("  • Orders automatically adjust as position size changes");
    println!("  • If TP triggers, SL is canceled (and vice versa)");
    println!("  • Orders are canceled when position sign changes");
    println!("  • Perfect for protecting existing positions");

    Ok(())
}

