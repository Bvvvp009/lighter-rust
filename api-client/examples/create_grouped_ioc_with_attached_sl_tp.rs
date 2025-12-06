use api_client::{LighterClient, CreateOrderRequest, CreateGroupedOrdersRequest};
use std::env;

/// Example: Create IOC Order with Attached Stop Loss and Take Profit
/// 
/// This example demonstrates creating a grouped order where:
/// - An IOC (Immediate or Cancel) order is created
/// - A Stop Loss (SL) order is attached to close position if price moves against you
/// - A Take Profit (TP) order is attached to close position if price moves in your favor
/// 
/// The SL/TP orders will be sized based on the executed size of the IOC order.
/// The SL/TP orders are canceled when the position sign changes.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔗 CREATE IOC ORDER WITH ATTACHED SL/TP EXAMPLE");
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

    // Grouping Type: ONE_TRIGGERS_A_ONE_CANCELS_THE_OTHER = 3
    // This means: When the IOC order executes, it triggers the SL/TP orders
    // If one SL/TP executes, the other is canceled

    // Step 1: Create the IOC (Immediate or Cancel) order
    let ioc_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 30001,
        base_amount: 1000,         // 0.1 ETH
        price: 300000,             // $3000 limit price
        is_ask: true,              // Sell order
        order_type: 0,            // Limit order
        time_in_force: 0,          // ImmediateOrCancel (IOC)
        reduce_only: false,
        trigger_price: 0,
    };

    // Step 2: Create Take Profit Limit order
    // BaseAmount=0 means it will match the executed size of the IOC order
    let take_profit_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 30002,
        base_amount: 0,            // 0 = match executed size of IOC order
        price: 300000,             // Limit price $3000
        is_ask: false,             // Buy order (to close short position)
        order_type: 5,             // TakeProfitLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 300000,     // Trigger at $3000
    };

    // Step 3: Create Stop Loss Limit order
    // BaseAmount=0 means it will match the executed size of the IOC order
    let stop_loss_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 30003,
        base_amount: 0,            // 0 = match executed size of IOC order
        price: 500000,             // Limit price $5000
        is_ask: false,             // Buy order (to close short position)
        order_type: 3,             // StopLossLimitOrder
        time_in_force: 1,          // GoodTillTime
        reduce_only: true,         // Only reduce position
        trigger_price: 500000,     // Trigger at $5000
    };

    println!("📝 Creating Grouped Order:");
    println!("  Grouping Type: ONE_TRIGGERS_A_ONE_CANCELS_THE_OTHER");
    println!();
    println!("  Order 1: IOC Limit Order");
    println!("    Amount: {} (0.1 ETH)", ioc_order.base_amount);
    println!("    Price: ${}", ioc_order.price as f64 / 100.0);
    println!("    Type: Immediate or Cancel");
    println!();
    println!("  Order 2: Take Profit Limit Order");
    println!("    Amount: {} (matches IOC execution)", take_profit_order.base_amount);
    println!("    Trigger Price: ${}", take_profit_order.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", take_profit_order.price as f64 / 100.0);
    println!();
    println!("  Order 3: Stop Loss Limit Order");
    println!("    Amount: {} (matches IOC execution)", stop_loss_order.base_amount);
    println!("    Trigger Price: ${}", stop_loss_order.trigger_price as f64 / 100.0);
    println!("    Limit Price: ${}", stop_loss_order.price as f64 / 100.0);
    println!();

    let grouped_request = CreateGroupedOrdersRequest {
        grouping_type: 3, // ONE_TRIGGERS_A_ONE_CANCELS_THE_OTHER
        orders: vec![ioc_order, take_profit_order, stop_loss_order],
    };

    let response = client.create_grouped_orders(grouped_request).await?;

    println!("✅ Grouped order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Grouped order created successfully!");
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
    println!("💡 How It Works:");
    println!("{}", "═".repeat(80));
    println!("  1. IOC order executes immediately (if price is available)");
    println!("  2. SL/TP orders are activated based on executed size");
    println!("  3. If TP triggers, SL is canceled (and vice versa)");
    println!("  4. SL/TP orders are canceled when position sign changes");

    Ok(())
}

