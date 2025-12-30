//! Test to verify scalar arithmetic: s = k - e*sk and k = s + e*sk

use goldilocks_crypto::ScalarField;
use hex;

#[test]
fn test_scalar_arithmetic_roundtrip() {
    println!("\n=== Testing Scalar Arithmetic Roundtrip ===\n");
    
    // Test: if s = k - e*sk, then k = s + e*sk should hold
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11; // k = 11
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    println!("Input values:");
    println!("  k: {}", hex::encode(&k.to_bytes_le()));
    println!("  e: {}", hex::encode(&e.to_bytes_le()));
    println!("  sk: {}", hex::encode(&sk.to_bytes_le()));
    
    // Compute e*sk
    let e_times_sk = e.mul(&sk);
    println!("\ne.mul(&sk) (Montgomery form):");
    println!("  Result: {}", hex::encode(&e_times_sk.to_bytes_le()));
    
    // Convert to canonical
    let e_times_sk_canonical = e_times_sk.to_canonical();
    println!("e.mul(&sk).to_canonical():");
    println!("  Result: {}", hex::encode(&e_times_sk_canonical.to_bytes_le()));
    
    // Compute s = k - e*sk (both in canonical)
    let s = k.sub(e_times_sk_canonical);
    println!("\ns = k - e*sk (canonical):");
    println!("  Result: {}", hex::encode(&s.to_bytes_le()));
    
    // Now verify: k = s + e*sk
    let k_reconstructed = s.add(e_times_sk_canonical);
    println!("\nk_reconstructed = s + e*sk (canonical):");
    println!("  Result: {}", hex::encode(&k_reconstructed.to_bytes_le()));
    
    let match_result = k.0 == k_reconstructed.0;
    
    if match_result {
        println!("\n✅ k == s + e*sk (roundtrip works)");
    } else {
        println!("\n❌ k != s + e*sk (roundtrip FAILS)");
        println!("  This is a bug in scalar arithmetic!");
        
        for i in 0..5 {
            if k.0[i] != k_reconstructed.0[i] {
                println!("  Limb[{}]: k=0x{:016x}, k_reconstructed=0x{:016x}, diff=0x{:016x}",
                    i, k.0[i], k_reconstructed.0[i], k.0[i] ^ k_reconstructed.0[i]);
            }
        }
    }
    
    assert!(match_result, "Scalar arithmetic roundtrip should work");
}

#[test]
fn test_scalar_arithmetic_with_montgomery() {
    println!("\n=== Testing Scalar Arithmetic with Montgomery Form ===\n");
    
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11;
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3;
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7;
        bytes
    }).unwrap();
    
    // Test 1: Using canonical form (correct)
    let e_times_sk_canonical = e.mul(&sk).to_canonical();
    let s_canonical = k.sub(e_times_sk_canonical);
    let k_reconstructed_canonical = s_canonical.add(e_times_sk_canonical);
    
    println!("Using canonical form:");
    println!("  s: {}", hex::encode(&s_canonical.to_bytes_le()));
    println!("  k_reconstructed: {}", hex::encode(&k_reconstructed_canonical.to_bytes_le()));
    println!("  Match: {}", k.0 == k_reconstructed_canonical.0);
    
    // Test 2: Using Montgomery form (incorrect - but let's see what happens)
    let e_times_sk_montgomery = e.mul(&sk);
    let s_montgomery = k.sub(e_times_sk_montgomery);
    let k_reconstructed_montgomery = s_montgomery.add(e_times_sk_montgomery);
    
    println!("\nUsing Montgomery form:");
    println!("  s: {}", hex::encode(&s_montgomery.to_bytes_le()));
    println!("  k_reconstructed: {}", hex::encode(&k_reconstructed_montgomery.to_bytes_le()));
    println!("  Match: {}", k.0 == k_reconstructed_montgomery.0);
    
    // Test 3: Mixed forms
    let s_mixed1 = k.sub(e_times_sk_montgomery); // k (canonical) - e*sk (Montgomery)
    let k_reconstructed_mixed1 = s_mixed1.add(e_times_sk_canonical); // s (mixed) + e*sk (canonical)
    
    println!("\nUsing mixed forms (k canonical, e*sk Montgomery):");
    println!("  s: {}", hex::encode(&s_mixed1.to_bytes_le()));
    println!("  k_reconstructed: {}", hex::encode(&k_reconstructed_mixed1.to_bytes_le()));
    println!("  Match: {}", k.0 == k_reconstructed_mixed1.0);
}












