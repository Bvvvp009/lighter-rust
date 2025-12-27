//! Test that our signatures verify correctly after the fix
//!
//! This validates that the scalar canonicalization fix works end-to-end.

use signer::KeyManager;
use goldilocks_crypto::schnorr::verify_signature;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Signature Roundtrip Test (Post-Fix Validation) ===\n");
    
    // Test with a known private key
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex)?;
    
    // Create a test message
    let test_message = "Test Message 123";
    let message_bytes = test_message.as_bytes();
    
    // Pad to 40 bytes
    let mut padded_message = [0u8; 40];
    let copy_len = message_bytes.len().min(40);
    padded_message[..copy_len].copy_from_slice(&message_bytes[..copy_len]);
    
    println!("Test Message: {}", test_message);
    println!("Message (hex): {}\n", hex::encode(&padded_message));
    
    // Sign the message multiple times
    let num_tests = 10;
    let mut success_count = 0;
    
    for i in 0..num_tests {
        let signature = key_manager.sign(&padded_message)?;
        
        // Get public key
        let public_key = key_manager.public_key_bytes();
        
        // Verify the signature (using bytes directly)
        let is_valid = verify_signature(&signature, &padded_message, &public_key)?;
        
        if is_valid {
            println!("  Signature {}: ✓ PASS", i + 1);
            success_count += 1;
        } else {
            println!("  Signature {}: ✗ FAIL", i + 1);
            println!("    Signature (hex): {}", hex::encode(&signature));
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("Results: {}/{} signatures verified successfully", success_count, num_tests);
    
    if success_count == num_tests {
        println!("✅ ALL SIGNATURES PASS! Fix is working correctly!");
    } else {
        println!("❌ Some signatures failed. Success rate: {:.1}%", 
                 (success_count as f64 / num_tests as f64) * 100.0);
        return Err("Not all signatures verified".into());
    }
    
    Ok(())
}
