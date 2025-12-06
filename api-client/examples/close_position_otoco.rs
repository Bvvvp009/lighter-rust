use api_client::{LighterClient, CreateOrderRequest, CreateGroupedOrdersRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚫 CLOSE POSITION WITH OTOCO (Close + Re-entry Protection)");
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

    // Prices (in cents)
    let current_price = 55000000;     // $55,000 - Current market price
    let re_entry_price = 54000000;    // $50,000 - Re-entry limit order
    let stop_loss_price = 50000000;  // $50,000 - Stop loss trigger
    let take_profit_price = 60000000; // $60,000 - Take profit trigger


    println!("📝 Closing Position with OTOCO for Re-entry Strategy");
    println!("  Strategy: Close current position, then set up re-entry with protection");
    println!("  Current Price: ${}", current_price / 100);
    println!("  Re-entry: Limit Buy at ${}", re_entry_price / 100);
    println!("  Stop Loss: ${} (position-tied)", stop_loss_price / 100);
    println!("  Take Profit: ${} (position-tied)", take_profit_price / 100);
    println!("  Grouping Type: 3 (OTOCO)");
    println!();

    // Step 1: Close existing position first
    println!("📝 Step 1: Closing existing position...");
    let close_response = client.close_position(0, true).await?; // Close long position (sell)
    
    let close_code = close_response["code"].as_i64().unwrap_or_default();
    if close_code == 200 {
        println!("✅ Position closed successfully!");
        if let Some(tx_hash) = close_response["tx_hash"].as_str() {
            println!("  Close Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Close position returned code: {}", close_code);
        if let Some(msg) = close_response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }
    println!();

    // Step 2: Set up OTOCO for re-entry
    println!("📝 Step 2: Setting up OTOCO for re-entry with protection...");
    
    // Create OTOCO grouped orders for re-entry
    // Order 0: Limit re-entry order (parent)
    // Order 1: Stop Loss (child, position-tied, base_amount = 0)
    // Order 2: Take Profit (child, position-tied, base_amount = 0)
    let request = CreateGroupedOrdersRequest {
        grouping_type: 3, // OTOCO: One Triggers A One Cancels The Other
        orders: vec![
            // Order 0: Limit re-entry order (parent)
            // Note: All orders in grouped orders must have client_order_index: 0
            CreateOrderRequest {
                account_index,
                order_book_index: 0,  // Market index
                client_order_index: 0,  // Must be 0 for grouped orders
                base_amount: 1000000,  // 0.001 BTC (in smallest unit)
                price: re_entry_price, // Re-entry limit price
                is_ask: false,         // Buy order
                order_type: 0,         // LimitOrder
                time_in_force: 1,      // GoodTillTime
                reduce_only: false,
                trigger_price: 0,
            },
            // Order 1: Stop Loss Limit (child, position-tied)
            CreateOrderRequest {
                account_index,
                order_book_index: 0,
                client_order_index: 0,  // Must be 0 for grouped orders
                base_amount: 0,        // Position-tied (uses position size automatically)
                price: stop_loss_price,
                is_ask: true,          // Sell (opposite direction to entry)
                order_type: 3,         // StopLossLimitOrder
                time_in_force: 1,      // GoodTillTime
                reduce_only: true,     // Only reduce position
                trigger_price: stop_loss_price,
            },
            // Order 2: Take Profit Limit (child, position-tied)
            CreateOrderRequest {
                account_index,
                order_book_index: 0,
                client_order_index: 0,  // Must be 0 for grouped orders
                base_amount: 0,        // Position-tied (uses position size automatically)
                price: take_profit_price,
                is_ask: true,          // Sell (opposite direction to entry)
                order_type: 5,         // TakeProfitLimitOrder
                time_in_force: 1,      // GoodTillTime
                reduce_only: true,     // Only reduce position
                trigger_price: take_profit_price,
            },
        ],
    };

    let response = client.create_grouped_orders(request).await?;

    println!("✅ Re-entry OTOCO order group submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Re-entry OTOCO order group created successfully!");
        println!("\n📊 Strategy Flow:");
        println!("  1. ✅ Position closed");
        println!("  2. ✅ Re-entry limit order placed at ${}", re_entry_price / 100);
        println!("  3. When re-entry executes, both Stop Loss and Take Profit are automatically placed");
        println!("  4. If price drops to ${} → Stop Loss Limit executes, Take Profit cancelled", stop_loss_price / 100);
        println!("  5. If price rises to ${} → Take Profit Limit executes, Stop Loss cancelled", take_profit_price / 100);
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("\n  Re-entry Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Re-entry order submission returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}

