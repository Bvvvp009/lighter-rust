//! Test sub() logic with simple values

use goldilocks_crypto::ScalarField;

fn main() {
    println!("Testing sub() logic\n");
    
    // Test case: k = 11, e*sk = 21
    // k - e*sk should be N - 10 (if k < e*sk, result is k - e*sk + N)
    // Then (k - e*sk) + e*sk should equal k
    
    let k = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 11;
        bytes
    }).unwrap();
    
    let e_sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 21;
        bytes
    }).unwrap();
    
    println!("k: {:?}", k.0);
    println!("e*sk: {:?}", e_sk.0);
    
    // Test subtraction
    let s = k.sub(e_sk);
    println!("\ns = k - e*sk: {:?}", s.0);
    
    // Test roundtrip
    let k_reconstructed = s.add(e_sk);
    println!("k_reconstructed = s + e*sk: {:?}", k_reconstructed.0);
    println!("k: {:?}", k.0);
    
    println!("\nk == k_reconstructed: {}", k.0 == k_reconstructed.0);
    
    // Check if k_reconstructed == k - N
    let k_minus_n = k.sub(ScalarField::N);
    println!("k - N: {:?}", k_minus_n.0);
    println!("k_reconstructed == k - N: {}", k_reconstructed.0 == k_minus_n.0);
    
    // Check if k_reconstructed == k + N (should be reduced to k)
    let k_plus_n = k.add(ScalarField::N);
    println!("k + N (should reduce to k): {:?}", k_plus_n.0);
    println!("k + N == k: {}", k_plus_n.0 == k.0);
}










