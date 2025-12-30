use api_client::LighterClient;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> GENERATE AUTH TOKEN (READ-ONLY)");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("Configuration:");
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    // Create a client (note: BASE_URL can be any valid URL for token generation)
    let base_url = "https://mainnet.zklighter.elliot.ai".to_string();
    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Generate token that expires in 1 hour (3600 seconds)
    println!("Generating authentication token (expiring in 1 hour)...");
    match client.create_auth_token(3600) {
        Ok(token) => {
            println!("SUCCESS - Auth token generated!");
            println!("  Token (first 50 chars): {}...", &token[..std::cmp::min(50, token.len())]);
            println!("  Full length: {} characters", token.len());
            println!();
            println!("Usage: Include this token in Authorization header for API requests");
            println!("  Header: Authorization: Bearer {}", &token[..std::cmp::min(20, token.len())]);
        }
        Err(e) => {
            println!("FAILED - Error generating token: {}", e);
        }
    }

    Ok(())
}
