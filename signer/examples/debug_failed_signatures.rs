//! Debug Failed Signatures - Investigate why some signatures fail verification
//!
//! This tool investigates why some signatures fail local verification.
//!
//! Usage:
//!   cargo run --example debug_failed_signatures --release

use signer::KeyManager;
use goldilocks_crypto::verify_signature;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug Failed Signatures");
    println!("{}", "=".repeat(80));
    
    let test_private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(test_private_key_hex)?;
    let public_key = key_manager.public_key_bytes();
    
    let deadline = 1766426073i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    println!("Testing auth token generation with:");
    println!("  Deadline: {}", deadline);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}\n", api_key_index);
    
    // Generate multiple tokens and find ones that fail
    let mut failed_tokens = Vec::new();
    let mut passed_tokens = Vec::new();
    
    println!("Generating tokens until we find failures...\n");
    
    for i in 1..=20 {
        let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
        let parts: Vec<&str> = auth_token.split(':').collect();
        let signature_hex = parts[3];
        
        let is_valid = key_manager.verify_auth_token(
            deadline,
            account_index,
            api_key_index,
            signature_hex,
        ).unwrap_or(false);
        
        if is_valid {
            passed_tokens.push((i, auth_token.clone(), signature_hex.to_string()));
        } else {
            failed_tokens.push((i, auth_token.clone(), signature_hex.to_string()));
            println!("❌ Token {} FAILED verification", i);
            println!("   Token: {}...", &auth_token[..auth_token.len().min(80)]);
            println!("   Signature: {}...", &signature_hex[..signature_hex.len().min(40)]);
            
            // Try to verify manually to see what's wrong
            let signature_bytes = hex::decode(signature_hex)?;
            
            // Reconstruct message hash
            let auth_data = format!("{}:{}:{}", deadline, account_index, api_key_index);
            let auth_bytes = auth_data.as_bytes();
            let missing = (8 - auth_bytes.len() % 8) % 8;
            
            use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
            let mut elements = Vec::new();
            let mut j = 0;
            while j < auth_bytes.len() {
                let next_start = (j + 8).min(auth_bytes.len());
                let chunk = &auth_bytes[j..next_start];
                let mut bytes = [0u8; 8];
                bytes[..chunk.len()].copy_from_slice(chunk);
                if chunk.len() < 8 && missing > 0 {
                    bytes[chunk.len()..].fill(0);
                }
                bytes.reverse();
                let val = u64::from_be_bytes(bytes);
                elements.push(Goldilocks::from_canonical_u64(val));
                j = next_start;
            }
            
            let hash_fp5 = hash_to_quintic_extension(&elements);
            let message_bytes = hash_fp5.to_bytes_le();
            
            println!("   Message hash: {}", hex::encode(&message_bytes));
            println!("   Signature s: {}", hex::encode(&signature_bytes[0..40]));
            println!("   Signature e: {}", hex::encode(&signature_bytes[40..80]));
            
            // Try verification with detailed error
            match verify_signature(&signature_bytes, &message_bytes, &public_key) {
                Ok(true) => println!("   ⚠️  Manual verification: PASSED (inconsistent!)"),
                Ok(false) => println!("   Manual verification: FAILED"),
                Err(e) => println!("   Verification error: {:?}", e),
            }
            
            println!();
            
            if failed_tokens.len() >= 3 {
                break; // Found enough failures
            }
        }
    }
    
    println!("{}", "=".repeat(80));
    println!("SUMMARY");
    println!("{}", "=".repeat(80));
    println!("Total tokens generated: {}", passed_tokens.len() + failed_tokens.len());
    println!("Passed verification: {}", passed_tokens.len());
    println!("Failed verification: {}", failed_tokens.len());
    
    if !failed_tokens.is_empty() {
        println!("\n❌ CRITICAL ISSUE FOUND:");
        println!("   Some signatures fail local verification!");
        println!("   This indicates a bug in signature generation.");
        println!("\nFailed tokens:");
        for (i, token, sig) in failed_tokens.iter() {
            println!("   Token {}: Signature {}...", i, &sig[..sig.len().min(40)]);
        }
        
        println!("\n🔍 Investigation needed:");
        println!("   1. Check nonce generation");
        println!("   2. Check signature assembly (s || e)");
        println!("   3. Check message hashing");
        println!("   4. Compare with working signatures");
    } else {
        println!("\n✅ All tokens passed verification");
    }
    
    Ok(())
}









