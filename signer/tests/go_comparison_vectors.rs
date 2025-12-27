//! Test vectors for comparing Rust vs Go signatures
//! 
//! These tests use fixed inputs to enable deterministic comparison with Go output.
//! Run the Go equivalent and compare signatures byte-by-byte.

use signer::KeyManager;
use hex;

/// Test case with fixed inputs for deterministic comparison
struct ComparisonTestCase {
    name: &'static str,
    private_key_hex: &'static str,
    message_hex: &'static str,
    deadline: i64,
    account_index: i64,
    api_key_index: u8,
}

const TEST_CASES: &[ComparisonTestCase] = &[
    ComparisonTestCase {
        name: "Standard case",
        private_key_hex: "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
        message_hex: "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        deadline: 1735689600,
        account_index: 271,
        api_key_index: 4,
    },
    ComparisonTestCase {
        name: "All zeros message",
        private_key_hex: "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
        message_hex: "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        deadline: 1735689600,
        account_index: 271,
        api_key_index: 4,
    },
    ComparisonTestCase {
        name: "All ones message",
        private_key_hex: "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
        message_hex: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        deadline: 1735689600,
        account_index: 271,
        api_key_index: 4,
    },
    ComparisonTestCase {
        name: "Edge case - minimum deadline",
        private_key_hex: "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
        message_hex: "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        deadline: 0,
        account_index: 0,
        api_key_index: 0,
    },
    ComparisonTestCase {
        name: "Edge case - maximum account index",
        private_key_hex: "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
        message_hex: "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        deadline: 1735689600,
        account_index: i64::MAX,
        api_key_index: u8::MAX,
    },
];

#[test]
fn test_signature_comparison_vectors() {
    for test_case in TEST_CASES {
        println!("\n=== Test Case: {} ===", test_case.name);
        
        let key_manager = KeyManager::from_hex(test_case.private_key_hex)
            .expect("Failed to create KeyManager");
        
        let message_bytes = hex::decode(test_case.message_hex)
            .expect("Failed to decode message hex");
        
        // Pad or truncate to 40 bytes
        let mut message = [0u8; 40];
        let copy_len = message_bytes.len().min(40);
        message[..copy_len].copy_from_slice(&message_bytes[..copy_len]);
        
        // Test signature generation
        let signature = key_manager.sign(&message)
            .expect("Failed to sign message");
        
        assert_eq!(signature.len(), 80, "Signature must be 80 bytes");
        
        println!("Private Key: {}", test_case.private_key_hex);
        println!("Message: {}", test_case.message_hex);
        println!("Signature: {}", hex::encode(&signature));
        
        // Test auth token generation
        let auth_token = key_manager.create_auth_token(
            test_case.deadline,
            test_case.account_index,
            test_case.api_key_index,
        ).expect("Failed to create auth token");
        
        println!("Auth Token: {}", auth_token);
        
        // Verify auth token format
        let parts: Vec<&str> = auth_token.split(':').collect();
        assert_eq!(parts.len(), 4, "Auth token must have 4 parts");
        assert_eq!(parts[0], test_case.deadline.to_string());
        assert_eq!(parts[1], test_case.account_index.to_string());
        assert_eq!(parts[2], test_case.api_key_index.to_string());
        
        let signature_hex = parts[3];
        assert_eq!(signature_hex.len(), 160, "Signature must be 160 hex chars");
        
        println!("✅ Test case passed");
    }
}

#[test]
fn test_signature_verification() {
    // Test that signatures can be verified (using crypto crate)
    // Note: Since sign() generates random nonces, we test verification separately
    use goldilocks_crypto::{sign, verify_signature, Point};
    
    let private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let private_key_bytes = hex::decode(private_key_hex).unwrap();
    let mut private_key_bytes_array = [0u8; 40];
    let copy_len = private_key_bytes.len().min(40);
    private_key_bytes_array[..copy_len].copy_from_slice(&private_key_bytes[..copy_len]);
    
    // Generate public key using crypto crate directly (for verification)
    use goldilocks_crypto::ScalarField;
    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes_array)
        .expect("Failed to parse private key");
    let public_key_point = Point::generator().mul(&private_scalar);
    let public_key_bytes = public_key_point.encode().to_bytes_le();
    
    // Sign a message multiple times and verify each
    let message = [0u8; 40];
    let mut all_valid = true;
    
    for i in 0..5 {
        let signature = sign(&private_key_bytes_array, &message)
            .expect("Failed to sign");
        
        // Verify signature
        let is_valid = verify_signature(&signature, &message, &public_key_bytes)
            .expect("Failed to verify signature");
        
        if !is_valid {
            println!("❌ Signature {} failed verification", i + 1);
            all_valid = false;
        }
    }
    
    assert!(all_valid, "All signatures should be valid");
    println!("✅ Signature verification passed (5 signatures verified)");
}

#[test]
fn test_key_consistency() {
    // Test that KeyManager produces consistent results
    let private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    
    // Create KeyManager multiple times with same key
    let km1 = KeyManager::from_hex(private_key_hex).unwrap();
    let km2 = KeyManager::from_hex(private_key_hex).unwrap();
    let km3 = KeyManager::from_hex(private_key_hex).unwrap();
    
    // Public keys should be identical
    assert_eq!(km1.public_key_bytes(), km2.public_key_bytes());
    assert_eq!(km2.public_key_bytes(), km3.public_key_bytes());
    
    // Private keys should be identical
    assert_eq!(km1.private_key_bytes(), km2.private_key_bytes());
    assert_eq!(km2.private_key_bytes(), km3.private_key_bytes());
    
    println!("✅ Key consistency test passed");
}

#[test]
fn test_message_variations() {
    // Test signing various message patterns
    let private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(private_key_hex).unwrap();
    
    let test_messages = vec![
        ([0u8; 40], "All zeros"),
        ([0xFFu8; 40], "All ones"),
        ([0xAAu8; 40], "Alternating pattern"),
        ([0x55u8; 40], "Alternating pattern 2"),
    ];
    
    for (message, description) in test_messages {
        let signature = key_manager.sign(&message)
            .expect(&format!("Failed to sign: {}", description));
        
        assert_eq!(signature.len(), 80);
        assert!(!signature.iter().all(|&b| b == 0));
        
        println!("✅ Signed message: {} - Signature: {}...", 
                 description, 
                 hex::encode(&signature[0..16]));
    }
}

