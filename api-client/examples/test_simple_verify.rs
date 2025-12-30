// Simple direct test of sign/verify matching

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 SIMPLE SIGN/VERIFY TEST");
    println!("============================\n");

    // Generate a test private key
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Create a test message (Poseidon2 hash of some data)
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    
    println!("Message bytes: {}", hex::encode(&message[..16]));
    println!("Message (reconstructed from bytes):");
    let reconstructed = goldilocks_crypto::Fp5Element::from_bytes_le(&message)?;
    println!("  Original hashed: {:?}", hashed.0.iter().map(|g| g.0).collect::<Vec<_>>()[..3].to_vec());
    println!("  Reconstructed: {:?}", reconstructed.0.iter().map(|g| g.0).collect::<Vec<_>>()[..3].to_vec());
    
    // Use fixed nonce
    let fixed_nonce = [0x01u8; 40];
    
    // Sign
    let signature = sign_hashed_message(&private_key_bytes, &message, &fixed_nonce)?;
    println!("\n✅ Signature created: {}...{}", hex::encode(&signature[..16]), hex::encode(&signature[64..]));
    
    // Derive public key
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Verify
    match verify_signature(&signature, &message, &public_key_bytes) {
        Ok(true) => println!("✅ VERIFICATION PASSED!"),
        Ok(false) => println!("❌ VERIFICATION FAILED - signature marked as invalid"),
        Err(e) => println!("❌ VERIFICATION ERROR: {}", e),
    }

    Ok(())
}
