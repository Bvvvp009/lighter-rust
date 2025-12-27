//! Compare Rust Signatures with Go Signatures
//!
//! This tool helps identify differences between our Rust implementation
//! and Go's implementation by comparing signatures for the same inputs.
//!
//! Usage:
//!   cargo run --example compare_with_go_signature --release
//!
//! Environment variables:
//!   API_PRIVATE_KEY - Your API private key (hex)
//!   API_KEY_INDEX   - API key index (default: 5)
//!   ACCOUNT_INDEX   - Account index (default: 361816)

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use signer::KeyManager;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Comparing Rust vs Go Signatures\n");
    println!("{}", "=".repeat(80));
    
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
    
    // Initialize our key manager
    let key_manager = KeyManager::from_hex(&api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Generate auth token
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    
    println!("Auth Token: {}", auth_token);
    println!("\nToken Format: deadline:account_index:api_key_index:signature");
    
    // Parse token
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() >= 4 {
        let signature_hex = parts[3];
        let signature_bytes = hex::decode(signature_hex)?;
        
        if signature_bytes.len() == 80 {
            let s_bytes = &signature_bytes[0..40];
            let e_bytes = &signature_bytes[40..80];
            
            println!("\nSignature Components:");
            println!("  s (40 bytes): {}", hex::encode(s_bytes));
            println!("  e (40 bytes): {}", hex::encode(e_bytes));
        }
    }
    
    // Reconstruct message hash for comparison
    let message = format!("{}:{}:{}", deadline, account_index, api_key_index);
    println!("\nMessage: {}", message);
    
    println!("\n{}", "=".repeat(80));
    println!("Key Insight from Go Code Analysis:");
    println!("{}", "=".repeat(80));
    println!("Go uses ECgFp5Point for both signing and verification:");
    println!("  - Signing: generator.Mul(&k).Encode()");
    println!("  - Verification: generator.Mul(&s).Add(publicPoint.Mul(&e))");
    println!("  - Go uses e DIRECTLY (no adjustment like e.monty_mul(&ONE))");
    println!("\nOur current approach:");
    println!("  - Signing: Point::generator().mul(&k).encode() ✅ (matches Go)");
    println!("  - Verification: Uses e.monty_mul(&ONE) adjustment ❌ (different from Go)");
    
    println!("\n{}", "=".repeat(80));
    println!("Recommendation:");
    println!("{}", "=".repeat(80));
    println!("Try using e DIRECTLY in verification (like Go does),");
    println!("but keep using Point arithmetic (not WeierstrassPoint).");
    println!("This might match what the server expects.");
    
    Ok(())
}





