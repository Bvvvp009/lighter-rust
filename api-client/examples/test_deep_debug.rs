// Deep debug of signature verification

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message, verify_signature};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension, Fp5Element};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 DEEP DEBUG OF SIGNATURE VERIFICATION\n");

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
    
    println!("Looking for a failing nonce...");
    
    for attempt in 0..1000 {
        let nonce_scalar = ScalarField::sample_crypto();
        let nonce = nonce_scalar.to_bytes_le();
        
        let sig = sign_hashed_message(&private_key_bytes, &message, &nonce)?;
        let valid = verify_signature(&sig, &message, &public_key_bytes)?;
        
        if !valid {
            println!("\n❌ Found failing signature at attempt {}!", attempt + 1);
            println!("\n=== SIGNATURE DEBUG ===");
            println!("Nonce bytes (first 16): {:?}", &nonce[..16]);
            let nonce_reconstructed = ScalarField::from_bytes_le(&nonce)?;
            let nonce_recon_bytes = nonce_reconstructed.to_bytes_le();
            println!("Nonce scalar reconstructed: {:?}", &nonce_recon_bytes[..16]);
            
            // Extract s and e from signature
            let s_bytes = &sig[0..40];
            let e_bytes = &sig[40..80];
            
            println!("\nSignature components:");
            println!("  s (first 16 bytes): {:?}", &s_bytes[..16]);
            println!("  e (first 16 bytes): {:?}", &e_bytes[..16]);
            
            // Reconstruct s and e as scalars
            let s = ScalarField::from_bytes_le(s_bytes)?;
            let e = ScalarField::from_bytes_le(e_bytes)?;
            
            println!("\nReconstructed scalars:");
            println!("  s reconstructed (first 16 bytes): {:?}", &s.to_bytes_le()[..16]);
            println!("  e reconstructed (first 16 bytes): {:?}", &e.to_bytes_le()[..16]);
            
            // Check if round-trip matches
            println!("\nRound-trip verification:");
            println!("  s round-trip matches: {}", s.to_bytes_le() == s_bytes);
            println!("  e round-trip matches: {}", e.to_bytes_le() == e_bytes);
            
            // Now manually verify the signature
            println!("\n=== MANUAL VERIFICATION ===");
            
            // Decode public key
            let public_key_fp5 = Fp5Element::from_bytes_le(&public_key_bytes)?;
            let public_decoded = Point::decode(&public_key_fp5).ok_or("Failed to decode public key")?;
            
            // Compute R = s*G + e*P
            let s_g = generator.mul(&s);
            let e_p = public_decoded.mul(&e);
            let r_computed = s_g.add(&e_p);
            let r_encoded = r_computed.encode();
            
            println!("R computed (first 16 bytes): {:?}", &r_encoded.to_bytes_le()[..16]);
            
            // Compute e' = H(R || m)
            let message_fp5 = Fp5Element::from_bytes_le(&message)?;
            let mut pre_image = [Goldilocks::zero(); 10];
            pre_image[..5].copy_from_slice(&r_encoded.0);
            pre_image[5..].copy_from_slice(&message_fp5.0);
            
            let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
            let e_prime = ScalarField::from_fp5_element(&e_prime_fp5);
            
            println!("e' computed (first 16 bytes): {:?}", &e_prime.to_bytes_le()[..16]);
            println!("e from sig  (first 16 bytes): {:?}", &e.to_bytes_le()[..16]);
            
            println!("\nComparison:");
            println!("  e == e': {}", e.equals(&e_prime));
            println!("  e bytes == e' bytes: {}", e.to_bytes_le() == e_prime.to_bytes_le());
            
            // Check each limb
            println!("\nLimb-by-limb comparison:");
            for i in 0..5 {
                println!("  Limb {}: e={:016x}, e'={:016x}, match={}", 
                    i, e.0[i], e_prime.0[i], e.0[i] == e_prime.0[i]);
            }
            
            break;
        }
    }
    
    Ok(())
}
