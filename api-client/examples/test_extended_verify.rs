// Extended simple test with multiple signatures

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 EXTENDED SIGN/VERIFY TEST - 10 Signatures");
    println!("=============================================\n");

    // Generate a test private key
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Create a test message (Poseidon2 hash of some data)
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    
    // Use fixed nonce
    let fixed_nonce = [0x01u8; 40];
    
    // Derive public key
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Sign 10 times
    println!("Creating 10 signatures with fixed nonce...");
    let mut signatures = Vec::new();
    for i in 0..10 {
        let signature = sign_hashed_message(&private_key_bytes, &message, &fixed_nonce)?;
        println!("  [{}] Sig: {}...{}", i+1, hex::encode(&signature[..8]), hex::encode(&signature[72..]));
        signatures.push(signature);
    }
    
    // Verify all
    println!("\nVerifying all 10 signatures...");
    let mut valid_count = 0;
    for (i, sig) in signatures.iter().enumerate() {
        match verify_signature(sig, &message, &public_key_bytes) {
            Ok(true) => {
                println!("  [{}] ✅ Valid", i + 1);
                valid_count += 1;
            }
            Ok(false) => {
                println!("  [{}] ❌ INVALID", i + 1);
            }
            Err(e) => {
                println!("  [{}] ❌ Error: {}", i + 1, e);
            }
        }
    }
    
    println!("\n📊 Results: {}/10 signatures verified", valid_count);
    if valid_count == 10 {
        println!("✅ ALL PASSED!");
    } else {
        println!("❌ SOME FAILED!");
    }
    
    Ok(())
}
