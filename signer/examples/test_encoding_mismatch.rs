//! Test Encoding Mismatch - Critical Root Cause Investigation
//!
//! This test verifies if Point::encode() and WeierstrassPoint::encode() produce
//! the same Fp5Element for the same point. If they differ, this is the root cause
//! of signature verification failures.
//!
//! Usage:
//!   cargo run --example test_encoding_mismatch --release

use goldilocks_crypto::{
    sign, verify_signature, ScalarField, Point, WeierstrassPoint, Fp5Element,
};
use hex;

fn test_encoding_consistency() {
    println!("🔍 Testing Encoding Consistency");
    println!("{}", "=".repeat(80));
    
    // Test with multiple random points
    let mut mismatches = 0;
    let mut matches = 0;
    
    for i in 0..10 {
        println!("\n--- Test {} ---", i + 1);
        
        // Generate a random point
        let scalar = ScalarField::sample_crypto();
        let point = Point::generator().mul(&scalar);
        
        // Encode using Point encoding (used during signing)
        let point_encoded = point.encode();
        
        // Decode as WeierstrassPoint
        let weierstrass_point = match WeierstrassPoint::decode_fp5_as_weierstrass(&point_encoded) {
            Some(p) => p,
            None => {
                println!("  ❌ Failed to decode Point encoding as WeierstrassPoint");
                mismatches += 1;
                continue;
            }
        };
        
        // Encode using WeierstrassPoint encoding (used during verification)
        let weierstrass_encoded = weierstrass_point.encode();
        
        // Check if encodings match
        let encodings_match = point_encoded.0.iter()
            .zip(weierstrass_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        
        if encodings_match {
            matches += 1;
            println!("  ✅ Encodings match");
        } else {
            mismatches += 1;
            println!("  ❌ Encodings DO NOT match!");
            println!("    Point encoding (hex):       {}", hex::encode(&point_encoded.to_bytes_le()));
            println!("    Weierstrass encoding (hex): {}", hex::encode(&weierstrass_encoded.to_bytes_le()));
            
            // Show the difference
            println!("    Difference:");
            for (idx, (p, w)) in point_encoded.0.iter().zip(weierstrass_encoded.0.iter()).enumerate() {
                if p.0 != w.0 {
                    println!("      Element {}: Point={}, Weierstrass={}, diff={}", 
                        idx, p.0, w.0, p.0.wrapping_sub(w.0));
                }
            }
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("SUMMARY:");
    println!("  Matches: {}", matches);
    println!("  Mismatches: {}", mismatches);
    
    if mismatches > 0 {
        println!("\n  ⚠️  CRITICAL: Encoding mismatch detected!");
        println!("  This is likely the root cause of signature verification failures.");
        println!("  Signing uses Point::encode(), verification uses WeierstrassPoint::encode()");
        println!("  If they differ, e != e' and verification fails!");
    } else {
        println!("\n  ✅ All encodings match - encoding is not the issue");
    }
}

fn test_signature_with_encoding_check() {
    println!("\n\n🔍 Testing Signature Generation with Encoding Check");
    println!("{}", "=".repeat(80));
    
    // Generate key pair
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    println!("Private key (hex): {}", hex::encode(&private_key_bytes));
    println!("Public key (hex):  {}", hex::encode(&public_key_bytes));
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    // Test multiple signatures
    for i in 0..20 {
        let message = [i as u8; 40]; // Different message each time
        
        // Sign
        let signature = match sign(&private_key_bytes, &message) {
            Ok(sig) => sig,
            Err(e) => {
                println!("  Test {}: ❌ Signing failed: {:?}", i + 1, e);
                failure_count += 1;
                continue;
            }
        };
        
        // Verify
        let is_valid = match verify_signature(&signature, &message, &public_key_bytes) {
            Ok(valid) => valid,
            Err(e) => {
                println!("  Test {}: ❌ Verification error: {:?}", i + 1, e);
                failure_count += 1;
                continue;
            }
        };
        
        if is_valid {
            success_count += 1;
            if i < 5 {
                println!("  Test {}: ✅ Signature verified", i + 1);
            }
        } else {
            failure_count += 1;
            println!("  Test {}: ❌ Signature verification FAILED", i + 1);
            
            // Analyze the failure
            let s_bytes = &signature[..40];
            let e_bytes = &signature[40..];
            let s = ScalarField::from_bytes_le(s_bytes).unwrap();
            let e = ScalarField::from_bytes_le(e_bytes).unwrap();
            
            // Reconstruct R from signing perspective
            let generator = Point::generator();
            let nonce_reconstructed = s.add(e.mul(&private_key));
            let r_point_signing = generator.mul(&nonce_reconstructed);
            let r_encoded_signing = r_point_signing.encode();
            
            // Reconstruct R from verification perspective
            let generator_ws = WeierstrassPoint::GENERATOR;
            let public_point_ws = WeierstrassPoint::decode_fp5_as_weierstrass(
                &Fp5Element::from_bytes_le(&public_key_bytes).unwrap()
            ).unwrap();
            let r_point_verification = WeierstrassPoint::mul_add2(&generator_ws, &public_point_ws, &s, &e);
            let r_encoded_verification = r_point_verification.encode();
            
            // Check if encodings match
            let r_match = r_encoded_signing.0.iter()
                .zip(r_encoded_verification.0.iter())
                .all(|(a, b)| a.0 == b.0);
            
            println!("    R encoding match: {}", if r_match { "✅ YES" } else { "❌ NO" });
            
            if !r_match {
                println!("    R from signing (Point):       {}", hex::encode(&r_encoded_signing.to_bytes_le()));
                println!("    R from verification (Weierstrass): {}", hex::encode(&r_encoded_verification.to_bytes_le()));
                println!("    ⚠️  This confirms the encoding mismatch is causing failures!");
            }
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("SIGNATURE TEST SUMMARY:");
    println!("  Successful verifications: {}", success_count);
    println!("  Failed verifications: {}", failure_count);
    let failure_rate = (failure_count as f64 / (success_count + failure_count) as f64) * 100.0;
    println!("  Failure rate: {:.1}%", failure_rate);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Encoding Mismatch Investigation");
    println!("{}", "=".repeat(80));
    println!("This tool tests if Point::encode() and WeierstrassPoint::encode()");
    println!("produce the same Fp5Element for the same point.\n");
    
    // Test 1: Encoding consistency
    test_encoding_consistency();
    
    // Test 2: Signature verification with encoding check
    test_signature_with_encoding_check();
    
    println!("\n\n{}", "=".repeat(80));
    println!("INVESTIGATION COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nIf encoding mismatches are found:");
    println!("  1. This confirms the root cause");
    println!("  2. Need to fix: Use same encoding in both sign() and verify_signature()");
    println!("  3. Check which encoding Go uses and match it");
    
    Ok(())
}









