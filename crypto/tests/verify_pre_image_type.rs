//! Verify the correct type for hash pre-image
//! 
//! This test checks if we should use Goldilocks elements or Fp5Elements
//! for the Poseidon2 hash pre-image.

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

#[test]
fn test_pre_image_type_verification() {
    println!("\n=== Pre-image Type Verification ===\n");
    
    // Use known values
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    
    // Reconstruct R
    let generator = Point::generator();
    let e_sk = e.mul(&sk);
    let e_sk_canonical = e_sk.to_canonical();
    let k = s.add(e_sk_canonical);
    let r_point = generator.mul(&k);
    let r_encoded = r_point.encode();
    
    // Convert message
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
    
    println!("R encoded type: Fp5Element (which contains [Goldilocks; 5])");
    println!("R encoded limbs:");
    for i in 0..5 {
        println!("  R[{}] = Goldilocks({})", i, r_encoded.0[i].0);
    }
    
    println!("\nMessage Fp5 type: Fp5Element (which contains [Goldilocks; 5])");
    println!("Message limbs:");
    for i in 0..5 {
        println!("  M[{}] = Goldilocks({})", i, message_fp5.0[i].0);
    }
    
    // Method 1: Use Goldilocks elements directly (what Rust does)
    println!("\n=== Method 1: Goldilocks Elements (Rust approach) ===");
    let mut pre_image_goldilocks = [Goldilocks::zero(); 10];
    pre_image_goldilocks[..5].copy_from_slice(&r_encoded.0);
    pre_image_goldilocks[5..].copy_from_slice(&message_fp5.0);
    
    println!("Pre-image (10 Goldilocks elements):");
    for i in 0..10 {
        if i < 5 {
            println!("  [{}] R[{}] = {}", i, i, pre_image_goldilocks[i].0);
        } else {
            println!("  [{}] M[{}] = {}", i, i-5, pre_image_goldilocks[i].0);
        }
    }
    
    let e1_fp5 = hash_to_quintic_extension(&pre_image_goldilocks);
    let e1_scalar = ScalarField::from_fp5_element(&e1_fp5);
    
    println!("Hash result: {}", hex::encode(&e1_scalar.to_bytes_le()));
    
    // Expected hash from signature
    println!("\nExpected e (from signature): {}", hex::encode(&e.to_bytes_le()));
    println!("Match: {}", e.0 == e1_scalar.0);
    
    // Check if R encoding matches what Go would use
    println!("\n=== R Encoding Check ===");
    println!("R encoded bytes: {}", hex::encode(&r_encoded.to_bytes_le()));
    println!("R limbs as u64:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_encoded.0[i].0);
    }
    
    // Compare with what Go showed us earlier
    println!("\n=== Comparison with Go Output ===");
    println!("When Go processed our R values, it showed:");
    println!("  R[0] = [18010422780608180324]");
    println!("  R[1] = [15143564317185692925]");
    println!("  R[2] = [6061725974849309129]");
    println!("  R[3] = [13273000297535928161]");
    println!("  R[4] = [11112181103645862154]");
    
    let go_r_values = [18010422780608180324u64, 15143564317185692925, 6061725974849309129, 13273000297535928161, 11112181103645862154];
    
    println!("\nOur R values:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_encoded.0[i].0);
    }
    
    let r_matches_go = r_encoded.0.iter().zip(go_r_values.iter()).all(|(a, &b)| a.0 == b);
    println!("\nR matches Go's internal representation: {}", r_matches_go);
    
    if !r_matches_go {
        println!("\n⚠️  R values differ! This could be the issue.");
        println!("Possible causes:");
        println!("  1. Go uses different R encoding during signing");
        println!("  2. Go's Encode() returns different values");
        println!("  3. Type conversion issue (Goldilocks vs Fp5Element)");
    }
}










