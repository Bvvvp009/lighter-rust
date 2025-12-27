//! Debug s computation to understand why it's incorrect

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

#[test]
fn test_s_computation_debug() {
    println!("\n=== Debugging s Computation ===\n");
    
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Sign
    println!("Signing...");
    let signature = sign(&private_key_bytes, &message).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[0..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..80]).unwrap();
    
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    
    // Reconstruct what should have happened during signing
    // We need to trace through sign() step by step
    
    // Step 1: Generate nonce (we can't know the actual nonce, but we can verify the relationship)
    // Step 2: Compute R = k*G
    // Step 3: Compute e = H(R || message)
    // Step 4: Compute s = k - e*sk
    
    // From the signature, we have s and e
    // We know: s = k - e*sk, so k = s + e*sk
    let e_sk = e.mul(&private_key);
    let k_reconstructed = s.add(e_sk);
    
    println!("\nReconstructing k = s + e*sk:");
    println!("  e*sk: {}...", hex::encode(&e_sk.to_bytes_le()[..8]));
    println!("  k_reconstructed: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Compute R from k
    let generator = Point::generator();
    let r_from_k = generator.mul(&k_reconstructed);
    let r_from_k_encoded = r_from_k.encode();
    
    println!("\nR from k_reconstructed:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_from_k_encoded.0[i].0);
    }
    
    // Compute e' from R
    fn message_to_fp5(message: &[u8]) -> goldilocks_crypto::Fp5Element {
        let mut message_elements = [Goldilocks::zero(); 5];
        for (i, chunk) in message.chunks(8).enumerate().take(5) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            bytes.reverse();
            message_elements[i] = Goldilocks::from_canonical_u64(u64::from_be_bytes(bytes));
        }
        goldilocks_crypto::Fp5Element(message_elements)
    }
    
    let message_fp5 = message_to_fp5(&message);
    
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_from_k_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_from_r_fp5 = hash_to_quintic_extension(&pre_image);
    let e_from_r_scalar = ScalarField::from_fp5_element(&e_from_r_fp5);
    
    println!("\ne computed from R (k_reconstructed):");
    println!("  e_from_r: {}", hex::encode(&e_from_r_scalar.to_bytes_le()));
    println!("  e from signature: {}", hex::encode(&e.to_bytes_le()));
    println!("  Match: {}", e_from_r_scalar.to_bytes_le() == e.to_bytes_le());
    
    if e_from_r_scalar.to_bytes_le() != e.to_bytes_le() {
        println!("  ❌ e from R doesn't match e from signature!");
        println!("  This means either:");
        println!("    1. s is wrong (so k_reconstructed is wrong, so R is wrong)");
        println!("    2. e in signature is wrong");
        println!("    3. The relationship s = k - e*sk doesn't hold");
    }
    
    // Now verify using the signature
    println!("\nVerifying signature...");
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    println!("Result: {}", is_valid);
    
    if !is_valid {
        // During verification, compute R = s*G + e*P
        let public_key_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&public_key_bytes).unwrap();
        let public_point = Point::decode(&public_key_fp5).unwrap();
        let r_verification = Point::mul_add2(&generator, &public_point, &s, &e);
        let r_verification_encoded = r_verification.encode();
        
        println!("\nR from verification (s*G + e*P):");
        for i in 0..5 {
            println!("  R[{}] = {}", i, r_verification_encoded.0[i].0);
        }
        
        // Compare R values
        println!("\nR comparison:");
        let r_match = r_from_k_encoded.to_bytes_le() == r_verification_encoded.to_bytes_le();
        println!("  R from k_reconstructed == R from s*G + e*P: {}", r_match);
        
        if !r_match {
            println!("  ❌ R values don't match!");
            println!("  This confirms that s is incorrect or the relationship doesn't hold");
            
            // Check if s = k - e*sk holds
            println!("\n  Checking relationship s = k - e*sk:");
            let s_computed = k_reconstructed.sub(e_sk);
            println!("    s (from signature): {}...", hex::encode(&s.to_bytes_le()[..8]));
            println!("    s (computed from k - e*sk): {}...", hex::encode(&s_computed.to_bytes_le()[..8]));
            println!("    Match: {}", s.to_bytes_le() == s_computed.to_bytes_le());
        }
    }
}



