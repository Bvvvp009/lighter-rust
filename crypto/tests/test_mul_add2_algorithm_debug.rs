//! Debug the mul_add2 algorithm step by step

use goldilocks_crypto::{ScalarField, Point, WeierstrassPoint, Fp5Element};
use hex;

#[test]
fn test_mul_add2_algorithm_step_by_step() {
    println!("\n=== Debugging mul_add2 Algorithm Step by Step ===\n");
    
    // Use simple values for debugging
    let s = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5; // s = 5
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
    
    println!("Test values:");
    println!("  s = 5");
    println!("  e = 3");
    println!("  sk = 7");
    
    // Compute expected k
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let k = s.add(e_times_sk_canonical);
    
    println!("\nScalar arithmetic:");
    println!("  e*sk (Montgomery): {}...", hex::encode(&e_times_sk.to_bytes_le()[..8]));
    println!("  e*sk (canonical): {}...", hex::encode(&e_times_sk_canonical.to_bytes_le()[..8]));
    println!("  k = s + e*sk: {}...", hex::encode(&k.to_bytes_le()[..8]));
    
    // Check limb splitting
    let s_limbs = s.split_to_4bit_limbs();
    let e_limbs = e.split_to_4bit_limbs();
    let k_limbs = k.split_to_4bit_limbs();
    
    println!("\n4-bit limbs (first 10):");
    println!("  s: {:?}", &s_limbs[0..10]);
    println!("  e: {:?}", &e_limbs[0..10]);
    println!("  k: {:?}", &k_limbs[0..10]);
    
    // Compute expected: k*G
    let generator = Point::generator();
    let expected_r = generator.mul(&k);
    let expected_encoded = expected_r.encode();
    
    println!("\nExpected R (k*G):");
    println!("  Encoded: {}...", hex::encode(&expected_encoded.to_bytes_le()[..16]));
    
    // Compute with separate multiplications: s*G and e*P
    let public_key = generator.mul(&sk);
    let s_g = generator.mul(&s);
    let e_p = public_key.mul(&e);
    let separate_result = s_g.add(&e_p);
    let separate_encoded = separate_result.encode();
    
    println!("\nSeparate computation (s*G).add(e*P):");
    println!("  s*G: {}...", hex::encode(&s_g.encode().to_bytes_le()[..16]));
    println!("  e*P: {}...", hex::encode(&e_p.encode().to_bytes_le()[..16]));
    println!("  Result: {}...", hex::encode(&separate_encoded.to_bytes_le()[..16]));
    
    // Check if separate matches expected
    let separate_match = expected_encoded.0.iter().zip(separate_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    println!("  Matches expected: {}", separate_match);
    
    if !separate_match {
        println!("\n❌ Even separate multiplication fails!");
        println!("  This means the issue is in Point::mul() or Point::add()");
        
        // Check if e*P is computed correctly
        // e*P should equal (e*sk)*G
        let e_sk_g = generator.mul(&e_times_sk_canonical);
        let e_sk_g_encoded = e_sk_g.encode();
        let e_p_encoded = e_p.encode();
        
        println!("\n  Checking e*P computation:");
        println!("    e*P: {}...", hex::encode(&e_p_encoded.to_bytes_le()[..16]));
        println!("    (e*sk)*G: {}...", hex::encode(&e_sk_g_encoded.to_bytes_le()[..16]));
        let e_p_match = e_p_encoded.0.iter().zip(e_sk_g_encoded.0.iter())
            .all(|(a, b)| a.0 == b.0);
        println!("    e*P == (e*sk)*G: {}", e_p_match);
        
        if !e_p_match {
            println!("    ❌ e*P is NOT equal to (e*sk)*G!");
            println!("    This is the root cause!");
        }
    }
}








