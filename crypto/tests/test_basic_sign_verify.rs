#[cfg(test)]
mod tests {
    use goldilocks_crypto::{ScalarField, schnorr::{sign_with_nonce, verify_signature, Point}};

    #[test]
    fn test_sign_verify_roundtrip() {
        // Create a private key
        let private_scalar = ScalarField::sample_crypto();
        let private_key_bytes = private_scalar.to_bytes_le();
        
        // Create a nonce
        let nonce_scalar = ScalarField::sample_crypto();
        let nonce_bytes = nonce_scalar.to_bytes_le();
        
        // Create a message
        let message = [42u8; 40];
        
        // Derive public key
        let generator = Point::generator();
        let public_point = generator.mul(&private_scalar);
        let public_key_bytes = public_point.encode().to_bytes_le();
        
        // Sign
        let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes)
            .expect("Failed to sign");
        
        // Verify
        let is_valid = verify_signature(&signature, &message, &public_key_bytes)
            .expect("Failed to verify");
        
        assert!(is_valid, "Signature verification failed");
        println!("✓ Signature verification passed");
    }
    
    #[test]
    fn test_multiple_signatures() {
        let private_scalar = ScalarField::sample_crypto();
        let private_key_bytes = private_scalar.to_bytes_le();
        
        let generator = Point::generator();
        let public_point = generator.mul(&private_scalar);
        let public_key_bytes = public_point.encode().to_bytes_le();
        
        let message = [99u8; 40];
        
        let mut pass_count = 0;
        let mut fail_count = 0;
        
        for i in 0..20 {
            let nonce_scalar = ScalarField::sample_crypto();
            let nonce_bytes = nonce_scalar.to_bytes_le();
            
            let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes)
                .expect("Failed to sign");
            
            match verify_signature(&signature, &message, &public_key_bytes) {
                Ok(true) => {
                    pass_count += 1;
                    println!("  Signature {}: ✓ PASS", i);
                }
                Ok(false) => {
                    fail_count += 1;
                    println!("  Signature {}: ✗ FAIL", i);
                }
                Err(e) => {
                    fail_count += 1;
                    println!("  Signature {}: ✗ ERROR: {}", i, e);
                }
            }
        }
        
        println!("\nResults: {}/{} signatures verified successfully", pass_count, pass_count + fail_count);
        
        if fail_count > 0 {
            panic!("{} signatures failed verification", fail_count);
        }
    }
}
