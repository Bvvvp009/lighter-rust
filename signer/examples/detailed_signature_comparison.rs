//! Detailed Signature Comparison: Rust vs Go
//!
//! This tool helps identify why Rust and Go produce different signatures
//! for the same inputs.

use std::env;
use hex;
use goldilocks_crypto::{schnorr, ScalarField, Point, Goldilocks, CryptoError, Result};
use poseidon_hash::hash_to_quintic_extension;

// Helper function to convert message bytes to Fp5Element (matches Go's FromCanonicalLittleEndianBytes)
fn message_to_fp5(message: &[u8]) -> Result<goldilocks_crypto::Fp5Element> {
    if message.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(message.len()));
    }
    let mut message_elements = [Goldilocks::zero(); 5];
    for (i, chunk) in message.chunks(8).enumerate().take(5) {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        // CRITICAL FIX: Go SDK's FromCanonicalLittleEndianBytes reverses bytes before
        // calling SetBytesCanonical (which expects big-endian). We need to match this.
        bytes.reverse();
        message_elements[i] = Goldilocks::from_canonical_u64(u64::from_be_bytes(bytes));
    }
    Ok(goldilocks_crypto::Fp5Element(message_elements))
}

fn load_dotenv() {
    if let Ok(current_dir) = env::current_dir() {
        let env_files = [
            current_dir.join(".env"),
            current_dir.join("..").join(".env"),
            current_dir.join("..").join("..").join(".env"),
        ];
        for env_file in env_files.iter() {
            if env_file.exists() {
                if let Ok(content) = std::fs::read_to_string(env_file) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"').trim_matches('\'');
                            if env::var(key).is_err() {
                                env::set_var(key, value);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn main() {
    load_dotenv();
    
    println!("🔍 Detailed Signature Comparison: Rust vs Go\n");
    println!("{}", "=".repeat(80));
    
    let api_private_key = env::var("API_PRIVATE_KEY")
        .expect("API_PRIVATE_KEY environment variable is required");
    
    // Test with all-zero message (deterministic for comparison)
    let message_bytes = [0u8; 40];
    let message_hex = hex::encode(&message_bytes);
    
    println!("Test Configuration:");
    println!("  Private Key: {}", api_private_key);
    println!("  Message (hex): {}", message_hex);
    println!("  Message: {} bytes of zeros\n", message_bytes.len());
    
    // Generate Rust signature
    println!("{}", "=".repeat(80));
    println!("Rust Signature Generation:");
    println!("{}", "=".repeat(80));
    
    let private_key_bytes = hex::decode(&api_private_key)
        .expect("Failed to decode private key");
    let rust_signature = schnorr::sign(&private_key_bytes, &message_bytes)
        .expect("Failed to sign");
    
    let rust_s = &rust_signature[0..40];
    let rust_e = &rust_signature[40..80];
    
    println!("Rust Signature:");
    println!("  s: {}", hex::encode(rust_s));
    println!("  e: {}", hex::encode(rust_e));
    
    // Parse scalars to analyze
    let rust_s_scalar = ScalarField::from_bytes_le(rust_s)
        .unwrap_or_else(|e| panic!("Failed to parse s: {}", e));
    let rust_e_scalar = ScalarField::from_bytes_le(rust_e)
        .unwrap_or_else(|e| panic!("Failed to parse e: {}", e));
    
    println!("\nRust Scalar Analysis:");
    println!("  s (canonical): {}", hex::encode(&rust_s_scalar.to_canonical().to_bytes_le()));
    println!("  e (canonical): {}", hex::encode(&rust_e_scalar.to_canonical().to_bytes_le()));
    
    // Reconstruct R from Rust signature
    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes)
        .unwrap_or_else(|e| panic!("Failed to parse private key: {}", e));
    let generator = Point::generator();
    let public_point = generator.mul(&private_scalar);
    
    // Try both verification methods
    println!("\n{}", "=".repeat(80));
    println!("Rust Verification (R = s*G + e*P):");
    println!("{}", "=".repeat(80));
    
    // Method 1: e directly (like Go)
    let s_g_direct = generator.mul(&rust_s_scalar);
    let e_pk_direct = public_point.mul(&rust_e_scalar);
    let r_point_direct = s_g_direct.add(&e_pk_direct);
    let r_encoded_direct = r_point_direct.encode();
    
    println!("Method 1: e directly (like Go):");
    println!("  R: {}", hex::encode(&r_encoded_direct.to_bytes_le()));
    
    // Method 2: e.monty_mul(&ONE) (our current working approach)
    let e_adjusted = rust_e_scalar.monty_mul(&ScalarField::ONE);
    let s_g_adjusted = generator.mul(&rust_s_scalar);
    let e_pk_adjusted = public_point.mul(&e_adjusted);
    let r_point_adjusted = s_g_adjusted.add(&e_pk_adjusted);
    let r_encoded_adjusted = r_point_adjusted.encode();
    
    println!("Method 2: e.monty_mul(&ONE) (our current approach):");
    println!("  R: {}", hex::encode(&r_encoded_adjusted.to_bytes_le()));
    
    // Compute e' from both R values
    let message_fp5 = message_to_fp5(&message_bytes)
        .expect("Failed to convert message");
    
    // For direct method
    let mut pre_image_direct = [Goldilocks::zero(); 10];
    pre_image_direct[..5].copy_from_slice(&r_encoded_direct.0);
    pre_image_direct[5..].copy_from_slice(&message_fp5.0);
    let e_prime_fp5_direct = hash_to_quintic_extension(&pre_image_direct);
    let e_prime_scalar_direct = ScalarField::from_fp5_element(&e_prime_fp5_direct);
    
    println!("\nVerification Results:");
    println!("  Method 1 (e directly):");
    println!("    Computed e': {}", hex::encode(&e_prime_scalar_direct.to_canonical().to_bytes_le()));
    println!("    Expected e:  {}", hex::encode(&rust_e_scalar.to_canonical().to_bytes_le()));
    let match_direct = e_prime_scalar_direct.to_canonical().equals(&rust_e_scalar.to_canonical());
    println!("    Match: {}", if match_direct { "✅" } else { "❌" });
    
    // For adjusted method
    let mut pre_image_adjusted = [Goldilocks::zero(); 10];
    pre_image_adjusted[..5].copy_from_slice(&r_encoded_adjusted.0);
    pre_image_adjusted[5..].copy_from_slice(&message_fp5.0);
    let e_prime_fp5_adjusted = hash_to_quintic_extension(&pre_image_adjusted);
    let e_prime_scalar_adjusted = ScalarField::from_fp5_element(&e_prime_fp5_adjusted);
    
    println!("  Method 2 (e.monty_mul(&ONE)):");
    println!("    Computed e': {}", hex::encode(&e_prime_scalar_adjusted.to_canonical().to_bytes_le()));
    println!("    Expected e:  {}", hex::encode(&rust_e_scalar.to_canonical().to_bytes_le()));
    let match_adjusted = e_prime_scalar_adjusted.to_canonical().equals(&rust_e_scalar.to_canonical());
    println!("    Match: {}", if match_adjusted { "✅" } else { "❌" });
    
    println!("\n{}", "=".repeat(80));
    println!("Go Signature (from trace_go_signing.go):");
    println!("{}", "=".repeat(80));
    println!("  s: d8068b6614b2f4efb26e27b2b593b47408c0dd680a0cd4e03cf73a9f3ca4ca88e4661bbf52ec8660");
    println!("  e: e0b9eeebbc46504e25e560e30365f55f0535c8bc1097d4a18ff1bae717bd29ae69d64e1026cd5738");
    println!("  R: 16de8498bcb3325b98d07ef94d4da3514baee73a8b8038b2048ed0ec6520a12f003175147ff32a05");
    
    println!("\n{}", "=".repeat(80));
    println!("Comparison:");
    println!("{}", "=".repeat(80));
    println!("Rust s: {}", hex::encode(rust_s));
    println!("Go   s: d8068b6614b2f4efb26e27b2b593b47408c0dd680a0cd4e03cf73a9f3ca4ca88e4661bbf52ec8660");
    println!("Match: {}", if hex::encode(rust_s) == "d8068b6614b2f4efb26e27b2b593b47408c0dd680a0cd4e03cf73a9f3ca4ca88e4661bbf52ec8660" { "✅" } else { "❌ DIFFERENT" });
    
    println!("\nRust e: {}", hex::encode(rust_e));
    println!("Go   e: e0b9eeebbc46504e25e560e30365f55f0535c8bc1097d4a18ff1bae717bd29ae69d64e1026cd5738");
    println!("Match: {}", if hex::encode(rust_e) == "e0b9eeebbc46504e25e560e30365f55f0535c8bc1097d4a18ff1bae717bd29ae69d64e1026cd5738" { "✅" } else { "❌ DIFFERENT" });
    
    println!("\n{}", "=".repeat(80));
    println!("Key Finding:");
    println!("{}", "=".repeat(80));
    println!("Signatures are DIFFERENT - this means:");
    println!("  1. Our signing produces different signatures than Go");
    println!("  2. This could be due to:");
    println!("     - Different nonce generation (random)");
    println!("     - Different scalar arithmetic");
    println!("     - Different point encoding");
    println!("  3. Need to investigate signing process step-by-step");
}
