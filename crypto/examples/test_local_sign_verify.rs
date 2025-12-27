use goldilocks_crypto::{ScalarField, schnorr::{sign_with_nonce, verify_signature, Point}};

fn main() {
    println!("Testing sign/verify cycle...");
    
    // Generate keys
    let private_scalar = ScalarField::sample_crypto();
    let private_key_bytes = private_scalar.to_bytes_le();
    
    let generator = Point::generator();
    let public_point = generator.mul(&private_scalar);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    // Create message
    let message = [42u8; 40];
    
    // Sign
    let nonce_scalar = ScalarField::sample_crypto();
    let nonce_bytes = nonce_scalar.to_bytes_le();
    
    match sign_with_nonce(&private_key_bytes, &message, &nonce_bytes) {
        Ok(signature) => {
            println!("✓ Signature created");
            
            // Verify
            match verify_signature(&signature, &message, &public_key_bytes) {
                Ok(true) => println!("✓ Signature verified successfully!"),
                Ok(false) => println!("✗ Signature verification FAILED"),
                Err(e) => println!("✗ Verification error: {}", e),
            }
        }
        Err(e) => println!("✗ Signing error: {}", e),
    }
}
