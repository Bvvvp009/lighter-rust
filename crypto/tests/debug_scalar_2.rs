//! Debug scalar 2 multiplication

use goldilocks_crypto::{ScalarField, Point};
use hex;

fn limbs_to_bytes(limbs: [u64; 5]) -> [u8; 40] {
    let mut bytes = [0u8; 40];
    for (i, &limb) in limbs.iter().enumerate() {
        let start = i * 8;
        bytes[start..start + 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes
}

#[test]
fn test_debug_scalar_2_multiplication() {
    println!("\n=== Debug Scalar 2 Multiplication ===\n");
    
    let generator = Point::generator();
    
    // Create scalar 2
    let two = ScalarField::from_bytes_le(&limbs_to_bytes([2, 0, 0, 0, 0])).unwrap();
    println!("Scalar: {:?}", two.0);
    
    // Recode it
    let digits = two.recode_signed(5);
    println!("Recoded digits (len={}): {:?}", digits.len(), digits);
    
    //  Manually check what the algorithm does
    println!("\nManual simulation:");
    
    // Start with last digit
    let last_idx = digits.len() - 1;
    println!("Starting with digits[{}] = {}", last_idx, digits[last_idx]);
    
    // Then process from right to left
    for i in (0..digits.len() - 1).rev() {
        println!("Step: double 5 times, then add digits[{}] = {}", i, digits[i]);
    }
    
    // Compare with simple multiplication
    let two_g_simple = generator.mul_simple(2);
    let two_g_mul = generator.mul(&two);
    
    let simple_encoded = two_g_simple.encode().to_bytes_le();
    let mul_encoded = two_g_mul.encode().to_bytes_le();
    
    println!("\nSimple mul result: {}", hex::encode(&simple_encoded));
    println!("Windowed mul result: {}", hex::encode(&mul_encoded));
    println!("Match: {}", simple_encoded == mul_encoded);
}
