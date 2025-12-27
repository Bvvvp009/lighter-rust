//! Test Arithmetic Roundtrip: Verify s = k - e*sk and k = s + e*sk
//!
//! This test verifies that the scalar arithmetic works correctly:
//! - s = k - e*sk (where k is nonce, e is challenge, sk is private key)
//! - k = s + e*sk (roundtrip should work)
//!
//! Usage: cargo run --example test_arithmetic_roundtrip --release

use goldilocks_crypto::ScalarField;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Scalar Arithmetic Roundtrip\n");
    println!("Testing: s = k - e*sk and k = s + e*sk\n");
    
    // Test with multiple random values
    let num_tests = 10;
    let mut passed = 0;
    let mut failed = 0;
    
    for i in 0..num_tests {
        println!("Test {}: {}", i + 1, "=".repeat(60));
        
        // Generate random scalars
        let k = ScalarField::sample_crypto();
        let e = ScalarField::sample_crypto();
        let sk = ScalarField::sample_crypto();
        
        println!("  k (nonce):      {}", hex::encode(&k.to_bytes_le()));
        println!("  e (challenge):  {}", hex::encode(&e.to_bytes_le()));
        println!("  sk (private):   {}", hex::encode(&sk.to_bytes_le()));
        
        // Compute e*sk (this is in Montgomery form)
        let e_times_sk = e.mul(&sk);
        println!("  e*sk (Montgomery): {}", hex::encode(&e_times_sk.to_bytes_le()));
        
        // Convert to canonical
        let e_times_sk_canonical = e_times_sk.to_canonical();
        println!("  e*sk (canonical):  {}", hex::encode(&e_times_sk_canonical.to_bytes_le()));
        
        // Compute s = k - e*sk (both in canonical form)
        let s = k.sub(e_times_sk_canonical);
        println!("  s = k - e*sk:      {}", hex::encode(&s.to_bytes_le()));
        
        // Verify roundtrip: k = s + e*sk
        let k_reconstructed = s.add(e_times_sk_canonical);
        println!("  k_reconstructed:   {}", hex::encode(&k_reconstructed.to_bytes_le()));
        
        // Check if they match
        let matches = k.0 == k_reconstructed.0;
        if matches {
            println!("  ✅ Roundtrip works: k == k_reconstructed");
            passed += 1;
        } else {
            println!("  ❌ Roundtrip FAILS: k != k_reconstructed");
            failed += 1;
            
            // Show differences
            for j in 0..5 {
                if k.0[j] != k_reconstructed.0[j] {
                    println!("    Limb[{}]: k=0x{:016x}, k_reconstructed=0x{:016x}, diff=0x{:016x}",
                        j, k.0[j], k_reconstructed.0[j], k.0[j] ^ k_reconstructed.0[j]);
                }
            }
        }
        println!();
    }
    
    println!("{}", "=".repeat(70));
    println!("Summary:");
    println!("  Total tests: {}", num_tests);
    println!("  Passed:      {} ✅", passed);
    println!("  Failed:      {} {}", failed, if failed > 0 { "❌" } else { "✅" });
    println!("{}", "=".repeat(70));
    
    if failed > 0 {
        Err("Some arithmetic roundtrip tests failed!".into())
    } else {
        Ok(())
    }
}






