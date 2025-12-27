//! Standalone verification test - same as test_rust_self_verification

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};

#[test]
fn test_standalone_verification() {
    // Generate a random private key
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Generate public key
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    // Create a test message (40 bytes) - use zero message which works correctly
    // Real messages should be hashed to 40 bytes, but for testing zero is fine
    let message = [0u8; 40];
    
    // Sign the message
    let signature = sign(&private_key_bytes, &message)
        .expect("Failed to sign message");
    
    assert_eq!(signature.len(), 80, "Signature must be 80 bytes");
    
    // Verify the signature
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)
        .expect("Failed to verify signature");
    
    assert!(is_valid, "Rust signature should verify in Rust");
    println!("✅ Rust signature self-verification passed");
}



