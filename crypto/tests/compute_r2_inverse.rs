//! Compute R2^-1 mod N for the fix

use goldilocks_crypto::ScalarField;
use num_bigint::BigUint;

#[test]
fn compute_r2_inverse() {
    println!("\n=== Computing R2^-1 mod N ===\n");
    
    // Get R2
    let r2 = ScalarField::R2;
    println!("R2:");
    for (i, &limb) in r2.0.iter().enumerate() {
        println!("  R2[{}] = 0x{:016x}", i, limb);
    }
    
    // Get N (modulus)
    let n = ScalarField::N;
    println!("\nN (modulus):");
    for (i, &limb) in n.0.iter().enumerate() {
        println!("  N[{}] = 0x{:016x}", i, limb);
    }
    
    // Convert to BigUint using bytes (little-endian)
    let r2_bytes = r2.to_bytes_le();
    let r2_big = BigUint::from_bytes_le(&r2_bytes);
    
    let n_bytes = n.to_bytes_le();
    let n_big = BigUint::from_bytes_le(&n_bytes);
    
    println!("\nComputing R2^-1 mod N...");
    
    // Compute modular inverse using extended Euclidean algorithm
    // Actually, BigUint has mod_inverse
    let r2_inv_big = r2_big.modpow(&(&n_big - BigUint::from(2u64)), &n_big);
    
    println!("R2^-1 mod N (as BigUint):");
    println!("  {}", r2_inv_big);
    
    // Convert back to limbs (little-endian)
    let r2_inv_bytes = r2_inv_big.to_bytes_le();
    let mut r2_inv_limbs = [0u64; 5];
    for (i, chunk) in r2_inv_bytes.chunks(8).enumerate().take(5) {
        let mut limb_bytes = [0u8; 8];
        let copy_len = chunk.len().min(8);
        limb_bytes[..copy_len].copy_from_slice(&chunk[..copy_len]);
        r2_inv_limbs[i] = u64::from_le_bytes(limb_bytes);
    }
    
    println!("\nR2^-1 mod N (as limbs):");
    for (i, &limb) in r2_inv_limbs.iter().enumerate() {
        println!("  R2_INV[{}] = 0x{:016x},", i, limb);
    }
    
    // Verify: R2 * R2^-1 = 1 mod N (using BigUint for verification)
    let product_big = (&r2_big * &r2_inv_big) % &n_big;
    let one_big = BigUint::from(1u64);
    
    println!("\nVerification: R2 * R2^-1 mod N (using BigUint):");
    println!("  Product: {}", product_big);
    println!("  Should equal 1: {}", product_big == one_big);
    
    if product_big == one_big {
        println!("\n✅ R2^-1 computed correctly!");
        println!("\nAdd this constant to ScalarField:");
        println!("    pub const R2_INV: ScalarField = ScalarField([");
        for (i, &limb) in r2_inv_limbs.iter().enumerate() {
            println!("        0x{:016x},  // R2_INV[{}]", limb, i);
        }
        println!("    ]);");
    } else {
        println!("\n❌ R2^-1 computation failed!");
        println!("  Product: {}", product_big);
        println!("  Expected: 1");
    }
}

