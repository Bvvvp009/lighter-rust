//! Test if Point::mul() expects Montgomery form scalars

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_point_mul_with_montgomery_conversion() {
    println!("\n=== Testing Point::mul() with Montgomery Conversion ===\n");
    
    let generator = Point::generator();
    
    // Test scalar = 3
    let e_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3;
        bytes
    }).unwrap();
    
    println!("e (canonical): {}...", hex::encode(&e_canonical.to_bytes_le()[..8]));
    
    // Convert to Montgomery form
    let e_montgomery = e_canonical.monty_mul(&ScalarField::R2);
    println!("e (Montgomery): {}...", hex::encode(&e_montgomery.to_bytes_le()[..8]));
    
    // Test 1: generator.mul(&e_canonical)
    let result1 = generator.mul(&e_canonical);
    let encoded1 = result1.encode();
    
    println!("\nTest 1: G.mul(&e_canonical):");
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Test 2: generator.mul(&e_montgomery)  
    let result2 = generator.mul(&e_montgomery);
    let encoded2 = result2.encode();
    
    println!("\nTest 2: G.mul(&e_montgomery):");
    println!("  Result: {}...", hex::encode(&encoded2.to_bytes_le()[..16]));
    
    // Test 3: Compute 3*G by addition
    let mut result3 = generator;
    for _ in 1..3 {
        result3 = result3.add(&generator);
    }
    let encoded3 = result3.encode();
    
    println!("\nTest 3: 3*G (by addition):");
    println!("  Result: {}...", hex::encode(&encoded3.to_bytes_le()[..16]));
    
    // Compare
    let match_1_2 = encoded1.0.iter().zip(encoded2.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_1_3 = encoded1.0.iter().zip(encoded3.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_2_3 = encoded2.0.iter().zip(encoded3.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  canonical == Montgomery: {}", match_1_2);
    println!("  canonical == 3*G: {}", match_1_3);
    println!("  Montgomery == 3*G: {}", match_2_3);
    
    if match_2_3 && !match_1_3 {
        println!("\n❌ Point::mul() expects Montgomery form scalars!");
        println!("  This is the bug - we're passing canonical scalars!");
    } else if match_1_3 {
        println!("\n✅ Point::mul() works with canonical scalars");
    }
}








