//! Test that our auth tokens pass local verification

use signer::KeyManager;
use goldilocks_crypto::schnorr::verify_signature;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Auth Token Verification Test ===\n");
    
    // Use test private key
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex)?;
    
    let deadline = 1766758395i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    println!("Creating auth token...");
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token: {}\n", auth_token);
    
    // Parse the auth token
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() != 4 {
        return Err("Invalid auth token format".into());
    }
    
    let signature_hex = parts[3];
    let signature_bytes = hex::decode(signature_hex)?;
    
    if signature_bytes.len() != 80 {
        return Err(format!("Invalid signature length: {}", signature_bytes.len()).into());
    }
    
    // Recreate the message hash that was signed
    let message = format!("{}:{}:{}", deadline, account_index, api_key_index);
    let auth_bytes = message.as_bytes();
    
    use goldilocks_crypto::Goldilocks;
    use poseidon_hash::hash_to_quintic_extension;
    
    let missing = (8 - auth_bytes.len() % 8) % 8;
    let mut elements = Vec::new();
    
    let mut i = 0;
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        
        let val = u64::from_le_bytes(bytes);
        elements.push(Goldilocks::from_canonical_u64(val));
        
        i = next_start;
    }
    
    let hash_fp5 = hash_to_quintic_extension(&elements);
    let message_bytes = hash_fp5.to_bytes_le();
    
    // Get public key
    let public_key = key_manager.public_key_bytes();
    
    println!("Verifying signature locally...");
    println!("  Message hash: {}", hex::encode(&message_bytes));
    println!("  Public key: {}", hex::encode(&public_key));
    println!("  Signature: {}\n", signature_hex);
    
    // Verify the signature
    let is_valid = verify_signature(&signature_bytes, &message_bytes, &public_key)?;
    
    if is_valid {
        println!("✅ PASS: Signature verifies correctly!");
        println!("✅ Our implementation is internally consistent");
    } else {
        println!("❌ FAIL: Signature does NOT verify!");
        println!("❌ This indicates a problem with our signing or verification");
        return Err("Signature verification failed".into());
    }
    
    Ok(())
}
