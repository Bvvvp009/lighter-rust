//! Debug test for point doubling vs scalar multiplication

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
fn test_debug_point_doubling() {
    println!("\n=== Debug Point Doubling ===\n");
    
    let generator = Point::generator();
    let generator_encoded = generator.encode();
    let generator_bytes = generator_encoded.to_bytes_le();
    println!("Generator G: {}", hex::encode(&generator_bytes));
    
    // Method 1: G + G using add()
    let double_g_add = generator.add(&generator);
    let double_g_add_encoded = double_g_add.encode();
    let double_g_add_bytes = double_g_add_encoded.to_bytes_le();
    println!("G + G (using add): {}", hex::encode(&double_g_add_bytes));
    
    // Method 2: 2*G using double()
    let double_g_double = generator.double();
    let double_g_double_encoded = double_g_double.encode();
    let double_g_double_bytes = double_g_double_encoded.to_bytes_le();
    println!("2*G (using double): {}", hex::encode(&double_g_double_bytes));
    
    // Method 3: G * 2 using scalar multiplication
    let two = ScalarField::from_bytes_le(&limbs_to_bytes([2, 0, 0, 0, 0])).unwrap();
    let two_g_mul = generator.mul(&two);
    let two_g_mul_encoded = two_g_mul.encode();
    let two_g_mul_bytes = two_g_mul_encoded.to_bytes_le();
    println!("G * 2 (using mul): {}", hex::encode(&two_g_mul_bytes));
    
    // Compare all three methods
    println!("\nComparison:");
    println!("G + G == 2*G: {}", double_g_add_bytes == double_g_double_bytes);
    println!("G + G == G * 2: {}", double_g_add_bytes == two_g_mul_bytes);
    println!("2*G == G * 2: {}", double_g_double_bytes == two_g_mul_bytes);
    
    // Check coordinates
    println!("\nCoordinates G + G:");
    println!("  x: {}", hex::encode(&double_g_add.x.to_bytes_le()));
    println!("  z: {}", hex::encode(&double_g_add.z.to_bytes_le()));
    println!("  u: {}", hex::encode(&double_g_add.u.to_bytes_le()));
    println!("  t: {}", hex::encode(&double_g_add.t.to_bytes_le()));
    
    println!("\nCoordinates 2*G:");
    println!("  x: {}", hex::encode(&double_g_double.x.to_bytes_le()));
    println!("  z: {}", hex::encode(&double_g_double.z.to_bytes_le()));
    println!("  u: {}", hex::encode(&double_g_double.u.to_bytes_le()));
    println!("  t: {}", hex::encode(&double_g_double.t.to_bytes_le()));
    
    println!("\nCoordinates G * 2:");
    println!("  x: {}", hex::encode(&two_g_mul.x.to_bytes_le()));
    println!("  z: {}", hex::encode(&two_g_mul.z.to_bytes_le()));
    println!("  u: {}", hex::encode(&two_g_mul.u.to_bytes_le()));
    println!("  t: {}", hex::encode(&two_g_mul.t.to_bytes_le()));
    
    // Verify they all should be equal
    assert_eq!(double_g_add_bytes, double_g_double_bytes, "G + G should equal 2*G");
    assert_eq!(double_g_add_bytes, two_g_mul_bytes, "G + G should equal G * 2");
}
