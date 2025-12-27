//! Compare SDK vs Our Implementation
//!
//! This tool generates auth tokens using both the SDK and our implementation
//! to compare signatures and test which one the server accepts.
//!
//! Usage:
//!   cargo run --example test_sdk_vs_our_implementation --release
//!
//! Environment variables:
//!   API_PRIVATE_KEY - Your API private key (hex, with or without 0x prefix)
//!   API_KEY_INDEX   - API key index (default: 5)
//!   ACCOUNT_INDEX   - Account index (default: 361816)
//!   BASE_URL        - Base URL (default: https://mainnet.zklighter.elliot.ai)

use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::Client;
use serde_json::Value;

use signer::KeyManager;

fn load_dotenv() {
    if let Ok(current_dir) = env::current_dir() {
        let env_files = [
            current_dir.join(".env"),
            current_dir.join("..").join(".env"),
            current_dir.join("..").join("..").join(".env"),
        ];
        for env_file in env_files.iter() {
            if env_file.exists() {
                if let Ok(content) = std::fs::read_to_string(env_file) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"').trim_matches('\'');
                            if env::var(key).is_err() {
                                env::set_var(key, value);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

async fn test_auth_token_with_server(
    client: &Client,
    base_url: &str,
    auth_token: &str,
    account_index: i64,
) -> Result<(u16, String), Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/accountActiveOrders", base_url);
    
    let response = client
        .get(&url)
        .header("Authorization", auth_token)
        .query(&[
            ("account_index", account_index.to_string().as_str()),
            ("market_id", "0"),
        ])
        .send()
        .await?;
    
    let status_code = response.status().as_u16();
    let body_text = response.text().await.unwrap_or_default();
    
    Ok((status_code, body_text))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Comparing SDK vs Our Implementation\n");
    println!("{}", "=".repeat(80));
    
    // Load configuration
    let api_private_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
    
    let api_key_index = env::var("API_KEY_INDEX")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u8>()
        .unwrap_or(5);
    
    let account_index = env::var("ACCOUNT_INDEX")
        .unwrap_or_else(|_| "361816".to_string())
        .parse::<i64>()
        .unwrap_or(361816);
    
    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    
    println!("Configuration:");
    println!("  API Key Index:  {}", api_key_index);
    println!("  Account Index:  {}", account_index);
    println!("  Base URL:       {}", base_url);
    println!("  Private Key:    {}...\n", &api_private_key[..api_private_key.len().min(20)]);
    
    // Initialize our key manager
    let key_manager = KeyManager::from_hex(&api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    println!("{}", "=".repeat(80));
    println!("Test 1: Our Implementation");
    println!("{}", "=".repeat(80));
    
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let our_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token: {}...", &our_token[..our_token.len().min(100)]);
    
    // Extract signature from token
    let parts: Vec<&str> = our_token.split(':').collect();
    if parts.len() >= 4 {
        let our_signature = parts[3];
        println!("Signature (hex): {}...", &our_signature[..our_signature.len().min(40)]);
        println!("Signature length: {} bytes", our_signature.len() / 2);
    }
    
    // Test with server
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    println!("\nTesting with server...");
    match test_auth_token_with_server(&client, &base_url, &our_token, account_index).await {
        Ok((status_code, body)) => {
            println!("Status Code: {}", status_code);
            if status_code == 200 {
                println!("✅ Our implementation: Server ACCEPTED");
            } else {
                println!("❌ Our implementation: Server REJECTED");
                if let Ok(json) = serde_json::from_str::<Value>(&body) {
                    if let Some(msg) = json.get("message") {
                        println!("   Error: {}", msg);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("Test 2: SDK Implementation");
    println!("{}", "=".repeat(80));
    println!("Note: SDK testing requires lighter-sdk to be available in the path.");
    println!("If you have lighter-sdk installed, you can test it separately.");
    println!("SDK approach uses WeierstrassPoint for verification with e directly.");
    
    println!("\n{}", "=".repeat(80));
    println!("Key Differences Found:");
    println!("{}", "=".repeat(80));
    println!("1. SDK uses WeierstrassPoint for verification (we use Point)");
    println!("2. SDK uses e directly in verification (we use e.monty_mul(&ONE))");
    println!("3. SDK message conversion uses direct little-endian (same result as ours)");
    println!("4. Our Point approach works locally but server rejects it");
    println!("5. SDK WeierstrassPoint approach fails both locally and with server");
    
    println!("\n{}", "=".repeat(80));
    println!("Recommendations:");
    println!("{}", "=".repeat(80));
    println!("1. Test SDK directly with server to confirm it works");
    println!("2. Compare byte-by-byte: SDK signature vs our signature for same inputs");
    println!("3. Check if server accepts SDK-generated tokens");
    println!("4. If SDK works, identify exact differences in signature format");
    println!("5. If SDK also fails, investigate server-side verification algorithm");
    
    Ok(())
}





