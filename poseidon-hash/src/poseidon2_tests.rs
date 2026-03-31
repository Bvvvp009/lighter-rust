#[cfg(test)]
mod tests {
    use crate::{Goldilocks, Fp5Element, hash_to_quintic_extension, hash_no_pad, hash_n_to_one, empty_hash_out};

    // ─── Determinism / Known-Answer Tests ────────────────────────────────────────

    #[test]
    fn hash_to_quintic_extension_is_deterministic() {
        let input: Vec<Goldilocks> = (1u64..=8)
            .map(Goldilocks::from_canonical_u64)
            .collect();
        let a = hash_to_quintic_extension(&input);
        let b = hash_to_quintic_extension(&input);
        assert_eq!(a.to_bytes_le(), b.to_bytes_le(), "same input must always give same hash");
    }

    #[test]
    fn hash_no_pad_is_deterministic() {
        let input: Vec<Goldilocks> = (10u64..=15)
            .map(Goldilocks::from_canonical_u64)
            .collect();
        let a = hash_no_pad(&input);
        let b = hash_no_pad(&input);
        assert_eq!(a, b, "hash_no_pad must be deterministic");
    }

    #[test]
    fn hash_n_to_one_is_deterministic() {
        let a = hash_no_pad(&[Goldilocks::from_canonical_u64(1), Goldilocks::from_canonical_u64(2)]);
        let b = hash_no_pad(&[Goldilocks::from_canonical_u64(3), Goldilocks::from_canonical_u64(4)]);
        let combined1 = hash_n_to_one(&[a, b]);
        let combined2 = hash_n_to_one(&[a, b]);
        assert_eq!(combined1, combined2, "hash_n_to_one must be deterministic");
    }

    // ─── Avalanche / Collision-resistance ────────────────────────────────────────

    #[test]
    fn single_bit_flip_changes_hash_to_quintic_extension() {
        let base: Vec<Goldilocks> = (1u64..=5).map(Goldilocks::from_canonical_u64).collect();
        let hash_base = hash_to_quintic_extension(&base);

        let mut modified = base.clone();
        modified[0] = Goldilocks::from_canonical_u64(2);          // flip first element
        let hash_mod = hash_to_quintic_extension(&modified);

        assert_ne!(
            hash_base.to_bytes_le(), hash_mod.to_bytes_le(),
            "a one-element change must alter the hash output"
        );
    }

    #[test]
    fn single_bit_flip_changes_hash_no_pad() {
        let base = vec![Goldilocks::from_canonical_u64(0xDEAD_BEEF_u64)];
        let alt  = vec![Goldilocks::from_canonical_u64(0xDEAD_BEEE_u64)];
        assert_ne!(hash_no_pad(&base), hash_no_pad(&alt));
    }

    #[test]
    fn different_lengths_produce_different_hashes() {
        let input: Vec<Goldilocks> = (0u64..8).map(Goldilocks::from_canonical_u64).collect();
        let h5 = hash_to_quintic_extension(&input[..5]);
        let h8 = hash_to_quintic_extension(&input[..8]);
        assert_ne!(
            h5.to_bytes_le(), h8.to_bytes_le(),
            "inputs of different lengths must yield different hashes"
        );
    }

    #[test]
    fn permutation_order_changes_hash() {
        let a = Goldilocks::from_canonical_u64(1);
        let b = Goldilocks::from_canonical_u64(2);
        let fwd = hash_to_quintic_extension(&[a, b]);
        let rev = hash_to_quintic_extension(&[b, a]);
        assert_ne!(fwd.to_bytes_le(), rev.to_bytes_le(), "order of inputs must matter");
    }

    // ─── Field edge-case handling ─────────────────────────────────────────────────

    #[test]
    fn hash_of_all_zeros_is_not_zero() {
        let zeros = vec![Goldilocks::zero(); 8];
        let result = hash_to_quintic_extension(&zeros);
        assert!(!result.is_zero(), "hash of all-zero input should not be zero");
    }

    #[test]
    fn hash_of_all_max_values_does_not_panic() {
        let max_val = Goldilocks::from_canonical_u64(Goldilocks::ORDER - 1);
        let input = vec![max_val; 8];
        let _ = hash_to_quintic_extension(&input); // must not panic or overflow
    }

    #[test]
    fn hash_of_single_element_does_not_panic() {
        let _ = hash_to_quintic_extension(&[Goldilocks::one()]);
    }

