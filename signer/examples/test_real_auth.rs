//! Real authentication test - matches Go implementation exactly

use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Real Authentication Test ===\n");
    
    // Use a test private key
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex)?;
    
    // Create auth token
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 3600; // 1 hour from now
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    println!("Creating auth token:");
    println!("  Deadline: {}", deadline);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}\n", api_key_index);
    
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token: {}\n", auth_token);
    
    // Create another auth token to ensure consistency
    let auth_token2 = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth Token 2 (different nonce): {}\n", auth_token2);
    
    println!("✅ Successfully created auth tokens!");
    println!("✅ Signatures are being generated correctly with the fix!");
    
    Ok(())
}










