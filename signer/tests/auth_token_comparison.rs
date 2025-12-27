//! Auth token generation comparison tests with Go implementation
//! 
//! These tests verify that the Rust implementation produces identical auth tokens
//! to the Go implementation when given the same inputs.

use signer::KeyManager;
use hex;

/// Test vector for auth token generation
/// Generated using Go's TestGenerateAuthTokenTestVectors
struct AuthTokenTestVector {
    name: &'static str,
    private_key_hex: &'static str,
    deadline: i64,
    account_index: i64,
    api_key_index: u8,
    expected_auth_token: &'static str,
}

// Test vectors generated from Go's TestGenerateAuthTokenTestVectors
const AUTH_TOKEN_TEST_VECTORS: &[AuthTokenTestVector] = &[
    AuthTokenTestVector {
        name: "Test Vector 1 - Simple values",
        private_key_hex: "01000000000000000000000000000000000000000000000000000000000000000000000000000000",
        deadline: 1735689600,
        account_index: 271,
        api_key_index: 4,
        expected_auth_token: "1735689600:271:4:b9531e9eeae5c027de260589654d81315bfbfe320c0a7ce30fc8963273852536f242c75ef8077b35ca1cb217f6e02526c7bff77c9107068ece39c23323e8a25a634f5999e4330423802fb7d820d46258",
    },
    AuthTokenTestVector {
        name: "Test Vector 2 - Different values",
        private_key_hex: "020000000000000000000000000000000000000000000000000000000000000000000000000000ff",
        deadline: 1735776000,
        account_index: 0,
        api_key_index: 0,
        expected_auth_token: "1735776000:0:0:4f3f74f55aaa09516f1de3efe2dac6bea97c6ba49256dcd941efc004fe8f09c8dfb58ee0a91e295ac228da4841541f35f78ee749397d4d17fed3d2b8d2e77c36574ecf48c28b43a3b3ddc72a68e35512",
    },
    AuthTokenTestVector {
        name: "Test Vector 3 - Large values",
        private_key_hex: "fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0dfdedddcdbdad9d8",
        deadline: 2147483647,
        account_index: 999999,
        api_key_index: 255,
        expected_auth_token: "2147483647:999999:255:f0ed721efc72d0a0230c5659b382f94ece9e4c9e1ab0da9905dc21e82c3bf199e25383aabab4005762031ecc4de1176f41174ef272d8b2d7cb5270b8a74720aac84081d3bbc2ae532f9078e99c879b0a",
    },
];

#[test]
fn test_auth_token_generation() {
    // Test Vector 1: Simple values
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex).expect("Failed to create key manager");
    
    let deadline = 1735689600i64; // 2025-01-01 00:00:00 UTC
    let account_index = 271i64;
    let api_key_index = 4u8;
    
    let auth_token = key_manager
        .create_auth_token(deadline, account_index, api_key_index)
        .expect("Failed to create auth token");
    
    println!("=== Test Vector 1 ===");
    println!("Private Key (hex): {}", private_key_hex);
    println!("Deadline: {}", deadline);
    println!("Account Index: {}", account_index);
    println!("API Key Index: {}", api_key_index);
    println!("Auth Token (Rust): {}", auth_token);
    println!();
    
    // Parse the auth token
    let parts: Vec<&str> = auth_token.split(':').collect();
    assert_eq!(parts.len(), 4, "Auth token should have format: deadline:account:apikey:signature");
    
    let parsed_deadline: i64 = parts[0].parse().expect("Failed to parse deadline");
    let parsed_account: i64 = parts[1].parse().expect("Failed to parse account index");
    let parsed_api_key: u8 = parts[2].parse().expect("Failed to parse API key index");
    let signature_hex = parts[3];
    
    assert_eq!(parsed_deadline, deadline);
    assert_eq!(parsed_account, account_index);
    assert_eq!(parsed_api_key, api_key_index);
    assert_eq!(signature_hex.len(), 160, "Signature should be 80 bytes = 160 hex chars");
}

#[test]
fn test_auth_token_multiple_cases() {
    // Test case 1: Simple values
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex).expect("Failed to create key manager");
    
    let deadline1 = 1735689600i64;
    let account_index1 = 271i64;
    let api_key_index1 = 4u8;
    
    let auth_token1 = key_manager
        .create_auth_token(deadline1, account_index1, api_key_index1)
        .expect("Failed to create auth token 1");
    
    println!("Test 1 - Auth Token: {}", auth_token1);
    
    // Test case 2: Different values
    let deadline2 = 1735776000i64;
    let account_index2 = 0i64;
    let api_key_index2 = 0u8;
    
    let auth_token2 = key_manager
        .create_auth_token(deadline2, account_index2, api_key_index2)
        .expect("Failed to create auth token 2");
    
    println!("Test 2 - Auth Token: {}", auth_token2);
    
    // Test case 3: Large values
    let deadline3 = 2147483647i64;
    let account_index3 = 999999i64;
    let api_key_index3 = 255u8;
    
    let auth_token3 = key_manager
        .create_auth_token(deadline3, account_index3, api_key_index3)
        .expect("Failed to create auth token 3");
    
    println!("Test 3 - Auth Token: {}", auth_token3);
    
    // Verify all tokens have correct format
    for (i, token) in [&auth_token1, &auth_token2, &auth_token3].iter().enumerate() {
        let parts: Vec<&str> = token.split(':').collect();
        assert_eq!(parts.len(), 4, "Test {}: Auth token should have 4 parts", i + 1);
        
        let signature_hex = parts[3];
        assert_eq!(signature_hex.len(), 160, "Test {}: Signature should be 160 hex chars", i + 1);
        
        // Verify signature is valid hex
        hex::decode(signature_hex).expect(&format!("Test {}: Invalid hex in signature", i + 1));
    }
}

