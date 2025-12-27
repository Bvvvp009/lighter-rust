//! Test to verify s computation during signing matches expected

use goldilocks_crypto::{ScalarField, Point, sign};
use hex;

#[test]
fn test_s_computation_verification() {
    println!("\n=== Testing s Computation During Signing ===\n");
    
    // Generate key pair
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let public_key_point = Point::generator().mul(&private_key);
    
    let message = [0u8; 40];
    
    // Sign the message
    let signature = sign(&private_key_bytes, &message).unwrap();
    
    // Extract s and e from signature
    let s_from_sig = ScalarField::from_bytes_le(&signature[..40]).unwrap();
    let e_from_sig = ScalarField::from_bytes_le(&signature[40..]).unwrap();
    
    println!("From signature:");
    println!("  s: {}...", hex::encode(&s_from_sig.to_bytes_le()[..8]));
    println!("  e: {}...", hex::encode(&e_from_sig.to_bytes_le()[..8]));
    
    // Now manually recompute what should have happened during signing
    // We need to simulate the signing process to get the nonce
    // But we don't have the nonce... so let's work backwards
    
    // Verify: s + e*sk should equal some k
    let e_times_sk = e_from_sig.mul(&private_key);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k_reconstructed = s_from_sig.add(e_times_sk_canonical);
    
    println!("\nReconstructed k = s + e*sk:");
    println!("  k: {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    
    // Compute expected R = k*G
    let generator = Point::generator();
    let expected_r = generator.mul(&k_reconstructed);
    let expected_r_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}...", hex::encode(&expected_r_encoded.to_bytes_le()[..16]));
    
    // Compute R using mul_add2
    let computed_r = Point::mul_add2(&generator, &public_key_point, &s_from_sig, &e_from_sig);
    let computed_r_encoded = computed_r.encode();
    
    println!("\nComputed R (s*G + e*P using mul_add2):");
    println!("  Encoded: {}...", hex::encode(&computed_r_encoded.to_bytes_le()[..16]));
    
    let match_result = expected_r_encoded.0.iter().zip(computed_r_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if match_result {
        println!("\n✅ s computation is correct, mul_add2 works!");
    } else {
        println!("\n❌ Issue found!");
        
        // Check if s is correct by verifying: s = k - e*sk
        // We have k_reconstructed, so let's verify
        let s_expected = k_reconstructed.sub(e_times_sk_canonical);
        println!("\nVerifying s computation:");
        println!("  s from signature: {}...", hex::encode(&s_from_sig.to_bytes_le()[..8]));
        println!("  s expected (k - e*sk): {}...", hex::encode(&s_expected.to_bytes_le()[..8]));
        println!("  s matches: {}", s_from_sig.0 == s_expected.0);
        
        if s_from_sig.0 != s_expected.0 {
            println!("\n  ❌ s is computed incorrectly during signing!");
            for i in 0..5 {
                if s_from_sig.0[i] != s_expected.0[i] {
                    println!("    Limb[{}]: sig=0x{:016x}, expected=0x{:016x}, diff=0x{:016x}",
                        i, s_from_sig.0[i], s_expected.0[i], s_from_sig.0[i] ^ s_expected.0[i]);
                }
            }
        } else {
            println!("\n  ✅ s is computed correctly");
            println!("  ❌ But mul_add2 still doesn't match - bug in mul_add2!");
        }
    }
}








