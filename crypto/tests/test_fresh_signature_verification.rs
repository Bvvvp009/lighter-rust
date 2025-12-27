//! Comprehensive test to verify fresh signatures work correctly after scalar arithmetic fix

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use hex;

#[test]
fn test_fresh_signature_generation_and_verification() {
    println!("\n=== Testing Fresh Signature Generation and Verification ===\n");
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    // Test with multiple random key pairs and messages
    for i in 0..20 {
        // Generate random private key
        let private_key = ScalarField::sample_crypto();
        let private_key_bytes = private_key.to_bytes_le();
        
        // Generate public key
        let public_key_point = Point::generator().mul(&private_key);
        let public_key_bytes = public_key_point.encode().to_bytes_le();
        
        // Create test message
        let message = {
            let mut msg = [0u8; 40];
            // Use different messages for variety
            for j in 0..40 {
                msg[j] = ((i * 40 + j) % 256) as u8;
            }
            msg
        };
        
        // Sign the message
        let signature = match sign(&private_key_bytes, &message) {
            Ok(sig) => sig,
            Err(e) => {
                println!("Test {}: ❌ Failed to sign: {}", i, e);
                failure_count += 1;
                continue;
            }
        };
        
        // Verify the signature
        let is_valid = match verify_signature(&signature, &message, &public_key_bytes) {
            Ok(valid) => valid,
            Err(e) => {
                println!("Test {}: ❌ Failed to verify: {}", i, e);
                failure_count += 1;
                continue;
            }
        };
        
        if is_valid {
            success_count += 1;
            if i < 5 {
                println!("Test {}: ✅ Signature verified successfully", i);
            }
        } else {
            failure_count += 1;
            println!("Test {}: ❌ Signature verification FAILED", i);
            println!("  Private key: {}...", hex::encode(&private_key_bytes[..8]));
            println!("  Public key: {}...", hex::encode(&public_key_bytes[..8]));
            println!("  Message: {}...", hex::encode(&message[..8]));
            println!("  Signature: {}...", hex::encode(&signature[..16]));
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("SUMMARY:");
    println!("  Successes: {}/20", success_count);
    println!("  Failures: {}/20", failure_count);
    println!("  Success Rate: {:.1}%", (success_count as f64 / 20.0) * 100.0);
    println!("{}", "=".repeat(80));
    
    if failure_count == 0 {
        println!("\n✅ ALL SIGNATURES VERIFIED SUCCESSFULLY!");
        println!("   The scalar arithmetic fix is working correctly!");
    } else {
        println!("\n⚠️  Some signatures still fail verification");
        println!("   This suggests there may be additional issues beyond the scalar arithmetic bug");
    }
    
    // We expect 100% success rate after the fix
    assert_eq!(failure_count, 0, "All fresh signatures should verify correctly after the scalar arithmetic fix");
}

#[test]
fn test_signature_roundtrip_detailed() {
    println!("\n=== Detailed Signature Roundtrip Test ===\n");
    
    // Generate key pair
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    // Test message
    let message = [0u8; 40];
    
    println!("Private key: {}...", hex::encode(&private_key_bytes[..8]));
    println!("Public key: {}...", hex::encode(&public_key_bytes[..8]));
    println!("Message: {}...", hex::encode(&message[..8]));
    
    // Sign
    let signature = sign(&private_key_bytes, &message)
        .expect("Failed to sign");
    
    println!("\nSignature generated:");
    println!("  s: {}...", hex::encode(&signature[..8]));
    println!("  e: {}...", hex::encode(&signature[40..48]));
    
    // Extract s and e
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    // Verify scalar arithmetic: k = s + e*sk should hold
    let e_times_sk = e.mul(&private_key);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k_reconstructed = s.add(e_times_sk_canonical);
    
    println!("\nScalar arithmetic verification:");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  k_reconstructed: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Verify signature
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)
        .expect("Failed to verify");
    
    println!("\nVerification result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    
    assert!(is_valid, "Signature should verify correctly");
}








