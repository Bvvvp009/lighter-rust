use api_client::LighterClient;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("🔍 CHECK API KEY STATUS");
    println!("{}", "=".repeat(80));
    println!();

    dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;

    println!("🔄 Checking API key on server...");

    client.check_api_key().await?;

    let local_pubkey = hex::encode(client.key_manager().public_key_bytes());
    println!();
    println!("🔑 Local Public Key:  0x{}", local_pubkey);
    println!();
    println!("✅ SUCCESS - API key is valid!");
    println!("   Account Index: {}", client.account_index());
    println!("   API Key Index: {}", client.api_key_index());

    Ok(())
}
