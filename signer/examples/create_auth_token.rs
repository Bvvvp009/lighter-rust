//! Create an authentication token
//!
//! This example creates an authentication token for API access.
//!
//! Usage:
//!   cargo run --example create_auth_token
//!
//! Environment variables (optional):
//!   API_PRIVATE_KEY - Your API private key (hex, with or without 0x prefix)
//!   API_KEY_INDEX   - API key index (default: 5)
//!   ACCOUNT_INDEX   - Account index (default: 361816)

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔐 Creating authentication token...\n");
    
    // Get configuration from environment or use defaults
    let api_private_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
    
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);
    
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .unwrap_or_else(|_| "361816".to_string())
        .parse()
        .unwrap_or(361816);
    
    println!("Configuration:");
    println!("  API Key Index:  {}", api_key_index);
    println!("  Account Index:  {}", account_index);
    println!();
    
    // Create key manager
    let key_manager = KeyManager::from_hex(&api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Calculate deadline (7 hours from now)
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + (7 * 3600);
    
    // Generate auth token
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)
        .map_err(|e| format!("Failed to create auth token: {}", e))?;
    
    println!("✅ Auth token generated successfully!\n");
    println!("Auth Token:");
    println!("  {}", auth_token);
    println!();
    println!("Token expires at: {} ({:.1} hours from now)", deadline, 7.0);
    println!();
    println!("Usage:");
    println!("  Add to HTTP header: Authorization: {}", &auth_token[..auth_token.len().min(50)]);
    println!("  Or as query param: ?auth={}", &auth_token[..auth_token.len().min(50)]);
    
    Ok(())
}













