use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> GET NONCE (READ-ONLY)");
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

    println!("Fetching current nonce...");
    match client.get_nonce().await {
        Ok(nonce) => {
            println!("SUCCESS - Current nonce: {}", nonce);
            println!("  (Used for transaction ordering and replay protection)");
        }
        Err(e) => {
            println!("FAILED - Error fetching nonce: {}", e);
        }
    }

    println!();
    println!("Refreshing nonce from server...");
    match client.refresh_nonce().await {
        Ok(new_nonce) => {
            println!("SUCCESS - Refreshed nonce: {}", new_nonce);
        }
        Err(e) => {
            println!("FAILED - Error refreshing nonce: {}", e);
        }
    }

    Ok(())
}
