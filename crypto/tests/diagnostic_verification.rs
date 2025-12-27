//! Diagnostic test to understand verification failures

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use hex;

#[test]
fn test_diagnostic_verification() {
    println!("\n=== Diagnostic Verification Test ===\n");
    
    // Test with zero message
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    println!("Private Key: {}...", hex::encode(&private_key_bytes[..8]));
    println!("Public Key: {}", hex::encode(&public_key_bytes));
    println!("Message: {} (all zeros)", hex::encode(&message[..8]));
    
    // Sign multiple times
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for i in 0..10 {
        let signature = sign(&private_key_bytes, &message)
            .expect("Failed to sign message");
        
        let is_valid = verify_signature(&signature, &message, &public_key_bytes)
            .expect("Failed to verify signature");
        
        if is_valid {
            success_count += 1;
            if i < 3 {
                println!("  Signature {}: ✅ VERIFIED", i + 1);
            }
        } else {
            fail_count += 1;
            println!("  Signature {}: ❌ FAILED", i + 1);
            println!("    Signature: {}", hex::encode(&signature[..20]));
        }
    }
    
    println!("\nResults:");
    println!("  Success: {}/10 ({:.1}%)", success_count, (success_count as f64 / 10.0) * 100.0);
    println!("  Failed: {}/10 ({:.1}%)", fail_count, (fail_count as f64 / 10.0) * 100.0);
    
    // Test with sequential bytes message
    println!("\n=== Testing Sequential Bytes Message ===");
    let mut seq_message = [0u8; 40];
    for i in 0..40 {
        seq_message[i] = i as u8;
    }
    
    let mut seq_success = 0;
    let mut seq_fail = 0;
    
    for i in 0..10 {
        let signature = sign(&private_key_bytes, &seq_message)
            .expect("Failed to sign message");
        
        let is_valid = verify_signature(&signature, &seq_message, &public_key_bytes)
            .expect("Failed to verify signature");
        
        if is_valid {
            seq_success += 1;
        } else {
            seq_fail += 1;
            if i < 3 {
                println!("  Signature {}: ❌ FAILED", i + 1);
            }
        }
    }
    
    println!("Sequential message results:");
    println!("  Success: {}/10", seq_success);
    println!("  Failed: {}/10", seq_fail);
    
    // Summary
    println!("\n=== Summary ===");
    println!("Zero message: {}/10 verified", success_count);
    println!("Sequential message: {}/10 verified", seq_success);
    
    if success_count == 10 && seq_success == 10 {
        println!("✅ All signatures verified successfully");
    } else {
        println!("❌ Intermittent failures detected");
        println!("   This suggests an issue with verification logic");
    }
}









