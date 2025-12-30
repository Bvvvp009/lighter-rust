//! Test to verify if converting scalars to Montgomery form fixes mul_add2

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_mul_add2_with_montgomery_scalars() {
    println!("\n=== Testing mul_add2 with Montgomery Form Scalars ===\n");
    
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
    
    let public_key = generator.mul(&sk_canonical); // P = 7*G
    
    println!("Test values (canonical):");
    println!("  s: {}...", hex::encode(&s_canonical.to_bytes_le()[..8]));
    println!("  e: {}...", hex::encode(&e_canonical.to_bytes_le()[..8]));
    println!("  sk: {}...", hex::encode(&sk_canonical.to_bytes_le()[..8]));
    
    // Convert to Montgomery form
    let s_montgomery = s_canonical.monty_mul(&ScalarField::R2);
    let e_montgomery = e_canonical.monty_mul(&ScalarField::R2);
    
    println!("\nMontgomery forms:");
    println!("  s (Montgomery): {}...", hex::encode(&s_montgomery.to_bytes_le()[..8]));
    println!("  e (Montgomery): {}...", hex::encode(&e_montgomery.to_bytes_le()[..8]));
    
    // Compute expected: k = s + e*sk (canonical)
    let e_times_sk = e_canonical.mul(&sk_canonical);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s_canonical.add(e_times_sk_canonical);
    let expected_r = generator.mul(&k);
    let expected_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G where k = s + e*sk canonical):");
    println!("  k: {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("  R: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    // Test 1: mul_add2 with canonical scalars
    let result1 = Point::mul_add2(&generator, &public_key, &s_canonical, &e_canonical);
    let encoded1 = result1.encode();
    
    println!("\nTest 1: mul_add2 with canonical scalars:");
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Test 2: mul_add2 with Montgomery scalars
    let result2 = Point::mul_add2(&generator, &public_key, &s_montgomery, &e_montgomery);
    let encoded2 = result2.encode();
    
    println!("\nTest 2: mul_add2 with Montgomery scalars:");
    println!("  Result: {}...", hex::encode(&encoded2.to_bytes_le()[..16]));
    
    // Test 3: Separate multiplication (s*G + e*P) with canonical
    let s_g = generator.mul(&s_canonical);
    let e_p = public_key.mul(&e_canonical);
    let result3 = s_g.add(&e_p);
    let encoded3 = result3.encode();
    
    println!("\nTest 3: (s*G).add(e*P) with canonical scalars:");
    println!("  Result: {}...", hex::encode(&encoded3.to_bytes_le()[..16]));
    
    // Compare
    let match_1 = encoded1.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_2 = encoded2.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_3 = encoded3.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  mul_add2 (canonical) == k*G: {}", match_1);
    println!("  mul_add2 (Montgomery) == k*G: {}", match_2);
    println!("  separate (canonical) == k*G: {}", match_3);
    
    if match_2 && !match_1 {
        println!("\n✅ FOUND THE FIX: mul_add2 needs Montgomery form scalars!");
    } else if match_1 {
        println!("\n✅ mul_add2 works with canonical scalars");
    } else {
        println!("\n❌ Neither form works - there's a different bug");
    }
}












