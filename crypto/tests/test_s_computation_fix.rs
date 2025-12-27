//! Test to verify the fix for s computation (k - e*sk)

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use hex;

#[test]
fn test_s_computation_fix() {
    println!("\n=== Testing s computation fix ===\n");
    
    // Generate test values
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Sign the message
    println!("Signing message...");
    let signature = sign(&private_key_bytes, &message)
        .expect("Failed to sign message");
    
    assert_eq!(signature.len(), 80, "Signature must be 80 bytes");
    
    // Extract s and e from signature
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];
    
    let s = ScalarField::from_bytes_le(s_bytes).unwrap();
    let e = ScalarField::from_bytes_le(e_bytes).unwrap();
    
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    
    // Verify the signature
    println!("\nVerifying signature...");
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)
        .expect("Failed to verify signature");
    
    if is_valid {
        println!("✅ Signature verification PASSED");
    } else {
        println!("❌ Signature verification FAILED");
        
        // Debug: Check what went wrong
        println!("\n=== Debug Info ===");
        
        // Reconstruct R using Point operations
        let public_key_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&public_key_bytes).unwrap();
        let public_point = Point::decode(&public_key_fp5).unwrap();
        let generator = Point::generator();
        let r_point = Point::mul_add2(&generator, &public_point, &s, &e);
        
        let r_encoded = r_point.encode();
        
        println!("R encoded: {}...", hex::encode(&r_encoded.to_bytes_le()[..16]));
        
        // Compute e'
        use poseidon_hash::hash_to_quintic_extension;
        let message_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&message).unwrap();
        let mut pre_image = [goldilocks_crypto::Goldilocks::zero(); 10];
        pre_image[..5].copy_from_slice(&r_encoded.0);
        pre_image[5..].copy_from_slice(&message_fp5.0);
        
        let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
        let e_prime_scalar = ScalarField::from_fp5_element(&e_prime_fp5);
        
        println!("e (from signature): {}...", hex::encode(&e.to_bytes_le()[..8]));
        println!("e' (computed):       {}...", hex::encode(&e_prime_scalar.to_bytes_le()[..8]));
        println!("e == e': {}", e.equals(&e_prime_scalar));
    }
    
    assert!(is_valid, "Signature should verify");
}

