// Debug: Check if signatures are being generated correctly

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 CHECKING SIGNATURE GENERATION\n");

    // Generate keys
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Create message
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    
    println!("Creating 5 signatures with random nonces and inspecting them...\n");
    
    for i in 0..5 {
        let nonce = ScalarField::sample_crypto().to_bytes_le();
        let sig = sign_hashed_message(&private_key_bytes, &message, &nonce)?;
        
        // Parse signature
        let s_bytes = &sig[..40];
        let e_bytes = &sig[40..80];
        
        let s = ScalarField::from_bytes_le(s_bytes)?;
        let e = ScalarField::from_bytes_le(e_bytes)?;
        
        println!("[{}]", i + 1);
        println!("  Nonce first 8 bytes:  {:?}", &nonce[..8]);
        println!("  s first 8 bytes:      {:?}", &s_bytes[..8]);
        println!("  e first 8 bytes:      {:?}", &e_bytes[..8]);
        println!("  s as scalar[0]: {}", s.0[0]);
        println!("  e as scalar[0]: {}", e.0[0]);
        println!();
    }
    
    Ok(())
}
