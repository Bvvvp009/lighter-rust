use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CLOSE PERPETUAL POSITION");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    let market_index: u8 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let avg_execution_price: i64 = env::var("AVG_EXECUTION_PRICE")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(350_000);

    // Close a position by creating a market order with reduce_only flag
    // This example closes a position on the configured market index.
    println!("📝 Closing ETH position...");
    println!("  Market index: {}", market_index);
    println!("  Type: Market order");
    println!("  Effect: Closes the current live position size");
    println!();
    println!(
        "⚠️  Note: Use with caution - this will close your position immediately at market price"
    );
    println!();

    let account = client.get_my_account().await?;
    let position_size = account
        .positions
        .unwrap_or_default()
        .into_iter()
        .find(|position| position.market_index == market_index as u32)
        .and_then(|position| {
            position
                .base_amount
                .as_deref()
                .and_then(|value| value.trim().parse::<f64>().ok())
                .map(|value| value.round() as i64)
        })
        .unwrap_or(0);

    if position_size == 0 {
        println!("ℹ No open position found on market {}", market_index);
        return Ok(());
    }

    let close_side_is_ask = position_size > 0;

    // Create a market close order on the exact live position size.
    let response = client
        .create_market_order(
            market_index, // order_book_index
            400,     // client_order_index
            position_size.abs(), // base_amount
            avg_execution_price, // avg_execution_price (acceptable slippage)
            close_side_is_ask,    // is_ask (true = sell, closes long position)
        )
        .await?;

    println!("✅ Close order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Position closed successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Close order returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}
