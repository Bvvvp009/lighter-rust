// Test to verify Schnorr signature determinism and correctness
// Run with: cargo test --package crypto --lib test_schnorr_determinism -- --nocapture

use goldilocks_crypto::{ScalarField, schnorr, Point};
use poseidon_hash::{Goldilocks, Fp5Element, hash_to_quintic_extension};

#[test]
fn test_schnorr_signature_components() {
    // Test private key (40 bytes)
    let private_key_hex = "825ed9fde4a049e5eb4a0a31dd3cc53ac657e4e0171f44ae1224ad301f8e51af5c4bbcafa28e1b55";
    let private_key = hex::decode(private_key_hex).unwrap();
    
    // Test message (transaction hash - 40 bytes)
    let message_hex = "1f1507bc68e6328fdd4a5d205159851b97f95feb7630874366e6862275a2d4bf8bd7f41b65612a26";
    let message = hex::decode(message_hex).unwrap();
    
    // Fixed nonce for deterministic testing (40 bytes)
    let nonce_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let nonce_bytes = hex::decode(nonce_hex).unwrap();
    
    // Sign with fixed nonce
    let signature = schnorr::sign_hashed_message(&private_key, &message, &nonce_bytes).unwrap();
    
    println!("Private Key: {}", hex::encode(&private_key));
    println!("Message:     {}", hex::encode(&message));
    println!("Nonce:       {}", hex::encode(&nonce_bytes));
    println!("Signature:   {}", hex::encode(&signature));
    println!("Sig length:  {} bytes", signature.len());
    
    // Parse signature components
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];
    
    println!("\nSignature Components:");
    println!("s (response): {}", hex::encode(s_bytes));
    println!("e (challenge): {}", hex::encode(e_bytes));
    
    // Verify signature manually
    // 1. Compute R = nonce * G
    let nonce_scalar = ScalarField::from_bytes_le(&nonce_bytes[0..40]).unwrap();
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded = r_point.encode();
    
    println!("\nVerification:");
    println!("R (nonce*G): {}", hex::encode(&r_encoded.to_bytes_le()));
    
    // 2. Compute challenge e = H(R || message)
    let message_fp5 = Fp5Element::from_bytes_le(&message).unwrap();
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let computed_e_fp5 = hash_to_quintic_extension(&pre_image);
    let computed_e_scalar = ScalarField::from_fp5_element(&computed_e_fp5);
    let computed_e_bytes = computed_e_scalar.to_bytes_le();
    
    println!("Computed e:  {}", hex::encode(&computed_e_bytes));
    println!("Signature e: {}", hex::encode(e_bytes));
    println!("e matches:   {}", computed_e_bytes == e_bytes);
    
    // 3. Verify s = nonce - e * private_key
    let private_scalar = ScalarField::from_bytes_le(&private_key).unwrap();
    let e_times_private = computed_e_scalar.mul(&private_scalar);
    let computed_s = nonce_scalar.sub(e_times_private);
    let computed_s_bytes = computed_s.to_bytes_le();
    
    println!("Computed s:  {}", hex::encode(&computed_s_bytes));
    println!("Signature s: {}", hex::encode(s_bytes));
    println!("s matches:   {}", computed_s_bytes == s_bytes);
    
    assert_eq!(computed_e_bytes, e_bytes, "Challenge e mismatch!");
    assert_eq!(computed_s_bytes, s_bytes, "Response s mismatch!");
    
    // Sign again with different random nonce to show non-determinism
    let signature2 = schnorr::sign(&private_key, &message).unwrap();
    println!("\n=== Second signature (random nonce) ===");
    println!("Signature 2: {}", hex::encode(&signature2));
    println!("Different:   {}", signature != signature2);
}

#[test]
fn test_schnorr_random_nonce_quality() {
    // Test that random nonces are actually random
    let private_key = vec![0u8; 40];
    let message = vec![0u8; 40];
    
    let mut signatures = Vec::new();
    for i in 0..10 {
        let sig = schnorr::sign(&private_key, &message).unwrap();
        println!("Signature {}: {}", i+1, hex::encode(&sig));
        signatures.push(sig);
    }
    
    // Check all signatures are different (probabilistically should be)
    for i in 0..signatures.len() {
        for j in (i+1)..signatures.len() {
            assert_ne!(signatures[i], signatures[j], 
                "Signatures {} and {} are identical - RNG might be broken!", i+1, j+1);
        }
    }
    
    println!("\n✅ All 10 signatures are different - RNG is working");
}
