// Test to verify that signing and verification work correctly

use goldilocks_crypto::*;

#[test]
fn test_sign_verify_consistency() {
    // Generate a key pair
    let private_key = ScalarField::sample_crypto();
    let public_key_point = Point::generator().mul(&private_key);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    // Sign a message
    let message = [0u8; 40];
    let private_key_bytes = private_key.to_bytes_le();
    let signature = sign(&private_key_bytes, &message).unwrap();
    
    // Verify the signature
    let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
    
    println!("Signature valid: {}", is_valid);
    
    if is_valid {
        println!("✅ Sign and verify work consistently!");
    } else {
        println!("❌ Signature verification failed!");
        panic!("R computed during verification doesn't match R computed during signing");
    }
}

