//! Test to verify Point::mul() works correctly with canonical scalars

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_point_mul_with_canonical_scalars() {
    println!("\n=== Testing Point::mul() with Canonical Scalars ===\n");
    
    let generator = Point::generator();
    
    // Test 1: Simple scalar = 1
    let scalar_one = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let result1 = generator.mul(&scalar_one);
    let expected1 = generator;
    
    println!("Test 1: 1 * G");
    println!("  Result: {}...", hex::encode(&result1.encode().to_bytes_le()[..16]));
    println!("  Expected: {}...", hex::encode(&expected1.encode().to_bytes_le()[..16]));
    println!("  Match: {}", result1.encode().0.iter().zip(expected1.encode().0.iter())
        .all(|(a, b)| a.0 == b.0));
    
    // Test 2: Scalar = 2
    let scalar_two = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 2;
        bytes
    }).unwrap();
    
    let result2 = generator.mul(&scalar_two);
    let expected2 = generator.double();
    
    println!("\nTest 2: 2 * G");
    println!("  Result: {}...", hex::encode(&result2.encode().to_bytes_le()[..16]));
    println!("  Expected (G.double()): {}...", hex::encode(&expected2.encode().to_bytes_le()[..16]));
    println!("  Match: {}", result2.encode().0.iter().zip(expected2.encode().0.iter())
        .all(|(a, b)| a.0 == b.0));
    
    // Test 3: Verify s*G + e*P = (s + e*sk)*G
    let s = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5;
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3;
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7;
        bytes
    }).unwrap();
    
    let public_key = generator.mul(&sk);
    
    println!("\nTest 3: Verify s*G + e*P = (s + e*sk)*G");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("  sk: {}...", hex::encode(&sk.to_bytes_le()[..8]));
    
    // Compute s*G + e*P using mul_add2
    let result_mul_add2 = Point::mul_add2(&generator, &public_key, &s, &e);
    
    // Compute (s + e*sk)*G
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s.add(e_times_sk_canonical);
    let expected_k_g = generator.mul(&k);
    
    println!("\n  s + e*sk = k: {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("  mul_add2 result: {}...", hex::encode(&result_mul_add2.encode().to_bytes_le()[..16]));
    println!("  k*G result: {}...", hex::encode(&expected_k_g.encode().to_bytes_le()[..16]));
    
    let match_result = result_mul_add2.encode().0.iter().zip(expected_k_g.encode().0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("  Match: {}", match_result);
    
    if !match_result {
        println!("\n  ❌ mul_add2 does NOT equal k*G!");
        println!("  This confirms the bug in mul_add2!");
    }
}












