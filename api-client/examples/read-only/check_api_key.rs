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

    println!("Validating API key...");
    match client.check_api_key().await {
        Ok(()) => {
            println!("SUCCESS - API key is valid!");
            println!("  Account Index: {}", client.account_index());
            println!("  API Key Index: {}", client.api_key_index());
        }
        Err(e) => {
            println!("Primary check_api_key failed: {}", e);
            println!("Retrying with tolerant parser...");

            let url = format!(
                "{}/api/v1/apiKey?account_index={}&api_key_index={}",
                base_url, account_index, api_key_index
            );

            let response = reqwest::Client::new().get(&url).send().await?;
            let status = response.status();
            let body = response.text().await?;
            let trimmed = body.trim();

            if !status.is_success() {
                println!("FAILED - HTTP status {}", status);
                println!("Body: {}", trimmed);
                return Ok(());
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(json) => {
                    if let Some(server_pubkey) = json["public_key"].as_str() {
                        let local_pubkey = hex::encode(client.key_manager().public_key_bytes());
                        let server_clean = server_pubkey.strip_prefix("0x").unwrap_or(server_pubkey);
                        if server_clean == local_pubkey {
                            println!("SUCCESS - API key is valid (tolerant parse)");
                            println!("  Account Index: {}", client.account_index());
                            println!("  API Key Index: {}", client.api_key_index());
                        } else {
                            println!("FAILED - Pubkey mismatch");
                            println!("  Server: {}", server_pubkey);
                            println!("  Local : {}", local_pubkey);
                        }
                    } else {
                        println!("FAILED - Missing public_key in response: {}", json);
                    }
                }
                Err(parse_err) => {
                    println!("WARN - Could not parse JSON ({}). Raw response: {}", parse_err, trimmed);
                    println!("Treating HTTP {} as success for connectivity check.", status);
                }
            }
        }
    }

    Ok(())
}
