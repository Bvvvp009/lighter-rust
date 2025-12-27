//! Debug Arithmetic Roundtrip Failures
//!
//! This tool provides detailed debugging output when the roundtrip fails

use goldilocks_crypto::ScalarField;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging Arithmetic Roundtrip Failures\n");
    
    // Use a failing case from the test
    let mut found_failure = false;
    
    for _attempt in 0..100 {
        let k = ScalarField::sample_crypto();
        let e = ScalarField::sample_crypto();
        let sk = ScalarField::sample_crypto();
        
        // Compute e*sk (Montgomery form)
        let e_times_sk_montgomery = e.mul(&sk);
        
        // Convert to canonical
        let e_times_sk_canonical = e_times_sk_montgomery.to_canonical();
        
        // Compute s = k - e*sk (both in canonical form)
        let s = k.sub(e_times_sk_canonical);
        
        // Verify roundtrip: k = s + e*sk
        let k_reconstructed = s.add(e_times_sk_canonical);
        
        if k.0 != k_reconstructed.0 {
            found_failure = true;
            println!("{}", "=".repeat(70));
            println!("FOUND FAILURE:");
            println!("{}", "=".repeat(70));
            println!("\nk (original):     {}", hex::encode(&k.to_bytes_le()));
            println!("k (reconstructed): {}", hex::encode(&k_reconstructed.to_bytes_le()));
            println!("\ne:                {}", hex::encode(&e.to_bytes_le()));
            println!("sk:               {}", hex::encode(&sk.to_bytes_le()));
            println!("\ne*sk (Montgomery): {}", hex::encode(&e_times_sk_montgomery.to_bytes_le()));
            println!("e*sk (canonical):  {}", hex::encode(&e_times_sk_canonical.to_bytes_le()));
            println!("\ns = k - e*sk:      {}", hex::encode(&s.to_bytes_le()));
            
            println!("\n--- Detailed Limb Analysis ---");
            for i in 0..5 {
                println!("Limb[{}]:", i);
                println!("  k[{}] = 0x{:016x}", i, k.0[i]);
                println!("  k_reconstructed[{}] = 0x{:016x}", i, k_reconstructed.0[i]);
                println!("  diff = 0x{:016x}", k.0[i] ^ k_reconstructed.0[i]);
                if k.0[i] < k_reconstructed.0[i] {
                    println!("  k[{}] < k_reconstructed[{}]", i, i);
                } else if k.0[i] > k_reconstructed.0[i] {
                    println!("  k[{}] > k_reconstructed[{}]", i, i);
                }
            }
            
            // Check if k_reconstructed == k + N or k - N
            let k_plus_n = k.add(ScalarField::N);
            let k_minus_n_attempt = k.sub(ScalarField::N);
            
            println!("\n--- Checking if k_reconstructed == k + N or k - N ---");
            println!("k + N:  {}", hex::encode(&k_plus_n.to_bytes_le()));
            println!("k - N:  {}", hex::encode(&k_minus_n_attempt.to_bytes_le()));
            println!("k_reconstructed == k + N: {}", k_reconstructed.0 == k_plus_n.0);
            println!("k_reconstructed == k - N: {}", k_reconstructed.0 == k_minus_n_attempt.0);
            
            // Check what s + e*sk actually gives us in different forms
            println!("\n--- Testing different forms of e*sk in addition ---");
            let k_from_montgomery = s.add(e_times_sk_montgomery);
            println!("s + e*sk (Montgomery form): {}", hex::encode(&k_from_montgomery.to_bytes_le()));
            println!("Matches k: {}", k_from_montgomery.0 == k.0);
            
            // Check the actual arithmetic: verify s + e*sk = k
            println!("\n--- Manual Verification ---");
            let s_plus_e_sk = s.add(e_times_sk_canonical);
            println!("s + e*sk (canonical): {}", hex::encode(&s_plus_e_sk.to_bytes_le()));
            
            break;
        }
    }
    
    if !found_failure {
        println!("No failures found in 100 attempts");
    }
    
    Ok(())
}





