//! Test Verification Arithmetic - Verify that s*G + e*P = k*G
//!
//! This test verifies the verification equation holds: s*G + e*P = k*G
//! where k is the nonce, s = k - e*sk, and P = sk*G
//!
//! Usage: cargo run --example test_verification_arithmetic --release

use goldilocks_crypto::{sign, ScalarField, Point, Fp5Element, Goldilocks};
use poseidon_hash::hash_to_quintic_extension;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Verification Arithmetic: s*G + e*P = k*G\n");
    println!("{}", "=".repeat(80));
    
    // Generate a key pair
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Compute public key
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    println!("Private Key: {}", hex::encode(&private_key_bytes));
    println!("Public Key:  {}", hex::encode(&public_key_bytes));
    println!();
    
    // Sign a message to get a signature
    let message = [0u8; 40];
    println!("Message: {}", hex::encode(&message));
    println!();
    
    let signature = sign(&private_key_bytes, &message)?;
    println!("Signature: {}", hex::encode(&signature));
    println!();
    
    // Extract s and e from signature
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];
    
    let s = ScalarField::from_bytes_le(s_bytes)?;
    let e = ScalarField::from_bytes_le(e_bytes)?;
    
    println!("From signature:");
    println!("  s: {}", hex::encode(&s.to_bytes_le()));
    println!("  e: {}", hex::encode(&e.to_bytes_le()));
    println!();
    
    // Convert message to Fp5Element for hash computation
    let message_fp5 = Fp5Element::from_bytes_le(&message)?;
    
    // Reconstruct k = s + e*sk
    // IMPORTANT: During signing, we compute s = k - e*sk where e*sk is converted to canonical
    // So when reconstructing, we must use the canonical form of e*sk
    println!("Reconstructing k = s + e*sk:");
    let e_times_sk = e.mul(&private_key);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    println!("  e*sk (Montgomery): {}", hex::encode(&e_times_sk.to_bytes_le()));
    println!("  e*sk (canonical):  {}", hex::encode(&e_times_sk_canonical.to_bytes_le()));
    
    // Try both forms to see which one works
    // Also try converting s to canonical explicitly
    let s_canonical = s.to_canonical();
    let k_reconstructed_canonical = s.add(e_times_sk_canonical);
    let k_reconstructed_canonical_s = s_canonical.add(e_times_sk_canonical);
    let k_reconstructed_montgomery = s.add(e_times_sk);
    println!("  k = s + e*sk (canonical):      {}", hex::encode(&k_reconstructed_canonical.to_bytes_le()));
    println!("  k = s_canon + e*sk (canonical): {}", hex::encode(&k_reconstructed_canonical_s.to_bytes_le()));
    println!("  k = s + e*sk (Montgomery):      {}", hex::encode(&k_reconstructed_montgomery.to_bytes_le()));
    
    // Test which k produces the correct e
    println!("\n  Testing which k produces correct e:");
    for (name, k_test) in [("canonical", k_reconstructed_canonical), ("canonical_s", k_reconstructed_canonical_s), ("montgomery", k_reconstructed_montgomery)] {
        let r_test = generator.mul(&k_test);
        let r_test_encoded = r_test.encode();
        let mut pre_image_test = [Goldilocks::zero(); 10];
        pre_image_test[..5].copy_from_slice(&r_test_encoded.0);
        pre_image_test[5..].copy_from_slice(&message_fp5.0);
        let e_test = hash_to_quintic_extension(&pre_image_test);
        let e_test_scalar = ScalarField::from_fp5_element(&e_test);
        let matches = e.equals(&e_test_scalar);
        println!("    {}: {}", name, if matches { "✅ MATCHES" } else { "❌ no match" });
        if matches {
            println!("      Using k from: {}", name);
        }
    }
    
    // Use the canonical version (as that's what signing uses) - but we'll test all
    let k_reconstructed = k_reconstructed_canonical;
    println!();
    
    // Compute R from k*G (this is what was computed during signing)
    println!("Computing R = k*G (from signing):");
    let r_from_k = generator.mul(&k_reconstructed);
    let r_from_k_encoded = r_from_k.encode();
    println!("  R: {}", hex::encode(&r_from_k_encoded.to_bytes_le()));
    
    // Verify that this R produces the correct e
    let message_fp5 = Fp5Element::from_bytes_le(&message)?;
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_from_k_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    let e_from_r = hash_to_quintic_extension(&pre_image);
    let e_from_r_scalar = ScalarField::from_fp5_element(&e_from_r);
    let e_matches = e.equals(&e_from_r_scalar);
    println!("  e from H(R||message): {}", hex::encode(&e_from_r_scalar.to_bytes_le()));
    println!("  e matches signature: {}", if e_matches { "✅ YES" } else { "❌ NO" });
    if !e_matches {
        println!("  ⚠️  WARNING: Reconstructed k doesn't produce correct e!");
        println!("     This suggests s + e*sk != k, meaning the arithmetic roundtrip fails!");
    }
    println!();
    
    // Compute R from s*G + e*P (verification method)
    println!("Computing R = s*G + e*P (verification method):");
    
    // Method 1: Direct e (no adjustment)
    println!("\n  Method 1: Using e directly");
    let s_g1 = generator.mul(&s);
    let e_p1 = public_point.mul(&e);
    let r_from_verify1 = s_g1.add(&e_p1);
    let r_from_verify1_encoded = r_from_verify1.encode();
    println!("    R: {}", hex::encode(&r_from_verify1_encoded.to_bytes_le()));
    let matches1 = r_from_k_encoded.0.iter().zip(r_from_verify1_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    println!("    Matches k*G: {}", if matches1 { "✅ YES" } else { "❌ NO" });
    
    // Method 2: e * R2_INV
    println!("\n  Method 2: Using e * R2_INV");
    let e_adjusted2 = e.mul(&ScalarField::R2_INV);
    let e_p2 = public_point.mul(&e_adjusted2);
    let r_from_verify2 = s_g1.add(&e_p2);
    let r_from_verify2_encoded = r_from_verify2.encode();
    println!("    e_adjusted: {}", hex::encode(&e_adjusted2.to_bytes_le()));
    println!("    R: {}", hex::encode(&r_from_verify2_encoded.to_bytes_le()));
    let matches2 = r_from_k_encoded.0.iter().zip(r_from_verify2_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    println!("    Matches k*G: {}", if matches2 { "✅ YES" } else { "❌ NO" });
    
    // Method 3: e.monty_mul(ONE) - converts to (e/R mod N) form
    println!("\n  Method 3: Using e.monty_mul(ONE)");
    let e_adjusted3 = e.monty_mul(&ScalarField::ONE);
    let e_p3 = public_point.mul(&e_adjusted3);
    let r_from_verify3 = s_g1.add(&e_p3);
    let r_from_verify3_encoded = r_from_verify3.encode();
    println!("    e_adjusted: {}", hex::encode(&e_adjusted3.to_bytes_le()));
    println!("    R: {}", hex::encode(&r_from_verify3_encoded.to_bytes_le()));
    let matches3 = r_from_k_encoded.0.iter().zip(r_from_verify3_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    println!("    Matches k*G: {}", if matches3 { "✅ YES" } else { "❌ NO" });
    
    // Method 4: Check if s needs adjustment too
    println!("\n  Method 4: Adjusting both s and e");
    let s_adjusted4 = s.monty_mul(&ScalarField::ONE);
    let e_adjusted4 = e.monty_mul(&ScalarField::ONE);
    let s_g4 = generator.mul(&s_adjusted4);
    let e_p4 = public_point.mul(&e_adjusted4);
    let r_from_verify4 = s_g4.add(&e_p4);
    let r_from_verify4_encoded = r_from_verify4.encode();
    println!("    R: {}", hex::encode(&r_from_verify4_encoded.to_bytes_le()));
    let matches4 = r_from_k_encoded.0.iter().zip(r_from_verify4_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    println!("    Matches k*G: {}", if matches4 { "✅ YES" } else { "❌ NO" });
    
    println!("\n{}", "=".repeat(80));
    println!("Summary:");
    println!("  Method 1 (e direct):        {}", if matches1 { "✅" } else { "❌" });
    println!("  Method 2 (e * R2_INV):      {}", if matches2 { "✅" } else { "❌" });
    println!("  Method 3 (e.monty_mul(ONE)): {}", if matches3 { "✅" } else { "❌" });
    println!("  Method 4 (both adjusted):   {}", if matches4 { "✅" } else { "❌" });
    
    if matches1 || matches2 || matches3 || matches4 {
        println!("\n✅ Found a working method!");
    } else {
        println!("\n❌ None of the methods work - there's a deeper issue");
    }
    
    Ok(())
}

