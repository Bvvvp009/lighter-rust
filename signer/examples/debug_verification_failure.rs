//! Debug Verification Failure - Detailed analysis of why signatures fail verification
//!
//! This tool generates a signature and then traces through the verification process
//! step-by-step to identify where it fails.
//!
//! Usage: cargo run --example debug_verification_failure --release

use goldilocks_crypto::{sign, verify_signature, ScalarField, Point, Fp5Element, Goldilocks};
use poseidon_hash::hash_to_quintic_extension;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug Signature Verification Failure\n");
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
    
    // Sign a message
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
    
    // Now verify step by step
    println!("{}", "=".repeat(80));
    println!("VERIFICATION PROCESS");
    println!("{}", "=".repeat(80));
    
    // Step 1: Convert message to Fp5Element (same way verification does)
    let message_fp5 = Fp5Element::from_bytes_le(&message)?;
    println!("Message Fp5Element: {:?}", message_fp5.0.iter().map(|g| g.0).collect::<Vec<_>>());
    println!();
    
    // Step 2: Decode public key
    let public_key_fp5 = Fp5Element::from_bytes_le(&public_key_bytes)?;
    let public_point_verify = Point::decode(&public_key_fp5)
        .ok_or("Failed to decode public key")?;
    println!("Public key decoded successfully");
    println!();
    
    // Step 3: Compute R = s*G + e*P
    println!("Computing R = s*G + e*P");
    let s_g = generator.mul(&s);
    println!("  s*G computed");
    
    let e_pk = public_point_verify.mul(&e);
    println!("  e*P computed");
    
    let r_point = s_g.add(&e_pk);
    let r_encoded = r_point.encode();
    println!("  R = s*G + e*P computed");
    println!("  R encoded: {}", hex::encode(&r_encoded.to_bytes_le()));
    println!();
    
    // Step 4: Compute e' = H(R || message)
    println!("Computing e' = H(R || message)");
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    println!("  Pre-image (R || message):");
    for (i, g) in pre_image.iter().enumerate() {
        println!("    [{}] = {}", i, g.0);
    }
    
    let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
    let e_prime_scalar = ScalarField::from_fp5_element(&e_prime_fp5);
    
    println!("  e' computed: {}", hex::encode(&e_prime_scalar.to_bytes_le()));
    println!();
    
    // Step 5: Compare e and e'
    println!("Comparison:");
    println!("  e  (from signature): {}", hex::encode(&e.to_bytes_le()));
    println!("  e' (computed):       {}", hex::encode(&e_prime_scalar.to_bytes_le()));
    
    // Convert both to canonical for comparison
    let e_canonical = e.to_canonical();
    let e_prime_canonical = e_prime_scalar.to_canonical();
    
    println!("  e  (canonical):      {}", hex::encode(&e_canonical.to_bytes_le()));
    println!("  e' (canonical):      {}", hex::encode(&e_prime_canonical.to_bytes_le()));
    
    let matches = e_canonical.equals(&e_prime_canonical);
    println!("  Match: {}", if matches { "✅ YES" } else { "❌ NO" });
    println!();
    
    // Also verify using the actual verify_signature function
    println!("Using verify_signature() function:");
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)?;
    println!("  Result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    println!();
    
    // Additional debugging: Check if s + e*sk = k (reconstruct k)
    println!("Arithmetic Check:");
    let e_times_sk = e.mul(&private_key);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k_reconstructed = s.add(e_times_sk_canonical);
    
    println!("  e*sk (canonical):     {}", hex::encode(&e_times_sk_canonical.to_bytes_le()));
    println!("  k = s + e*sk:         {}", hex::encode(&k_reconstructed.to_bytes_le()));
    
    // Compute R from k*G to compare
    let r_from_k = generator.mul(&k_reconstructed);
    let r_from_k_encoded = r_from_k.encode();
    println!("  R from k*G:           {}", hex::encode(&r_from_k_encoded.to_bytes_le()));
    println!("  R from s*G + e*P:     {}", hex::encode(&r_encoded.to_bytes_le()));
    
    let r_matches = r_from_k_encoded.0.iter().zip(r_encoded.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  R matches: {}", if r_matches { "✅ YES" } else { "❌ NO" });
    
    if !r_matches {
        println!("\n  ⚠️  R mismatch detected!");
        println!("  This means s*G + e*P != k*G, which suggests an issue with point arithmetic.");
        for i in 0..5 {
            if r_from_k_encoded.0[i].0 != r_encoded.0[i].0 {
                println!("    R[{}]: k*G={}, s*G+e*P={}, diff={}",
                    i, r_from_k_encoded.0[i].0, r_encoded.0[i].0,
                    r_from_k_encoded.0[i].0.wrapping_sub(r_encoded.0[i].0));
            }
        }
    }
    
    Ok(())
}

