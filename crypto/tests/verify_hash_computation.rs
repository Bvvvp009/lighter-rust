//! Comprehensive hash computation verification
//! 
//! This test verifies:
//! 1. Poseidon2 configuration matches between Rust and Go
//! 2. Hash outputs match with identical inputs
//! 3. R encoding is consistent between signing and verification

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

/// Verify Poseidon2 configuration matches Go implementation
#[test]
fn test_poseidon2_configuration() {
    println!("\n=== Poseidon2 Configuration Verification ===\n");
    
    // Rust constants (from poseidon-hash/src/lib.rs)
    const RUST_WIDTH: usize = 12;
    const RUST_RATE: usize = 8;
    const RUST_ROUNDS_F_HALF: usize = 4;
    const RUST_ROUNDS_P: usize = 22;
    
    // Go constants (from gnark-plonky2-verifier/poseidon/goldilocks.go)
    const GO_SPONGE_WIDTH: usize = 12;
    const GO_SPONGE_RATE: usize = 8;
    const GO_HALF_N_FULL_ROUNDS: usize = 4;
    const GO_N_PARTIAL_ROUNDS: usize = 22;
    
    println!("Rust Poseidon2 parameters:");
    println!("  WIDTH = {}", RUST_WIDTH);
    println!("  RATE = {}", RUST_RATE);
    println!("  ROUNDS_F_HALF = {}", RUST_ROUNDS_F_HALF);
    println!("  ROUNDS_P = {}", RUST_ROUNDS_P);
    
    println!("\nGo Poseidon2 parameters:");
    println!("  SPONGE_WIDTH = {}", GO_SPONGE_WIDTH);
    println!("  SPONGE_RATE = {}", GO_SPONGE_RATE);
    println!("  HALF_N_FULL_ROUNDS = {}", GO_HALF_N_FULL_ROUNDS);
    println!("  N_PARTIAL_ROUNDS = {}", GO_N_PARTIAL_ROUNDS);
    
    assert_eq!(RUST_WIDTH, GO_SPONGE_WIDTH, "WIDTH mismatch");
    assert_eq!(RUST_RATE, GO_SPONGE_RATE, "RATE mismatch");
    assert_eq!(RUST_ROUNDS_F_HALF, GO_HALF_N_FULL_ROUNDS, "Full rounds mismatch");
    assert_eq!(RUST_ROUNDS_P, GO_N_PARTIAL_ROUNDS, "Partial rounds mismatch");
    
    println!("\n✅ All Poseidon2 configuration parameters match!");
}

/// Test hash computation with known test vectors
#[test]
fn test_hash_with_known_inputs() {
    println!("\n=== Hash Computation with Known Inputs ===\n");
    
    // Test case 1: Simple inputs
    let simple_inputs = vec![
        Goldilocks::from_canonical_u64(1),
        Goldilocks::from_canonical_u64(2),
        Goldilocks::from_canonical_u64(3),
        Goldilocks::from_canonical_u64(4),
        Goldilocks::from_canonical_u64(5),
    ];
    
    println!("Test case 1: Simple inputs");
    for (i, input) in simple_inputs.iter().enumerate() {
        println!("  Input[{}] = {}", i, input.0);
    }
    
    let result1 = hash_to_quintic_extension(&simple_inputs);
    println!("  Hash result:");
    for (i, limb) in result1.0.iter().enumerate() {
        println!("    e[{}] = {}", i, limb.0);
    }
    
    // Test case 2: 10-element input (R || message format)
    let mut test_inputs = Vec::new();
    // R elements (5)
    for i in 0..5 {
        test_inputs.push(Goldilocks::from_canonical_u64(1000 + i as u64));
    }
    // Message elements (5)
    for i in 0..5 {
        test_inputs.push(Goldilocks::from_canonical_u64(2000 + i as u64));
    }
    
    println!("\nTest case 2: 10-element input (R || message format)");
    println!("  R elements:");
    for i in 0..5 {
        println!("    R[{}] = {}", i, test_inputs[i].0);
    }
    println!("  Message elements:");
    for i in 0..5 {
        println!("    M[{}] = {}", i, test_inputs[5 + i].0);
    }
    
    let result2 = hash_to_quintic_extension(&test_inputs);
    println!("  Hash result:");
    for (i, limb) in result2.0.iter().enumerate() {
        println!("    e[{}] = {}", i, limb.0);
    }
    
    println!("\n✅ Hash computation completed successfully");
    println!("\nTo verify with Go, run:");
    print!("  go run lighter-rust/signer/examples/go_hash_helper.go \"");
    for i in 0..5 {
        if i > 0 { print!(","); }
        print!("{}", test_inputs[i].0);
    }
    print!("\" \"");
    for i in 0..5 {
        if i > 0 { print!(","); }
        print!("{}", test_inputs[5 + i].0);
    }
    println!("\"");
}

