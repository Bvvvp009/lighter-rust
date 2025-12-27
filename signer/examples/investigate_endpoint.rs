//! Investigate accountActiveOrders Endpoint - Why does it fail?
//!
//! This tool specifically tests the accountActiveOrders endpoint to understand
//! why it has an 80% failure rate while other endpoints work perfectly.
//!
//! Usage:
//!   cargo run --example investigate_endpoint --release

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use tokio::time::sleep;
use std::time::Duration;

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

struct Config {
    api_private_key: String,
    api_key_index: u8,
    account_index: i64,
    base_url: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let api_private_key = env::var("API_PRIVATE_KEY")
            .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
        
        let api_key_index = env::var("API_KEY_INDEX")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u8>()
            .map_err(|_| "API_KEY_INDEX must be a valid u8")?;
        
        let account_index = env::var("ACCOUNT_INDEX")
            .unwrap_or_else(|_| "361816".to_string())
            .parse::<i64>()
            .map_err(|_| "ACCOUNT_INDEX must be a valid i64")?;
        
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
        
        Ok(Config {
            api_private_key,
            api_key_index,
            account_index,
            base_url,
        })
    }
}

async fn test_endpoint(
    client: &Client,
    config: &Config,
    key_manager: &KeyManager,
    endpoint: &str,
    query_params: &[(&str, &str)],
    use_auth_header: bool,
    use_auth_query: bool,
) -> Result<(u16, String, bool), Box<dyn std::error::Error>> {
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let auth_token = key_manager.create_auth_token(
        deadline,
        config.account_index,
        config.api_key_index,
    )?;
    
    // Verify locally first (for debugging)
    let parts: Vec<&str> = auth_token.split(':').collect();
    let signature_hex = parts[3];
    let _local_verify = key_manager.verify_auth_token(
        deadline,
        config.account_index,
        config.api_key_index,
        signature_hex,
    ).unwrap_or(false);
    
    let url = format!("{}{}", config.base_url, endpoint);
    let mut request = client.get(&url);
    
    if use_auth_header {
        request = request.header("Authorization", &auth_token);
    }
    
    for (key, value) in query_params {
        request = request.query(&[(key, value)]);
    }
    
    if use_auth_query {
        request = request.query(&[("auth", &auth_token)]);
    }
    
    let response = request.send().await?;
    let status = response.status();
    let status_code = status.as_u16();
    let body_text = response.text().await?;
    let success = status.is_success();
    
    Ok((status_code, body_text, success))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Endpoint Investigation Tool");
    println!("{}", "=".repeat(80));
    println!("This tool investigates why accountActiveOrders endpoint fails");
    println!("while other endpoints work perfectly.\n");
    
    let config = Config::from_env()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    println!("Configuration:");
    println!("  API Key Index:  {}", config.api_key_index);
    println!("  Account Index:  {}", config.account_index);
    println!("  Base URL:       {}\n", config.base_url);
    
    let key_manager = KeyManager::from_hex(&config.api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    // Test different endpoint configurations
    let endpoints = vec![
        ("/api/v1/accountActiveOrders", vec![
            ("account_index", config.account_index.to_string()),
            ("market_id", "0".to_string()),
        ]),
        ("/api/v1/accountLimits", vec![
            ("account_index", config.account_index.to_string()),
        ]),
        ("/api/v1/accountMetadata", vec![
            ("by", "index".to_string()),
            ("value", config.account_index.to_string()),
        ]),
    ];
    
    println!("Testing different request configurations:\n");
    
    for (endpoint, params) in endpoints.iter() {
        println!("\n{}", "=".repeat(80));
        println!("Endpoint: {}", endpoint);
        println!("{}", "=".repeat(80));
        
        let query_params: Vec<(&str, &str)> = params.iter()
            .map(|(k, v)| {
                let k_str: &str = k;
                let v_str: &str = v;
                (k_str, v_str)
            })
            .collect();
        
        // Test 1: Both header and query param
        println!("\nTest 1: Auth in header + query param");
        match test_endpoint(
            &client, &config, &key_manager,
            endpoint, &query_params, true, true
        ).await {
            Ok((code, body, success)) => {
                println!("  Status: {} ({})", code, if success { "✅" } else { "❌" });
                if !success {
                    println!("  Error: {}", if body.len() > 200 { &body[..200] } else { &body });
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
        sleep(Duration::from_millis(500)).await;
        
        // Test 2: Only header
        println!("\nTest 2: Auth in header only");
        match test_endpoint(
            &client, &config, &key_manager,
            endpoint, &query_params, true, false
        ).await {
            Ok((code, body, success)) => {
                println!("  Status: {} ({})", code, if success { "✅" } else { "❌" });
                if !success {
                    println!("  Error: {}", if body.len() > 200 { &body[..200] } else { &body });
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
        sleep(Duration::from_millis(500)).await;
        
        // Test 3: Only query param
        println!("\nTest 3: Auth in query param only");
        match test_endpoint(
            &client, &config, &key_manager,
            endpoint, &query_params, false, true
        ).await {
            Ok((code, body, success)) => {
                println!("  Status: {} ({})", code, if success { "✅" } else { "❌" });
                if !success {
                    println!("  Error: {}", if body.len() > 200 { &body[..200] } else { &body });
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
        sleep(Duration::from_millis(500)).await;
        
        // Test 4: Different market_id values
        if endpoint.contains("accountActiveOrders") {
            println!("\nTest 4: Testing different market_id values");
            for market_id in ["0", "1", "2"] {
                let mut test_params: Vec<(&str, &str)> = query_params.iter()
                    .map(|(k, v)| {
                        let k_str: &str = k;
                        let v_str: &str = v;
                        (k_str, v_str)
                    })
                    .collect();
                test_params.push(("market_id", market_id));
                
                match test_endpoint(
                    &client, &config, &key_manager,
                    endpoint, &test_params, true, true
                ).await {
                    Ok((code, _body, success)) => {
                        println!("  market_id={}: Status {} ({})", 
                            market_id, code, if success { "✅" } else { "❌" });
                    }
                    Err(e) => println!("  market_id={}: Error {}", market_id, e),
                }
                sleep(Duration::from_millis(300)).await;
            }
        }
    }
    
    println!("\n\n{}", "=".repeat(80));
    println!("INVESTIGATION COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nKey findings:");
    println!("- Compare success rates across different configurations");
    println!("- Check if accountActiveOrders requires different auth format");
    println!("- Verify if market_id parameter affects authentication");
    println!("- Check if endpoint has different validation requirements");
    
    Ok(())
}

