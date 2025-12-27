//! Debug auth token hash generation to compare with Go
//!
//! This helps identify differences in message hashing

use signer::KeyManager;
use goldilocks_crypto::Goldilocks;
use poseidon_hash::hash_to_quintic_extension;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Auth Token Hash Generation Debug ===\n");
    
    // Use same test values
    let deadline = 1766758395i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    // Create message string (matches Go)
    let message = format!("{}:{}:{}", deadline, account_index, api_key_index);
    println!("Message: {}", message);
    println!("Message bytes: {:?}", message.as_bytes());
    println!("Message hex: {}\n", hex::encode(message.as_bytes()));
    
    // Convert to Goldilocks elements (match Go's ArrayFromCanonicalLittleEndianBytes)
    let auth_bytes = message.as_bytes();
    let missing = (8 - auth_bytes.len() % 8) % 8;
    
    println!("Total bytes: {}", auth_bytes.len());
    println!("Missing bytes for padding: {}\n", missing);
    
    let mut elements = Vec::new();
    let mut i = 0;
    let mut chunk_num = 0;
    
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        
        // Pad with zeros if this is the last chunk and it's incomplete
        if chunk.len() < 8 && missing > 0 {
            // Already padded with zeros from initialization
        }
        
        let val = u64::from_le_bytes(bytes);
        let elem = Goldilocks::from_canonical_u64(val);
        
        println!("Chunk {}: {:?}", chunk_num, chunk);
        println!("  Bytes (padded): {:?}", bytes);
        println!("  As u64 (LE): {}", val);
        println!("  Goldilocks element: {:?}\n", elem);
        
        elements.push(elem);
        i = next_start;
        chunk_num += 1;
    }
    
    println!("Total Goldilocks elements: {}\n", elements.len());
    
    // Hash using Poseidon2
    let hash_fp5 = hash_to_quintic_extension(&elements);
    println!("Poseidon2 hash (Fp5):");
    let hash_bytes = hash_fp5.to_bytes_le();
    println!("  Hex: {}", hex::encode(&hash_bytes));
    println!("  Bytes: {:?}\n", &hash_bytes[..]);
    
    // Now test with actual KeyManager
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex)?;
    
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Generated auth token:");
    println!("{}\n", auth_token);
    
    // Parse signature
    let parts: Vec<&str> = auth_token.split(':').collect();
    if parts.len() >= 4 {
        println!("Signature (hex): {}", parts[3]);
    }
    
    Ok(())
}
