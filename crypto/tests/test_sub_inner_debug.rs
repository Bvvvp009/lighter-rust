//! Debug test for sub_inner to understand borrow flag

use goldilocks_crypto::ScalarField;

#[test]
fn test_sub_inner_debug() {
    println!("\n=== Testing sub_inner Debug ===\n");
    
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11; // k = 11
        bytes
    }).unwrap();
    
    let e_times_sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 21; // e*sk = 21
        bytes
    }).unwrap();
    
    println!("k = 11:");
    println!("  Limbs: {:?}", k.0);
    
    println!("\ne*sk = 21:");
    println!("  Limbs: {:?}", e_times_sk.0);
    
    // Test sub_inner
    let (r0, c) = k.sub_inner(&e_times_sk);
    println!("\nsub_inner(k, e*sk):");
    println!("  Result: {:?}", r0.0);
    println!("  Borrow flag c: 0x{:016x}", c);
    println!("  c == 0: {}", c == 0);
    println!("  c != 0: {}", c != 0);
    
    // Expected: 11 - 21 = -10, which should trigger a borrow
    // So c should be 0xFFFFFFFFFFFFFFFF
    println!("\nExpected: 11 - 21 = -10 (should borrow)");
    println!("  Borrow flag should be: 0xFFFFFFFFFFFFFFFF");
    println!("  Actual borrow flag: 0x{:016x}", c);
    
    // Now test the full sub() function
    let s = k.sub(e_times_sk);
    println!("\nsub(k, e*sk):");
    println!("  Result: {:?}", s.0);
    
    // Check if s + e*sk = k
    let k_reconstructed = s.add(e_times_sk);
    println!("\nk_reconstructed = s + e*sk:");
    println!("  Result: {:?}", k_reconstructed.0);
    println!("  Matches k: {}", k.0 == k_reconstructed.0);
}








