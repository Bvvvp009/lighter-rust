//! Compare mul_add2 with separate multiplications to find the bug

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_mul_add2_vs_separate() {
    println!("\n=== Comparing mul_add2 vs Separate Multiplications ===\n");
    
    let generator = Point::generator();
    
    // Use simple test values
    let s = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // s = 5
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    let public_key = generator.mul(&sk); // P = 7*G
    
    println!("Test values:");
    println!("  s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    println!("  e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("  sk: {}...", hex::encode(&sk.to_bytes_le()[..8]));
    
    // Method 1: mul_add2
    let result1 = Point::mul_add2(&generator, &public_key, &s, &e);
    let encoded1 = result1.encode();
    
    println!("\nMethod 1: mul_add2(s*G, e*P):");
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Method 2: Separate multiplications and addition
    let s_g = generator.mul(&s);
    let e_p = public_key.mul(&e);
    let result2 = s_g.add(&e_p);
    let encoded2 = result2.encode();
    
    println!("\nMethod 2: (s*G).add(e*P):");
    println!("  Result: {}...", hex::encode(&encoded2.to_bytes_le()[..16]));
    
    // Method 3: Compute k = s + e*sk, then k*G
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s.add(e_times_sk_canonical);
    let result3 = generator.mul(&k);
    let encoded3 = result3.encode();
    
    println!("\nMethod 3: (s + e*sk)*G:");
    println!("  k = s + e*sk: {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("  Result: {}...", hex::encode(&encoded3.to_bytes_le()[..16]));
    
    // Compare
    let match_1_2 = encoded1.0.iter().zip(encoded2.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_1_3 = encoded1.0.iter().zip(encoded3.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_2_3 = encoded2.0.iter().zip(encoded3.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  mul_add2 == separate: {}", match_1_2);
    println!("  mul_add2 == k*G: {}", match_1_3);
    println!("  separate == k*G: {}", match_2_3);
    
    if !match_2_3 {
        println!("\n❌ Even separate multiplication fails!");
        println!("  This suggests the issue is in Point::mul() or Point::add()");
    } else if !match_1_2 {
        println!("\n❌ mul_add2 != separate multiplication");
        println!("  This suggests the bug is specifically in mul_add2");
    } else {
        println!("\n✅ All methods match!");
    }
}








