//! Debug Signature Failures - Comprehensive Investigation Tool
//!
//! This tool investigates why some signatures fail validation on the server.
//! It performs:
//! 1. Local signature verification before sending to API
//! 2. Detailed logging of message hashing steps
//! 3. Comparison with expected Go behavior
//! 4. Testing with fixed nonces to check randomness issues
//!
//! Usage:
//!   cargo run --example debug_signature_failures --release

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use tokio::time::sleep;
use std::time::Duration;

use signer::KeyManager;
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use hex;

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

/// Verify signature locally before sending to API
fn verify_auth_token_locally(
    key_manager: &KeyManager,
    deadline: i64,
    account_index: i64,
    api_key_index: u8,
    signature_hex: &str,
) -> Result<bool, String> {
    key_manager.verify_auth_token(deadline, account_index, api_key_index, signature_hex)
        .map_err(|e| format!("Verification error: {}", e))
}

/// Detailed logging of message hashing process
fn log_message_hashing(
    deadline: i64,
    account_index: i64,
    api_key_index: u8,
) {
    println!("\n{}", "=".repeat(80));
    println!("MESSAGE HASHING DEBUG");
    println!("{}", "=".repeat(80));
    
    let auth_data = format!("{}:{}:{}", deadline, account_index, api_key_index);
    println!("Auth data string: \"{}\"", auth_data);
    println!("Auth data bytes: {:?}", auth_data.as_bytes());
    println!("Auth data length: {} bytes", auth_data.len());
    
    let auth_bytes = auth_data.as_bytes();
    let missing = (8 - auth_bytes.len() % 8) % 8;
    println!("Missing bytes to pad: {}", missing);
    
    let mut elements = Vec::new();
    let mut i = 0;
    let mut chunk_idx = 0;
    
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        
        if chunk.len() < 8 && missing > 0 {
            bytes[chunk.len()..].fill(0);
        }
        
        println!("\nChunk {}:", chunk_idx);
        println!("  Original bytes: {:?}", chunk);
        println!("  Padded bytes: {:?}", bytes);
        
        bytes.reverse();
        let val = u64::from_be_bytes(bytes);
        let goldi = Goldilocks::from_canonical_u64(val);
        elements.push(goldi);
        
        println!("  Reversed bytes: {:?}", bytes);
        println!("  u64 value: {} (0x{:x})", val, val);
        println!("  Goldilocks element: {} (0x{:x})", goldi.0, goldi.0);
        
        i = next_start;
        chunk_idx += 1;
    }
    
    println!("\nAll Goldilocks elements:");
    for (idx, elem) in elements.iter().enumerate() {
        println!("  [{}]: {} (0x{:x})", idx, elem.0, elem.0);
    }
    
    let hash_fp5 = hash_to_quintic_extension(&elements);
    println!("\nPoseidon2 Hash (Fp5Element):");
    println!("  Elements: [{}, {}, {}, {}, {}]",
        hash_fp5.0[0].0, hash_fp5.0[1].0, hash_fp5.0[2].0,
        hash_fp5.0[3].0, hash_fp5.0[4].0);
    
    let message_bytes = hash_fp5.to_bytes_le();
    println!("  Message bytes (hex): {}", hex::encode(&message_bytes));
    println!("  Message bytes length: {} bytes", message_bytes.len());
    println!("{}", "=".repeat(80));
}