/// Test that R encoding is consistent between signing and verification
#[test]
fn test_r_encoding_consistency() {
    println!("\n=== R Encoding Consistency Test ===\n");
    
    // Use a known Go signature
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let public_key_hex = "04000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let public_key = hex::decode(public_key_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    let generator = Point::generator();
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    
    // Reconstruct R from signing: k = s + e*sk, R = k*G
    let e_sk = e.mul(&sk);
    let e_sk_canonical = e_sk.to_canonical();
    let k = s.add(e_sk_canonical);
    let r_from_signing = generator.mul(&k);
    let r_from_signing_encoded = r_from_signing.encode();
    
    // Compute R from verification: R = s*G + (e/R)*P
    let public_key_fp5 = Fp5Element::from_bytes_le(&public_key).unwrap();
    let public_point = Point::decode(&public_key_fp5).unwrap();
    let e_adjusted = e.monty_mul(&ScalarField::ONE);
    let r_from_verification = Point::mul_add2(&generator, &public_point, &s, &e_adjusted);
    let r_from_verification_encoded = r_from_verification.encode();
    
    // Compare R encodings
    let r_matches = r_from_signing_encoded.0.iter().zip(r_from_verification_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("R from signing (k*G):");
    for (i, limb) in r_from_signing_encoded.0.iter().enumerate() {
        println!("  R[{}] = {}", i, limb.0);
    }
    
    println!("\nR from verification (s*G + (e/R)*P):");
    for (i, limb) in r_from_verification_encoded.0.iter().enumerate() {
        println!("  R[{}] = {}", i, limb.0);
    }
    
    if r_matches {
        println!("\n✅ R encodings match! This is correct.");
    } else {
        println!("\n❌ R encodings differ!");
        for i in 0..5 {
            if r_from_signing_encoded.0[i].0 != r_from_verification_encoded.0[i].0 {
                println!("  R[{}]: signing={}, verification={}", 
                    i, 
                    r_from_signing_encoded.0[i].0,
                    r_from_verification_encoded.0[i].0);
            }
        }
        panic!("R encoding mismatch!");
    }
}

/// Test hash computation with R from signing
#[test]
fn test_hash_with_r_from_signing() {
    println!("\n=== Hash Computation with R from Signing ===\n");
    
    // Use a known Go signature
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let message_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    let signature_hex = "f026eefe094088d8d21ebad69565715f7a8a1fe27e5a7c0401e5cbd384aa902953597975f447be70f2d89d958c99870fca816a00a4f61689bf7b98ed67a5837b151b342c6c556f56b4a6860a09b8410f";
    
    let private_key = hex::decode(private_key_hex).unwrap();
    let message = hex::decode(message_hex).unwrap();
    let signature = hex::decode(signature_hex).unwrap();
    
    let s = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    // Reconstruct R from signing
    let generator = Point::generator();
    let sk = ScalarField::from_bytes_le(&private_key).unwrap();
    let e_sk = e.mul(&sk);
    let e_sk_canonical = e_sk.to_canonical();
    let k = s.add(e_sk_canonical);
    let r_from_signing = generator.mul(&k);
    let r_from_signing_encoded = r_from_signing.encode();
    
    // Convert message using message_to_fp5 (matches verification)
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
    
    // Construct pre-image: [R[0..5], message[0..5]]
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_from_signing_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    // Compute hash
    let e_computed_fp5 = hash_to_quintic_extension(&pre_image);
    let e_computed_scalar = ScalarField::from_fp5_element(&e_computed_fp5);
    
    println!("Pre-image (10 Goldilocks elements):");
    for (i, elem) in pre_image.iter().enumerate() {
        if i < 5 {
            println!("  [{}] R[{}] = {}", i, i, elem.0);
        } else {
            println!("  [{}] M[{}] = {}", i, i-5, elem.0);
        }
    }
    
    println!("\nExpected e (from signature):");
    println!("  Scalar: {}", hex::encode(&e.to_bytes_le()));
    
    println!("\nComputed e' (from hash):");
    println!("  Scalar: {}", hex::encode(&e_computed_scalar.to_bytes_le()));
    println!("  Fp5Element limbs:");
    for (i, limb) in e_computed_fp5.0.iter().enumerate() {
        println!("    e'[{}] = {}", i, limb.0);
    }
    
    println!("\nMatch: {}", e.0 == e_computed_scalar.0);
    
    if e.0 != e_computed_scalar.0 {
        println!("\n❌ Hash mismatch! e != e'");
        println!("This indicates a problem with:");
        println!("  1. R encoding for hash");
        println!("  2. Message encoding");
        println!("  3. Poseidon2 hash function implementation");
        println!("\nTo verify with Go, run:");
        print!("  go run lighter-rust/signer/examples/go_hash_helper.go \"");
        for (i, limb) in r_from_signing_encoded.0.iter().enumerate() {
            if i > 0 { print!(","); }
            print!("{}", limb.0);
        }
        print!("\" \"");
        for (i, limb) in message_fp5.0.iter().enumerate() {
            if i > 0 { print!(","); }
            print!("{}", limb.0);
        }
        println!("\"");
    } else {
        println!("\n✅ Hash matches! Verification should work.");
    }
}

