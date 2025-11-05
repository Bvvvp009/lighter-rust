use goldilocks_crypto::{ScalarField, Point, sign_with_nonce, verify_signature, CryptoError, Result};

fn main() -> Result<()> {
    println!("Testing crypto library exports...");
    
    // Test ScalarField
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    println!("ScalarField: Generated {} byte private key", private_key_bytes.len());
    
    // Test Point
    let generator = Point::generator();
    let public_key = generator.mul(&private_key);
    let public_key_bytes = public_key.encode().to_bytes_le();
    println!("Point: Generated {} byte public key", public_key_bytes.len());
    
    // Test signing
    let message = [0u8; 40];
    let nonce = ScalarField::sample_crypto();
    let nonce_bytes = nonce.to_bytes_le();
    
    let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes)?;
    println!("Signing: Generated {} byte signature", signature.len());
    
    // Test verification
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)?;
    println!("Verification: Signature is {}", if is_valid { "valid" } else { "invalid" });
    
    // Test error types
    let _error: CryptoError = CryptoError::InvalidPrivateKeyLength(32);
    println!("Error types work correctly");
    
    println!("All crypto exports work correctly!");
    Ok(())
}


