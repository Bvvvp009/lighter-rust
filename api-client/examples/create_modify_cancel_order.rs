use api_client::{LighterClient, CreateOrderRequest, ModifyOrderRequest};
use std::env;

/// Example: Create, Modify, and Cancel Order
/// 
/// This example demonstrates the full order lifecycle:
/// 1. Create a limit order
/// 2. Modify the order (change price and size)
/// 3. Cancel the order
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔄 CREATE, MODIFY, CANCEL ORDER EXAMPLE");
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

    // Step 1: Create a limit order
    println!("{}", "─".repeat(80));
    println!("📝 Step 1: Create Limit Order");
    println!("{}", "─".repeat(80));
    let client_order_index = 12345u64; // Use this to track the order
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // ETH/USDC market
        client_order_index,
        base_amount: 1000,         // 0.1 ETH
        price: 405000,             // $4050 limit price
        is_ask: true,             // Sell order
        order_type: 0,            // Limit order
        time_in_force: 1,         // GoodTillTime
        reduce_only: false,
        trigger_price: 0,
    };

    println!("  Creating sell order:");
    println!("    Amount: {} (0.1 ETH)", order.base_amount);
    println!("    Price: ${}", order.price as f64 / 100.0);
    println!("    Client Order Index: {}", client_order_index);
    println!();

    let response = client.create_order(order).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    
    if code != 200 {
        println!("  ❌ Failed to create order: {}", response["message"].as_str().unwrap_or("Unknown error"));
        return Ok(());
    }

    println!("  ✅ Order created successfully!");
    if let Some(tx_hash) = response["tx_hash"].as_str() {
        println!("    Transaction Hash: {}", tx_hash);
    }

    // Get the order index from the response (or use client_order_index)
    // Note: In production, you'd get the actual order_index from the API response
    // For this example, we'll use the client_order_index
    let order_index = client_order_index as i64;
    println!("    Order Index: {}", order_index);
    println!();

    // Step 2: Modify the order
    println!("{}", "─".repeat(80));
    println!("📝 Step 2: Modify Order");
    println!("{}", "─".repeat(80));
    let modify_request = ModifyOrderRequest {
        market_index: 0,
        order_index,
        base_amount: 1100,         // Increase to 0.11 ETH
        price: 410000,             // Increase price to $4100
        trigger_price: 0,
    };

    println!("  Modifying order:");
    println!("    Order Index: {}", order_index);
    println!("    New Amount: {} (0.11 ETH)", modify_request.base_amount);
    println!("    New Price: ${}", modify_request.price as f64 / 100.0);
    println!();

    let response = client.modify_order(modify_request).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    
    if code != 200 {
        println!("  ⚠️  Failed to modify order: {}", response["message"].as_str().unwrap_or("Unknown error"));
        println!("  Note: Order might not exist yet or may have been filled");
    } else {
        println!("  ✅ Order modified successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("    Transaction Hash: {}", tx_hash);
        }
    }
    println!();

    // Step 3: Cancel the order
    println!("{}", "─".repeat(80));
    println!("📝 Step 3: Cancel Order");
    println!("{}", "─".repeat(80));
    println!("  Canceling order:");
    println!("    Market Index: 0");
    println!("    Order Index: {}", order_index);
    println!();

    let response = client.cancel_order(0, order_index).await?;
    let code = response["code"].as_i64().unwrap_or_default();
    
    if code != 200 {
        println!("  ⚠️  Failed to cancel order: {}", response["message"].as_str().unwrap_or("Unknown error"));
        println!("  Note: Order might not exist, may have been filled, or already canceled");
    } else {
        println!("  ✅ Order canceled successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("    Transaction Hash: {}", tx_hash);
        }
    }
    println!();

    println!("{}", "═".repeat(80));
    println!("✅ Order Lifecycle Example Complete");
    println!("{}", "═".repeat(80));
    println!();
    println!("💡 Key Points:");
    println!("  • Use client_order_index to track your orders");
    println!("  • Modify order to change price and/or size");
    println!("  • Cancel order when no longer needed");
    println!("  • Orders can be filled before modification/cancellation");

    Ok(())
}

