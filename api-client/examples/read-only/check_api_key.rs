use api_client::LighterClient;
use std::env;
use serde_json::Value;
use hex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> CHECK API KEY STATUS (READ-ONLY)");
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

    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;

    println!("Checking API key on server...");
    println!("  Account: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let url = format!(
        "{}/api/v1/apiKey?account_index={}&api_key_index={}",
        base_url, account_index, api_key_index
    );

    let response = reqwest::Client::new().get(&url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    let trimmed = body.trim();

    println!("📡 Response Status: {}", status);
    
    if trimmed.is_empty() {
        println!("⚠️  Empty response from server");
        println!("Note: API key may not be registered or endpoint changed");
        return Ok(());
    }

    if !status.is_success() {
        println!("❌ HTTP error: {}", status);
        println!("Response: {}", trimmed);
        return Ok(());
    }

    match serde_json::from_str::<Value>(trimmed) {
        Ok(json) => {
            if let Some(server_pubkey) = json["public_key"].as_str() {
                let local_pubkey = hex::encode(client.key_manager().public_key_bytes());
                let server_clean = server_pubkey.strip_prefix("0x").unwrap_or(server_pubkey);
                
                println!("🔑 Server Public Key: {}", server_pubkey);
                println!("🔑 Local Public Key:  0x{}", local_pubkey);
                println!();
                
                if server_clean == local_pubkey {
                    println!("✅ SUCCESS - API key is valid!");
                    println!("  Account Index: {}", client.account_index());
                    println!("  API Key Index: {}", client.api_key_index());
                } else {
                    println!("❌ FAILED - Public key mismatch");
                    println!("  The private key does not match the registered API key");
                }
            } else {
                println!("⚠️  Response does not contain 'public_key' field");
                println!("Response JSON: {}", json);
            }
        }
        Err(parse_err) => {
            println!("⚠️  Could not parse JSON response");
            println!("Error: {}", parse_err);
            println!("Raw response: {}", trimmed);
            println!();
            println!("Note: The API endpoint may have changed or requires different authentication");
        }
    }

    Ok(())
}