async fn test_auth_token_with_verification(
    client: &Client,
    config: &Config,
    key_manager: &KeyManager,
    request_num: usize,
    endpoint: &str,
    query_params: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    // Log message hashing details
    log_message_hashing(deadline, config.account_index, config.api_key_index);
    
    // Generate auth token
    let auth_token = key_manager.create_auth_token(
        deadline,
        config.account_index,
        config.api_key_index,
    )?;
    
    // Extract signature from token
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid auth token format: {}", auth_token).into());
    }
    let signature_hex = parts[3];
    
    println!("\n{}", "=".repeat(80));
    println!("REQUEST #{}: {}", request_num, endpoint);
    println!("{}", "=".repeat(80));
    println!("Deadline: {}", deadline);
    println!("Auth token: {}...", &auth_token[..auth_token.len().min(80)]);
    
    // Verify signature locally BEFORE sending to API
    println!("\n🔍 Local Signature Verification:");
    match verify_auth_token_locally(
        key_manager,
        deadline,
        config.account_index,
        config.api_key_index,
        signature_hex,
    ) {
        Ok(true) => {
            println!("  ✅ Local verification: PASSED");
        }
        Ok(false) => {
            println!("  ❌ Local verification: FAILED");
            println!("  ⚠️  WARNING: Signature fails local verification!");
            println!("  This signature will definitely fail on the server.");
            return Ok(()); // Don't send to API if local verification fails
        }
        Err(e) => {
            println!("  ⚠️  Local verification error: {}", e);
            println!("  Continuing to send to API anyway...");
        }
    }
    
    // Send request to API
    let url = format!("{}{}", config.base_url, endpoint);
    let mut request = client.get(&url);
    request = request.header("Authorization", &auth_token);
    
    for (key, value) in query_params {
        request = request.query(&[(key, value)]);
    }
    
    request = request.query(&[("auth", &auth_token)]);
    
    let response = request.send().await?;
    let status = response.status();
    let status_code = status.as_u16();
    
    let body_text = response.text().await?;
    
    println!("\n📡 API Response:");
    println!("  Status Code: {}", status_code);
    println!("  Success: {}", if status.is_success() { "✅ YES" } else { "❌ NO" });
    
    if !status.is_success() {
        println!("  Error Response: {}", body_text);
        
        // Check if it's an invalid signature error
        if body_text.contains("invalid signature") {
            println!("\n  🔴 INVALID SIGNATURE ERROR DETECTED");
            println!("  This signature passed local verification but failed on server!");
            println!("  This indicates a mismatch between local and server verification.");
        }
    } else {
        println!("  Response: {}", if body_text.len() > 200 {
            format!("{}...", &body_text[..200])
        } else {
            body_text.clone()
        });
    }
    
    println!("{}", "=".repeat(80));
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Signature Failure Debug Tool");
    println!("{}", "=".repeat(80));
    println!("This tool investigates signature failures by:");
    println!("  1. Verifying signatures locally before sending to API");
    println!("  2. Logging detailed message hashing steps");
    println!("  3. Comparing with expected Go behavior");
    println!("  4. Identifying mismatches between local and server verification\n");
    
    let config = Config::from_env()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    println!("Configuration:");
    println!("  API Key Index:  {}", config.api_key_index);
    println!("  Account Index:  {}", config.account_index);
    println!("  Base URL:       {}", config.base_url);
    println!("  Private Key:    {}...\n", &config.api_private_key[..config.api_private_key.len().min(20)]);
    
    let key_manager = KeyManager::from_hex(&config.api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Print public key for reference
    let public_key = key_manager.public_key_bytes();
    println!("Public Key (hex): {}\n", hex::encode(&public_key));
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    // Test the problematic endpoint
    let endpoint = "/api/v1/accountActiveOrders";
    let query_params = vec![
        ("account_index", config.account_index.to_string()),
        ("market_id", "0".to_string()),
    ];
    
    println!("Testing {} requests to {} endpoint\n", 5, endpoint);
    
    for i in 1..=5 {
        println!("\n\n{}", "🔄".repeat(40));
        println!("TEST {} of 5", i);
        println!("{}", "🔄".repeat(40));
        
        let params: Vec<(&str, &str)> = query_params.iter()
            .map(|(k, v)| {
                let k_str: &str = k;
                let v_str: &str = v;
                (k_str, v_str)
            })
            .collect();
        
        if let Err(e) = test_auth_token_with_verification(
            &client,
            &config,
            &key_manager,
            i,
            endpoint,
            &params,
        ).await {
            eprintln!("Error in test {}: {}", i, e);
        }
        
        if i < 5 {
            sleep(Duration::from_millis(500)).await;
        }
    }
    
    println!("\n\n{}", "=".repeat(80));
    println!("INVESTIGATION COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nKey Findings:");
    println!("  - Check if signatures pass local verification");
    println!("  - Compare message hashing with Go implementation");
    println!("  - Look for patterns in failures (same deadline, etc.)");
    println!("  - If local verification passes but server rejects, there's a mismatch");
    
    Ok(())
}

