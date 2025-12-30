//! Isolate the R encoding issue
//! 
//! This test focuses on understanding why R values differ when passed to Go.
//! We'll compare:
//! 1. R from our reconstruction
//! 2. R that Go would use (if we can determine it)
//! 3. Byte-level encoding differences

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_isolate_r_encoding_issue() {
    println!("\n=== Isolating R Encoding Issue ===\n");
    
    // Known signature
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let _message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    
    // Reconstruct R (Method 1 - we know this is correct)
    let generator = Point::generator();
    let e_sk = e.mul(&sk);
    let e_sk_canonical = e_sk.to_canonical();
    let k = s.add(e_sk_canonical);
    let r_point = generator.mul(&k);
    let r_encoded = r_point.encode();
    
    println!("=== Our R Values ===");
    println!("R encoded bytes: {}", hex::encode(&r_encoded.to_bytes_le()));
    println!("R as Goldilocks elements:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_encoded.0[i].0);
    }
    
    // Convert to bytes and show byte-level representation
    println!("\n=== R Byte-Level Representation ===");
    let r_bytes = r_encoded.to_bytes_le();
    for i in 0..5 {
        let start = i * 8;
        let end = start + 8;
        println!("  R[{}] bytes: {}", i, hex::encode(&r_bytes[start..end]));
        println!("    As u64 (little-endian): {}", u64::from_le_bytes(r_bytes[start..end].try_into().unwrap()));
    }
    
    // Compare with what Go showed us
    println!("\n=== Comparison with Go Output ===");
    println!("When we passed our R values to Go, Go showed:");
    let go_r_values = [18010422780608180324u64, 15143564317185692925, 6061725974849309129, 13273000297535928161, 11112181103645862154];
    
    println!("Go's internal R values:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, go_r_values[i]);
    }
    
    println!("\nOur R values:");
    for i in 0..5 {
        println!("  R[{}] = {}", i, r_encoded.0[i].0);
    }
    
    // Check if there's a pattern in the difference
    println!("\n=== Difference Analysis ===");
    for i in 0..5 {
        let diff = r_encoded.0[i].0 as i128 - go_r_values[i] as i128;
        println!("  R[{}]: ours={}, go={}, diff={}", 
            i, r_encoded.0[i].0, go_r_values[i], diff);
    }
    
    // Key question: Are these the same values modulo the Goldilocks prime?
    // Goldilocks prime: p = 2^64 - 2^32 + 1 = 18446744069414584321
    const GOLDILOCKS_PRIME: u64 = 18446744069414584321;
    
    println!("\n=== Modulo Goldilocks Prime Check ===");
    for i in 0..5 {
        let ours_mod = r_encoded.0[i].0 % GOLDILOCKS_PRIME;
        let go_mod = go_r_values[i] % GOLDILOCKS_PRIME;
        println!("  R[{}]: ours_mod={}, go_mod={}, match={}", 
            i, ours_mod, go_mod, ours_mod == go_mod);
    }
    
    // Check if Go is interpreting our values differently
    println!("\n=== Hypothesis: Type Conversion Issue ===");
    println!("When we pass R values to Go as gFp5.Element(limbVal), Go might:");
    println!("  1. Interpret them as Fp5Element limbs (correct)");
    println!("  2. Convert them through some encoding");
    println!("  3. Use them directly as Goldilocks elements");
    
    println!("\nTo verify, we need to:");
    println!("  1. See what Go's Encode() actually returns during signing");
    println!("  2. Compare byte-for-byte with our R");
    println!("  3. Check if there's a type conversion happening");
}










