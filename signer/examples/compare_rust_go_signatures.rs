//! Compare Rust vs Go Signatures Byte-by-Byte
//!
//! This tool generates a signature with our Rust implementation and provides
//! the exact inputs needed to generate a signature with Go for comparison.
//!
//! Usage:
//!   1. Run: cargo run --example compare_rust_go_signatures --release
//!   2. Use the output to run Go's trace_go_signing.go with the same inputs
//!   3. Compare the signatures byte-by-byte
//!
//! Environment variables:
//!   API_PRIVATE_KEY - Your API private key (hex)

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
    
    println!("🔍 Rust vs Go Signature Comparison Tool\n");
    println!("{}", "=".repeat(80));
    
    let api_private_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
    
    // Initialize our key manager
    let key_manager = KeyManager::from_hex(&api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Generate a simple message (all zeros for easy comparison)
    let message_bytes = [0u8; 40];
    let message_hex = hex::encode(&message_bytes);
    
    println!("Test Configuration:");
    println!("  Private Key: {}", api_private_key);
    println!("  Message (hex): {}", message_hex);
    println!("  Message (bytes): {} bytes of zeros\n", message_bytes.len());
    
    // Create signature using our implementation
    // Note: We need to access the internal signing, so let's just create an auth token
    // and extract the signature, or we could expose a direct signing method
    // For now, let's use a deterministic deadline
    let deadline = 1234567890i64;  // Fixed for reproducibility
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    // Build the message that would be hashed
    let message_string = format!("{}:{}:{}", deadline, account_index, api_key_index);
    println!("Auth Token Message: {}", message_string);
    
    // Create auth token
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token: {}", auth_token);
    
    // Extract signature
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() >= 4 {
        let rust_signature = parts[3];
        println!("\nRust Signature: {}", rust_signature);
        
        let sig_bytes = hex::decode(rust_signature)?;
        if sig_bytes.len() == 80 {
            let s = &sig_bytes[0..40];
            let e = &sig_bytes[40..80];
            println!("  s: {}", hex::encode(s));
            println!("  e: {}", hex::encode(e));
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("To Compare with Go:");
    println!("{}", "=".repeat(80));
    println!("1. In lighter-go directory, run:");
    println!("   go run trace_go_signing.go {} <message_hex>", api_private_key);
    println!("\n2. For auth token message hash:");
    println!("   - First, convert message string to bytes: {:?}", message_string.as_bytes());
    println!("   - Then hash with Poseidon2 to get the 40-byte hash");
    println!("   - Use that hash as the message_hex for Go");
    println!("\n3. Compare the signatures byte-by-byte");
    println!("   - If they match: Our signing is correct");
    println!("   - If they differ: Our signing has a bug");
    
    // Also test with simple zero message directly
    println!("\n{}", "=".repeat(80));
    println!("Direct Message Test (all zeros):");
    println!("{}", "=".repeat(80));
    println!("Message hex: {}", message_hex);
    println!("Run: go run trace_go_signing.go {} {}", api_private_key, message_hex);
    println!("This will help isolate if the issue is in signing or verification.");
    
    Ok(())
}
