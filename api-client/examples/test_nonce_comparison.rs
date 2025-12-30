// Test random vs fixed nonces

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 TESTING RANDOM vs FIXED NONCES\n");

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
    
    // Test 1: Fixed nonce (always works)
    println!("Test 1: Fixed nonce [0x01; 40]");
    let fixed_nonce = [0x01u8; 40];
    let sig_fixed = sign_hashed_message(&private_key_bytes, &message, &fixed_nonce)?;
    let valid_fixed = verify_signature(&sig_fixed, &message, &public_key_bytes)?;
    println!("  Result: {}\n", if valid_fixed { "✅ VALID" } else { "❌ INVALID" });
    
    // Test 2: Random nonce generated with sample_crypto
    println!("Test 2: Random nonce from ScalarField::sample_crypto");
    let random_nonce = ScalarField::sample_crypto().to_bytes_le();
    println!("  Random nonce first 8 bytes: {:?}", &random_nonce[..8]);
    let sig_random = sign_hashed_message(&private_key_bytes, &message, &random_nonce)?;
    let valid_random = verify_signature(&sig_random, &message, &public_key_bytes)?;
    println!("  Result: {}\n", if valid_random { "✅ VALID" } else { "❌ INVALID" });
    
    // Test 3: Multiple random nonces
    println!("Test 3: 20 random nonces");
    let mut failures = 0;
    for i in 0..20 {
        let random_scalar = ScalarField::sample_crypto();
        let nonce = random_scalar.to_bytes_le();
        
        // Debug: Verify round-trip
        let nonce_reconstructed = ScalarField::from_bytes_le(&nonce)?;
        let matches = random_scalar.equals(&nonce_reconstructed);
        
        let sig = sign_hashed_message(&private_key_bytes, &message, &nonce)?;
        let valid = verify_signature(&sig, &message, &public_key_bytes)?;
        if !valid {
            failures += 1;
            if i < 3 {
                println!("  [{}] ❌ INVALID (round-trip matches: {})", i + 1, matches);
            }
        }
    }
    println!("  Failures: {}/20\n", failures);
    
    if failures == 0 {
        println!("✅ All random nonces work!");
    } else {
        println!("❌ {} random nonces failed!", failures);
        println!("\n🔍 Investigating the pattern...");
        
        // Generate more failures to find pattern
        println!("\nGenerating random nonces to find failures:");
        let mut sample_failure_nonce = None;
        for _ in 0..1000 {
            let nonce = ScalarField::sample_crypto().to_bytes_le();
            let sig = sign_hashed_message(&private_key_bytes, &message, &nonce)?;
            if !verify_signature(&sig, &message, &public_key_bytes)? {
                sample_failure_nonce = Some(nonce);
                println!("Found failing nonce: {:?}", &nonce[..16]);
                break;
            }
        }
    }
    
    Ok(())
}
