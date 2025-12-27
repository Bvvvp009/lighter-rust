//! Test both Rust and Go signature verification
//! 
//! This test checks if we need different handling for Rust vs Go signatures

use goldilocks_crypto::{ScalarField, Point, verify_signature};
use hex;

#[test]
fn test_rust_signature_with_e_direct() {
    println!("\n=== Testing Rust Signature with e Direct ===\n");
    
    // Create a Rust signature
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Sign using Rust
    let signature = goldilocks_crypto::sign(&private_key_bytes, &message).unwrap();
    
    // Verify using current code (e direct, no adjustment)
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    
    println!("Rust signature verification: {}", if is_valid { "✅ PASS" } else { "❌ FAIL" });
    
    if !is_valid {
        println!("\n⚠️  Rust signature fails with e direct!");
        println!("This suggests Rust signatures need e_adjusted.");
    }
}

#[test]
fn test_go_signature_with_e_direct() {
    println!("\n=== Testing Go Signature with e Direct ===\n");
    
    // Known Go signature
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    // Get public key
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    let public_key_point = Point::generator().mul(&sk);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    // Verify using current code (e direct, no adjustment)
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    
    println!("Go signature verification: {}", if is_valid { "✅ PASS" } else { "❌ FAIL" });
    
    if is_valid {
        println!("\n✅ Go signature works with e direct!");
    } else {
        println!("\n❌ Go signature fails with e direct!");
    }
}






