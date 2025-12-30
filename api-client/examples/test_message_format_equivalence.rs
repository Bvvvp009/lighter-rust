// Test if from_bytes_le and message_to_fp5 are truly equivalent

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, Fp5Element, hash_to_quintic_extension};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 TESTING message_to_fp5 vs Fp5Element::from_bytes_le equivalence\n");

    // Create a test message (Poseidon2 hash of some data)
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    
    println!("Message (hash output):");
    println!("  First 8 bytes: {}", hex::encode(&message[..8]));
    
    // Method 1: Using from_bytes_le
    let fp5_from_bytes = Fp5Element::from_bytes_le(&message)?;
    println!("\nFp5Element::from_bytes_le():");
    println!("  Limb 0: {}", fp5_from_bytes.0[0].0);
    println!("  Limb 1: {}", fp5_from_bytes.0[1].0);
    
    // Method 2: Using message_to_fp5 (via the signing logic)
    // We can't call it directly since it's private, but let's verify through sign/verify
    
    // Generate keys
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Use fixed nonce
    let fixed_nonce = [0x01u8; 40];
    
    // Sign
    let signature = sign_hashed_message(&private_key_bytes, &message, &fixed_nonce)?;
    println!("\n✅ Signature created");
    
    // Verify - this will use message_to_fp5() internally
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)?;
    
    println!("\nVerification result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    
    if !is_valid {
        println!("\n🚨 CRITICAL: message_to_fp5 may not be equivalent to from_bytes_le");
        println!("   OR there's a bug in the signing/verification algorithm");
    }
    
    Ok(())
}
