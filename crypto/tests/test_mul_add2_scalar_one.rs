//! Test to debug why mul_add2(G, P, 1, 0) fails

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_mul_add2_with_scalar_one() {
    println!("\n=== Debugging mul_add2(G, P, 1, 0) ===\n");
    
    let generator = Point::generator();
    let p = generator.mul(&ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5;
        bytes
    }).unwrap());
    
    let one = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let zero = ScalarField::from_bytes_le(&[0u8; 40]).unwrap();
    
    println!("Scalar values:");
    println!("  one: {:?}", one.0);
    println!("  zero: {:?}", zero.0);
    
    // Check 4-bit limb splitting
    let one_limbs = one.split_to_4bit_limbs();
    let zero_limbs = zero.split_to_4bit_limbs();
    
    println!("\n4-bit limbs:");
    println!("  one_limbs[0] (LSB): {}", one_limbs[0]);
    println!("  one_limbs[79] (MSB): {}", one_limbs[79]);
    println!("  zero_limbs[0]: {}", zero_limbs[0]);
    println!("  zero_limbs[79]: {}", zero_limbs[79]);
    
    // Find where the 1 is
    let mut one_positions = Vec::new();
    for (i, &limb) in one_limbs.iter().enumerate() {
        if limb != 0 {
            one_positions.push((i, limb));
        }
    }
    println!("  Non-zero limbs in one: {:?}", one_positions);
    
    // Expected result
    let expected = generator.encode();
    println!("\nExpected result (G):");
    println!("  Encoded: {}", hex::encode(&expected.to_bytes_le()));
    
    // Computed result
    let computed = Point::mul_add2(&generator, &p, &one, &zero);
    let computed_encoded = computed.encode();
    println!("Computed result:");
    println!("  Encoded: {}", hex::encode(&computed_encoded.to_bytes_le()));
    
    // Also try with scalar 2 to compare
    let two = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 2;
        bytes
    }).unwrap();
    let computed2 = Point::mul_add2(&generator, &p, &two, &zero);
    let computed2_encoded = computed2.encode();
    let expected2 = generator.mul(&two).encode();
    println!("\nFor comparison, mul_add2(G, P, 2, 0):");
    println!("  Expected (2*G): {}", hex::encode(&expected2.to_bytes_le()));
    println!("  Computed: {}", hex::encode(&computed2_encoded.to_bytes_le()));
    println!("  Match: {}", expected2.0.iter().zip(computed2_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
}













