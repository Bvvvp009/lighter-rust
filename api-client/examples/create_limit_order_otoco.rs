use api_client::{LighterClient, CreateOrderRequest, CreateGroupedOrdersRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE LIMIT ORDER WITH OTOCO (Entry + Stop Loss + Take Profit)");
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

     // Market order prices (in cents)
     let entry_price = 890000;      // $89,000 - Market order will execute at current market price
     let stop_loss_price = 880000;  // $88,000 - Stop loss trigger
     let take_profit_price = 920000; // $92,000 - Take profit trigger
    // Calculate order expiry: 30 days from now (in milliseconds)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    let order_expiry = now + (30 * 24 * 60 * 60 * 1000); // 30 days

    println!("📝 Creating Limit Order with OTOCO (One Triggers A One Cancels The Other)");
    println!("  Entry: Limit Buy at ${}", entry_price / 100);
    println!("  Stop Loss: ${} (position-tied)", stop_loss_price / 100);
    println!("  Take Profit: ${} (position-tied)", take_profit_price / 100);
    println!("  Grouping Type: 3 (OTOCO)");
    println!("  Order Expiry: 28 days");
    println!();

    // Create OTOCO grouped orders
    // Order 0: Limit entry order (parent)
    // Order 1: Stop Loss (child, position-tied, base_amount = 0)
    // Order 2: Take Profit (child, position-tied, base_amount = 0)
    let request = CreateGroupedOrdersRequest {
        grouping_type: 3, // OTOCO: One Triggers A One Cancels The Other
        orders: vec![
            // Order 0: Limit entry order (parent)
            // Note: All orders in grouped orders must have client_order_index: 0
            CreateOrderRequest {
                account_index,
                order_book_index: 1,  // Market index
                client_order_index: 0,  // Must be 0 for grouped orders
                base_amount: 1000000,  // 0.001 BTC (in smallest unit)
                price: entry_price,    // Limit price
                is_ask: false,         // Buy order
                order_type: 0,         // LimitOrder
                time_in_force: 1,      // GoodTillTime
                reduce_only: false,
                trigger_price: 0,
            },
            // Order 1: Stop Loss Limit (child, position-tied)
            CreateOrderRequest {
                account_index,
                order_book_index: 1,
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
                order_book_index: 1,
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

    println!("✅ Limit Order with OTOCO submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ OTOCO order group created successfully!");
        println!("\n📊 How OTOCO Works:");
        println!("  1. Limit order placed at ${}", entry_price / 100);
        println!("  2. When limit order executes, both Stop Loss and Take Profit are automatically placed");
        println!("  3. If price drops to ${} → Stop Loss Limit executes, Take Profit cancelled", stop_loss_price / 100);
        println!("  4. If price rises to ${} → Take Profit Limit executes, Stop Loss cancelled", take_profit_price / 100);
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


