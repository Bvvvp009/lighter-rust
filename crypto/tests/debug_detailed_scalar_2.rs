//! Detailed debug of scalar 2 multiplication

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
fn test_detailed_scalar_2() {
    println!("\n=== Detailed Scalar 2 Multiplication ===\n");
    
    let generator = Point::generator();
    
    // Test mul_simple(2) - this should be correct
    let two_g_simple = generator.mul_simple(2);
    let simple_encoded = two_g_simple.encode().to_bytes_le();
    println!("mul_simple(2): {}", hex::encode(&simple_encoded));
    
    // Test double() - this should also be correct
    let two_g_double = generator.double();
    let double_encoded = two_g_double.encode().to_bytes_le();
    println!("double(): {}", hex::encode(&double_encoded));
    
    // Test add(generator, generator) - should also be correct
    let two_g_add = generator.add(&generator);
    let add_encoded = two_g_add.encode().to_bytes_le();
    println!("add(G, G): {}", hex::encode(&add_encoded));
    
    // Now test mul with scalar 2
    let two = ScalarField::from_bytes_le(&limbs_to_bytes([2, 0, 0, 0, 0])).unwrap();
    let digits = two.recode_signed(5);
    
    println!("\nScalar 2 recoded digits:");
    for (i, &d) in digits.iter().enumerate() {
        if d != 0 {
            println!("  digits[{}] = {}", i, d);
        }
    }
    
    // Find first non-zero
    let mut start_idx = digits.len() - 1;
    while start_idx > 0 && digits[start_idx] == 0 {
        start_idx -= 1;
    }
    println!("\nFirst non-zero digit at index: {}", start_idx);
    println!("digits[{}] = {}", start_idx, digits[start_idx]);
    
    // Manually do what mul() should do
    let window = generator.make_window_affine();
    println!("\nWindow[0] (1*G): {}", hex::encode(&window[0].x.to_bytes_le()[..16]));
    println!("Window[1] (2*G): {}", hex::encode(&window[1].x.to_bytes_le()[..16]));
    
    // Lookup digits[start_idx]
    println!("\nLooking up digits[{}] = {} from window", start_idx, digits[start_idx]);
    
    let mut result = if digits[start_idx] == 0 {
        println!("  → neutral point");
        window[0].to_point()  // This is wrong! neutral should not use window[0]
    } else if digits[start_idx] > 0 {
        let idx = (digits[start_idx] as usize) - 1;
        println!("  → window[{}]", idx);
        window[idx].to_point()
    } else {
        panic!("Negative digit");
    };
    
    let result_encoded = result.encode().to_bytes_le();
    println!("After lookup: {}", hex::encode(&result_encoded));
    
    // Check if we need to process more digits
    if start_idx > 0 {
        println!("\nProcessing remaining {} digits", start_idx);
        for i in (0..start_idx).rev() {
            if digits[i] != 0 {
                println!("  Step {}: double 5 times, add digits[{}] = {}", start_idx - i, i, digits[i]);
            }
        }
    }
    
    let two_g_mul = generator.mul(&two);
    let mul_encoded = two_g_mul.encode().to_bytes_le();
    println!("\nmul(&two): {}", hex::encode(&mul_encoded));
    
    println!("\nComparison:");
    println!("  simple == double: {}", simple_encoded == double_encoded);
    println!("  simple == add: {}", simple_encoded == add_encoded);
    println!("  simple == mul: {}", simple_encoded == mul_encoded);
    
    assert_eq!(simple_encoded, mul_encoded, "mul should match mul_simple");
}
