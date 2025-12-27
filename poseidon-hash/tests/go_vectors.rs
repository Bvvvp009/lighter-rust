//! Poseidon hash test vectors from Go implementation
//! 
//! These test vectors are extracted from Go's TestHashToQuinticExtension
//! to verify byte-for-byte compatibility.

use poseidon_hash::{Goldilocks, hash_to_quintic_extension, Fp5Element};

#[test]
fn test_hash_to_quintic_extension_go_vector() {
    // Test vector from Go's TestHashToQuinticExtension
    // Go: poseidon2_goldilocks/poseidon2_test.go:256-279
    
    let inputs = vec![
        Goldilocks::from_canonical_u64(3451004116618606032),
        Goldilocks::from_canonical_u64(11263134342958518251),
        Goldilocks::from_canonical_u64(10957204882857370932),
        Goldilocks::from_canonical_u64(5369763041201481933),
        Goldilocks::from_canonical_u64(7695734348563036858),
        Goldilocks::from_canonical_u64(1393419330378128434),
        Goldilocks::from_canonical_u64(7387917082382606332),
    ];
    
    let result = hash_to_quintic_extension(&inputs);
    
    // Expected output from Go test
    let expected = Fp5Element::from_uint64_array([
        17992684813643984528,
        5243896189906434327,
        7705560276311184368,
        2785244775876017560,
        14449776097783372302,
    ]);
    
    // Verify each limb matches
    for i in 0..5 {
        assert_eq!(
            result.0[i].to_canonical_u64(),
            expected.0[i].to_canonical_u64(),
            "Limb {} mismatch: expected {}, got {}",
            i,
            expected.0[i].to_canonical_u64(),
            result.0[i].to_canonical_u64()
        );
    }
    
    println!("✅ Poseidon hash output matches Go test vector");
}

#[test]
fn test_hash_single_element() {
    // Test hashing a single element
    let inputs = vec![Goldilocks::from_canonical_u64(1)];
    let result = hash_to_quintic_extension(&inputs);
    
    // Verify result is 40 bytes (5 limbs)
    let bytes = result.to_bytes_le();
    assert_eq!(bytes.len(), 40);
    
    // Verify not all zeros
    assert!(!bytes.iter().all(|&b| b == 0));
    
    println!("✅ Single element hash works");
}

#[test]
fn test_hash_empty_input() {
    // Test hashing empty input (should pad and hash)
    let inputs = vec![];
    let result = hash_to_quintic_extension(&inputs);
    
    // Verify result is 40 bytes
    let bytes = result.to_bytes_le();
    assert_eq!(bytes.len(), 40);
    
    println!("✅ Empty input hash works");
}

#[test]
fn test_hash_consistency() {
    // Test that same input produces same output
    let inputs = vec![
        Goldilocks::from_canonical_u64(1),
        Goldilocks::from_canonical_u64(2),
        Goldilocks::from_canonical_u64(3),
    ];
    
    let result1 = hash_to_quintic_extension(&inputs);
    let result2 = hash_to_quintic_extension(&inputs);
    
    assert_eq!(result1, result2, "Hash should be deterministic");
    println!("✅ Hash consistency verified");
}













