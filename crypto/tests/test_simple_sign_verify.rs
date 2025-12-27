//! Simple sign and verify test

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use hex;

#[test]
fn test_simple_sign_verify() {
    println!("\n=== Simple Sign and Verify Test ===\n");
    
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    println!("Private key: {}...", hex::encode(&private_key_bytes[..8]));
    println!("Public key: {}...", hex::encode(&public_key_bytes[..8]));
    
    // Sign
    println!("\nSigning...");
    let signature = sign(&private_key_bytes, &message).unwrap();
    println!("Signature: {}...", hex::encode(&signature[..16]));
    
    // Extract s and e
    let s = ScalarField::from_bytes_le(&signature[0..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..80]).unwrap();
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    
    // Verify
    println!("\nVerifying...");
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    println!("Result: {}", is_valid);
    
    if !is_valid {
        // Debug: reconstruct k and check R
        let e_sk = e.mul(&private_key);
        let k_reconstructed = s.add(e_sk);
        
        let generator = Point::generator();
        let r_signing = generator.mul(&k_reconstructed);
        let r_signing_encoded = r_signing.encode();
        
        let public_key_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&public_key_bytes).unwrap();
        let public_point = Point::decode(&public_key_fp5).unwrap();
        let r_verification = Point::mul_add2(&generator, &public_point, &s, &e);
        let r_verification_encoded = r_verification.encode();
        
        println!("\nDebug info:");
        println!("  R (from k): {}...", hex::encode(&r_signing_encoded.to_bytes_le()[..16]));
        println!("  R (from verification): {}...", hex::encode(&r_verification_encoded.to_bytes_le()[..16]));
        println!("  R match: {}", r_signing_encoded.to_bytes_le() == r_verification_encoded.to_bytes_le());
        
        // Check hash computation
        use poseidon_hash::hash_to_quintic_extension;
        
        // Recreate message_to_fp5 logic
        fn message_to_fp5(message: &[u8]) -> Result<goldilocks_crypto::Fp5Element, String> {
            if message.len() != 40 {
                return Err(format!("Invalid message length: {}", message.len()));
            }
            let mut message_elements = [poseidon_hash::Goldilocks::zero(); 5];
            for (i, chunk) in message.chunks(8).enumerate().take(5) {
                let mut bytes = [0u8; 8];
                bytes[..chunk.len()].copy_from_slice(chunk);
                bytes.reverse();
                message_elements[i] = poseidon_hash::Goldilocks::from_canonical_u64(u64::from_be_bytes(bytes));
            }
            Ok(goldilocks_crypto::Fp5Element(message_elements))
        }
        
        let message_fp5 = message_to_fp5(&message).unwrap();
        
        let mut pre_image = [poseidon_hash::Goldilocks::zero(); 10];
        pre_image[..5].copy_from_slice(&r_verification_encoded.0);
        pre_image[5..].copy_from_slice(&message_fp5.0);
        
        let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
        let e_prime_scalar = ScalarField::from_fp5_element(&e_prime_fp5);
        
        println!("\nHash comparison:");
        println!("  e (from signature): {}", hex::encode(&e.to_bytes_le()));
        println!("  e' (computed):      {}", hex::encode(&e_prime_scalar.to_bytes_le()));
        println!("  e == e': {}", e.to_bytes_le() == e_prime_scalar.to_bytes_le());
        println!("  e.equals(&e'): {}", e.equals(&e_prime_scalar));
        
        // Check if it's a limb-by-limb comparison issue
        println!("\nLimb-by-limb comparison:");
        for i in 0..5 {
            if e.0[i] != e_prime_scalar.0[i] {
                println!("  Limb[{}]: e={}, e'={}", 
                    i, e.0[i], e_prime_scalar.0[i]);
            }
        }
        
        // Check pre-image values
        println!("\nPre-image values:");
        for i in 0..10 {
            if i < 5 {
                println!("  Pre-image[{}] = R[{}] = {}", i, i, pre_image[i].0);
            } else {
                println!("  Pre-image[{}] = M[{}] = {}", i, i-5, pre_image[i].0);
            }
        }
        
        // Also check what R was during signing
        println!("\nR during signing (from k):");
        for i in 0..5 {
            println!("  R[{}] = {}", i, r_signing_encoded.0[i].0);
        }
        
        println!("\nMessage Fp5Element:");
        for i in 0..5 {
            println!("  M[{}] = {}", i, message_fp5.0[i].0);
        }
    }
    
    assert!(is_valid, "Signature should verify");
}

