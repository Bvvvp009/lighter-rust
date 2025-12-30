use api_client::{LighterClient, CreateOrderRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> SPOT MARKET BUY ORDER");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Spot market buy order - market_index must be u8 (0-255)
    // Example: market_index 1 (typical spot market)
    // Use a realistic price to satisfy testnet validation (ETH ~$2,982.49)
    let price_usd_cents: i64 = 2_982_49; // $2,982.49

    let buy_order = CreateOrderRequest {
        account_index,
        order_book_index: 1,      // Spot market (ETH or other spot pair)
        client_order_index: 1001,
        base_amount: 500,         // 0.05 ETH (or equivalent)
        price: price_usd_cents,   // non-zero to satisfy testnet validation
        is_ask: false,            // false = buy
        order_type: 0,            // limit order type (IoC when price provided)
        time_in_force: 2,         // immediate or cancel
        reduce_only: false,
        trigger_price: 0,
    };

    println!("Creating spot buy order...");
    println!("  Market Index: 1");
    println!("  Amount: 0.05 tokens");
    println!("  Price: ${:.2}", (price_usd_cents as f64) / 100.0);
    println!("  Order Type: Market (IoC)");
    println!();

    match client.create_market_order(
        buy_order.order_book_index,       // u8 for order_book_index
        buy_order.client_order_index,
        buy_order.base_amount,
        buy_order.price,
        buy_order.is_ask,
    ).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("SUCCESS - Spot buy order placed!");
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!("FAILED - Code: {}", code);
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }

    Ok(())
}
