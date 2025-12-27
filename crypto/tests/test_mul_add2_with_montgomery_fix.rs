//! Test mul_add2 with Montgomery form scalars to verify the fix

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_mul_add2_with_montgomery_fix() {
    println!("\n=== Testing mul_add2 with Montgomery Form Fix ===\n");
    
    let generator = Point::generator();
    
    // Test values
    let s_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // s = 5
        bytes
    }).unwrap();
    
    let e_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    let sk_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    // Convert to Montgomery
    let s_montgomery = s_canonical.monty_mul(&ScalarField::R2);
    let e_montgomery = e_canonical.monty_mul(&ScalarField::R2);
    
    let public_key = generator.mul(&sk_canonical);
    
    // Compute expected: k = s + e*sk (canonical)
    let e_times_sk = e_canonical.mul(&sk_canonical);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s_canonical.add(e_times_sk_canonical);
    let expected_r = generator.mul(&k);
    let expected_encoded = expected_r.encode();
    
    println!("Expected R (k*G where k = s + e*sk canonical):");
    println!("  k: {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("  R: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    // Test: mul_add2 with Montgomery scalars
    let result = Point::mul_add2(&generator, &public_key, &s_montgomery, &e_montgomery);
    let result_encoded = result.encode();
    
    println!("\nmul_add2 with Montgomery scalars:");
    println!("  Result: {}...", hex::encode(&result_encoded.to_bytes_le()[..16]));
    
    // Also test separate: s*G + e*P with Montgomery
    let s_g = generator.mul(&s_montgomery);
    let e_p = public_key.mul(&e_montgomery);
    let separate_result = s_g.add(&e_p);
    let separate_encoded = separate_result.encode();
    
    println!("\nSeparate (s*G).add(e*P) with Montgomery scalars:");
    println!("  Result: {}...", hex::encode(&separate_encoded.to_bytes_le()[..16]));
    
    // Compare
    let match_mul_add2 = result_encoded.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_separate = separate_encoded.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  mul_add2 (Montgomery) == k*G: {}", match_mul_add2);
    println!("  separate (Montgomery) == k*G: {}", match_separate);
    
    if match_mul_add2 {
        println!("\n✅ FIX WORKS: mul_add2 with Montgomery scalars produces correct result!");
    } else {
        println!("\n❌ Fix doesn't work - need to investigate further");
        
        // Check if s*G is correct
        let s_g_canonical = generator.mul(&s_canonical);
        let s_g_canonical_encoded = s_g_canonical.encode();
        let s_g_encoded = s_g.encode();
        let s_g_match = s_g_canonical_encoded.0.iter().zip(s_g_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("  s*G (canonical) == s*G (Montgomery): {}", s_g_match);
        
        if !s_g_match {
            println!("  ❌ s*G computation differs between canonical and Montgomery!");
        }
    }
}

