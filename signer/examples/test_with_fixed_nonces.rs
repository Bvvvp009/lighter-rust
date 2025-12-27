//! Test with Fixed Nonces - Isolate randomness vs deterministic issues
//!
//! This tool tests signature generation with fixed nonces to check if the issue
//! is related to randomness or deterministic signature generation.
//!
//! Usage:
//!   cargo run --example test_with_fixed_nonces --release

use signer::KeyManager;
use goldilocks_crypto::{ScalarField, verify_signature};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Test with Fixed Nonces");
    println!("{}", "=".repeat(80));
    println!("This tool tests signature generation with fixed nonces to isolate");
    println!("whether the issue is related to randomness or deterministic generation.\n");
    
    let test_private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(test_private_key_hex)?;
    let public_key = key_manager.public_key_bytes();
    
    println!("Private Key (hex): {}...", &test_private_key_hex[..20]);
    println!("Public Key (hex): {}\n", hex::encode(&public_key));
    
    // Test message hash (from auth token)
    let deadline = 1766426073i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    // Reconstruct message hash
    let auth_data = format!("{}:{}:{}", deadline, account_index, api_key_index);
    let auth_bytes = auth_data.as_bytes();
    let missing = (8 - auth_bytes.len() % 8) % 8;
    
    use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
    let mut elements = Vec::new();
    let mut i = 0;
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        if chunk.len() < 8 && missing > 0 {
            bytes[chunk.len()..].fill(0);
        }
        bytes.reverse();
        let val = u64::from_be_bytes(bytes);
        elements.push(Goldilocks::from_canonical_u64(val));
        i = next_start;
    }
    
    let hash_fp5 = hash_to_quintic_extension(&elements);
    let message_bytes = hash_fp5.to_bytes_le();
    
    println!("Message hash: {}", hex::encode(&message_bytes));
    println!("Testing with fixed nonces...\n");
    
    // Test with different fixed nonces
    let test_nonces = vec![
        [0u8; 40],  // All zeros
        [1u8; 40],  // All ones
        [0xFFu8; 40], // All 0xFF
        {
            let mut n = [0u8; 40];
            n[0] = 1;
            n
        }, // Just first byte set
        {
            let mut n = [0u8; 40];
            n[39] = 1;
            n
        }, // Just last byte set
    ];
    
    // We need to use sign_with_nonce, but it's only available in test builds
    // So we'll test the regular sign() function multiple times and see if we can
    // identify patterns
    
    println!("{}", "=".repeat(80));
    println!("Test 1: Generate Multiple Signatures (Random Nonces)");
    println!("{}", "=".repeat(80));
    
    let mut signatures = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for i in 1..=20 {
        let signature = key_manager.sign(&message_bytes)?;
        let is_valid = verify_signature(&signature, &message_bytes, &public_key)?;
        
        signatures.push((i, signature.clone(), is_valid));
        
        if is_valid {
            valid_count += 1;
            println!("Signature {}: ✅ VALID", i);
        } else {
            invalid_count += 1;
            println!("Signature {}: ❌ INVALID", i);
            println!("  Signature: {}...", hex::encode(&signature[..20]));
        }
    }
    
    println!("\nSummary:");
    println!("  Valid: {} ({:.1}%)", valid_count, (valid_count as f64 / 20.0) * 100.0);
    println!("  Invalid: {} ({:.1}%)", invalid_count, (invalid_count as f64 / 20.0) * 100.0);
    
    // Analyze failed signatures
    if invalid_count > 0 {
        println!("\n{}", "=".repeat(80));
        println!("Analysis of Failed Signatures");
        println!("{}", "=".repeat(80));
        
        let failed_sigs: Vec<_> = signatures.iter().filter(|(_, _, valid)| !*valid).collect();
        
        println!("Failed signatures:");
        for (i, sig, _) in failed_sigs.iter() {
            println!("  Signature {}:", i);
            println!("    s (first 40 bytes): {}", hex::encode(&sig[0..40]));
            println!("    e (last 40 bytes): {}", hex::encode(&sig[40..80]));
            
            // Check if s or e are all zeros or have unusual patterns
            let s_all_zero = sig[0..40].iter().all(|&b| b == 0);
            let e_all_zero = sig[40..80].iter().all(|&b| b == 0);
            
            if s_all_zero {
                println!("    ⚠️  s component is all zeros!");
            }
            if e_all_zero {
                println!("    ⚠️  e component is all zeros!");
            }
        }
        
        // Compare with valid signatures
        let valid_sigs: Vec<_> = signatures.iter().filter(|(_, _, valid)| *valid).collect();
        if !valid_sigs.is_empty() {
            println!("\nComparing with valid signatures:");
            let (_, valid_sig, _) = valid_sigs[0];
            println!("  Valid signature s: {}...", hex::encode(&valid_sig[0..20]));
            println!("  Valid signature e: {}...", hex::encode(&valid_sig[40..60]));
        }
    }
    
    // Test 2: Check if the issue is consistent
    println!("\n{}", "=".repeat(80));
    println!("Test 2: Consistency Check");
    println!("{}", "=".repeat(80));
    
    println!("Generating same signature multiple times (should be different due to randomness)...");
    let mut same_message_sigs = Vec::new();
    for i in 1..=5 {
        let sig = key_manager.sign(&message_bytes)?;
        let is_valid = verify_signature(&sig, &message_bytes, &public_key)?;
        same_message_sigs.push((i, sig, is_valid));
    }
    
    // Check if all are unique
    let mut unique_sigs = std::collections::HashSet::new();
    for (_, sig, _) in &same_message_sigs {
        unique_sigs.insert(hex::encode(sig));
    }
    
    println!("  Unique signatures: {}", unique_sigs.len());
    println!("  Total signatures: {}", same_message_sigs.len());
    
    let all_valid = same_message_sigs.iter().all(|(_, _, valid)| *valid);
    println!("  All valid: {}", if all_valid { "✅ YES" } else { "❌ NO" });
    
    if !all_valid {
        println!("\n⚠️  Some signatures fail even with same message!");
        println!("This suggests a bug in signature generation, not randomness.");
    }
    
    println!("\n{}", "=".repeat(80));
    println!("CONCLUSION");
    println!("{}", "=".repeat(80));
    
    if invalid_count == 0 {
        println!("✅ All signatures passed verification");
        println!("The issue might be intermittent or related to specific conditions.");
    } else {
        println!("❌ {}% of signatures failed verification", (invalid_count as f64 / 20.0) * 100.0);
        println!("This indicates a bug in signature generation.");
        println!("\nPossible causes:");
        println!("  1. Nonce generation issue");
        println!("  2. Signature assembly bug");
        println!("  3. Arithmetic overflow/underflow");
        println!("  4. Form conversion issue (Montgomery vs canonical)");
    }
    
    Ok(())
}









