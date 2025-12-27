use goldilocks_crypto::{ScalarField, Fp5Element, Point};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use hex;

fn main() {
    let privkey_hex = "c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51";
    let privkey_bytes = hex::decode(privkey_hex).unwrap();
    
    // Go's hash
    let hash_hex = "b63f2674b8fd44b604c142d0c8c1bdf8e3a2d3dab4ad8e6b15e64803becef90d59e7f02c9807fb4c";
    let hash_bytes = hex::decode(hash_hex).unwrap();
    
    // Nonce as scalar 2
    let nonce_scalar = ScalarField::new([2, 0, 0, 0, 0]);
    let nonce_bytes = nonce_scalar.to_bytes_le();
    
    // Load privkey and nonce
    let private_scalar = ScalarField::from_bytes_le(&privkey_bytes).unwrap();
    let nonce_loaded = ScalarField::from_bytes_le(&nonce_bytes).unwrap();
    
    // Load message hash
    let message_fp5 = Fp5Element::from_bytes_le(&hash_bytes).unwrap();
    
    println!("=== Debug Components ===");
    println!("Private key scalar limbs: {:?}", private_scalar.limbs());
    println!("Nonce scalar limbs: {:?}", nonce_loaded.limbs());
    println!("Message Fp5 limbs: {:?}", message_fp5.0);
    
    // Compute R = nonce * G
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_loaded);
    let r_encoded = r_point.encode();
    let r_bytes = r_encoded.to_bytes_le();
    println!("\nR = nonce * G:");
    println!("  Limbs: {:?}", r_encoded.0);
    println!("  Bytes: {}", hex::encode(&r_bytes));
    
    // Compute e = H(R || m)
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    println!("\nPre-image for hash H(R || m):");
    println!("  R part (first 5 Goldilocks elements):");
    for i in 0..5 {
        println!("    [{}]: {:?}", i, pre_image[i].to_canonical_u64());
    }
    println!("  m part (next 5 Goldilocks elements):");
    for i in 5..10 {
        println!("    [{}]: {:?}", i, pre_image[i].to_canonical_u64());
    }
    
    let e_fp5 = hash_to_quintic_extension(&pre_image);
    let e_scalar = ScalarField::from_fp5_element(&e_fp5);
    println!("\nChallenge e = H(R || m):");
    println!("  Fp5 limbs: {:?}", e_fp5.0);
    println!("  Scalar limbs: {:?}", e_scalar.limbs());
    println!("  Scalar bytes: {}", hex::encode(&e_scalar.to_bytes_le()));
    
    // Compute s = nonce - e * privkey
    let e_times_private = e_scalar.mul(&private_scalar);
    println!("\ne * privkey:");
    println!("  Limbs: {:?}", e_times_private.limbs());
    println!("  Bytes: {}", hex::encode(&e_times_private.to_bytes_le()));
    
    let s = nonce_loaded.sub(e_times_private);
    println!("\ns = nonce - e * privkey:");
    println!("  Limbs: {:?}", s.limbs());
    println!("  Bytes: {}", hex::encode(&s.to_bytes_le()));
    
    // Assemble signature
    let mut signature = [0u8; 80];
    signature[..40].copy_from_slice(&s.to_bytes_le());
    signature[40..].copy_from_slice(&e_scalar.to_bytes_le());
    
    let sig_hex = hex::encode(&signature);
    println!("\nFinal Signature (s || e):");
    println!("  Rust: {}", sig_hex);
    println!("  Go:   eeb76862d2f64fa1f2b5e4e1bbee455ea45b7cb2dddf603456d829044ff0c339c101f10e8fa7a74bff6bdc4971eeb11a9dcc05b854867553ccb4309ca188141fde370e00b52849515fc31c4af8f2f714");
}
