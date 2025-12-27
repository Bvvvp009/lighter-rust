//! Test computing e_adjusted = e * R2_INV in canonical form

use goldilocks_crypto::{ScalarField, Point};
use num_bigint::BigUint;
use hex;

#[test]
fn test_e_adjusted_canonical_computation() {
    println!("\n=== Testing e_adjusted = e * R2_INV (Canonical) ===\n");
    
    let generator = Point::generator();
    
    let e_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    let sk_canonical = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    let public_key = generator.mul(&sk_canonical);
    
    // Compute expected: (e*sk canonical)*G
    let e_times_sk = e_canonical.mul(&sk_canonical);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let expected = generator.mul(&e_times_sk_canonical);
    let expected_encoded = expected.encode();
    
    println!("Expected (e*sk canonical)*G:");
    println!("  Result: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    // Method 1: Compute e * R2_INV using BigUint (canonical multiplication)
    let e_bytes = e_canonical.to_bytes_le();
    let r2_inv_bytes = ScalarField::R2_INV.to_bytes_le();
    
    let e_big = BigUint::from_bytes_le(&e_bytes);
    let r2_inv_big = BigUint::from_bytes_le(&r2_inv_bytes);
    let n_big = {
        let n_bytes = ScalarField::N.to_bytes_le();
        BigUint::from_bytes_le(&n_bytes)
    };
    
    // Compute e * R2_INV mod N in canonical form
    let product_big = (&e_big * &r2_inv_big) % &n_big;
    let product_bytes = product_big.to_bytes_le();
    let mut product_limbs = [0u64; 5];
    for (i, chunk) in product_bytes.chunks(8).enumerate().take(5) {
        let mut limb_bytes = [0u8; 8];
        let copy_len = chunk.len().min(8);
        limb_bytes[..copy_len].copy_from_slice(&chunk[..copy_len]);
        product_limbs[i] = u64::from_le_bytes(limb_bytes);
    }
    let e_adjusted_canonical = ScalarField(product_limbs);
    
    println!("\nMethod 1: e * R2_INV (canonical, using BigUint):");
    println!("  e_adjusted: {}...", hex::encode(&e_adjusted_canonical.to_bytes_le()[..8]));
    
    let result1 = public_key.mul(&e_adjusted_canonical);
    let encoded1 = result1.encode();
    
    println!("  Result: {}...", hex::encode(&encoded1.to_bytes_le()[..16]));
    
    // Compare
    let match_1 = encoded1.0.iter().zip(expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparison:");
    println!("  e_adjusted_canonical*P == (e*sk canonical)*G: {}", match_1);
    
    if match_1 {
        println!("\n✅ FIX WORKS: Using canonical multiplication for e * R2_INV!");
        println!("   We need to add a function to multiply canonical scalars");
    } else {
        println!("\n❌ Still doesn't work - need to investigate further");
    }
}








