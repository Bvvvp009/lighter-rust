//! Comprehensive Auth Token Testing
//! Tests auth token generation and verification with multiple scenarios

use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Comprehensive Auth Token Test");
    println!("{}", "=".repeat(80));
    
    // Use test private key
    let test_private_key_hex = "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c";
    let key_manager = KeyManager::from_hex(test_private_key_hex)
        .map_err(|e| format!("Failed to create KeyManager: {}", e))?;
    
    println!("\n📋 Test Configuration:");
    println!("  Private Key: {}...", &test_private_key_hex[..20]);
    let public_key = key_manager.public_key_bytes();
    println!("  Public Key: {}\n", hex::encode(&public_key));
    
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    
    // Test 1: Generate and verify single auth token
    println!("{}", "=".repeat(80));
    println!("Test 1: Single Auth Token Generation and Verification");
    println!("{}", "=".repeat(80));
    total_tests += 1;
    
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 3600; // 1 hour from now
    let account_index = 361816i64;
    let api_key_index = 5u8;
    
    match key_manager.create_auth_token(deadline, account_index, api_key_index) {
        Ok(token) => {
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() == 4 {
                let sig_hex = parts[3];
                match key_manager.verify_auth_token(deadline, account_index, api_key_index, sig_hex) {
                    Ok(true) => {
                        println!("✅ PASSED: Auth token generated and verified successfully");
                        passed_tests += 1;
                    }
                    Ok(false) => {
                        println!("❌ FAILED: Auth token verification returned false");
                        failed_tests += 1;
                    }
                    Err(e) => {
                        println!("❌ FAILED: Auth token verification error: {}", e);
                        failed_tests += 1;
                    }
                }
            } else {
                println!("❌ FAILED: Invalid auth token format");
                failed_tests += 1;
            }
        }
        Err(e) => {
            println!("❌ FAILED: Failed to create auth token: {}", e);
            failed_tests += 1;
        }
    }
    
    // Test 2: Generate multiple tokens with same parameters
    println!("\n{}", "=".repeat(80));
    println!("Test 2: Multiple Tokens (Same Parameters)");
    println!("{}", "=".repeat(80));
    
    let mut tokens_generated = 0;
    let mut tokens_verified = 0;
    
    for i in 1..=10 {
        total_tests += 1;
        match key_manager.create_auth_token(deadline, account_index, api_key_index) {
            Ok(token) => {
                tokens_generated += 1;
                let parts: Vec<&str> = token.split(':').collect();
                if parts.len() == 4 {
                    let sig_hex = parts[3];
                    match key_manager.verify_auth_token(deadline, account_index, api_key_index, sig_hex) {
                        Ok(true) => {
                            tokens_verified += 1;
                            if i <= 3 {
                                println!("  Token {}: ✅ Generated and verified", i);
                            }
                        }
                        Ok(false) | Err(_) => {
                            if i <= 3 {
                                println!("  Token {}: ❌ Verification failed", i);
                            }
                        }
                    }
                }
            }
            Err(_) => {
                if i <= 3 {
                    println!("  Token {}: ❌ Generation failed", i);
                }
            }
        }
    }
    
    println!("  Generated: {}/10 tokens", tokens_generated);
    println!("  Verified: {}/10 tokens", tokens_verified);
    
    if tokens_generated == 10 && tokens_verified == 10 {
        println!("✅ PASSED: All 10 tokens generated and verified");
        passed_tests += 1;
    } else {
        println!("❌ FAILED: Only {}/10 tokens verified successfully", tokens_verified);
        failed_tests += 1;
    }
    
    // Test 3: Different deadlines
    println!("\n{}", "=".repeat(80));
    println!("Test 3: Different Deadlines");
    println!("{}", "=".repeat(80));
    
    let deadlines = vec![
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 300,  // 5 min
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 3600, // 1 hour
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 86400, // 1 day
    ];
    
    let mut deadline_passed = 0;
    for (i, &dl) in deadlines.iter().enumerate() {
        total_tests += 1;
        match key_manager.create_auth_token(dl, account_index, api_key_index) {
            Ok(token) => {
                let parts: Vec<&str> = token.split(':').collect();
                if parts.len() == 4 {
                    let sig_hex = parts[3];
                    if key_manager.verify_auth_token(dl, account_index, api_key_index, sig_hex).unwrap_or(false) {
                        deadline_passed += 1;
                        println!("  Deadline {}: ✅ PASSED", i + 1);
                    } else {
                        println!("  Deadline {}: ❌ FAILED", i + 1);
                    }
                }
            }
            Err(e) => {
                println!("  Deadline {}: ❌ FAILED - {}", i + 1, e);
            }
        }
    }
    
    if deadline_passed == 3 {
        passed_tests += 1;
    } else {
        failed_tests += 1;
    }
    
    // Test 4: Different account indices
    println!("\n{}", "=".repeat(80));
    println!("Test 4: Different Account Indices");
    println!("{}", "=".repeat(80));
    
    let account_indices = vec![0, 1, 100, 361816, 999999];
    let mut account_passed = 0;
    
    for &acc_idx in &account_indices {
        total_tests += 1;
        match key_manager.create_auth_token(deadline, acc_idx, api_key_index) {
            Ok(token) => {
                let parts: Vec<&str> = token.split(':').collect();
                if parts.len() == 4 {
                    let sig_hex = parts[3];
                    if key_manager.verify_auth_token(deadline, acc_idx, api_key_index, sig_hex).unwrap_or(false) {
                        account_passed += 1;
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    println!("  Verified: {}/5 account indices", account_passed);
    if account_passed == 5 {
        passed_tests += 1;
        println!("✅ PASSED: All account indices work");
    } else {
        failed_tests += 1;
        println!("❌ FAILED: Only {}/5 account indices verified", account_passed);
    }
    
    // Summary
    println!("\n{}", "=".repeat(80));
    println!("TEST SUMMARY");
    println!("{}", "=".repeat(80));
    println!("Total Tests: {}", total_tests);
    println!("Passed: {} ({:.1}%)", passed_tests, (passed_tests as f64 / total_tests as f64) * 100.0);
    println!("Failed: {} ({:.1}%)", failed_tests, (failed_tests as f64 / total_tests as f64) * 100.0);
    println!("{}", "=".repeat(80));
    
    if failed_tests == 0 {
        println!("✅ ALL TESTS PASSED!");
        Ok(())
    } else {
        println!("❌ SOME TESTS FAILED");
        Err("Some tests failed".into())
    }
}













