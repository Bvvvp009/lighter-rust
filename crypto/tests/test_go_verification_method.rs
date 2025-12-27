//! Test Go's verification method: R = s*G + e*P (without e adjustment)
//! 
//! Go's test showed this works, so let's verify it works in Rust too.

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

#[test]
fn test_go_verification_method() {
    println!("\n=== Testing Go's Verification Method ===\n");
    
    // Known Go signature
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
    println!("  s: {}", hex::encode(&s.to_bytes_le()));
    println!("  e: {}", hex::encode(&e.to_bytes_le()));
    
    // Method 1: Current Rust approach (with e_adjusted)
    println!("\n=== Method 1: Current Rust (with e_adjusted) ===");
    let generator = Point::generator();
    let public_point = generator.mul(&sk);
    
    let e_adjusted = e.monty_mul(&ScalarField::ONE);
    let r1 = Point::mul_add2(&generator, &public_point, &s, &e_adjusted);
    let r1_encoded = r1.encode();
    
    println!("R encoded: {}", hex::encode(&r1_encoded.to_bytes_le()));
    println!("R elements:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r1_encoded.0[i].0);
    }
    
    // Method 2: Go's approach (e without adjustment)
    println!("\n=== Method 2: Go's Approach (e without adjustment) ===");
    let r2 = Point::mul_add2(&generator, &public_point, &s, &e);
    let r2_encoded = r2.encode();
    
    println!("R encoded: {}", hex::encode(&r2_encoded.to_bytes_le()));
    println!("R elements:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r2_encoded.0[i].0);
    }
    
    // Compare R values
    println!("\n=== R Comparison ===");
    let r_match = r1_encoded.0.iter().zip(r2_encoded.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("Method 1 == Method 2: {}", r_match);
    
    if !r_match {
        println!("R values differ! This explains the hash mismatch.");
        for i in 0..5 {
            if r1_encoded.0[i].0 != r2_encoded.0[i].0 {
                println!("  R[{}]: Method1={}, Method2={}, diff={}", 
                    i, r1_encoded.0[i].0, r2_encoded.0[i].0,
                    r1_encoded.0[i].0 as i128 - r2_encoded.0[i].0 as i128);
            }
        }
    }
    
    // Compute hash with Method 2 (Go's approach)
    println!("\n=== Hash Computation with Method 2 ===");
    
    fn message_to_fp5(message: &[u8]) -> Result<Fp5Element, String> {
        if message.len() != 40 {
            return Err(format!("Invalid message length: {}", message.len()));
        }
        let mut message_elements = [Goldilocks::zero(); 5];
        for (i, chunk) in message.chunks(8).enumerate().take(5) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            bytes.reverse();
            message_elements[i] = Goldilocks::from_canonical_u64(u64::from_be_bytes(bytes));
        }
        Ok(Fp5Element(message_elements))
    }
    
    let message_fp5 = message_to_fp5(&message).unwrap();
    
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r2_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_computed_fp5 = hash_to_quintic_extension(&pre_image);
    let e_computed_scalar = ScalarField::from_fp5_element(&e_computed_fp5);
    
    println!("Computed e': {}", hex::encode(&e_computed_scalar.to_bytes_le()));
    println!("Expected e:  {}", hex::encode(&e.to_bytes_le()));
    
    let hash_match = e.0 == e_computed_scalar.0;
    println!("Hash match: {}", hash_match);
    
    if hash_match {
        println!("\n✅ SUCCESS! Method 2 (Go's approach) works!");
        println!("The issue is that we shouldn't adjust e - use it directly!");
    } else {
        println!("\n❌ Hash still doesn't match with Method 2");
        println!("This suggests a deeper issue with Point::mul_add2 or scalar forms");
    }
}