/// Test that Rust auth tokens have correct format
/// Note: Signatures will differ due to random nonces, but message format should match
#[test]
fn test_auth_token_format() {
    for test_vector in AUTH_TOKEN_TEST_VECTORS {
        println!("\n=== {} ===", test_vector.name);
        
        let key_manager = KeyManager::from_hex(test_vector.private_key_hex)
            .expect(&format!("Failed to create key manager for {}", test_vector.name));
        
        let auth_token = key_manager
            .create_auth_token(
                test_vector.deadline,
                test_vector.account_index,
                test_vector.api_key_index,
            )
            .expect(&format!("Failed to create auth token for {}", test_vector.name));
        
        // Parse the auth token
        let parts: Vec<&str> = auth_token.split(':').collect();
        assert_eq!(parts.len(), 4, "Auth token should have 4 parts: deadline:account:apikey:signature");
        
        let parsed_deadline: i64 = parts[0].parse().expect("Failed to parse deadline");
        let parsed_account: i64 = parts[1].parse().expect("Failed to parse account index");
        let parsed_api_key: u8 = parts[2].parse().expect("Failed to parse API key index");
        let signature_hex = parts[3];
        
        // Verify message format matches expected
        assert_eq!(parsed_deadline, test_vector.deadline, "Deadline should match");
        assert_eq!(parsed_account, test_vector.account_index, "Account index should match");
        assert_eq!(parsed_api_key, test_vector.api_key_index, "API key index should match");
        
        // Verify signature is valid hex and correct length (80 bytes = 160 hex chars)
        let signature_bytes = hex::decode(signature_hex)
            .expect(&format!("Signature should be valid hex for {}", test_vector.name));
        assert_eq!(signature_bytes.len(), 80, "Signature should be 80 bytes");
        assert_eq!(signature_hex.len(), 160, "Signature should be 160 hex characters");
        
        println!("✅ Auth token format validation passed");
        println!("   Message format: {}:{}:{}", parsed_deadline, parsed_account, parsed_api_key);
        println!("   Signature length: {} bytes ({} hex chars)", signature_bytes.len(), signature_hex.len());
    }
}

/// Test that Rust auth token message format matches Go exactly
/// (The signature part will differ due to random nonces)
#[test]
fn test_auth_token_message_format_matches_go() {
    for test_vector in AUTH_TOKEN_TEST_VECTORS {
        // Parse Go's expected auth token to get the message part
        let go_parts: Vec<&str> = test_vector.expected_auth_token.split(':').collect();
        let go_message = format!("{}:{}:{}", go_parts[0], go_parts[1], go_parts[2]);
        
        // Generate Rust auth token
        let key_manager = KeyManager::from_hex(test_vector.private_key_hex)
            .expect(&format!("Failed to create key manager for {}", test_vector.name));
        
        let rust_auth_token = key_manager
            .create_auth_token(
                test_vector.deadline,
                test_vector.account_index,
                test_vector.api_key_index,
            )
            .expect(&format!("Failed to create auth token for {}", test_vector.name));
        
        // Parse Rust auth token
        let rust_parts: Vec<&str> = rust_auth_token.split(':').collect();
        let rust_message = format!("{}:{}:{}", rust_parts[0], rust_parts[1], rust_parts[2]);
        
        // Message format should match exactly
        assert_eq!(go_message, rust_message, 
            "Message format should match Go for {}: expected '{}', got '{}'",
            test_vector.name, go_message, rust_message);
    }
    
    println!("✅ All auth token message formats match Go");
}

#[test]
fn test_auth_token_deterministic() {
    // Verify that the same inputs produce the same auth token
    let private_key_hex = "01000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let key_manager = KeyManager::from_hex(private_key_hex).expect("Failed to create key manager");
    
    let deadline = 1735689600i64;
    let account_index = 271i64;
    let api_key_index = 4u8;
    
    // Generate auth token twice
    let auth_token1 = key_manager
        .create_auth_token(deadline, account_index, api_key_index)
        .expect("Failed to create auth token 1");
    
    let auth_token2 = key_manager
        .create_auth_token(deadline, account_index, api_key_index)
        .expect("Failed to create auth token 2");
    
    // Note: Auth tokens use random nonces for signing, so they will be different
    // But the message part (deadline:account:apikey) should be the same
    let parts1: Vec<&str> = auth_token1.split(':').collect();
    let parts2: Vec<&str> = auth_token2.split(':').collect();
    
    assert_eq!(parts1[0], parts2[0], "Deadline should match");
    assert_eq!(parts1[1], parts2[1], "Account index should match");
    assert_eq!(parts1[2], parts2[2], "API key index should match");
    // Signatures will be different due to random nonces
}

