//! Test Signature Verification - Verify signatures locally
//!
//! This tool tests signature generation and verification locally without needing API access.
//! It helps identify if signatures are being generated correctly.
//!
//! Usage:
//!   cargo run --example test_signature_verification --release

use signer::KeyManager;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Signature Verification Test");
    println!("{}", "=".repeat(80));
    println!("This tool tests signature generation and verification locally.\n");
    
    // Use a test private key (you can change this)
    let test_private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(test_private_key_hex)
        .map_err(|e| format!("Failed to create KeyManager: {}", e))?;
    
    println!("Private Key (hex): {}...", &test_private_key_hex[..20]);
    let public_key = key_manager.public_key_bytes();
    println!("Public Key (hex): {}\n", hex::encode(&public_key));
    
    // Test 1: Sign and verify a simple message
    println!("{}", "=".repeat(80));
    println!("Test 1: Sign and Verify Simple Message");
    println!("{}", "=".repeat(80));
    
    let message = [0u8; 40];
    println!("Message: {} (all zeros)", hex::encode(&message));
    
    let signature = key_manager.sign(&message)?;
    println!("Signature (hex): {}", hex::encode(&signature));
    println!("Signature length: {} bytes", signature.len());
    
    // Verify locally
    use goldilocks_crypto::verify_signature;
    let is_valid = verify_signature(&signature, &message, &public_key)?;
    
    if is_valid {
        println!("✅ Signature verification: PASSED");
    } else {
        println!("❌ Signature verification: FAILED");
        println!("⚠️  WARNING: Signature fails local verification!");
        return Err("Signature verification failed".into());
    }
    
    // Test 2: Generate auth token and verify
    println!("\n{}", "=".repeat(80));
    println!("Test 2: Generate Auth Token and Verify");
    println!("{}", "=".repeat(80));
    
    let deadline = 1766426073i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    println!("Deadline: {}", deadline);
    println!("Account Index: {}", account_index);
    println!("API Key Index: {}\n", api_key_index);
    
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token: {}...", &auth_token[..auth_token.len().min(80)]);
    
    // Extract signature
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() != 4 {
        return Err("Invalid auth token format".into());
    }
    
    let signature_hex = parts[3];
    println!("Signature (hex): {}...", &signature_hex[..signature_hex.len().min(40)]);
    
    // Verify auth token locally
    let is_valid = key_manager.verify_auth_token(
        deadline,
        account_index,
        api_key_index,
        signature_hex,
    )?;
    
    if is_valid {
        println!("✅ Auth token verification: PASSED");
    } else {
        println!("❌ Auth token verification: FAILED");
        println!("⚠️  WARNING: Auth token fails local verification!");
        return Err("Auth token verification failed".into());
    }
    
    // Test 3: Generate multiple tokens with same deadline
    println!("\n{}", "=".repeat(80));
    println!("Test 3: Generate Multiple Tokens with Same Deadline");
    println!("{}", "=".repeat(80));
    
    println!("Generating 5 tokens with the same deadline...\n");
    let mut tokens = Vec::new();
    let mut all_valid = true;
    
    for i in 1..=5 {
        let token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
        let parts: Vec<&str> = token.split(':').collect();
        let sig_hex = parts[3];
        
        let is_valid = key_manager.verify_auth_token(
            deadline,
            account_index,
            api_key_index,
            sig_hex,
        ).unwrap_or(false);
        
        tokens.push((token.clone(), is_valid));
        
        println!("Token {}: {}...", i, &token[..token.len().min(60)]);
        println!("  Signature: {}...", &sig_hex[..sig_hex.len().min(40)]);
        println!("  Verification: {}", if is_valid { "✅ PASSED" } else { "❌ FAILED" });
        
        if !is_valid {
            all_valid = false;
        }
    }
    
    println!("\nSummary:");
    println!("  Total tokens: {}", tokens.len());
    println!("  Valid tokens: {}", tokens.iter().filter(|(_, valid)| *valid).count());
    println!("  Invalid tokens: {}", tokens.iter().filter(|(_, valid)| !*valid).count());
    
    if all_valid {
        println!("✅ All tokens pass local verification");
    } else {
        println!("❌ Some tokens fail local verification");
        return Err("Some tokens failed verification".into());
    }
    
    // Test 4: Check signature uniqueness
    println!("\n{}", "=".repeat(80));
    println!("Test 4: Check Signature Uniqueness");
    println!("{}", "=".repeat(80));
    
    let mut signatures: Vec<String> = tokens.iter()
        .map(|(token, _)| {
            let parts: Vec<&str> = token.split(':').collect();
            parts[3].to_string()
        })
        .collect();
    
    signatures.sort();
    signatures.dedup();
    
    println!("Unique signatures: {}", signatures.len());
    println!("Total signatures: {}", tokens.len());
    
    if signatures.len() == tokens.len() {
        println!("✅ All signatures are unique (nonce is random)");
    } else {
        println!("⚠️  Some signatures are identical (very unlikely with secure randomness)");
    }
    
    println!("\n{}", "=".repeat(80));
    println!("ALL TESTS COMPLETE");
    println!("{}", "=".repeat(80));
    
    if all_valid {
        println!("✅ All signature tests PASSED");
        println!("\nConclusion:");
        println!("  - Signature generation works correctly");
        println!("  - Local verification works correctly");
        println!("  - Auth token generation works correctly");
        println!("\nIf API requests still fail, the issue is likely:");
        println!("  1. Server-side validation mismatch");
        println!("  2. Endpoint-specific requirements");
        println!("  3. Network/request format issues");
    } else {
        println!("❌ Some signature tests FAILED");
        println!("\nConclusion:");
        println!("  - There is a bug in signature generation or verification");
        println!("  - Need to debug signature generation code");
    }
    
    Ok(())
}









