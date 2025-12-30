//! Debug test to understand mul_add2 behavior with actual signature values

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use goldilocks_crypto::schnorr::sign_with_nonce;
use hex;

#[test]
fn test_mul_add2_with_signature_values() {
    println!("\n=== Testing mul_add2 with Actual Signature Values ===\n");
    
    let mut private_key_bytes = [0u8; 40];
    private_key_bytes[0] = 1;
    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes).unwrap();
    
    let public_key_point = Point::generator().mul(&private_scalar);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Use fixed nonce for reproducibility
    let mut nonce_bytes = [0u8; 40];
    nonce_bytes[0] = 1;
    let nonce_scalar = ScalarField::from_bytes_le(&nonce_bytes).unwrap();
    
    // Sign
    let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
    
    // Extract s and e
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];
    let s = ScalarField::from_bytes_le(s_bytes).unwrap();
    let e = ScalarField::from_bytes_le(e_bytes).unwrap();
    
    println!("Values:");
    println!("  k (nonce): {:?}", nonce_scalar.0);
    println!("  sk (private): {:?}", private_scalar.0);
    println!("  s: {:?}", s.0);
    println!("  e: {:?}", e.0);
    
    // Check 4-bit limb splitting
    let s_limbs = s.split_to_4bit_limbs();
    let e_limbs = e.split_to_4bit_limbs();
    
    println!("\n4-bit limbs (first 10 and last 10):");
    println!("  s_limbs[0..10]: {:?}", &s_limbs[0..10]);
    println!("  s_limbs[70..80]: {:?}", &s_limbs[70..80]);
    println!("  e_limbs[0..10]: {:?}", &e_limbs[0..10]);
    println!("  e_limbs[70..80]: {:?}", &e_limbs[70..80]);
    
    // Expected: k*G
    let generator = Point::generator();
    let expected_r = generator.mul(&nonce_scalar);
    let expected_r_encoded = expected_r.encode();
    
    // Computed using mul_add2
    let public_point = Point::decode(&Fp5Element::from_bytes_le(&public_key_bytes).unwrap()).unwrap();
    let computed_r = Point::mul_add2(&generator, &public_point, &s, &e);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nR comparison:");
    println!("  Expected (k*G): {}", hex::encode(&expected_r_encoded.to_bytes_le()));
    println!("  Computed (s*G + e*P): {}", hex::encode(&computed_r_encoded.to_bytes_le()));
    
    // Also try computing separately to see where it diverges
    let s_g = generator.mul(&s);
    let e_p = public_point.mul(&e);
    println!("  s*G: {}", hex::encode(&s_g.encode().to_bytes_le()));
    println!("  e*P: {}", hex::encode(&e_p.encode().to_bytes_le()));
    
    // Check if s + e*sk = k (should be true)
    let e_times_sk = e.mul(&private_scalar);
    let s_plus_e_times_sk = s.add(e_times_sk);
    println!("\nVerification of s + e*sk = k:");
    println!("  s + e*sk: {:?}", s_plus_e_times_sk.0);
    println!("  k: {:?}", nonce_scalar.0);
    println!("  Match: {}", s_plus_e_times_sk.0 == nonce_scalar.0);
    
    let match_result = expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ R matches!");
    } else {
        println!("\n❌ R does NOT match!");
        println!("  Expected limbs: {:?}", expected_r_encoded.0);
        println!("  Computed limbs: {:?}", computed_r_encoded.0);
    }
}













