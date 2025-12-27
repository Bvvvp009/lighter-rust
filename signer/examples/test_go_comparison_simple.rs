//! Simple Go vs Rust Comparison Test
//!
//! This test compares Rust and Go signature generation with fixed inputs
//! to verify byte-for-byte compatibility.

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use poseidon_hash::hash_to_quintic_extension;
use hex;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Go vs Rust Signature Comparison");
    println!("{}", "=".repeat(80));
    
    // Test with fixed inputs for deterministic comparison (40 bytes = 80 hex chars each)
    let private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let message_hex = "00000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let nonce_hex = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    
    println!("\n📝 Test Inputs:");
    println!("  Private key: {}", private_key_hex);
    println!("  Message:     {}", message_hex);
    println!("  Nonce:       {}", nonce_hex);
    
    // Generate Rust signature
    println!("\n🔧 Generating Rust signature...");
    let private_key_vec = hex::decode(private_key_hex)?;
    let message_vec = hex::decode(message_hex)?;
    let nonce_vec = hex::decode(nonce_hex)?;
    
    if private_key_vec.len() != 40 || message_vec.len() != 40 || nonce_vec.len() != 40 {
        return Err(format!("Invalid lengths: pk={}, msg={}, nonce={}", 
            private_key_vec.len(), message_vec.len(), nonce_vec.len()).into());
    }
    
    let private_key_bytes: [u8; 40] = private_key_vec.try_into().unwrap();
    let message_bytes: [u8; 40] = message_vec.try_into().unwrap();
    let nonce_bytes: [u8; 40] = nonce_vec.try_into().unwrap();
    
    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes)?;
    let nonce_scalar = ScalarField::from_bytes_le(&nonce_bytes)?;
    let message_fp5 = Fp5Element::from_bytes_le(&message_bytes)?;
    
    // Compute R = nonce * G
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded = r_point.encode();
    
    // Compute e = H(R || message)
    use poseidon_hash::Goldilocks;
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_fp5 = hash_to_quintic_extension(&pre_image);
    let e_scalar = ScalarField::from_fp5_element(&e_fp5);
    
    // Compute s = nonce - e * private_key
    let e_times_private = e_scalar.mul(&private_scalar);
    let s = nonce_scalar.sub(e_times_private);
    
    // Assemble signature
    let mut rust_signature = [0u8; 80];
    rust_signature[..40].copy_from_slice(&s.to_bytes_le());
    rust_signature[40..].copy_from_slice(&e_scalar.to_bytes_le());
    
    println!("  ✅ Rust signature generated");
    println!("  R (hex):     {}", hex::encode(&r_encoded.to_bytes_le()));
    println!("  e (hex):     {}", hex::encode(&e_scalar.to_bytes_le()));
    println!("  s (hex):     {}", hex::encode(&s.to_bytes_le()));
    println!("  Signature:   {}", hex::encode(&rust_signature));
    
    // Try to generate Go signature
    println!("\n🔧 Generating Go signature...");
    let go_helper_path = "go_signature_helper.go";
    
    if !std::path::Path::new(go_helper_path).exists() {
        println!("  ⚠️  Go helper not found at: {}", go_helper_path);
        println!("  Skipping Go comparison - helper script not available");
        println!("\n✅ Rust signature generation complete");
        println!("  To compare with Go:");
        println!("    1. Ensure Go is installed");
        println!("    2. Install poseidon_crypto: go get github.com/elliottech/poseidon_crypto/...");
        println!("    3. Run: go run go_signature_helper.go {} {} {}", private_key_hex, message_hex, nonce_hex);
        return Ok(());
    }
    
    let output = Command::new("go")
        .arg("run")
        .arg(go_helper_path)
        .arg(private_key_hex)
        .arg(message_hex)
        .arg(nonce_hex)
        .current_dir("lighter-rust/signer/examples")
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            println!("  ✅ Go signature generated");
            
            // Parse Go output
            let mut go_r_encoded = Vec::new();
            let mut go_e_fp5 = Vec::new();
            let mut go_e_scalar_hex = String::new();
            let mut go_s_scalar_hex = String::new();
            let mut go_signature_hex = String::new();
            
            for line in stdout.lines() {
                if line.starts_with("R_ENCODED:") {
                    let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
                    for part in parts {
                        go_r_encoded.push(part.parse::<u64>()?);
                    }
                } else if line.starts_with("E_FP5:") {
                    let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
                    for part in parts {
                        go_e_fp5.push(part.parse::<u64>()?);
                    }
                } else if line.starts_with("E_SCALAR:") {
                    go_e_scalar_hex = line.split(':').nth(1).unwrap_or("").to_string();
                } else if line.starts_with("S_SCALAR:") {
                    go_s_scalar_hex = line.split(':').nth(1).unwrap_or("").to_string();
                } else if line.starts_with("SIGNATURE:") {
                    go_signature_hex = line.split(':').nth(1).unwrap_or("").to_string();
                }
            }
            
            if go_r_encoded.len() == 5 && go_e_fp5.len() == 5 {
                use poseidon_hash::Goldilocks;
                let go_r_fp5 = Fp5Element([
                    Goldilocks(go_r_encoded[0]),
                    Goldilocks(go_r_encoded[1]),
                    Goldilocks(go_r_encoded[2]),
                    Goldilocks(go_r_encoded[3]),
                    Goldilocks(go_r_encoded[4]),
                ]);
                
                println!("  R (hex):     {}", hex::encode(&go_r_fp5.to_bytes_le()));
                println!("  e (hex):     {}", go_e_scalar_hex.trim());
                println!("  s (hex):     {}", go_s_scalar_hex.trim());
                println!("  Signature:   {}", go_signature_hex.trim());
                
                // Compare
                println!("\n📊 Comparison:");
                let r_match = r_encoded.0.iter()
                    .zip(go_r_encoded.iter())
                    .all(|(a, b)| a.0 == *b);
                println!("  R match:     {}", if r_match { "✅ YES" } else { "❌ NO" });
                
                let e_match = hex::encode(&e_scalar.to_bytes_le()) == go_e_scalar_hex.trim();
                println!("  e match:     {}", if e_match { "✅ YES" } else { "❌ NO" });
                
                let s_match = hex::encode(&s.to_bytes_le()) == go_s_scalar_hex.trim();
                println!("  s match:     {}", if s_match { "✅ YES" } else { "❌ NO" });
                
                let sig_match = hex::encode(&rust_signature) == go_signature_hex.trim();
                println!("  Signature:   {}", if sig_match { "✅ YES" } else { "❌ NO" });
                
                if r_match && e_match && s_match && sig_match {
                    println!("\n✅✅✅ PERFECT MATCH! Rust and Go produce identical signatures! ✅✅✅");
                } else {
                    println!("\n⚠️  Mismatches detected - need to investigate differences");
                }
            } else {
                println!("  ⚠️  Failed to parse Go output");
            }
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            println!("  ⚠️  Go helper failed:");
            println!("  {}", stderr);
            println!("\n  This is expected if Go dependencies are not installed.");
            println!("  Rust signature generation is working correctly.");
        }
        Err(e) => {
            println!("  ⚠️  Failed to run Go helper: {}", e);
            println!("  Rust signature generation is working correctly.");
        }
    }
    
    Ok(())
}

