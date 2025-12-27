//! Test to verify the mathematical relationship: s*G + e*P = k*G where k = s + e*sk

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_mul_add2_mathematical_relationship() {
    println!("\n=== Testing mul_add2 Mathematical Relationship ===\n");
    
    // Generate test values
    let sk = ScalarField::sample_crypto();
    let k = ScalarField::sample_crypto();
    let e = ScalarField::sample_crypto();
    
    // Compute s = k - e*sk (as done in signing)
    let e_sk = e.mul(&sk); // mul() returns canonical
    let s = k.sub(e_sk);
    
    println!("sk: {}...", hex::encode(&sk.to_bytes_le()[..8]));
    println!("k: {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("e: {}...", hex::encode(&e.to_bytes_le()[..8]));
    println!("s: {}...", hex::encode(&s.to_bytes_le()[..8]));
    
    // Verify: k should equal s + e*sk
    let k_reconstructed = s.add(e_sk);
    println!("\nVerifying: k == s + e*sk");
    println!("k (original): {}...", hex::encode(&k.to_bytes_le()[..8]));
    println!("k (reconstructed): {}...", hex::encode(&k_reconstructed.to_bytes_le()[..8]));
    println!("Match: {}", k.to_bytes_le() == k_reconstructed.to_bytes_le());
    
    assert_eq!(k.to_bytes_le(), k_reconstructed.to_bytes_le(), "k should equal s + e*sk");
    
    // Compute points
    let generator = Point::generator();
    let public_point = generator.mul(&sk);
    
    // Compute k*G (what R should be during signing)
    let r_signing = generator.mul(&k);
    let r_signing_encoded = r_signing.encode();
    println!("\nR from signing (k*G):");
    println!("  Encoded: {}...", hex::encode(&r_signing_encoded.to_bytes_le()[..16]));
    
    // Compute s*G + e*P using mul_add2 (what verification does)
    let r_verification = Point::mul_add2(&generator, &public_point, &s, &e);
    let r_verification_encoded = r_verification.encode();
    println!("\nR from verification (s*G + e*P using mul_add2):");
    println!("  Encoded: {}...", hex::encode(&r_verification_encoded.to_bytes_le()[..16]));
    
    // Compare
    let match_encoded = r_signing_encoded.to_bytes_le() == r_verification_encoded.to_bytes_le();
    println!("\nR (signing) == R (verification): {}", match_encoded);
    
    if !match_encoded {
        println!("\n❌ FAILED: R from verification doesn't match R from signing!");
        println!("  This means mul_add2 is not computing s*G + e*P correctly");
        
        // Try computing separately
        println!("\n  Trying separate computation: (s*G).add(e*P)");
        let s_g = generator.mul(&s);
        let e_p = public_point.mul(&e);
        let r_separate = s_g.add(&e_p);
        let r_separate_encoded = r_separate.encode();
        println!("  R (separate): {}...", hex::encode(&r_separate_encoded.to_bytes_le()[..16]));
        let match_separate = r_signing_encoded.to_bytes_le() == r_separate_encoded.to_bytes_le();
        println!("  R (signing) == R (separate): {}", match_separate);
    } else {
        println!("\n✅ PASSED: mul_add2 correctly computes s*G + e*P = k*G");
    }
    
    assert_eq!(r_signing_encoded.to_bytes_le(), r_verification_encoded.to_bytes_le(), 
               "R from verification should match R from signing");
}



