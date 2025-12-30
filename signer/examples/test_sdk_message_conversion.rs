//! Test: Compare SDK's message conversion approach vs our current approach
//!
//! This test compares how the SDK converts message bytes vs our current implementation
//! to identify if message conversion is causing server signature rejections

use std::env;
use signer::KeyManager;
use hex;
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

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

// SDK's approach: direct little-endian (no reversal)
fn array_from_le_bytes_sdk_style(bytes: &[u8]) -> Vec<Goldilocks> {
    let mut result = Vec::with_capacity((bytes.len() + 7) / 8);
    for chunk in bytes.chunks(8) {
        let mut padded = [0u8; 8];
        padded[..chunk.len()].copy_from_slice(chunk);
        // SDK: Direct little-endian interpretation
        let val = u64::from_le_bytes(padded);
        result.push(Goldilocks::from_canonical_u64(val));
    }
    result
}

// Our current approach: reverse bytes then interpret as big-endian
fn array_from_le_bytes_our_style(bytes: &[u8]) -> Vec<Goldilocks> {
    let mut result = Vec::with_capacity((bytes.len() + 7) / 8);
    for chunk in bytes.chunks(8) {
        let mut padded = [0u8; 8];
        padded[..chunk.len()].copy_from_slice(chunk);
        // Our current: Reverse bytes, then interpret as big-endian
        padded.reverse();
        let val = u64::from_be_bytes(padded);
        result.push(Goldilocks::from_canonical_u64(val));
    }
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Testing SDK vs Our Message Conversion Approach\n");
    println!("{}", "=".repeat(80));
    
    // Test with auth token message format
    let deadline = 1766426073i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    let message = format!("{}:{}:{}", deadline, account_index, api_key_index);
    let message_bytes = message.as_bytes();
    
    println!("Message: {}", message);
    println!("Message bytes (hex): {}", hex::encode(message_bytes));
    println!("Message length: {} bytes\n", message_bytes.len());
    
    // Convert using SDK's approach
    println!("{}", "=".repeat(80));
    println!("SDK's Approach: Direct Little-Endian (no reversal)");
    println!("{}", "=".repeat(80));
    let elements_sdk = array_from_le_bytes_sdk_style(message_bytes);
    println!("Goldilocks elements (SDK style):");
    for (i, elem) in elements_sdk.iter().enumerate() {
        println!("  [{}]: {} (0x{:x})", i, elem.0, elem.0);
    }
    
    let hash_sdk_fp5 = hash_to_quintic_extension(&elements_sdk);
    let hash_sdk_bytes = hash_sdk_fp5.to_bytes_le();
    println!("\nHash (SDK style): {}", hex::encode(&hash_sdk_bytes));
    
    // Convert using our current approach
    println!("\n{}", "=".repeat(80));
    println!("Our Current Approach: Reverse bytes, then big-endian");
    println!("{}", "=".repeat(80));
    let elements_our = array_from_le_bytes_our_style(message_bytes);
    println!("Goldilocks elements (Our style):");
    for (i, elem) in elements_our.iter().enumerate() {
        println!("  [{}]: {} (0x{:x})", i, elem.0, elem.0);
    }
    
    let hash_our_fp5 = hash_to_quintic_extension(&elements_our);
    let hash_our_bytes = hash_our_fp5.to_bytes_le();
    println!("\nHash (Our style): {}", hex::encode(&hash_our_bytes));
    
    // Compare
    println!("\n{}", "=".repeat(80));
    println!("Comparison");
    println!("{}", "=".repeat(80));
    let hashes_match = hash_sdk_bytes == hash_our_bytes;
    println!("Hashes match: {}", if hashes_match { "✅ YES" } else { "❌ NO" });
    
    if !hashes_match {
        println!("\n⚠️  CRITICAL: Hashes differ! This could explain server rejections.");
        println!("   The SDK approach produces a different hash than our current approach.");
        println!("   If the server uses SDK-style conversion, our signatures will fail.");
        
        // Show differences in elements
        println!("\nElement-by-element comparison:");
        for i in 0..elements_sdk.len().max(elements_our.len()) {
            let sdk_val = elements_sdk.get(i).map(|e| e.0).unwrap_or(0);
            let our_val = elements_our.get(i).map(|e| e.0).unwrap_or(0);
            if sdk_val != our_val {
                println!("  [{}]: SDK={} (0x{:x}), Ours={} (0x{:x}), diff={}", 
                    i, sdk_val, sdk_val, our_val, our_val, 
                    sdk_val as i128 - our_val as i128);
            } else {
                println!("  [{}]: Match = {} (0x{:x})", i, sdk_val, sdk_val);
            }
        }
    } else {
        println!("\n✅ Hashes match - message conversion is not the issue");
    }
    
    Ok(())
}









