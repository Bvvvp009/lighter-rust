// Test 2: Signature Determinism Test
// Verify that the same input always produces the same signature
// when using a fixed nonce

use goldilocks_crypto::{ScalarField, Point, sign_hashed_message};
use poseidon_hash::Goldilocks;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Test 2: Signature Determinism");
    println!("=================================");
    println!("Testing if fixed inputs produce identical signatures\n");

    // Generate a test private key
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    println!("Test Configuration:");
    println!("  Private key: {}...{}", 
        hex::encode(&private_key_bytes[..8]),
        hex::encode(&private_key_bytes[32..]));

    // Create a test message that is ALREADY a Poseidon2 hash output
    // (This matches what the API client sends to KeyManager::sign())
    // For this test, use a deterministic Poseidon2 hash as the message
    use poseidon_hash::hash_to_quintic_extension;
    let plain_data = [Goldilocks::from_canonical_u64(42u64); 10];
    let pre_hashed = hash_to_quintic_extension(&plain_data);
    let message = pre_hashed.to_bytes_le();
    
    println!("  Message (pre-hashed): {}...{}", 
        hex::encode(&message[..8]),
        hex::encode(&message[32..]));

    // Use a fixed nonce for deterministic testing
    let fixed_nonce = [0x01u8; 40];
    println!("  Nonce: {}...{}", 
        hex::encode(&fixed_nonce[..8]),
        hex::encode(&fixed_nonce[32..]));
    println!();

    // Test 1: Generate signature 1000 times with same inputs
    println!("🧪 Generating 1000 signatures with identical inputs...");
    let mut signatures = Vec::new();
    
    for i in 0..1000 {
        // CRITICAL: Use sign_hashed_message() not sign()
        // because our message is ALREADY a Poseidon2 hash output
        let sig = sign_hashed_message(&private_key_bytes, &message, &fixed_nonce)?;
        
        if i < 3 {
            println!("  [{}] Sig: {}...{}", 
                i + 1,
                hex::encode(&sig[..8]),
                hex::encode(&sig[sig.len()-8..]));
        }
        
        signatures.push(sig);
    }

    // Check if all signatures are identical
    let first_sig = &signatures[0];
    let all_identical = signatures.iter().all(|sig| sig == first_sig);

    println!();
    println!("📊 Signature Consistency:");
    if all_identical {
        println!("  ✅ PASS: All 1000 signatures are identical");
        println!("  This proves signature generation is deterministic");
    } else {
        println!("  ❌ FAIL: Signatures differ!");
        println!("  🚨 CRITICAL BUG in signing implementation!");
        
        // Find first difference
        for i in 1..signatures.len() {
            if signatures[i] != *first_sig {
                println!();
                println!("  First difference at signature {}:", i + 1);
                println!("    Sig 1: {}", hex::encode(&signatures[0]));
                println!("    Sig {}: {}", i + 1, hex::encode(&signatures[i]));
                break;
            }
        }
    }

    // Test 2: Verify all signatures
    println!();
    println!("🧪 Verifying all signatures...");
    
    // Derive public key
    let generator = Point::generator();
    let public_point = generator.mul(&private_key);
    let public_key_bytes = public_point.encode().to_bytes_le();
    
    let mut verification_failures = 0;
    for (i, sig) in signatures.iter().enumerate() {
        // CRITICAL: message must be passed as-is (pre-hashed bytes)
        match goldilocks_crypto::verify_signature(sig, &message, &public_key_bytes) {
            Ok(true) => {
                if i < 3 {
                    println!("  [{}] ✅ Valid", i + 1);
                }
            }
            Ok(false) => {
                println!("  [{}] ❌ INVALID signature!", i + 1);
                verification_failures += 1;
            }
            Err(e) => {
                println!("  [{}] ❌ Verification error: {}", i + 1, e);
                verification_failures += 1;
            }
        }
    }

    println!();
    println!("📊 Verification Results:");
    println!("  Total signatures: {}", signatures.len());
    println!("  Valid: {}", signatures.len() - verification_failures);
    println!("  Invalid: {}", verification_failures);
    
    if verification_failures == 0 {
        println!("  ✅ All signatures verified successfully");
    } else {
        println!("  ❌ {} signatures failed verification!", verification_failures);
        println!("  🚨 CRITICAL BUG in signature algorithm!");
    }

    // Test 3: Different nonces should produce different signatures
    println!();
    println!("🧪 Testing nonce variation...");
    
    let nonce1 = [0x01u8; 40];
    let nonce2 = [0x02u8; 40];
    let nonce3 = [0x03u8; 40];
    
    let sig1 = sign_hashed_message(&private_key_bytes, &message, &nonce1)?;
    let sig2 = sign_hashed_message(&private_key_bytes, &message, &nonce2)?;
    let sig3 = sign_hashed_message(&private_key_bytes, &message, &nonce3)?;
    
    if sig1 != sig2 && sig2 != sig3 && sig1 != sig3 {
        println!("  ✅ Different nonces produce different signatures (expected)");
    } else {
        println!("  ❌ Different nonces produce same signature (BUG!)");
    }

    // Verify all three
    let all_valid = [&sig1, &sig2, &sig3].iter().all(|sig| {
        goldilocks_crypto::verify_signature(sig, &message, &public_key_bytes)
            .unwrap_or(false)
    });
    
    if all_valid {
        println!("  ✅ All three signatures verify correctly");
    } else {
        println!("  ❌ Some signatures with different nonces failed verification!");
    }

    // Test 4: Test with random nonces (non-deterministic)
    println!();
    println!("🧪 Testing with random nonces...");
    
    let mut random_sigs = Vec::new();
    for i in 0..10 {
        // Generate random nonce for each signature
        let random_nonce = ScalarField::sample_crypto().to_bytes_le();
        let sig = sign_hashed_message(&private_key_bytes, &message, &random_nonce)?;
        
        println!("  [{}] Sig: {}...{}", 
            i + 1,
            hex::encode(&sig[..8]),
            hex::encode(&sig[sig.len()-8..]));
        
        random_sigs.push(sig);
    }

    // Random signatures should all be different
    let mut all_different = true;
    for i in 0..random_sigs.len() {
        for j in (i+1)..random_sigs.len() {
            if random_sigs[i] == random_sigs[j] {
                println!("  ❌ Signatures {} and {} are identical (should be different)!", i+1, j+1);
                all_different = false;
            }
        }
    }
    
    if all_different {
        println!();
        println!("  ✅ All random signatures are unique (expected)");
    }

    // Verify all random signatures
    let all_valid = random_sigs.iter().all(|sig| {
        goldilocks_crypto::verify_signature(sig, &message, &public_key_bytes)
            .unwrap_or(false)
    });
    
    if all_valid {
        println!("  ✅ All random signatures verify correctly");
    } else {
        println!("  ❌ Some random signatures failed verification!");
    }

    println!();
    println!("================================");
    println!("📋 Summary");
    println!("================================");
    
    if all_identical && verification_failures == 0 && all_different && all_valid {
        println!("✅ Signature algorithm is working correctly");
        println!("   → Fixed nonces produce deterministic signatures");
        println!("   → Random nonces produce unique signatures");
        println!("   → All signatures verify successfully");
        println!("   → Signature generation is NOT the root cause");
        println!();
        println!("👉 Continue to Test 3 (server behavior)");
    } else {
        println!("❌ Signature algorithm has issues:");
        if !all_identical {
            println!("   → Fixed inputs produce different signatures");
        }
        if verification_failures > 0 {
            println!("   → {} signatures failed verification", verification_failures);
        }
        if !all_different {
            println!("   → Random signatures are not unique");
        }
        if !all_valid {
            println!("   → Random signatures fail verification");
        }
        println!();
        println!("🚨 This IS a root cause - signature algorithm needs fixing");
    }

    Ok(())
}

