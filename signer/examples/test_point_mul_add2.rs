//! Test Point::mul_add2 correctness
//!
//! This test verifies that Point::mul_add2 correctly computes s*G + e*P
//! by comparing it with the equivalent computation: (s*G).add(e*P)

use goldilocks_crypto::{Point, ScalarField};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Point::mul_add2 correctness");
    println!("{}", "=".repeat(80));
    
    let generator = Point::generator();
    let mut failures = 0;
    let mut successes = 0;
    
    for i in 0..20 {
        // Generate random scalars
        let s = ScalarField::sample_crypto();
        let e = ScalarField::sample_crypto();
        let private_key = ScalarField::sample_crypto();
        let public_key = generator.mul(&private_key);
        
        // Method 1: Using mul_add2
        let r1 = Point::mul_add2(&generator, &public_key, &s, &e);
        
        // Method 2: Using separate multiplications and addition
        let s_g = generator.mul(&s);
        let e_pk = public_key.mul(&e);
        let r2 = s_g.add(&e_pk);
        
        // Check if they match
        let r1_encoded = r1.encode();
        let r2_encoded = r2.encode();
        
        let match_result = r1_encoded.0.iter()
            .zip(r2_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        
        if match_result {
            successes += 1;
            if i < 5 {
                println!("Test {}: ✅ mul_add2 matches separate computation", i + 1);
            }
        } else {
            failures += 1;
            println!("Test {}: ❌ mul_add2 does NOT match!", i + 1);
            println!("  mul_add2 result:     {}", hex::encode(&r1_encoded.to_bytes_le()));
            println!("  separate result:     {}", hex::encode(&r2_encoded.to_bytes_le()));
            
            // Also verify: s*G + e*PK should equal (s + e*sk)*G
            let k_reconstructed = s.add(e.mul(&private_key));
            let r3 = generator.mul(&k_reconstructed);
            let r3_encoded = r3.encode();
            println!("  (s + e*sk)*G result: {}", hex::encode(&r3_encoded.to_bytes_le()));
            
            let match_k = r1_encoded.0.iter()
                .zip(r3_encoded.0.iter())
                .all(|(a, b)| a.0 == b.0);
            println!("  Matches (s + e*sk)*G: {}", if match_k { "✅ YES" } else { "❌ NO" });
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("SUMMARY:");
    println!("  Successes: {}", successes);
    println!("  Failures: {}", failures);
    
    if failures > 0 {
        println!("\n  ⚠️  Point::mul_add2 has bugs!");
    } else {
        println!("\n  ✅ Point::mul_add2 works correctly");
    }
    
    Ok(())
}













