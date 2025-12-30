// Investigate failing nonce values

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 INVESTIGATING FAILING NONCE VALUES\n");

    // Generate keys
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Create message
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    
    println!("Testing 100 random nonces and collecting failing values...\n");
    
    let mut failing_nonces = Vec::new();
    let mut passing_nonces = Vec::new();
    
    for i in 0..100 {
        let nonce_scalar = ScalarField::sample_crypto();
        let nonce = nonce_scalar.to_bytes_le();
        
        let sig = sign_hashed_message(&private_key_bytes, &message, &nonce)?;
        let valid = verify_signature(&sig, &message, &public_key_bytes)?;
        
        if valid {
            passing_nonces.push(nonce);
        } else {
            failing_nonces.push(nonce);
            if failing_nonces.len() <= 5 {
                println!("[{}] ❌ FAILED:", i + 1);
                println!("  First 8 bytes: {:?}", &nonce[..8]);
                println!("  As u64: {}", u64::from_le_bytes(nonce[..8].try_into().unwrap()));
                println!();
            }
        }
    }
    
    println!("Results:");
    println!("  Passed: {}/100", passing_nonces.len());
    println!("  Failed: {}/100\n", failing_nonces.len());
    
    if !failing_nonces.is_empty() {
        println!("Pattern analysis:");
        
        // Check if failing nonces have a pattern
        let mut high_bits_set = 0;
        for nonce in &failing_nonces {
            let first_u64 = u64::from_le_bytes(nonce[..8].try_into().unwrap());
            // Check if highest bit is set
            if first_u64 & (1u64 << 63) != 0 {
                high_bits_set += 1;
            }
        }
        
        println!("  Failing nonces with high bit set: {}/{}", high_bits_set, failing_nonces.len());
        println!("  Pattern: {}%", (high_bits_set as f64 / failing_nonces.len() as f64) * 100.0);
    }
    
    Ok(())
}
