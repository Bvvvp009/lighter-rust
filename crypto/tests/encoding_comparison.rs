// Test to verify that Point encoding works correctly

use goldilocks_crypto::{Point, ScalarField};

#[test]
fn test_point_encoding() {
    // Generate a random point
    let scalar = ScalarField::sample_crypto();
    let point = Point::generator().mul(&scalar);
    
    // Encode the point
    let point_encoded = point.encode();
    
    // Decode it back
    let decoded_point = Point::decode(&point_encoded)
        .expect("Should decode successfully");
    
    // Verify the decoded point encodes to the same value
    let re_encoded = decoded_point.encode();
    
    println!("Original encoded: {:?}", point_encoded.to_bytes_le());
    println!("Re-encoded: {:?}", re_encoded.to_bytes_le());
    
    // They should be equal if encoding is consistent
    let encodings_match = point_encoded.0.iter().zip(re_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if !encodings_match {
        println!("❌ Encodings do NOT match!");
        println!("Point encoding: t/u");
        println!("Weierstrass encoding: Y/(A/3-X)");
    } else {
        println!("✅ Encodings match!");
    }
    
    // For now, just print the result - we expect them to be different
    // but we need to understand why Go's verification works
}

