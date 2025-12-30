//! Debug Point::add() to see if it's the issue

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_point_add_debug() {
    println!("\n=== Debugging Point::add() ===\n");
    
    let generator = Point::generator();
    
    // Test values
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
    
    // Compute points
    let s_g = generator.mul(&s);
    let public_key = generator.mul(&sk);
    let e_p = public_key.mul(&e);
    
    println!("Points:");
    println!("  s*G encoded: {}...", hex::encode(&s_g.encode().to_bytes_le()[..16]));
    println!("  e*P encoded: {}...", hex::encode(&e_p.encode().to_bytes_le()[..16]));
    
    // Test addition
    let result = s_g.add(&e_p);
    let result_encoded = result.encode();
    
    println!("\n(s*G).add(e*P):");
    println!("  Result: {}...", hex::encode(&result_encoded.to_bytes_le()[..16]));
    
    // Check if result is valid (not all 0xffff...)
    let is_valid = !result_encoded.0.iter().all(|elem| elem.0 == 0xFFFFFFFFFFFFFFFF);
    println!("  Is valid point: {}", is_valid);
    
    // Compare with expected
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s.add(e_times_sk_canonical);
    let expected = generator.mul(&k);
    let expected_encoded = expected.encode();
    
    println!("\nExpected (k*G):");
    println!("  Result: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    let match_result = result_encoded.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparison:");
    println!("  (s*G).add(e*P) == k*G: {}", match_result);
    
    if !match_result && !is_valid {
        println!("\n❌ Point::add() produces invalid point!");
        println!("   This is the bug - Point::add() fails when adding s*G and e*P");
    }
}












