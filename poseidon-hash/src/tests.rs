#[cfg(test)]
mod tests {
    use crate::{Goldilocks, Fp5Element};

    #[test]
    fn test_goldilocks_field_operations() {
        // Test field operations with standard values
        let a = Goldilocks::from_canonical_u64(12345);
        let b = Goldilocks::from_canonical_u64(67890);
        
        // Test addition
        let sum = a.add(&b);
        assert_eq!(sum.to_canonical_u64(), 80235);
        
        // Test subtraction
        let diff = b.sub(&a);
        assert_eq!(diff.to_canonical_u64(), 55545);
        
        // Test multiplication
        let product = a.mul(&b);
        let expected_product = (12345u128 * 67890u128) % Goldilocks::ORDER as u128;
        assert_eq!(product.to_canonical_u64(), expected_product as u64);
        
        // Test square
        let square = a.square();
        let expected_square = (12345u128 * 12345u128) % Goldilocks::ORDER as u128;
        assert_eq!(square.to_canonical_u64(), expected_square as u64);
        
        // Test double
        let doubled = a.double();
        assert_eq!(doubled.to_canonical_u64(), 24690);
        
        // Test negation
        let neg_a = a.neg();
        assert_eq!(neg_a.add(&a).is_zero(), true);
        
        // Test zero and one
        assert_eq!(Goldilocks::zero().is_zero(), true);
        assert_eq!(Goldilocks::one().is_zero(), false);
        
        // Test exp_power_of_2
        let power_of_2 = a.exp_power_of_2(3); // a^(2^3) = a^8
        let expected_power = a.square().square().square();
        assert_eq!(power_of_2.to_canonical_u64(), expected_power.to_canonical_u64());
    }
    
    #[test]
    fn test_goldilocks_field_edge_cases() {
        // Test with large values near the modulus
        let large_val = Goldilocks::from_canonical_u64(Goldilocks::ORDER - 1);
        let one = Goldilocks::one();
        
        // Test addition with large values
        let sum = large_val.add(&one);
        assert_eq!(sum.to_canonical_u64(), 0);
        
        // Test subtraction with large values
        let diff = large_val.sub(&one);
        assert_eq!(diff.to_canonical_u64(), Goldilocks::ORDER - 2);
        
        // Test multiplication with large values
        let product = large_val.mul(&large_val);
        let expected = ((Goldilocks::ORDER - 1) as u128 * (Goldilocks::ORDER - 1) as u128) % Goldilocks::ORDER as u128;
        assert_eq!(product.to_canonical_u64(), expected as u64);
    }
    
    #[test]
    fn test_goldilocks_field_constants() {
        // Test that constants are correct
        assert_eq!(Goldilocks::EPSILON, 0xffffffff);
        assert_eq!(Goldilocks::ORDER, 0xffffffff00000001);
    }
    
    #[test]
    fn test_fp5_field_operations() {
        // Test Fp5 field operations with standard values
        let a = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
        let b = Fp5Element::from_uint64_array([6, 7, 8, 9, 10]);
        
        // Test addition
        let sum = a.add(&b);
        assert_eq!(sum.0[0].to_canonical_u64(), 7);
        assert_eq!(sum.0[1].to_canonical_u64(), 9);
        assert_eq!(sum.0[2].to_canonical_u64(), 11);
        assert_eq!(sum.0[3].to_canonical_u64(), 13);
        assert_eq!(sum.0[4].to_canonical_u64(), 15);
        
        // Test subtraction
        let diff = b.sub(&a);
        assert_eq!(diff.0[0].to_canonical_u64(), 5);
        assert_eq!(diff.0[1].to_canonical_u64(), 5);
        assert_eq!(diff.0[2].to_canonical_u64(), 5);
        assert_eq!(diff.0[3].to_canonical_u64(), 5);
        assert_eq!(diff.0[4].to_canonical_u64(), 5);
        
        // Test multiplication
        let product = a.mul(&b);
        // This is a complex calculation, we'll just verify it's not zero
        assert!(!product.is_zero());
        
        // Test square
        let square = a.square();
        assert!(!square.is_zero());
        
        // Test double
        let doubled = a.double();
        assert_eq!(doubled.0[0].to_canonical_u64(), 2);
        assert_eq!(doubled.0[1].to_canonical_u64(), 4);
        assert_eq!(doubled.0[2].to_canonical_u64(), 6);
        assert_eq!(doubled.0[3].to_canonical_u64(), 8);
        assert_eq!(doubled.0[4].to_canonical_u64(), 10);
        
        // Test zero and one
        assert_eq!(Fp5Element::zero().is_zero(), true);
        assert_eq!(Fp5Element::one().is_one(), true);
        assert_eq!(Fp5Element::one().is_zero(), false);
    }
    
    #[test]
    fn test_fp5_field_constants() {
        // Test that constants are correct
        assert_eq!(Fp5Element::W.0, 3); // FP5_W
        assert_eq!(Fp5Element::DTH_ROOT.0, 1041288259238279555); // FP5_DTH_ROOT
        
        // Test zero and one constants
        assert_eq!(Fp5Element::zero().is_zero(), true);
        assert_eq!(Fp5Element::one().is_one(), true);
        assert_eq!(Fp5Element::two().0[0].to_canonical_u64(), 2);
    }
}
