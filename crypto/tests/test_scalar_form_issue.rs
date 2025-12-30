//! Test to check if scalar form mismatch is causing the issue

use goldilocks_crypto::ScalarField;
use hex;

#[test]
fn test_scalar_forms() {
    println!("\n=== Testing Scalar Forms ===\n");
    
    // Create test values
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3;
        bytes
    }).unwrap();
    
    println!("Input values (should be canonical):");
    println!("  k: {:?}", k.0);
    println!("  sk: {:?}", sk.0);
    println!("  e: {:?}", e.0);
    
    // Compute e*sk
    let e_times_sk = e.mul(&sk);
    println!("\ne.mul(&sk):");
    println!("  Result: {:?}", e_times_sk.0);
    println!("  Is this Montgomery form?");
    
    // Convert to canonical
    let e_times_sk_canonical = e_times_sk.to_canonical();
    println!("  Canonical: {:?}", e_times_sk_canonical.0);
    
    // Compute s = k - e*sk
    let s1 = k.sub(e_times_sk);
    let s2 = k.sub(e_times_sk_canonical);
    
    println!("\ns = k - e*sk (using Montgomery form):");
    println!("  s: {:?}", s1.0);
    println!("s = k - e*sk (using canonical form):");
    println!("  s: {:?}", s2.0);
    
    // Check if s + e*sk = k
    let check1 = s1.add(e_times_sk);
    let check2 = s2.add(e_times_sk_canonical);
    
    println!("\ns + e*sk (both in Montgomery):");
    println!("  Result: {:?}", check1.0);
    println!("  Matches k: {}", check1.0 == k.0);
    
    println!("\ns + e*sk (both in canonical):");
    println!("  Result: {:?}", check2.0);
    println!("  Matches k: {}", check2.0 == k.0);
    
    // Also check what happens if we convert s to canonical after computing
    let s1_canonical = s1.to_canonical();
    let check3 = s1_canonical.add(e_times_sk_canonical);
    println!("\ns (canonical) + e*sk (canonical):");
    println!("  s canonical: {:?}", s1_canonical.0);
    println!("  Result: {:?}", check3.0);
    println!("  Matches k: {}", check3.0 == k.0);
}













