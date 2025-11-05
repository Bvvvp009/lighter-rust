#[cfg(test)]
mod tests {
    use crate::{Goldilocks, Fp5Element, hash_to_quintic_extension};

    #[test]
    fn test_poseidon2_hash_to_quintic_extension() {
        // Test hash_to_quintic_extension function
        let input = vec![
            Goldilocks::from_canonical_u64(1),
            Goldilocks::from_canonical_u64(2),
            Goldilocks::from_canonical_u64(3),
            Goldilocks::from_canonical_u64(4),
            Goldilocks::from_canonical_u64(5),
        ];
        
        let result = hash_to_quintic_extension(&input);
        
        // Verify it's not zero
        assert!(!result.is_zero());
        
        // Verify it's a valid Fp5 element
        assert!(result.0.iter().any(|&x| x != Goldilocks::zero()));
    }
    
    #[test]
    fn test_poseidon2_basic_functionality() {
        // Test basic Poseidon2 functionality without complex constants
        let input = vec![
            Goldilocks::from_canonical_u64(1),
            Goldilocks::from_canonical_u64(2),
            Goldilocks::from_canonical_u64(3),
            Goldilocks::from_canonical_u64(4),
        ];
        
        let result = hash_to_quintic_extension(&input);
        
        // Verify it produces a valid result
        assert!(!result.is_zero());
        
        // Test that same input produces same output
        let result2 = hash_to_quintic_extension(&input);
        assert_eq!(result.to_bytes_le(), result2.to_bytes_le());
    }
}
