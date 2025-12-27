//! Verify that our R reconstruction matches what Go would compute during signing
//! 
//! This test checks:
//! 1. Does k = s + e*sk reconstruction match Go's approach?
//! 2. Does R = k*G match what Go would compute?
//! 3. Are we using the correct scalar forms?

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use hex;

#[test]
fn test_r_reconstruction_verification() {
    println!("\n=== R Reconstruction Verification ===\n");
    
    // Use known Go signature
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    
    println!("Inputs:");
    println!("  sk: {}", hex::encode(&sk.to_bytes_le()));
    println!("  s:  {}", hex::encode(&s.to_bytes_le()));
    println!("  e:  {}", hex::encode(&e.to_bytes_le()));
    
    // Method 1: Our current approach (e*sk Montgomery -> canonical -> add s)
    println!("\n=== Method 1: Current Approach ===");
    let e_sk_montgomery = e.mul(&sk);
    println!("  e*sk (Montgomery): {}", hex::encode(&e_sk_montgomery.to_bytes_le()));
    
    let e_sk_canonical = e_sk_montgomery.to_canonical();
    println!("  e*sk (canonical): {}", hex::encode(&e_sk_canonical.to_bytes_le()));
    
    let k1 = s.add(e_sk_canonical);
    println!("  k = s + e*sk (canonical): {}", hex::encode(&k1.to_bytes_le()));
    
    let generator = Point::generator();
    let r1 = generator.mul(&k1);
    let r1_encoded = r1.encode();
    println!("  R = k*G:");
    for i in 0..5 {
        println!("    R[{}] = {}", i, r1_encoded.0[i].0);
    }
    
    // Method 2: Try without converting to canonical (keep Montgomery)
    println!("\n=== Method 2: Keep Montgomery Form ===");
    let k2 = s.add(e_sk_montgomery);
    println!("  k = s + e*sk (Montgomery): {}", hex::encode(&k2.to_bytes_le()));
    
    let r2 = generator.mul(&k2);
    let r2_encoded = r2.encode();
    println!("  R = k*G:");
    for i in 0..5 {
        println!("    R[{}] = {}", i, r2_encoded.0[i].0);
    }
    
    // Method 3: Try with e in Montgomery form
    println!("\n=== Method 3: e in Montgomery Form ===");
    let e_montgomery = e.monty_mul(&ScalarField::ONE);
    println!("  e (Montgomery): {}", hex::encode(&e_montgomery.to_bytes_le()));
    
    let e_sk_montgomery2 = e_montgomery.mul(&sk);
    println!("  e*sk (Montgomery): {}", hex::encode(&e_sk_montgomery2.to_bytes_le()));
    
    let e_sk_canonical2 = e_sk_montgomery2.to_canonical();
    println!("  e*sk (canonical): {}", hex::encode(&e_sk_canonical2.to_bytes_le()));
    
    let k3 = s.add(e_sk_canonical2);
    println!("  k = s + e*sk: {}", hex::encode(&k3.to_bytes_le()));
    
    let r3 = generator.mul(&k3);
    let r3_encoded = r3.encode();
    println!("  R = k*G:");
    for i in 0..5 {
        println!("    R[{}] = {}", i, r3_encoded.0[i].0);
    }
    
    // Compare all methods
    println!("\n=== Comparison ===");
    println!("Method 1 == Method 2: {}", r1_encoded.0.iter().zip(r2_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
    println!("Method 1 == Method 3: {}", r1_encoded.0.iter().zip(r3_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
    println!("Method 2 == Method 3: {}", r2_encoded.0.iter().zip(r3_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
    
    // Expected R from our test (what we computed earlier)
    let expected_r_limbs = [9893616195988480026u64, 18353553311365906309, 5420542358697513897, 14520833540904659828, 11757474890942670296];
    println!("\n=== Expected R (from earlier test) ===");
    for i in 0..5 {
        println!("  R[{}] = {}", i, expected_r_limbs[i]);
    }
    
    println!("\n=== Match Check ===");
    let match1 = r1_encoded.0.iter().zip(expected_r_limbs.iter()).all(|(a, &b)| a.0 == b);
    let match2 = r2_encoded.0.iter().zip(expected_r_limbs.iter()).all(|(a, &b)| a.0 == b);
    let match3 = r3_encoded.0.iter().zip(expected_r_limbs.iter()).all(|(a, &b)| a.0 == b);
    
    println!("Method 1 matches expected: {}", match1);
    println!("Method 2 matches expected: {}", match2);
    println!("Method 3 matches expected: {}", match3);
    
    if !match1 && !match2 && !match3 {
        println!("\n❌ None of the methods match expected R!");
        println!("This suggests our k reconstruction is incorrect.");
        println!("\nPossible issues:");
        println!("  1. Go uses different scalar forms during signing");
        println!("  2. Go computes e*sk differently");
        println!("  3. Go uses different addition/subtraction forms");
    } else {
        let correct_method = if match1 { "Method 1" } else if match2 { "Method 2" } else { "Method 3" };
        println!("\n✅ {} matches expected R!", correct_method);
    }
}






