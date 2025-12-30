//! Test if hash function is deterministic

use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
use goldilocks_crypto::ScalarField;

#[test]
fn test_hash_deterministic() {
    println!("\n=== Testing Hash Determinism ===\n");
    
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[0] = Goldilocks::from_canonical_u64(100);
    pre_image[5] = Goldilocks::from_canonical_u64(200);
    
    println!("Pre-image:");
    for i in 0..10 {
        println!("  Pre-image[{}] = {}", i, pre_image[i].0);
    }
    
    // Call hash twice
    let e1_fp5 = hash_to_quintic_extension(&pre_image);
    let e2_fp5 = hash_to_quintic_extension(&pre_image);
    
    println!("\nHash results:");
    println!("  e1_fp5 limbs: {:?}", e1_fp5.0.iter().map(|g| g.0).collect::<Vec<_>>());
    println!("  e2_fp5 limbs: {:?}", e2_fp5.0.iter().map(|g| g.0).collect::<Vec<_>>());
    
    let match_fp5 = e1_fp5.0.iter().zip(e2_fp5.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  e1_fp5 == e2_fp5: {}", match_fp5);
    
    let e1_scalar = ScalarField::from_fp5_element(&e1_fp5);
    let e2_scalar = ScalarField::from_fp5_element(&e2_fp5);
    
    println!("\nScalar results:");
    println!("  e1_scalar: {}", hex::encode(&e1_scalar.to_bytes_le()));
    println!("  e2_scalar: {}", hex::encode(&e2_scalar.to_bytes_le()));
    println!("  e1_scalar == e2_scalar: {}", e1_scalar.to_bytes_le() == e2_scalar.to_bytes_le());
    
    assert!(match_fp5, "Hash should be deterministic");
    assert_eq!(e1_scalar.to_bytes_le(), e2_scalar.to_bytes_le(), "Scalar conversion should be deterministic");
}







