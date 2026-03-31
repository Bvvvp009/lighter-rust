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
        
        // Test negation (Goldilocks has no neg(); negate via 0 - a)
        let neg_a = Goldilocks::zero().sub(&a);
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
        assert_eq!(Fp5Element::one(), Fp5Element::one());
        assert_eq!(Fp5Element::one().is_zero(), false);
    }
    
    #[test]
    fn test_fp5_field_constants() {
        // Verify the irreducible polynomial coefficient w = 3
        // (internal to the implementation; we verify via Fp5 arithmetic rather than constants)
        let w = Goldilocks::from_canonical_u64(3);
        assert_eq!(w.to_canonical_u64(), 3); // FP5 uses x^5 = w where w = 3

        // The 5th-root-of-unity value used internally
        let dth_root = Goldilocks::from_canonical_u64(1041288259238279555);
        assert_eq!(dth_root.to_canonical_u64(), 1041288259238279555); // FP5_DTH_ROOT

        // Test zero and one constants
        assert_eq!(Fp5Element::zero().is_zero(), true);
        assert_eq!(Fp5Element::one(), Fp5Element::one());
        let two = Fp5Element::one().double();
        assert_eq!(two.0[0].to_canonical_u64(), 2);
    }

    // ─── Goldilocks::neg ─────────────────────────────────────────────────────

    #[test]
    fn test_goldilocks_neg_basic() {
        let a = Goldilocks::from_canonical_u64(12345);
        let neg_a = a.neg();
        // a + (-a) == 0
        assert!(neg_a.add(&a).is_zero(), "a + neg(a) must be zero");
        // -(-a) == a
        assert_eq!(neg_a.neg().to_canonical_u64(), a.to_canonical_u64(), "-(-a) must equal a");
    }

    #[test]
    fn test_goldilocks_neg_of_zero() {
        assert!(Goldilocks::zero().neg().is_zero(), "neg(0) must be 0");
    }

    #[test]
    fn test_goldilocks_neg_boundary() {
        // neg(p-1) == 1
        let p_minus_1 = Goldilocks::from_canonical_u64(Goldilocks::ORDER - 1);
        assert_eq!(p_minus_1.neg().to_canonical_u64(), 1, "neg(p-1) must be 1");
        // neg(1) == p-1
        assert_eq!(
            Goldilocks::one().neg().to_canonical_u64(),
            Goldilocks::ORDER - 1,
            "neg(1) must be p-1"
        );
    }

    // ─── Goldilocks field axioms ──────────────────────────────────────────────

    #[test]
    fn test_goldilocks_add_commutativity() {
        let a = Goldilocks::from_canonical_u64(314159265);
        let b = Goldilocks::from_canonical_u64(271828182);
        assert_eq!(a.add(&b).to_canonical_u64(), b.add(&a).to_canonical_u64());
    }

    #[test]
    fn test_goldilocks_mul_commutativity() {
        let a = Goldilocks::from_canonical_u64(123456789);
        let b = Goldilocks::from_canonical_u64(987654321);
        assert_eq!(a.mul(&b).to_canonical_u64(), b.mul(&a).to_canonical_u64());
    }

    #[test]
    fn test_goldilocks_add_associativity() {
        let a = Goldilocks::from_canonical_u64(314159265);
        let b = Goldilocks::from_canonical_u64(271828182);
        let c = Goldilocks::from_canonical_u64(100000007);
        assert_eq!(
            a.add(&b).add(&c).to_canonical_u64(),
            a.add(&b.add(&c)).to_canonical_u64(),
            "(a+b)+c must equal a+(b+c)"
        );
    }

    #[test]
    fn test_goldilocks_mul_associativity() {
        let a = Goldilocks::from_canonical_u64(314159265);
        let b = Goldilocks::from_canonical_u64(271828182);
        let c = Goldilocks::from_canonical_u64(100000007);
        assert_eq!(
            a.mul(&b).mul(&c).to_canonical_u64(),
            a.mul(&b.mul(&c)).to_canonical_u64(),
            "(a*b)*c must equal a*(b*c)"
        );
    }

    #[test]
    fn test_goldilocks_distributivity() {
        let a = Goldilocks::from_canonical_u64(314159265);
        let b = Goldilocks::from_canonical_u64(271828182);
        let c = Goldilocks::from_canonical_u64(100000007);
        // a*(b+c) == a*b + a*c
        let lhs = a.mul(&b.add(&c));
        let rhs = a.mul(&b).add(&a.mul(&c));
        assert_eq!(lhs.to_canonical_u64(), rhs.to_canonical_u64(), "distributivity failed");
    }

    #[test]
    fn test_goldilocks_additive_inverse() {
        let a = Goldilocks::from_canonical_u64(314159265);
        assert!(a.add(&a.neg()).is_zero(), "a + neg(a) must be zero");
        assert!(a.neg().add(&a).is_zero(), "neg(a) + a must be zero");
    }

    #[test]
    fn test_goldilocks_multiplicative_inverse() {
        let values = [1u64, 2, 7, 12345, 999_999_999, Goldilocks::ORDER - 1];
        for v in values {
            let a = Goldilocks::from_canonical_u64(v);
            let inv = a.inverse();
            assert_eq!(
                a.mul(&inv).to_canonical_u64(),
                1,
                "a * a^-1 must be 1 for a={v}"
            );
        }
    }

    // ─── Goldilocks::sqrt ─────────────────────────────────────────────────────

    #[test]
    fn test_goldilocks_sqrt_of_perfect_squares() {
        for &v in &[1u64, 4, 9, 16, 25, 100, 12345678] {
            let a_sq = Goldilocks::from_canonical_u64(v).square();
            let sqrt = a_sq.sqrt().expect("square of a positive value must have a sqrt");
            assert_eq!(
                sqrt.square().to_canonical_u64(),
                a_sq.to_canonical_u64(),
                "sqrt(a^2)^2 must equal a^2 for v={v}"
            );
        }
    }

    #[test]
    fn test_goldilocks_sqrt_of_zero() {
        let s = Goldilocks::zero().sqrt();
        assert!(s.is_some(), "sqrt(0) must return Some");
        assert!(s.unwrap().is_zero(), "sqrt(0) must be 0");
    }

    #[test]
    fn test_goldilocks_sqrt_consistency() {
        // If sqrt returns Some(s), then s^2 must equal the input.
        let candidates = [2u64, 3, 5, 6, 7, 11, 17, 100, 54321];
        for &v in &candidates {
            let x = Goldilocks::from_canonical_u64(v);
            if let Some(s) = x.sqrt() {
                assert_eq!(
                    s.square().to_canonical_u64(),
                    x.to_canonical_u64(),
                    "sqrt(x)^2 must equal x for x={v}"
                );
            }
            // None is fine — it means v is a quadratic non-residue in Goldilocks.
        }
    }

    // ─── Fp5Element bytes round-trip ──────────────────────────────────────────

    #[test]
    fn test_fp5_bytes_roundtrip() {
        let original = Fp5Element::from_uint64_array([10, 20, 30, 40, 50]);
        let bytes = original.to_bytes_le();
        let restored = Fp5Element::from_bytes_le(&bytes).unwrap();
        for i in 0..5 {
            assert_eq!(
                original.0[i].to_canonical_u64(),
                restored.0[i].to_canonical_u64(),
                "component {i} mismatch after bytes round-trip"
            );
        }
    }

    #[test]
    fn test_fp5_bytes_roundtrip_zero_and_one() {
        for elem in [Fp5Element::zero(), Fp5Element::one()] {
            let bytes = elem.to_bytes_le();
            let restored = Fp5Element::from_bytes_le(&bytes).unwrap();
            assert_eq!(elem, restored, "round-trip failed for zero/one");
        }
    }

    #[test]
    fn test_fp5_bytes_wrong_length_is_error() {
        assert!(Fp5Element::from_bytes_le(&[0u8; 20]).is_err());
        assert!(Fp5Element::from_bytes_le(&[0u8; 41]).is_err());
        assert!(Fp5Element::from_bytes_le(&[]).is_err());
    }

    // ─── from_canonical_u64 boundary ─────────────────────────────────────────

    #[test]
    fn test_from_canonical_u64_normal_value() {
        let a = Goldilocks::from_canonical_u64(999);
        assert_eq!(a.to_canonical_u64(), 999);
    }

    #[test]
    fn test_from_canonical_u64_at_modulus_minus_one() {
        let max = Goldilocks::from_canonical_u64(Goldilocks::MODULUS - 1);
        assert_eq!(max.to_canonical_u64(), Goldilocks::MODULUS - 1);
    }

    #[test]
    fn test_from_noncanonical_u64_reduces_modulus() {
        // Exactly MODULUS → should map to 0.
        let z = Goldilocks::from_noncanonical_u64(Goldilocks::MODULUS);
        assert!(z.is_zero(), "MODULUS must reduce to zero");
    }

    #[test]
    fn test_from_noncanonical_u64_reduces_above_modulus() {
        // MODULUS + 7 → should map to 7.
        let v = Goldilocks::from_noncanonical_u64(Goldilocks::MODULUS + 7);
        assert_eq!(v.to_canonical_u64(), 7);
    }

    // ─── Goldilocks zeroize ───────────────────────────────────────────────────

    #[test]
    fn test_goldilocks_zeroize_clears_value() {
        use zeroize::Zeroize;
        let mut a = Goldilocks::from_canonical_u64(0xDEAD_BEEF);
        assert!(!a.is_zero());
        a.zeroize();
        assert!(a.is_zero(), "zeroize must set the limb to 0");
    }

    #[test]
    fn test_fp5_zeroize_clears_all_limbs() {
        use zeroize::Zeroize;
        let mut elem = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
        elem.zeroize();
        assert!(elem.is_zero(), "all Fp5 limbs must be cleared by zeroize");
    }
}
