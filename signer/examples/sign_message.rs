//! Sign a message
//!
//! This example demonstrates signing a 40-byte message hash using Schnorr signatures.
//!
//! Usage:
//!   cargo run --example sign_message
//!
//! Environment variables (optional):
//!   API_PRIVATE_KEY - Your API private key (hex, with or without 0x prefix)

use std::env;
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
    
    println!("✍️  Signing message...\n");
    
    // Get private key from environment or use example
    let api_private_key = env::var("API_PRIVATE_KEY")
        .unwrap_or_else(|_| "01000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string());
    
    // Create key manager
    let key_manager = KeyManager::from_hex(&api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Example: Sign a 40-byte message hash (all zeros for demo)
    let message: [u8; 40] = [0u8; 40];
    
    println!("Message to sign (40 bytes):");
    println!("  {}", hex::encode(&message));
    println!();
    
    // Sign the message
    let signature = key_manager.sign(&message)
        .map_err(|e| format!("Failed to sign message: {}", e))?;
    
    println!("✅ Message signed successfully!\n");
    println!("Signature (80 bytes: 40 bytes s + 40 bytes e):");
    println!("  {}", hex::encode(&signature));
    println!();
    
    // Get public key for verification
    let public_key = key_manager.public_key_bytes();
    println!("Public Key (for verification):");
    println!("  {}", hex::encode(&public_key));
    
    Ok(())
}













