use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> SPOT LIMIT ORDER");
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

    // Spot limit order - specify exact price
    println!("Creating spot limit buy order...");
    println!("  Market Index: 1 (Spot)");
    println!("  Amount: 0.1 tokens");
    println!("  Price: $1800 (limit)");
    println!("  Time in Force: Good Till Time (GTT)");
    println!();

    match client.create_market_order(
        1,              // order_book_index (spot market)
        1003,           // client_order_index
        1000,           // base_amount (0.1 tokens)
        1800_00,        // price $1800
        false,          // is_ask (buy)
    ).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("SUCCESS - Spot limit order placed!");
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
