//! Test to identify the bug in scalar add/sub operations

use goldilocks_crypto::ScalarField;

#[test]
fn test_scalar_add_sub_bug() {
    println!("\n=== Testing Scalar Add/Sub Bug ===\n");
    
    // Simple test: 11 - 21 should be -10 mod N, then -10 + 21 should be 11
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11; // k = 11
        bytes
    }).unwrap();
    
    let e_times_sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 21; // e*sk = 21 (for testing)
        bytes
    }).unwrap();
    
    println!("k = 11:");
    println!("  Limbs: {:?}", k.0);
    
    println!("\ne*sk = 21:");
    println!("  Limbs: {:?}", e_times_sk.0);
    
    // Compute s = k - e*sk = 11 - 21 = -10 mod N
    let s = k.sub(e_times_sk);
    println!("\ns = k - e*sk = 11 - 21:");
    println!("  Limbs: {:?}", s.0);
    
    // Verify: s + e*sk should equal k
    let k_reconstructed = s.add(e_times_sk);
    println!("\nk_reconstructed = s + e*sk:");
    println!("  Limbs: {:?}", k_reconstructed.0);
    
    println!("\nComparison:");
    println!("  k: {:?}", k.0);
    println!("  k_reconstructed: {:?}", k_reconstructed.0);
    println!("  Match: {}", k.0 == k_reconstructed.0);
    
    if k.0 != k_reconstructed.0 {
        println!("\n❌ BUG FOUND: add/sub roundtrip fails!");
        for i in 0..5 {
            if k.0[i] != k_reconstructed.0[i] {
                println!("  Limb[{}]: k=0x{:016x}, k_recon=0x{:016x}, diff=0x{:016x}",
                    i, k.0[i], k_reconstructed.0[i], k.0[i] ^ k_reconstructed.0[i]);
            }
        }
        
        // Check if k_reconstructed == k - N (modular reduction issue)
        let n = ScalarField::N;
        let k_minus_n = k.sub(n);
        println!("\n  k - N: {:?}", k_minus_n.0);
        println!("  k_reconstructed == k - N: {}", k_reconstructed.0 == k_minus_n.0);
    } else {
        println!("\n✅ add/sub roundtrip works");
    }
}












