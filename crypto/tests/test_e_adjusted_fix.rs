//! Test if e_adjusted = e * R2_INV fixes the issue

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_e_adjusted_fix() {
    println!("\n=== Testing e_adjusted Fix ===\n");
    
    let generator = Point::generator();
    
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
    
    let public_key = generator.mul(&sk_canonical);
    
    println!("Test: e*P should equal (e*sk canonical)*G");
    println!("  e (canonical): {}...", hex::encode(&e_canonical.to_bytes_le()[..8]));
    println!("  sk (canonical): {}...", hex::encode(&sk_canonical.to_bytes_le()[..8]));
    
    // Compute expected: (e*sk canonical)*G
    let e_times_sk = e_canonical.mul(&sk_canonical);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let expected = generator.mul(&e_times_sk_canonical);
    let expected_encoded = expected.encode();
    
    println!("\nExpected (e*sk canonical)*G:");
    println!("  Result: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    // Test 1: e*P (current buggy behavior)
    let e_p = public_key.mul(&e_canonical);
    let e_p_encoded = e_p.encode();
    
    println!("\nTest 1: e*P (canonical e):");
    println!("  Result: {}...", hex::encode(&e_p_encoded.to_bytes_le()[..16]));
    
    // Test 2: e_adjusted*P where e_adjusted = e * R2_INV
    // We need to compute e * R2_INV mod N in canonical form
    // Since both are canonical, we can: convert to Montgomery, multiply, convert back
    let e_montgomery = e_canonical.monty_mul(&ScalarField::R2);
    let r2_inv_montgomery = ScalarField::R2_INV.monty_mul(&ScalarField::R2);
    let product_montgomery = e_montgomery.monty_mul(&r2_inv_montgomery);
    let e_adjusted = product_montgomery.to_canonical();
    
    println!("\nComputing e_adjusted = e * R2_INV:");
    println!("  e (Montgomery): {}...", hex::encode(&e_montgomery.to_bytes_le()[..8]));
    println!("  R2_INV (Montgomery): {}...", hex::encode(&r2_inv_montgomery.to_bytes_le()[..8]));
    println!("  e_adjusted (canonical): {}...", hex::encode(&e_adjusted.to_bytes_le()[..8]));
    
    let e_adjusted_p = public_key.mul(&e_adjusted);
    let e_adjusted_p_encoded = e_adjusted_p.encode();
    
    println!("\nTest 2: e_adjusted*P:");
    println!("  Result: {}...", hex::encode(&e_adjusted_p_encoded.to_bytes_le()[..16]));
    
    // Compare
    let match_1 = e_p_encoded.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_2 = e_adjusted_p_encoded.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  e*P == (e*sk canonical)*G: {}", match_1);
    println!("  e_adjusted*P == (e*sk canonical)*G: {}", match_2);
    
    if match_2 {
        println!("\n✅ FIX WORKS: e_adjusted*P produces correct result!");
    } else {
        println!("\n❌ Fix doesn't work - need different approach");
    }
}












