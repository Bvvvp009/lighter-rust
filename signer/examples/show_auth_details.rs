//! Generate auth token and show all details for debugging

use signer::KeyManager;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

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
    
    let api_private_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
    let api_key_index = env::var("API_KEY_INDEX")
        .unwrap_or_else(|_| "6".to_string())
        .parse::<u8>()?;
    let account_index = env::var("ACCOUNT_INDEX")
        .unwrap_or_else(|_| "361816".to_string())
        .parse::<i64>()?;
    
    let key_manager = KeyManager::from_hex(&api_private_key)?;
    
    // Generate auth token
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + (7 * 3600);
    
    println!("=== Auth Token Generation Details ===\n");
    println!("Account Index: {}", account_index);
    println!("API Key Index: {}", api_key_index);
    println!("Deadline: {}\n", deadline);
    
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    
    println!("Full Auth Token:");
    println!("{}\n", auth_token);
    
    // Parse components
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() == 4 {
        println!("Components:");
        println!("  deadline: {}", parts[0]);
        println!("  account_index: {}", parts[1]);
        println!("  api_key_index: {}", parts[2]);
        println!("  signature: {}\n", parts[3]);
        
        println!("Signature length: {} characters ({} bytes)", parts[3].len(), parts[3].len() / 2);
        
        if parts[3].len() == 160 {
            println!("✓ Signature length correct (80 bytes = 160 hex chars)");
        } else {
            println!("✗ Signature length incorrect!");
        }
    }
    
    println!("\nPublic Key: {}", hex::encode(&key_manager.public_key_bytes()));
    
    println!("\n=== How to test manually ===");
    println!("curl -H \"Authorization: {}\" \\", auth_token);
    println!("  \"https://mainnet.zklighter.elliot.ai/api/v1/accountActiveOrders?account_index={}&market_id=0\"", account_index);
    
    Ok(())
}
