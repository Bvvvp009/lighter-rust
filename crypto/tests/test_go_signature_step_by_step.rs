//! Step-by-step comparison with Go signature generation using real test vector

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use goldilocks_crypto::schnorr::sign_with_nonce;
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use hex;

#[test]
fn test_go_signature_step_by_step() {
    println!("\n=== Step-by-Step Go Signature Analysis ===\n");
    
    // Real Go test vector
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let private_scalar = ScalarField::from_bytes_le(&private_key).unwrap();
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    println!("Inputs:");
    println!("  Private key: {}", private_key_hex);
    println!("  Message: {}", message_hex);
    println!("  Signature: {}", signature_hex);
    println!("  s: {}", hex::encode(&s.to_bytes_le()));
    println!("  e: {}", hex::encode(&e.to_bytes_le()));
    
    // Reconstruct k = s + e*sk
    let e_times_sk = e.mul(&private_scalar);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s.add(e_times_sk_canonical);
    
    println!("\nReconstructing k:");
    println!("  e*sk (Montgomery): {}...", hex::encode(&e_times_sk.to_bytes_le()[..8]));
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  k = s + e*sk: {}...", hex::encode(&k.to_bytes_le()[..8]));
    
    // Compute R = k*G (what R should be)
    let generator = Point::generator();
    let expected_r = generator.mul(&k);
    let expected_r_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}", hex::encode(&expected_r_encoded.to_bytes_le()));
    
    // Compute e' = H(R || message) to verify
    let message_fp5 = Fp5Element::from_bytes_le(&message).unwrap();
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&expected_r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
    let e_prime = ScalarField::from_fp5_element(&e_prime_fp5);
    
    println!("\nComputed e' from expected R:");
    println!("  e': {}", hex::encode(&e_prime.to_bytes_le()));
    println!("  e (from signature): {}", hex::encode(&e.to_bytes_le()));
    println!("  Match: {}", e.0 == e_prime.0);
    
    if e.0 == e_prime.0 {
        println!("\n✅ Expected R is correct! The issue is in how we compute R during verification");
    } else {
        println!("\n❌ Even expected R doesn't match - there's an issue with k reconstruction");
    }
    
    // Now try to compute R using verification method
    let public_key_point = generator.mul(&private_scalar);
    let e_adjusted = e.mul_canonical(&ScalarField::R2_INV);
    let computed_r = Point::mul_add2(&generator, &public_key_point, &s, &e_adjusted);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nComputed R (verification method):");
    println!("  Encoded: {}", hex::encode(&computed_r_encoded.to_bytes_le()));
    println!("  Matches expected: {}", expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0));
}












