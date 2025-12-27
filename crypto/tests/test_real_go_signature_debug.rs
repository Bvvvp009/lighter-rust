//! Debug real Go signature to understand the exact issue

use goldilocks_crypto::{ScalarField, Point, verify_signature, Fp5Element};
use hex;

#[test]
fn test_real_go_signature_debug() {
    println!("\n=== Debugging Real Go Signature ===\n");
    
    // Real Go test vector from cross_validation.rs
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let public_key_hex = "04000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let public_key = hex::decode(public_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    println!("Inputs:");
    println!("  Private key: {}", private_key_hex);
    println!("  Public key: {}", public_key_hex);
    println!("  Message: {}", message_hex);
    println!("  Signature: {}", signature_hex);
    
    // Extract s and e from signature
    let s_bytes = &signature[..40];
    let e_bytes = &signature[40..];
    let s = ScalarField::from_bytes_le(s_bytes).unwrap();
    let e = ScalarField::from_bytes_le(e_bytes).unwrap();
    
    println!("\nSignature components:");
    println!("  s: {}", hex::encode(s_bytes));
    println!("  e: {}", hex::encode(e_bytes));
    
    // Decode public key
    let public_key_fp5 = Fp5Element::from_bytes_le(&public_key).unwrap();
    let public_point = Point::decode(&public_key_fp5).unwrap();
    
    // Compute expected R from signing process
    // We need to reconstruct what R should be
    // R was computed as: R = nonce * G during signing
    // We can't get nonce directly, but we can verify: s + e*sk = k = nonce
    let private_scalar = ScalarField::from_bytes_le(&private_key).unwrap();
    let e_times_sk = e.mul(&private_scalar);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k_reconstructed = s.add(e_times_sk_canonical);
    
    println!("\nReconstructing k = s + e*sk:");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  k = s + e*sk: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Compute expected R = k*G
    let generator = Point::generator();
    let expected_r = generator.mul(&k_reconstructed);
    let expected_r_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}", hex::encode(&expected_r_encoded.to_bytes_le()));
    
    // Compute R using current verification method
    // FIXED: Point::mul() now normalizes scalars to canonical form before recoding,
    // so e*P correctly equals (e*sk canonical)*G. No adjustment needed.
    let computed_r = Point::mul_add2(&generator, &public_point, &s, &e);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nComputed R (s*G + e*P):");
    println!("  e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("  Encoded: {}", hex::encode(&computed_r_encoded.to_bytes_le()));
    
    // Compare
    let match_result = expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparison:");
    println!("  Expected R == Computed R: {}", match_result);
    
    if !match_result {
        println!("\n❌ R computation is incorrect!");
        println!("  This is why verification fails");
        
        // Try alternative: separate multiplications
        let s_g = generator.mul(&s);
        let e_p = public_point.mul(&e);
        let separate_r = s_g.add(&e_p);
        let separate_r_encoded = separate_r.encode();
        
        println!("\nAlternative: (s*G).add(e*P):");
        println!("  Encoded: {}", hex::encode(&separate_r_encoded.to_bytes_le()));
        let separate_match = expected_r_encoded.0.iter().zip(separate_r_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("  Matches expected: {}", separate_match);
    } else {
        println!("\n✅ R computation is correct!");
        println!("  The issue must be elsewhere in verification");
    }
    
    // Now test actual verification
    let is_valid = verify_signature(&signature, &message, &public_key).unwrap();
    println!("\nActual verification result: {}", is_valid);
}


