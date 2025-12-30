//! Test to verify mul_add2 computes correctly: mul_add2(a, b, s, e) should equal a.mul(&s).add(&b.mul(&e))
//! Even if Point::add() has issues, we can test with simple cases

use goldilocks_crypto::{ScalarField, Point};

#[test]
fn test_mul_add2_basic() {
    println!("\n=== Testing mul_add2 Basic Correctness ===\n");
    
    let generator = Point::generator();
    
    // Test with simple scalars: s=1, e=1
    // mul_add2(G, G, 1, 1) should equal G + G = 2*G
    let s_one = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let e_one = s_one; // Same scalar
    
    let result_mul_add2 = Point::mul_add2(&generator, &generator, &s_one, &e_one);
    let result_expected = generator.double(); // 2*G
    
    let result_mul_add2_encoded = result_mul_add2.encode();
    let result_expected_encoded = result_expected.encode();
    
    println!("mul_add2(G, G, 1, 1):");
    println!("  Encoded: {:?}", hex::encode(&result_mul_add2_encoded.to_bytes_le()));
    println!("Expected (2*G):");
    println!("  Encoded: {:?}", hex::encode(&result_expected_encoded.to_bytes_le()));
    
    let match_result = result_mul_add2_encoded.0.iter().zip(result_expected_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ mul_add2(G, G, 1, 1) matches 2*G");
    } else {
        println!("\n❌ mul_add2(G, G, 1, 1) does NOT match 2*G");
        println!("  mul_add2 limbs: {:?}", result_mul_add2_encoded.0);
        println!("  expected limbs: {:?}", result_expected_encoded.0);
    }
    
    // Test: mul_add2(G, P, s, e) where P = k*G, s=1, e=0 should equal G
    let k_scalar = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // k=5
        bytes
    }).unwrap();
    
    let public_point = generator.mul(&k_scalar);
    let zero_scalar = ScalarField::from_bytes_le(&[0u8; 40]).unwrap();
    
    let result2 = Point::mul_add2(&generator, &public_point, &s_one, &zero_scalar);
    let result2_encoded = result2.encode();
    let generator_encoded = generator.encode();
    
    println!("\nmul_add2(G, 5*G, 1, 0):");
    println!("  Encoded: {:?}", hex::encode(&result2_encoded.to_bytes_le()));
    println!("Expected (G):");
    println!("  Encoded: {:?}", hex::encode(&generator_encoded.to_bytes_le()));
    
    let match_result2 = result2_encoded.0.iter().zip(generator_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result2 {
        println!("\n✅ mul_add2(G, 5*G, 1, 0) matches G");
    } else {
        println!("\n❌ mul_add2(G, 5*G, 1, 0) does NOT match G");
    }
    
    // Test: mul_add2(G, P, s, e) where P = k*G, s=0, e=1 should equal P
    let result3 = Point::mul_add2(&generator, &public_point, &zero_scalar, &s_one);
    let result3_encoded = result3.encode();
    
    println!("\nmul_add2(G, 5*G, 0, 1):");
    println!("  Encoded: {:?}", hex::encode(&result3_encoded.to_bytes_le()));
    println!("Expected (5*G):");
    println!("  Encoded: {:?}", hex::encode(&public_point.encode().to_bytes_le()));
    
    let match_result3 = result3_encoded.0.iter().zip(public_point.encode().0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result3 {
        println!("\n✅ mul_add2(G, 5*G, 0, 1) matches 5*G");
    } else {
        println!("\n❌ mul_add2(G, 5*G, 0, 1) does NOT match 5*G");
    }
}

#[test]
fn test_mul_add2_reconstruction() {
    println!("\n=== Testing mul_add2 for R Reconstruction ===\n");
    
    // Test the actual verification scenario:
    // If s = k - e*sk, then s*G + e*P should equal k*G
    // where P = sk*G (public key)
    
    let generator = Point::generator();
    
    // Create a private key (sk)
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    let public_key = generator.mul(&sk); // P = 7*G
    
    // Create a nonce (k)
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11; // k = 11
        bytes
    }).unwrap();
    
    // Create e (challenge, arbitrary value for test)
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    // Compute s = k - e*sk = 11 - 3*7 = 11 - 21 = -10 mod ORDER
    let e_times_sk = e.mul(&sk);
    let s = k.sub(e_times_sk);
    
    println!("Test values:");
    println!("  sk (private key): {}", hex::encode(&sk.to_bytes_le()));
    println!("  k (nonce): {}", hex::encode(&k.to_bytes_le()));
    println!("  e (challenge): {}", hex::encode(&e.to_bytes_le()));
    println!("  s (response): {}", hex::encode(&s.to_bytes_le()));
    
    // Expected: k*G
    let expected_r = generator.mul(&k);
    let expected_r_encoded = expected_r.encode();
    
    // Compute with mul_add2: s*G + e*P
    let computed_r = Point::mul_add2(&generator, &public_key, &s, &e);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}", hex::encode(&expected_r_encoded.to_bytes_le()));
    println!("Computed R (s*G + e*P using mul_add2):");
    println!("  Encoded: {}", hex::encode(&computed_r_encoded.to_bytes_le()));
    
    let match_result = expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ mul_add2 correctly computes s*G + e*P = k*G");
    } else {
        println!("\n❌ mul_add2 does NOT correctly compute s*G + e*P = k*G");
        println!("  Expected limbs: {:?}", expected_r_encoded.0);
        println!("  Computed limbs: {:?}", computed_r_encoded.0);
    }
    
    assert!(match_result, "mul_add2 should compute s*G + e*P correctly");
}













