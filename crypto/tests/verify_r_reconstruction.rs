//! Test to verify that R reconstructed during verification matches R from signing

use goldilocks_crypto::{ScalarField, Point, Fp5Element};
use goldilocks_crypto::schnorr::sign_with_nonce;
use hex;

#[test]
fn test_r_reconstruction() {
    println!("\n=== R Reconstruction Test ===\n");
    
    let mut private_key_bytes = [0u8; 40];
    private_key_bytes[0] = 1;
    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes).unwrap();
    
    let public_key_point = Point::generator().mul(&private_scalar);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    let message = [0u8; 40];
    
    // Use fixed nonce for reproducibility
    let mut nonce_bytes = [0u8; 40];
    nonce_bytes[0] = 1;
    let nonce_scalar = ScalarField::from_bytes_le(&nonce_bytes).unwrap();
    
    // Sign
    let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
    
    // Extract s and e
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];
    let s = ScalarField::from_bytes_le(s_bytes).unwrap();
    let e = ScalarField::from_bytes_le(e_bytes).unwrap();
    
    println!("Signature components:");
    println!("  s: {}", hex::encode(s_bytes));
    println!("  e: {}", hex::encode(e_bytes));
    println!("  s limbs: {:?}", s.0);
    println!("  e limbs: {:?}", e.0);
    println!("  k (nonce) limbs: {:?}", nonce_scalar.0);
    println!("  private_key limbs: {:?}", private_scalar.0);
    
    // Verify s = k - e*sk using the same scalars
    let e_times_private_test = e.mul(&private_scalar);
    let s_reconstructed_test = nonce_scalar.sub(e_times_private_test);
    println!("  e*sk limbs: {:?}", e_times_private_test.0);
    println!("  s reconstructed limbs: {:?}", s_reconstructed_test.0);
    println!("  s matches reconstructed: {}", s.0 == s_reconstructed_test.0);
    
    // Compute R during signing (for comparison)
    let generator = Point::generator();
    let r_signing = generator.mul(&nonce_scalar);
    let r_signing_encoded = r_signing.encode();
    
    println!("\nR from signing (k*G):");
    println!("  Encoded: {}", hex::encode(&r_signing_encoded.to_bytes_le()));
    
    // Reconstruct R during verification using mul_add2 (same as verify_signature does)
    let public_point = Point::decode(&Fp5Element::from_bytes_le(&public_key_bytes).unwrap()).unwrap();
    let r_verification = Point::mul_add2(&generator, &public_point, &s, &e);
    let r_verification_encoded = r_verification.encode();
    
    println!("\nR from verification (s*G + e*P):");
    println!("  Encoded: {}", hex::encode(&r_verification_encoded.to_bytes_le()));
    
    // Compare
    let r_match = r_signing_encoded.0.iter().zip(r_verification_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    if r_match {
        println!("\n✅ R encodings match!");
    } else {
        println!("\n❌ R encodings do NOT match!");
        println!("  Signing R limbs: {:?}", r_signing_encoded.0);
        println!("  Verification R limbs: {:?}", r_verification_encoded.0);
    }
    
    // Also verify the mathematical relationship: s = k - e*sk
    let e_times_private = e.mul(&private_scalar);
    let s_reconstructed = nonce_scalar.sub(e_times_private);
    
    println!("\nVerifying s = k - e*sk:");
    println!("  s from signature: {}", hex::encode(&s.to_bytes_le()));
    println!("  s reconstructed: {}", hex::encode(&s_reconstructed.to_bytes_le()));
    
    let s_match = s.0 == s_reconstructed.0;
    if s_match {
        println!("  ✅ s matches!");
    } else {
        println!("  ❌ s does NOT match!");
        println!("    s limbs: {:?}", s.0);
        println!("    s_reconstructed limbs: {:?}", s_reconstructed.0);
    }
    
    // If R matches, the signature should verify
    if r_match {
        use goldilocks_crypto::verify_signature;
        let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
        println!("\nVerification result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
        
        if !is_valid && r_match {
            println!("⚠️  WARNING: R matches but verification fails!");
            println!("   This suggests an issue with e computation or comparison.");
        }
    }
}

