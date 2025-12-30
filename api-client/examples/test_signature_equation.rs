// Test if s = k - e*sk equation holds

use goldilocks_crypto::{ScalarField, Point};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension, Fp5Element};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 TESTING SIGNATURE EQUATION: s = k - e*sk\n");

    // Generate keys
    let sk = ScalarField::sample_crypto();
    let sk_bytes = sk.to_bytes_le();
    let generator = Point::generator();
    let public_point = generator.mul(&sk);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Create message
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    let message_fp5 = Fp5Element::from_bytes_le(&message)?;
    
    println!("Looking for a failing nonce...\n");
    
    for attempt in 0..1000 {
        let k = ScalarField::sample_crypto();
        let k_bytes = k.to_bytes_le();
        
        // SIGN: Compute R = k*G
        let r_point = generator.mul(&k);
        let r_encoded = r_point.encode();
        
        // Compute e = H(R || m)
        let mut pre_image = [Goldilocks::zero(); 10];
        pre_image[..5].copy_from_slice(&r_encoded.0);
        pre_image[5..].copy_from_slice(&message_fp5.0);
        let e_fp5 = hash_to_quintic_extension(&pre_image);
        let e = ScalarField::from_fp5_element(&e_fp5);
        
        // Compute s = k - e*sk
        let e_times_sk = e.mul(&sk);
        let s = k.sub(e_times_sk);
        
        // VERIFY: Compute R' = s*G + e*P
        let s_g = generator.mul(&s);
        let e_p = public_point.mul(&e);
        let r_reconstructed = s_g.add(&e_p);
        let r_recon_encoded = r_reconstructed.encode();
        
        // Check if R == R'
        let r_match = r_encoded.to_bytes_le() == r_recon_encoded.to_bytes_le();
        
        if !r_match {
            println!("❌ Found mismatch at attempt {}!", attempt + 1);
            println!("\n=== ALGEBRAIC VERIFICATION ===");
            
            // Verify the equation algebraically
            // We should have: s*G + e*P = k*G
            // Which means: s + e*sk = k (mod N)
            // Or: s = k - e*sk (mod N)
            
            let s_plus_e_sk = s.add(e_times_sk);
            let algebraic_match = k.equals(&s_plus_e_sk);
            
            println!("Equation check: s + e*sk == k: {}", algebraic_match);
            
            if !algebraic_match {
                println!("\nThe subtraction s = k - e*sk is NOT working correctly!");
                println!("\nValues:");
                println!("  k (nonce)  : {:?}", &k.to_bytes_le()[..16]);
                println!("  e (challenge): {:?}", &e.to_bytes_le()[..16]);
                println!("  sk (privkey): {:?}", &sk.to_bytes_le()[..16]);
                println!("  e*sk       : {:?}", &e_times_sk.to_bytes_le()[..16]);
                println!("  s (response): {:?}", &s.to_bytes_le()[..16]);
                println!("  s+e*sk     : {:?}", &s_plus_e_sk.to_bytes_le()[..16]);
                
                // Check limb-by-limb
                println!("\nLimb comparison (k vs s+e*sk):");
                for i in 0..5 {
                    println!("  Limb {}: k={:016x}, s+e*sk={:016x}, match={}", 
                        i, k.0[i], s_plus_e_sk.0[i], k.0[i] == s_plus_e_sk.0[i]);
                }
            } else {
                println!("\nThe subtraction is correct, but point operations differ!");
                println!("  R (signed)   : {:?}", &r_encoded.to_bytes_le()[..16]);
                println!("  R' (verified): {:?}", &r_recon_encoded.to_bytes_le()[..16]);
            }
            
            break;
        }
    }
    
    Ok(())
}
