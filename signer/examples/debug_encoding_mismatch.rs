//! Debug Encoding Mismatch - Point vs WeierstrassPoint
//!
//! This tool investigates if there's an encoding mismatch between Point::encode()
//! used during signing and WeierstrassPoint::encode() used during verification.
//!
//! Usage:
//!   cargo run --example debug_encoding_mismatch --release

use signer::KeyManager;
use goldilocks_crypto::{ScalarField, Point, verify_signature};
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug Encoding Mismatch");
    println!("{}", "=".repeat(80));
    println!("Investigating Point::encode() vs WeierstrassPoint::encode() mismatch\n");
    
    let test_private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(test_private_key_hex)?;
    let public_key = key_manager.public_key_bytes();
    
    let message = [0u8; 40];
    
    println!("Testing signature generation and verification...\n");
    
    // Generate a signature
    let signature = key_manager.sign(&message)?;
    println!("Signature generated: {}...", hex::encode(&signature[..20]));
    
    // Try verification
    let is_valid = verify_signature(&signature, &message, &public_key)?;
    println!("Verification result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    
    if !is_valid {
        println!("\n{}", "=".repeat(80));
        println!("INVESTIGATING ENCODING MISMATCH");
        println!("{}", "=".repeat(80));
        
        // Parse signature
        let s_bytes = &signature[0..40];
        let e_bytes = &signature[40..80];
        
        let s = ScalarField::from_bytes_le(s_bytes)
            .map_err(|_| "Failed to parse s")?;
        let e = ScalarField::from_bytes_le(e_bytes)
            .map_err(|_| "Failed to parse e")?;
        
        println!("Signature components:");
        println!("  s: {}...", hex::encode(&s_bytes[..20]));
        println!("  e: {}...", hex::encode(&e_bytes[..20]));
        
        // Get private key
        let private_key_bytes = key_manager.private_key_bytes();
        let private_scalar = ScalarField::from_bytes_le(&private_key_bytes)
            .map_err(|_| "Failed to parse private key")?;
        
        // Reconstruct what happened during signing
        println!("\nReconstructing signing process...");
        
        // During signing: R = nonce * G (we don't have nonce, but we can compute R from signature)
        // R should satisfy: R = s*G + e*P where P is public key
        
        let generator = Point::generator();
        let public_point = generator.mul(&private_scalar);
        
        // Compute R using signature: R = s*G + e*P
        use goldilocks_crypto::WeierstrassPoint;
        let generator_ws = WeierstrassPoint::GENERATOR;
        let public_key_fp5 = poseidon_hash::Fp5Element::from_bytes_le(&public_key)
            .map_err(|_| "Failed to parse public key")?;
        let public_point_ws = WeierstrassPoint::decode_fp5_as_weierstrass(&public_key_fp5)
            .ok_or("Failed to decode public key")?;
        
        let r_point_ws = WeierstrassPoint::mul_add2(&generator_ws, &public_point_ws, &s, &e);
        let r_encoded_ws = r_point_ws.encode();
        
        // Also compute using Point encoding
        let r_point = generator.mul(&s).add(&public_point.mul(&e));
        let r_encoded_point = r_point.encode();
        
        println!("R computed using WeierstrassPoint: {}...", hex::encode(&r_encoded_ws.to_bytes_le()[..20]));
        println!("R computed using Point: {}...", hex::encode(&r_encoded_point.to_bytes_le()[..20]));
        
        // Check if they match
        let ws_bytes = r_encoded_ws.to_bytes_le();
        let point_bytes = r_encoded_point.to_bytes_le();
        
        if ws_bytes == point_bytes {
            println!("✅ R encodings match!");
        } else {
            println!("❌ R encodings DO NOT match!");
            println!("  Difference at bytes:");
            for i in 0..40 {
                if ws_bytes[i] != point_bytes[i] {
                    println!("    Byte {}: WS={:02x}, Point={:02x}", i, ws_bytes[i], point_bytes[i]);
                }
            }
        }
        
        // During signing, we used Point::encode() for R
        // During verification, we use WeierstrassPoint::encode() for R
        // If these don't match, verification will fail!
        
        println!("\n⚠️  POTENTIAL ISSUE:");
        println!("  Signing uses: Point::encode()");
        println!("  Verification uses: WeierstrassPoint::encode()");
        println!("  If encodings differ, verification will fail!");
    }
    
    Ok(())
}













