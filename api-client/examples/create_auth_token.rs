use api_client::LighterClient;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔐 CREATE AUTH TOKEN EXAMPLE");
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

    // Create auth token with default expiry (7 hours = 25200 seconds)
    println!("📝 Creating auth token...");
    let default_expiry_seconds = 7 * 60 * 60; // 7 hours
    let token = client.create_auth_token(default_expiry_seconds)?;

    println!("✅ Auth token created!");
    println!();
    println!("🔑 Token:");
    println!("{}", token);
    println!();
    println!("📝 Token Format: deadline:account_index:api_key_index:signature");
    println!("  Expiry: {} seconds ({} hours)", default_expiry_seconds, default_expiry_seconds / 3600);

    // Example: Create a short-lived token (10 minutes)
    println!();
    println!("📝 Creating short-lived token (10 minutes)...");
    let short_expiry_seconds = 10 * 60; // 10 minutes
    let short_token = client.create_auth_token(short_expiry_seconds)?;
    println!("✅ Short-lived token created!");
    println!("  Token: {}", short_token.chars().take(50).collect::<String>() + "...");
    println!("  Expiry: {} seconds ({} minutes)", short_expiry_seconds, short_expiry_seconds / 60);

    Ok(())
}
