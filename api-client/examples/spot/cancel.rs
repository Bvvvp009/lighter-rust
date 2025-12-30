use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> SPOT CANCEL ORDER");
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

    // Cancel spot order
    println!("Cancelling spot order...");
    println!("  Order Book Index: 1");
    println!("  Order Index: 1001");
    println!();

    match client.cancel_order(1, 1001).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("SUCCESS - Spot order cancelled!");
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
