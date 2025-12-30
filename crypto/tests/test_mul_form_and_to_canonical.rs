//! Test to verify what form mul() returns and if to_canonical() works correctly

use goldilocks_crypto::ScalarField;
use hex;

#[test]
fn test_mul_form_and_to_canonical() {
    println!("\n=== Testing mul() form and to_canonical() ===\n");
    
    // Test 1: Simple scalar multiplication
    println!("=== Test 1: Simple scalar (5 * 7 = 35) ===");
    let a = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5;
        bytes
    }).unwrap();
    
    let b = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7;
        bytes
    }).unwrap();
    
    println!("a (canonical): {}", hex::encode(&a.to_bytes_le()[..8]));
    println!("b (canonical): {}", hex::encode(&b.to_bytes_le()[..8]));
    
    // Convert to Montgomery form
    let a_mont = a.monty_mul(&ScalarField::R2);
    let b_mont = b.monty_mul(&ScalarField::R2);
    println!("a (Montgomery): {}", hex::encode(&a_mont.to_bytes_le()[..8]));
    println!("b (Montgomery): {}", hex::encode(&b_mont.to_bytes_le()[..8]));
    
    // Test mul() - should return Montgomery form
    let product = a.mul(&b);
    println!("a.mul(&b) (result): {}", hex::encode(&product.to_bytes_le()[..8]));
    
    // Convert to canonical
    let product_canonical = product.to_canonical();
    println!("a.mul(&b).to_canonical(): {}", hex::encode(&product_canonical.to_bytes_le()[..8]));
    
    // Expected: 5 * 7 = 35
    let expected = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 35;
        bytes
    }).unwrap();
    println!("expected (35): {}", hex::encode(&expected.to_bytes_le()[..8]));
    
    let matches = product_canonical.to_bytes_le() == expected.to_bytes_le();
    println!("product_canonical == expected: {}", matches);
    
    if !matches {
        println!("❌ FAILED: to_canonical() doesn't work for simple multiplication!");
        println!("  product_canonical limbs: {:?}", product_canonical.0);
        println!("  expected limbs: {:?}", expected.0);
    } else {
        println!("✅ PASSED: to_canonical() works for simple multiplication");
    }
    
    // Test 2: e*sk case (the problematic case)
    println!("\n=== Test 2: e*sk case (e=3, sk=7, expected=21) ===");
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3;
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7;
        bytes
    }).unwrap();
    
    println!("e (canonical): {}", hex::encode(&e.to_bytes_le()[..8]));
    println!("sk (canonical): {}", hex::encode(&sk.to_bytes_le()[..8]));
    
    // Compute e*sk using mul()
    let e_sk = e.mul(&sk);
    println!("e.mul(&sk) (Montgomery form): {}", hex::encode(&e_sk.to_bytes_le()[..8]));
    println!("e.mul(&sk) limbs: {:?}", e_sk.0);
    
    // Convert to canonical
    let e_sk_canonical = e_sk.to_canonical();
    println!("e.mul(&sk).to_canonical(): {}", hex::encode(&e_sk_canonical.to_bytes_le()[..8]));
    println!("e.mul(&sk).to_canonical() limbs: {:?}", e_sk_canonical.0);
    
    // Expected: 3 * 7 = 21
    let expected_21 = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 21;
        bytes
    }).unwrap();
    println!("expected (21): {}", hex::encode(&expected_21.to_bytes_le()[..8]));
    println!("expected limbs: {:?}", expected_21.0);
    
    let matches_21 = e_sk_canonical.to_bytes_le() == expected_21.to_bytes_le();
    println!("e_sk_canonical == expected_21: {}", matches_21);
    
    if !matches_21 {
        println!("❌ FAILED: to_canonical() doesn't work for e*sk!");
        println!("  This is the bug we need to fix!");
        
        // Try alternative: multiply by R2_INV
        println!("\n  Trying alternative: e_sk.monty_mul(&R2_INV)");
        let e_sk_alt = e_sk.monty_mul(&ScalarField::R2_INV);
        println!("  e_sk.monty_mul(&R2_INV): {}", hex::encode(&e_sk_alt.to_bytes_le()[..8]));
        println!("  e_sk.monty_mul(&R2_INV) limbs: {:?}", e_sk_alt.0);
        let matches_alt = e_sk_alt.to_bytes_le() == expected_21.to_bytes_le();
        println!("  matches: {}", matches_alt);
        
        // Try using BigUint approach
        println!("\n  Trying BigUint approach:");
        use num_bigint::BigUint;
        let e_bytes = e.to_bytes_le();
        let sk_bytes = sk.to_bytes_le();
        let n_bytes = ScalarField::N.to_bytes_le();
        
        let e_big = BigUint::from_bytes_le(&e_bytes);
        let sk_big = BigUint::from_bytes_le(&sk_bytes);
        let n_big = BigUint::from_bytes_le(&n_bytes);
        
        let product_big = (&e_big * &sk_big) % &n_big;
        let product_bytes = product_big.to_bytes_le();
        
        let mut product_limbs = [0u64; 5];
        for (i, chunk) in product_bytes.chunks(8).enumerate().take(5) {
            let mut limb_bytes = [0u8; 8];
            let copy_len = chunk.len().min(8);
            limb_bytes[..copy_len].copy_from_slice(&chunk[..copy_len]);
            product_limbs[i] = u64::from_le_bytes(limb_bytes);
        }
        let e_sk_biguint = ScalarField(product_limbs);
        println!("  BigUint result: {}", hex::encode(&e_sk_biguint.to_bytes_le()[..8]));
        println!("  BigUint limbs: {:?}", e_sk_biguint.0);
        let matches_biguint = e_sk_biguint.to_bytes_le() == expected_21.to_bytes_le();
        println!("  matches: {}", matches_biguint);
    } else {
        println!("✅ PASSED: to_canonical() works for e*sk");
    }
    
    // Test 3: Verify mul() returns Montgomery form
    println!("\n=== Test 3: Verify mul() returns Montgomery form ===");
    // If mul() returns Montgomery form, then:
    // product = (a * R2) * (b * R2) / R2 = a * b * R2 (Montgomery form)
    // So product should equal (a * b) * R2
    
    let ab_expected = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 35; // 5 * 7
        bytes
    }).unwrap();
    let ab_expected_mont = ab_expected.monty_mul(&ScalarField::R2);
    println!("(a*b) * R2 (expected Montgomery): {}", hex::encode(&ab_expected_mont.to_bytes_le()[..8]));
    println!("a.mul(&b) (actual): {}", hex::encode(&product.to_bytes_le()[..8]));
    
    let is_montgomery = product.to_bytes_le() == ab_expected_mont.to_bytes_le();
    println!("mul() returns Montgomery form: {}", is_montgomery);
    
    if is_montgomery {
        println!("✅ CONFIRMED: mul() returns Montgomery form");
    } else {
        println!("❌ UNEXPECTED: mul() doesn't return expected Montgomery form");
        println!("  This suggests mul() implementation might be wrong");
    }
}







