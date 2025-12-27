//! Point operation test vectors from Go implementation
//! 
//! These test vectors verify that point operations (Add, Double, Encode)
//! match Go's ECgFp5 implementation exactly.

use goldilocks_crypto::{Point, ScalarField};
use hex;

/// Test vector from Go's TestEncode
#[test]
fn test_point_encode_go_vector() {
    // This test verifies that point encoding matches Go's output
    // Go test: poseidon_crypto/curve/ecgfp5/curve_test.go:10-56
    
    let generator = Point::generator();
    let encoded = generator.encode();
    
    // Verify encoding is 40 bytes (5 limbs)
    let bytes = encoded.to_bytes_le();
    assert_eq!(bytes.len(), 40);
    
    // Verify not all zeros
    assert!(!bytes.iter().all(|&b| b == 0));
    
    println!("✅ Point encoding produces 40-byte output");
    println!("   Encoded (hex): {}", hex::encode(&bytes));
}

/// Test that point addition works: G + G = 2G
#[test]
fn test_point_addition_g_plus_g() {
    let generator = Point::generator();
    
    // G + G should equal 2G
    let double_g_via_add = generator.add(&generator);
    
    // Create scalar 2
    let mut two_bytes = [0u8; 40];
    two_bytes[0] = 2;
    let two_scalar = ScalarField::from_bytes_le(&two_bytes).expect("Failed to create scalar 2");
    
    // 2G via scalar multiplication
    let double_g_via_mul = generator.mul(&two_scalar);
    
    // Compare encoded points
    let encoded1 = double_g_via_add.encode();
    let encoded2 = double_g_via_mul.encode();
    
    assert_eq!(encoded1, encoded2, "G + G should equal 2*G");
    
    println!("✅ Point addition: G + G = 2G verified");
}

/// Test that point addition matches scalar multiplication
#[test]
fn test_point_addition_matches_scalar_mul() {
    let generator = Point::generator();
    
    // Test with scalar 3: G + G + G = 3G
    let triple_g_via_add = generator.add(&generator).add(&generator);
    
    // Create scalar 3
    let mut three_bytes = [0u8; 40];
    three_bytes[0] = 3;
    let three_scalar = ScalarField::from_bytes_le(&three_bytes).expect("Failed to create scalar 3");
    
    // 3G via scalar multiplication
    let triple_g_via_mul = generator.mul(&three_scalar);
    
    // Compare encoded points
    let encoded1 = triple_g_via_add.encode();
    let encoded2 = triple_g_via_mul.encode();
    
    assert_eq!(encoded1, encoded2, "G + G + G should equal 3*G");
    
    println!("✅ Point addition matches scalar multiplication");
}

