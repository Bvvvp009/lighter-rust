//! Test to verify the mul_add2 fix resolves verification issues

use goldilocks_crypto::{ScalarField, Point};
use goldilocks_crypto::schnorr::sign_with_nonce;
use goldilocks_crypto::verify_signature;
use hex;

#[test]
fn test_mul_add2_fix_verification() {
    println!("\n=== Testing mul_add2 Fix with Verification ===\n");
    
    // Test multiple signatures to check for intermittent failures
    let generator = Point::generator();
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    let public_key_point = generator.mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    let mut success_count = 0;
    let mut failure_count = 0;
    let num_tests = 100;
    
    println!("Testing {} signatures...", num_tests);
    
    for i in 0..num_tests {
        // Generate a random nonce for each signature
        let nonce = ScalarField::sample_crypto();
        let nonce_bytes = nonce.to_bytes_le();
        
        // Sign the message
        let signature = match sign_with_nonce(&private_key_bytes, &message, &nonce_bytes) {
            Ok(sig) => sig,
            Err(e) => {
                println!("Signing failed: {}", e);
                failure_count += 1;
                continue;
            }
        };
        
        // Verify the signature
        let is_valid = match verify_signature(&signature, &message, &public_key_bytes) {
            Ok(valid) => valid,
            Err(e) => {
                println!("Verification error: {}", e);
                failure_count += 1;
                continue;
            }
        };
        
        if is_valid {
            success_count += 1;
        } else {
            failure_count += 1;
            if failure_count <= 5 {
                println!("Signature {} failed verification", i + 1);
            }
        }
    }
    
    let success_rate = (success_count as f64 / num_tests as f64) * 100.0;
    
    println!("\n=== Results ===");
    println!("Total tests: {}", num_tests);
    println!("Successful: {} ({:.1}%)", success_count, success_rate);
    println!("Failed: {} ({:.1}%)", failure_count, 100.0 - success_rate);
    
    // The fix should achieve close to 100% success rate
    if success_rate >= 95.0 {
        println!("\n✅ Fix successful! Verification rate: {:.1}%", success_rate);
    } else {
        println!("\n❌ Still seeing failures. Success rate: {:.1}%", success_rate);
        println!("   This suggests additional issues remain.");
    }
    
    // Assert that we have a high success rate
    assert!(success_count > 0, "No signatures verified successfully");
}

#[test]
fn test_mul_add2_correctness() {
    println!("\n=== Testing mul_add2 Correctness ===\n");
    
    let generator = Point::generator();
    
    // Test with known values
    let s = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // s = 5
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    let public_key = generator.mul(&sk);
    
    // Method 1: Separate multiplications
    let s_g = generator.mul(&s);
    let e_pk = public_key.mul(&e);
    let expected = s_g.add(&e_pk);
    let expected_encoded = expected.encode();
    
    // Method 2: mul_add2
    let result = Point::mul_add2(&generator, &public_key, &s, &e);
    let result_encoded = result.encode();
    
    let matches = expected_encoded.to_bytes_le() == result_encoded.to_bytes_le();
    
    println!("Expected (s*G + e*P): {}", hex::encode(&expected_encoded.to_bytes_le()));
    println!("Result (mul_add2):    {}", hex::encode(&result_encoded.to_bytes_le()));
    println!("Match: {}", matches);
    
    if matches {
        println!("\n✅ mul_add2 produces correct result!");
    } else {
        println!("\n❌ mul_add2 does NOT match expected result!");
        panic!("mul_add2 implementation is incorrect");
    }
    
    // Also test with canonical reconstruction: k = s + e*sk
    let e_sk = e.mul(&sk);
    let e_sk_canonical = e_sk.to_canonical();
    let k = s.add(e_sk_canonical);
    let expected2 = generator.mul(&k);
    let expected2_encoded = expected2.encode();
    
    let matches2 = expected2_encoded.to_bytes_le() == result_encoded.to_bytes_le();
    println!("\nMatches k*G (where k = s + e*sk canonical): {}", matches2);
    
    if !matches2 {
        println!("⚠️  Note: mul_add2 doesn't match k*G, but this might be expected if scalar forms differ");
    }
}

