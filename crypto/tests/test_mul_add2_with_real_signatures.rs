//! Test mul_add2 with real signature values to identify the issue

use goldilocks_crypto::{ScalarField, Point, sign};
use hex;

#[test]
fn test_mul_add2_with_real_signature() {
    println!("\n=== Testing mul_add2 with Real Signature Values ===\n");
    
    // Generate a key pair and sign a message
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Sign the message
    let signature = sign(&private_key_bytes, &message).unwrap();
    
    // Extract s and e
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    println!("Signature components:");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("  private_key: {}...", hex::encode(&private_key_bytes[..8]));
    
    // Verify scalar arithmetic: k = s + e*sk
    let e_times_sk = e.mul(&private_key);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k_reconstructed = s.add(e_times_sk_canonical);
    
    println!("\nScalar arithmetic:");
    println!("  e*sk (Montgomery): {}...", hex::encode(&e_times_sk.to_bytes_le()[..8]));
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  k_reconstructed: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Compute expected: k*G
    let generator = Point::generator();
    let expected_r = generator.mul(&k_reconstructed);
    let expected_r_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}...", hex::encode(&expected_r_encoded.to_bytes_le()[..16]));
    
    // Compute with mul_add2: s*G + e*P
    let computed_r = Point::mul_add2(&generator, &public_key_point, &s, &e);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nComputed R (s*G + e*P using mul_add2):");
    println!("  Encoded: {}...", hex::encode(&computed_r_encoded.to_bytes_le()[..16]));
    
    // Compare
    let match_result = expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ mul_add2 correctly computes s*G + e*P = k*G");
    } else {
        println!("\n❌ mul_add2 does NOT correctly compute s*G + e*P = k*G");
        println!("  This is the bug!");
        
        // Detailed comparison
        for i in 0..5 {
            let expected = expected_r_encoded.0[i].0;
            let computed = computed_r_encoded.0[i].0;
            if expected != computed {
                println!("  Element[{}]: expected=0x{:016x}, computed=0x{:016x}, diff=0x{:016x}",
                    i, expected, computed, expected ^ computed);
            }
        }
        
        // Also try computing separately: s*G and e*P, then add
        let s_g = generator.mul(&s);
        let e_p = public_key_point.mul(&e);
        let separate_result = s_g.add(&e_p);
        let separate_encoded = separate_result.encode();
        
        println!("\nAlternative: (s*G).add(e*P):");
        println!("  Encoded: {}...", hex::encode(&separate_encoded.to_bytes_le()[..16]));
        
        let match_separate = expected_r_encoded.0.iter().zip(separate_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("  Matches expected: {}", match_separate);
        println!("  Matches mul_add2: {}", computed_r_encoded.0.iter().zip(separate_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0));
    }
    
    // This should pass if mul_add2 is correct
    assert!(match_result, "mul_add2 should correctly compute s*G + e*P = k*G");
}








