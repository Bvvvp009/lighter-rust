use goldilocks_crypto::schnorr;

fn main() {
    // Signatures from the stress test that failed with code 21120
    // We verify if they're cryptographically valid locally
    
    println!("\n{}", "=".repeat(80));
    println!("Verifying Failed Signatures from Stress Test");
    println!("{}\n", "=".repeat(80));
    
    // From the captured [SIG_DEBUG] output, we have multiple test cases
    // Testing with real data from orders that failed on server
    
    let test_cases = vec![
        (
            "stress_test_order",
            // pubkey
            hex::decode("99f3473027655c41eebb21afd06b516b438b42ad70c27ac8208cdb56b60be7d5c9ddfb05e3cf9518").unwrap(),
            // signature (from order 15)
            hex::decode("5a18141c89794b0dc6abea84edfa5eea1f7d38be59c637aefb29b7ea2d2eacd789e891ef31c0956b025ebbd692054b8573009b2ae178586c348d24811f12955f83e62693").unwrap(),
            // hash_bytes
            hex::decode("6437b3bbe682ca2b763a0c85d7ff3922a1ed67918b16f3f0b36bbc7fbe47354d5c7e83077127967e").unwrap(),
        ),
    ];
    
    for (label, pubkey_bytes, sig_bytes, hash_bytes) in test_cases {
        println!("Testing: {}", label);
        println!("  Pubkey (40B): {}", hex::encode(&pubkey_bytes[..20]));
        println!("  Sig (80B):    {}", hex::encode(&sig_bytes[..40]));
        println!("  Hash (40B):   {}", hex::encode(&hash_bytes[..20]));
        
        match schnorr::verify_signature(&sig_bytes, hash_bytes.as_slice(), pubkey_bytes.as_slice()) {
            Ok(true) => {
                println!("  Result: ✅ VALID\n");
            }
            Ok(false) => {
                println!("  Result: ❌ INVALID\n");
            }
            Err(e) => {
                println!("  Result: ❌ ERROR - {}\n", e);
            }
        }
    }
    
    println!("{}", "=".repeat(80));
    println!("DIAGNOSIS:");
    println!("{}", "=".repeat(80));
    println!("If all signatures verify VALID here but server still rejects with 21120,");
    println!("this indicates:");
    println!("");
    println!("  ❌ NOT a crypto algorithm problem");
    println!("  ✅ Likely a field mismatch or account/credential issue");
    println!("");
    println!("Next steps:");
    println!("  1. Compare which fields server includes in transaction hash");
    println!("  2. Verify account_index and api_key_index are correct");
    println!("  3. Check if public key derivation matches server's expectations");
    println!("{}\n", "=".repeat(80));
}
