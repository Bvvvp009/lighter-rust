use api_client::LighterClient;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("Checking API key on server...");
    println!("  Account: {}", account_index);
    println!("  API Key Index: {}", api_key_index);

    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;

    // Check if our public key matches server
    match client.check_api_key().await {
        Ok(()) => {
            println!("\n✅ Public key matches server!");
            println!("  Local public key: {}", hex::encode(client.key_manager().public_key_bytes()));
        }
        Err(e) => {
            println!("\n❌ Public key mismatch!");
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
