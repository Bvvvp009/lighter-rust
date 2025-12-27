#[cfg(test)]
mod diagnostic {
    use goldilocks_crypto::{ScalarField, schnorr::{sign_with_nonce, Point}, Goldilocks, Fp5Element};
    use poseidon_hash::hash_to_quintic_extension;

    #[test]
    fn diagnose_failure_pattern() {
        let private_scalar = ScalarField::sample_crypto();
        let private_key_bytes = private_scalar.to_bytes_le();
        
        let nonce_scalar = ScalarField::sample_crypto();
        let nonce_bytes = nonce_scalar.to_bytes_le();
        
        let message = [99u8; 40];
        
        let generator = Point::generator();
        let public_point = generator.mul(&private_scalar);
        
        // Sign
        let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes)
            .expect("Failed to sign");
        
        // Extract s and e from signature
        let s_bytes = &signature[0..40];
        let e_bytes = &signature[40..80];
        
        let s = ScalarField::from_bytes_le(s_bytes).unwrap();
        let e = ScalarField::from_bytes_le(e_bytes).unwrap();
        
        // Manually compute R from signature = s * G + e * P
        let s_g = generator.mul(&s);
        let e_p = public_point.mul(&e);
        let r_from_sig = s_g.add(&e_p);
        let r_enc_from_sig = r_from_sig.encode();
        
        // Compute R from signing = k * G
        let k = ScalarField::from_bytes_le(&nonce_bytes).unwrap();
        let r_from_nonce = generator.mul(&k);
        let r_enc_from_nonce = r_from_nonce.encode();
        
        println!("R from nonce (k*G):        {:?}", r_enc_from_nonce.to_bytes_le());
        println!("R from signature (s*G+e*P): {:?}", r_enc_from_sig.to_bytes_le());
        println!("Match: {}", r_enc_from_nonce.to_bytes_le() == r_enc_from_sig.to_bytes_le());
        
        // Now hash and check e'
        let msg_fp5 = Fp5Element::from_bytes_le(&message).unwrap();
        let mut pre_image = [Goldilocks::zero(); 10];
        pre_image[..5].copy_from_slice(&r_enc_from_nonce.0);
        pre_image[5..].copy_from_slice(&msg_fp5.0);
        
        let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
        let e_prime = ScalarField::from_fp5_element(&e_prime_fp5);
        
        println!("\ne (from signature): {:?}", e.to_bytes_le());
        println!("e' (computed):      {:?}", e_prime.to_bytes_le());
        println!("Match: {}", e.to_bytes_le() == e_prime.to_bytes_le());
        
        // Check if R values match
        if r_enc_from_nonce.to_bytes_le() != r_enc_from_sig.to_bytes_le() {
            println!("\n❌ CRITICAL: R values don't match!");
            println!("   s*G + e*P != k*G");
            println!("   This means the Schnorr equation is broken");
        } else {
            println!("\n✓ R values match!");
            if e.to_bytes_le() != e_prime.to_bytes_le() {
                println!("❌ But e values don't match!");
                println!("   This means the hash is computing differently");
            } else {
                println!("✓ e values also match!");
            }
        }
    }
}
