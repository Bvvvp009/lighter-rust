//! Test to verify that set_m_double(4) is equivalent to 4 separate double() calls

use goldilocks_crypto::{Point, ScalarField};

#[test]
fn test_set_m_double_vs_multiple_doubles() {
    println!("\n=== Testing set_m_double(4) vs 4×double() ===\n");
    
    let generator = Point::generator();
    
    // Test 1: set_m_double(4)
    let result1 = generator.set_m_double(4);
    let encoded1 = result1.encode();
    
    // Test 2: 4 separate double() calls
    let mut result2 = generator;
    for _ in 0..4 {
        result2 = result2.double();
    }
    let encoded2 = result2.encode();
    
    println!("Generator point:");
    println!("  Encoded: {}", hex::encode(&generator.encode().to_bytes_le()));
    
    println!("\nset_m_double(4) result:");
    println!("  Encoded: {}", hex::encode(&encoded1.to_bytes_le()));
    
    println!("\n4×double() result:");
    println!("  Encoded: {}", hex::encode(&encoded2.to_bytes_le()));
    
    // Compare
    let match_result = encoded1.0.iter().zip(encoded2.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ set_m_double(4) == 4×double()");
    } else {
        println!("\n❌ set_m_double(4) != 4×double()");
        println!("  This could be the source of verification failures!");
    }
    
    // Also test with a random point
    println!("\n=== Testing with random point ===\n");
    let random_scalar = ScalarField::sample_crypto();
    let random_point = generator.mul(&random_scalar);
    
    let result3 = random_point.set_m_double(4);
    let encoded3 = result3.encode();
    
    let mut result4 = random_point;
    for _ in 0..4 {
        result4 = result4.double();
    }
    let encoded4 = result4.encode();
    
    println!("Random point:");
    println!("  Encoded: {}", hex::encode(&random_point.encode().to_bytes_le()));
    
    println!("\nset_m_double(4) result:");
    println!("  Encoded: {}", hex::encode(&encoded3.to_bytes_le()));
    
    println!("\n4×double() result:");
    println!("  Encoded: {}", hex::encode(&encoded4.to_bytes_le()));
    
    let match_result2 = encoded3.0.iter().zip(encoded4.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result2 {
        println!("\n✅ set_m_double(4) == 4×double() (random point)");
    } else {
        println!("\n❌ set_m_double(4) != 4×double() (random point)");
        println!("  This confirms the issue!");
    }
    
    assert!(match_result && match_result2, "set_m_double(4) should equal 4×double()");
}