    #[test]
    fn hash_of_large_input_does_not_panic() {
        let input: Vec<Goldilocks> = (0u64..256).map(Goldilocks::from_canonical_u64).collect();
        let _ = hash_to_quintic_extension(&input);
    }

    // ─── hash_no_pad / hash_n_to_one consistency ─────────────────────────────────

    #[test]
    fn empty_hash_is_stable() {
        let h = empty_hash_out();
        // Calling again should return the same value
        let h2 = empty_hash_out();
        assert_eq!(h, h2);
    }

    #[test]
    fn hash_n_to_one_single_child_equals_idempotent() {
        let child = hash_no_pad(&[
            Goldilocks::from_canonical_u64(42),
            Goldilocks::from_canonical_u64(99),
        ]);
        // Combining one child with empty_hash_out should differ from the child alone
        let empty = empty_hash_out();
        let combined = hash_n_to_one(&[child, empty]);
        // They should not be equal (hashing changes the value)
        assert_ne!(combined, child, "hash_n_to_one must actually mix inputs");
    }

    #[test]
    fn hash_n_to_one_is_non_commutative() {
        let a = hash_no_pad(&[Goldilocks::from_canonical_u64(1)]);
        let b = hash_no_pad(&[Goldilocks::from_canonical_u64(2)]);
        let ab = hash_n_to_one(&[a, b]);
        let ba = hash_n_to_one(&[b, a]);
        assert_ne!(ab, ba, "hash_n_to_one must be order-sensitive");
    }

    // ─── Fp5 inverse and field axioms ────────────────────────────────────────────

    #[test]
    fn fp5_neg_double_is_identity() {
        let x = Fp5Element::from_uint64_array([3, 1, 4, 1, 5]);
        let nn = x.neg().neg();
        assert_eq!(
            nn.0.iter().map(|g| g.to_canonical_u64()).collect::<Vec<_>>(),
            x.0.iter().map(|g| g.to_canonical_u64()).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn fp5_sub_self_is_zero() {
        let x = Fp5Element::from_uint64_array([7, 6, 5, 4, 3]);
        let zero = x.sub(&x);
        assert!(zero.is_zero(), "x - x must equal zero");
    }

    #[test]
    fn fp5_mul_inverse_is_one_for_identity() {
        // The simplest case: one.inverse() == one, and one * one == one
        let one = Fp5Element::one();
        let inv = one.inverse();
        assert_eq!(inv, one, "one.inverse() must equal one");
        assert_eq!(one.mul(&inv), one, "one * one.inverse() must equal one");
    }

    #[test]
    fn fp5_mul_inverse_is_one_for_base_elements() {
        // Base field elements [a,0,0,0,0]: inverse must satisfy x*x^{-1}=1
        // Previously failed because Goldilocks::mul could produce non-canonical raw values
        // (e.g. p+1 instead of 1) that compared unequal under the old derived PartialEq.
        for a in [2u64, 3, 7, 100, 1_000_000, u32::MAX as u64] {
            let x = Fp5Element::from_uint64_array([a, 0, 0, 0, 0]);
            let inv = x.inverse();
            let product = x.mul(&inv);
            assert_eq!(
                product,
                Fp5Element::one(),
                "inverse failed for a={a}: product canonical = {:?}",
                product.0.iter().map(|g| g.to_canonical_u64()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn fp5_inverse_of_zero_is_zero() {
        // inverse_or_zero: zero input => zero output
        let z = Fp5Element::zero();
        assert!(z.inverse().is_zero(), "zero.inverse() must return zero");
    }

    #[test]
    fn fp5_add_is_commutative() {
        let a = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
        let b = Fp5Element::from_uint64_array([9, 8, 7, 6, 5]);
        let ab = a.add(&b);
        let ba = b.add(&a);
        for i in 0..5 {
            assert_eq!(ab.0[i].to_canonical_u64(), ba.0[i].to_canonical_u64());
        }
    }

    #[test]
    fn fp5_mul_is_commutative() {
        let a = Fp5Element::from_uint64_array([3, 1, 4, 0, 0]);
        let b = Fp5Element::from_uint64_array([1, 5, 9, 0, 0]);
        let ab = a.mul(&b);
        let ba = b.mul(&a);
        for i in 0..5 {
            assert_eq!(ab.0[i].to_canonical_u64(), ba.0[i].to_canonical_u64());
        }
    }
}
