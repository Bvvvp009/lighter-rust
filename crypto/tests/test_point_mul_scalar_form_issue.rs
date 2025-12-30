//! Test to verify the scalar form issue in Point::mul()

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_point_mul_scalar_form_issue() {
    println!("\n=== Testing Point::mul() Scalar Form Issue ===\n");
    
    let generator = Point::generator();
    
    // Test: When we compute P = sk*G, then e*P, does it match (e*sk)*G?
    let sk = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 7; // sk = 7
        bytes
    }).unwrap();
    
    let e = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 3; // e = 3
        bytes
    }).unwrap();
    
    // Compute P = sk*G
    let public_key = generator.mul(&sk);
    
    // Compute e*P
    let e_p = public_key.mul(&e);
    let e_p_encoded = e_p.encode();
    
    println!("e*P (where P = sk*G):");
    println!("  Result: {}...", hex::encode(&e_p_encoded.to_bytes_le()[..16]));
    
    // Compute (e*sk)*G with canonical form
    let e_times_sk = e.mul(&sk);
    let e_times_sk_canonical = e_times_sk.to_canonical();
    let e_sk_g_canonical = generator.mul(&e_times_sk_canonical);
    let e_sk_g_canonical_encoded = e_sk_g_canonical.encode();
    
    println!("\n(e*sk canonical)*G:");
    println!("  Result: {}...", hex::encode(&e_sk_g_canonical_encoded.to_bytes_le()[..16]));
    
    // Compute (e*sk)*G with Montgomery form
    let e_sk_g_montgomery = generator.mul(&e_times_sk);
    let e_sk_g_montgomery_encoded = e_sk_g_montgomery.encode();
    
    println!("\n(e*sk Montgomery)*G:");
    println!("  Result: {}...", hex::encode(&e_sk_g_montgomery_encoded.to_bytes_le()[..16]));
    
    // Compare
    let match_canonical = e_p_encoded.0.iter().zip(e_sk_g_canonical_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    let match_montgomery = e_p_encoded.0.iter().zip(e_sk_g_montgomery_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\nComparisons:");
    println!("  e*P == (e*sk canonical)*G: {}", match_canonical);
    println!("  e*P == (e*sk Montgomery)*G: {}", match_montgomery);
    
    if match_montgomery && !match_canonical {
        println!("\n✅ FOUND IT: Point::mul() treats scalars as Montgomery form!");
        println!("   When P was computed from sk*G, then e*P uses e as if it's in Montgomery form");
        println!("   But e from signature is in canonical form!");
        println!("\n   SOLUTION: Convert e to Montgomery form before calling P.mul(&e)");
    } else if match_canonical {
        println!("\n✅ Point::mul() works correctly with canonical scalars");
    } else {
        println!("\n❌ Neither form matches - there's a different bug");
    }
}












