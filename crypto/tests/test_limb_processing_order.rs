//! Test to verify limb processing order in mul_add2

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_limb_processing_order() {
    println!("\n=== Testing Limb Processing Order ===\n");
    
    let generator = Point::generator();
    
    // Create a scalar with a known pattern
    // Set only the most significant 4-bit limb to 1
    let mut scalar_bytes = [0u8; 40];
    // Set the most significant byte's high nibble to 1
    scalar_bytes[39] = 0x10; // This is the most significant 4 bits of the most significant byte
    let scalar = ScalarField::from_bytes_le(&scalar_bytes).unwrap();
    
    println!("Scalar (most significant byte = 0x10):");
    println!("  Bytes: {}...", hex::encode(&scalar_bytes[35..]));
    
    let limbs = scalar.split_to_4bit_limbs();
    println!("\n4-bit limbs (last 10):");
    for i in 70..80 {
        println!("  limbs[{}] = 0x{:x}", i, limbs[i]);
    }
    
    // Find which limb index has the 1
    let mut one_pos = None;
    for i in (0..80).rev() {
        if limbs[i] != 0 {
            one_pos = Some(i);
            break;
        }
    }
    
    if let Some(pos) = one_pos {
        println!("\nMost significant non-zero limb: limbs[{}] = 0x{:x}", pos, limbs[pos]);
        
        // Test: G * scalar should equal G * (1 << (4 * (79 - pos)))
        // But actually, if limbs[pos] = 1, then the scalar value is approximately 2^(4*pos)
        // So G * scalar should be approximately G * 2^(4*pos)
        
        let result = generator.mul(&scalar);
        let result_encoded = result.encode();
        println!("\nG * scalar:");
        println!("  Result: {}...", hex::encode(&result_encoded.to_bytes_le()[..16]));
        
        // Also test with a simpler scalar to verify Point::mul works
        let simple_scalar = ScalarField::from_bytes_le(&{
            let mut bytes = [0u8; 40];
            bytes[0] = 1;
            bytes
        }).unwrap();
        let simple_result = generator.mul(&simple_scalar);
        let simple_encoded = simple_result.encode();
        println!("\nG * 1:");
        println!("  Result: {}...", hex::encode(&simple_encoded.to_bytes_le()[..16]));
        println!("  Should equal G: {}", simple_encoded.0.iter().zip(generator.encode().0.iter())
            .all(|(a, b)| a.0 == b.0));
    }
}












