//! Test to verify e*P computation

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_e_p_computation() {
    println!("\n=== Testing e*P Computation ===\n");
    
    let generator = Point::generator();
    
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
    println!("  e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("  sk: {}...", hex::encode(&sk.to_bytes_le()[..8]));
    
    // Method 1: e*P = P.mul(&e)
    let result1 = public_key.mul(&e);
    let encoded1 = result1.encode();
    
    println!("\nMethod 1: P.mul(&e):");
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Method 2: e*P = (e*sk)*G
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let result2 = generator.mul(&e_times_sk_canonical);
    let encoded2 = result2.encode();
    
    println!("\nMethod 2: (e*sk)*G (canonical):");
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  Result: {}...", hex::encode(&encoded2.to_bytes_le()[..16]));
    
    // Method 3: e*P = (e*sk)*G (Montgomery)
    let result3 = generator.mul(&e_times_sk);
    let encoded3 = result3.encode();
    
    println!("\nMethod 3: (e*sk)*G (Montgomery):");
    println!("  e*sk (Montgomery): {}...", hex::encode(&e_times_sk.to_bytes_le()[..8]));
    println!("  Result: {}...", hex::encode(&encoded3.to_bytes_le()[..16]));
    
    // Compare
    let match_1_2 = encoded1.0.iter().zip(encoded2.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_1_3 = encoded1.0.iter().zip(encoded3.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  P.mul(&e) == (e*sk canonical)*G: {}", match_1_2);
    println!("  P.mul(&e) == (e*sk Montgomery)*G: {}", match_1_3);
    
    if !match_1_2 {
        println!("\n❌ P.mul(&e) != (e*sk canonical)*G");
        println!("  This is the bug! Point::mul() may not handle scalars correctly");
    } else {
        println!("\n✅ e*P computation is correct");
    }
}

#[test]
fn test_s_g_computation() {
    println!("\n=== Testing s*G Computation ===\n");
    
    let generator = Point::generator();
    
    let s = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // s = 5
        bytes
    }).unwrap();
    
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    
    // Method 1: s*G
    let result1 = generator.mul(&s);
    let encoded1 = result1.encode();
    
    println!("\nMethod 1: G.mul(&s):");
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Method 2: Compute 5*G by repeated addition
    let mut result2 = generator;
    for _ in 1..5 {
        result2 = result2.add(&generator);
    }
    let encoded2 = result2.encode();
    
    println!("\nMethod 2: G + G + G + G + G:");
    println!("  Result: {}...", hex::encode(&encoded2.to_bytes_le()[..16]));
    
    // Compare
    let match_result = encoded1.0.iter().zip(encoded2.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparison:");
    println!("  G.mul(&s) == 5*G (by addition): {}", match_result);
    
    if !match_result {
        println!("\n❌ G.mul(&s) is incorrect!");
    } else {
        println!("\n✅ G.mul(&s) is correct");
    }
}












