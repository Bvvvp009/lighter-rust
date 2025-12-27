//! Trace exact signing process and compare with verification

use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

#[test]
fn test_sign_verify_trace() {
    println!("\n=== Sign and Verify Trace ===\n");
    
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // We need to trace the exact signing process
    // Since sign() generates a random nonce, we can't directly trace it
    // But we can sign and then verify what happened
    
    println!("Signing...");
    let signature = sign(&private_key_bytes, &message).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[0..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..80]).unwrap();
    
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    
    // Reconstruct k = s + e*sk
    // But wait - if e in signature is wrong, then k_reconstructed will be wrong!
    // Let's check if s = k - e*sk holds for the e in the signature
    let e_sk = e.mul(&private_key);
    let k_reconstructed = s.add(e_sk);
    
    println!("\nReconstructing k = s + e*sk:");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e*sk: {}...", hex::encode(&e_sk.to_bytes_le()[..8]));
    println!("  k_reconstructed: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Compute R from k (what signing did)
    let generator = Point::generator();
    let r_signing = generator.mul(&k_reconstructed);
    let r_signing_encoded = r_signing.encode();
    
    // Also compute R directly from s and e (what verification does)
    let public_key_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&public_key_bytes).unwrap();
    let public_point = Point::decode(&public_key_fp5).unwrap();
    let r_verification_direct = Point::mul_add2(&generator, &public_point, &s, &e);
    let r_verification_direct_encoded = r_verification_direct.encode();
    
    println!("\nR comparison:");
    println!("  R from k_reconstructed: {}...", hex::encode(&r_signing_encoded.to_bytes_le()[..16]));
    println!("  R from s*G + e*P: {}...", hex::encode(&r_verification_direct_encoded.to_bytes_le()[..16]));
    println!("  Match: {}", r_signing_encoded.to_bytes_le() == r_verification_direct_encoded.to_bytes_le());
    
    println!("\nR from signing (k*G):");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_signing_encoded.0[i].0);
    }
    
    // Message encoding
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
    println!("\nMessage Fp5Element:");
    for i in 0..5 {
        println!("  M[{}] = {}", i, message_fp5.0[i].0);
    }
    
    // Pre-image for hash (what signing used)
    let mut pre_image_signing = [Goldilocks::zero(); 10];
    pre_image_signing[..5].copy_from_slice(&r_signing_encoded.0);
    pre_image_signing[5..].copy_from_slice(&message_fp5.0);
    
    println!("\nPre-image for hash (during signing):");
    for i in 0..10 {
        println!("  Pre-image[{}] = {}", i, pre_image_signing[i].0);
    }
    
        // Hash (what signing computed)
        let e_signing_fp5 = hash_to_quintic_extension(&pre_image_signing);
        let e_signing_scalar = ScalarField::from_fp5_element(&e_signing_fp5);
        
        println!("\nHash result (e from signing):");
        println!("  e_fp5 limbs: {:?}", e_signing_fp5.0.iter().map(|g| g.0).collect::<Vec<_>>());
        println!("  e_scalar: {}", hex::encode(&e_signing_scalar.to_bytes_le()));
        println!("  e from signature: {}", hex::encode(&e.to_bytes_le()));
        println!("  Match: {}", e_signing_scalar.to_bytes_le() == e.to_bytes_le());
        
        if e_signing_scalar.to_bytes_le() != e.to_bytes_le() {
            println!("  ❌ e from hash doesn't match e from signature!");
            println!("  This means either:");
            println!("    1. The hash is non-deterministic (unlikely)");
            println!("    2. There's a bug in from_fp5_element");
            println!("    3. The signature was created with different code");
        }
    
    // Now verify
    println!("\nVerifying...");
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    println!("Result: {}", is_valid);
    
    if !is_valid {
        // During verification, compute R
        let public_key_fp5 = goldilocks_crypto::Fp5Element::from_bytes_le(&public_key_bytes).unwrap();
        let public_point = Point::decode(&public_key_fp5).unwrap();
        let r_verification = Point::mul_add2(&generator, &public_point, &s, &e);
        let r_verification_encoded = r_verification.encode();
        
        println!("\nR from verification (s*G + e*P):");
        for i in 0..5 {
            println!("  R[{}] = {}", i, r_verification_encoded.0[i].0);
        }
        
        // Pre-image for hash (what verification uses)
        let mut pre_image_verification = [Goldilocks::zero(); 10];
        pre_image_verification[..5].copy_from_slice(&r_verification_encoded.0);
        pre_image_verification[5..].copy_from_slice(&message_fp5.0);
        
        println!("\nPre-image for hash (during verification):");
        for i in 0..10 {
            println!("  Pre-image[{}] = {}", i, pre_image_verification[i].0);
        }
        
        // Compare pre-images
        let pre_image_match = pre_image_signing.iter().zip(pre_image_verification.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("\nPre-image match: {}", pre_image_match);
        
        if !pre_image_match {
            println!("❌ Pre-images differ!");
            for i in 0..10 {
                if pre_image_signing[i].0 != pre_image_verification[i].0 {
                    println!("  Pre-image[{}]: signing={}, verification={}", 
                        i, pre_image_signing[i].0, pre_image_verification[i].0);
                }
            }
        } else {
            println!("✅ Pre-images match, but hash differs - this is a hash bug!");
        }
        
        // Hash (what verification computes)
        let e_verification_fp5 = hash_to_quintic_extension(&pre_image_verification);
        let e_verification_scalar = ScalarField::from_fp5_element(&e_verification_fp5);
        
        println!("\nHash result (e' from verification):");
        println!("  e'_fp5 limbs: {:?}", e_verification_fp5.0.iter().map(|g| g.0).collect::<Vec<_>>());
        println!("  e'_scalar: {}", hex::encode(&e_verification_scalar.to_bytes_le()));
        println!("  e from signature: {}", hex::encode(&e.to_bytes_le()));
        println!("  Match: {}", e_verification_scalar.to_bytes_le() == e.to_bytes_le());
        
        // Compare Fp5 elements
        println!("\nFp5 element comparison:");
        let e_fp5_match = e_signing_fp5.0.iter().zip(e_verification_fp5.0.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("  e_fp5 (signing) == e'_fp5 (verification): {}", e_fp5_match);
        
        if !e_fp5_match {
            println!("  ❌ Fp5 elements differ even though pre-images match!");
            for i in 0..5 {
                if e_signing_fp5.0[i].0 != e_verification_fp5.0[i].0 {
                    println!("    Fp5[{}]: signing={}, verification={}", 
                        i, e_signing_fp5.0[i].0, e_verification_fp5.0[i].0);
                }
            }
        } else {
            println!("  ✅ Fp5 elements match!");
            
            // If Fp5 elements match, scalars should match too
            println!("\n  Testing from_fp5_element on same Fp5Element:");
            let e_test1 = ScalarField::from_fp5_element(&e_signing_fp5);
            let e_test2 = ScalarField::from_fp5_element(&e_signing_fp5);
            println!("    e_test1: {}", hex::encode(&e_test1.to_bytes_le()[..16]));
            println!("    e_test2: {}", hex::encode(&e_test2.to_bytes_le()[..16]));
            println!("    e_test1 == e_test2: {}", e_test1.to_bytes_le() == e_test2.to_bytes_le());
            println!("    e_test1 == e (from signature): {}", e_test1.to_bytes_le() == e.to_bytes_le());
            println!("    e_test1 == e'_scalar: {}", e_test1.to_bytes_le() == e_verification_scalar.to_bytes_le());
        }
    }
    
    assert!(is_valid, "Signature should verify");
}

