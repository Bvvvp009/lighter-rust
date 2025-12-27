//! Test if from_fp5_element is deterministic

use goldilocks_crypto::{ScalarField, Fp5Element};
use poseidon_hash::Goldilocks;

#[test]
fn test_from_fp5_element_deterministic() {
    println!("\n=== Testing from_fp5_element Determinism ===\n");
    
    // Create an Fp5Element
    let mut elements = [Goldilocks::zero(); 5];
    elements[0] = Goldilocks::from_canonical_u64(100);
    elements[1] = Goldilocks::from_canonical_u64(200);
    elements[2] = Goldilocks::from_canonical_u64(300);
    elements[3] = Goldilocks::from_canonical_u64(400);
    elements[4] = Goldilocks::from_canonical_u64(500);
    
    let fp5 = Fp5Element(elements);
    
    println!("Fp5Element limbs:");
    for i in 0..5 {
        println!("  Fp5[{}] = {}", i, fp5.0[i].0);
    }
    
    // Call from_fp5_element twice
    let scalar1 = ScalarField::from_fp5_element(&fp5);
    let scalar2 = ScalarField::from_fp5_element(&fp5);
    
    println!("\nScalar results:");
    println!("  scalar1: {}", hex::encode(&scalar1.to_bytes_le()));
    println!("  scalar2: {}", hex::encode(&scalar2.to_bytes_le()));
    println!("  Match: {}", scalar1.to_bytes_le() == scalar2.to_bytes_le());
    
    // Check limbs
    println!("\nLimb comparison:");
    for i in 0..5 {
        if scalar1.0[i] != scalar2.0[i] {
            println!("  Limb[{}]: scalar1={}, scalar2={}", i, scalar1.0[i], scalar2.0[i]);
        }
    }
    
    assert_eq!(scalar1.to_bytes_le(), scalar2.to_bytes_le(), "from_fp5_element should be deterministic");
}



