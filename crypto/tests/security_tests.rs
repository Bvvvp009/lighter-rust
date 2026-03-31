/// Security-focused integration tests for goldilocks-crypto.
///
/// These tests verify that the cryptographic primitives resist common attacks
/// and reject malformed inputs correctly. They are *not* a substitute for a
/// formal audit, but provide a measurable robustness baseline.
use goldilocks_crypto::{
    batch_verify, sign, sign_with_nonce, validate_public_key, verify_signature, Point, ScalarField,
};
use zeroize::Zeroize;

// ─── helpers ────────────────────────────────────────────────────────────────

fn random_keypair() -> ([u8; 40], [u8; 40]) {
    let sk = ScalarField::sample_crypto();
    let sk_bytes = sk.to_bytes_le();
    let pk_bytes = Point::generator().mul(&sk).encode().to_bytes_le();
    (sk_bytes, pk_bytes)
}

/// Deterministic small private key whose public key can be re-derived any time.
fn fixed_sk_bytes() -> [u8; 40] {
    let mut sk = [0u8; 40];
    sk[0] = 7; // small non-zero value
    sk
}

fn fixed_pk_bytes() -> [u8; 40] {
    let sk = ScalarField::from_bytes_le(&fixed_sk_bytes()).unwrap();
    Point::generator().mul(&sk).encode().to_bytes_le()
}

fn fixed_nonce() -> [u8; 40] {
    let mut n = [0u8; 40];
    n[0] = 0xDE;
    n[1] = 0xAD;
    n
}

// ─── 1. Basic correctness ────────────────────────────────────────────────────

#[test]
fn sign_verify_roundtrip_random_key() {
    let (sk, pk) = random_keypair();
    let msg = [0xABu8; 40];
    let sig = sign(&sk, &msg).unwrap();
    assert!(
        verify_signature(&sig, &msg, &pk).unwrap(),
        "valid signature must verify"
    );
}

#[test]
fn sign_verify_roundtrip_various_messages() {
    let (sk, pk) = random_keypair();
    // Only patterns whose 8-byte repetition is < Goldilocks MODULUS (0xFFFF_FFFF_0000_0001)
    // are valid canonical messages.  0xFF repeating = 0xFFFF_FFFF_FFFF_FFFF >= MODULUS and
    // must be rejected rather than silently accepted (avoids binding-property break).
    for pattern in [0x00u8, 0x01, 0x7F, 0x80] {
        let msg = [pattern; 40];
        let sig = sign(&sk, &msg).unwrap();
        assert!(
            verify_signature(&sig, &msg, &pk).unwrap(),
            "failed for message pattern 0x{:02x}",
            pattern
        );
    }
}

#[test]
fn sign_rejects_non_canonical_message_bytes() {
    // 0xFF repeated gives chunks of 0xFFFF_FFFF_FFFF_FFFF which is >= Goldilocks MODULUS.
    // Both sign() and verify_signature() must return Err, not Ok(false).
    let (sk, pk) = random_keypair();
    let non_canonical_msg = [0xFFu8; 40];
    assert!(
        sign(&sk, &non_canonical_msg).is_err(),
        "sign must reject non-canonical message bytes"
    );
    // Craft a fake sig to confirm verify also rejects the message.
    let valid_msg = [0x01u8; 40];
    let sig = sign(&sk, &valid_msg).unwrap();
    assert!(
        verify_signature(&sig, &non_canonical_msg, &pk).is_err(),
        "verify must reject non-canonical message bytes"
    );
}

#[test]
fn sign_verify_with_fixed_key() {
    let sk = fixed_sk_bytes();
    let pk = fixed_pk_bytes();
    let msg = [0x11u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    assert!(verify_signature(&sig, &msg, &pk).unwrap());
}

// ─── 2. Signature-integrity checks (attack resistance) ──────────────────────

#[test]
fn tampered_s_component_rejected() {
    let (sk, pk) = random_keypair();
    let msg = [0u8; 40];
    let mut sig = sign(&sk, &msg).unwrap();
    sig[0] ^= 0xFF; // corrupt first byte of s
    assert!(
        !verify_signature(&sig, &msg, &pk).unwrap(),
        "tampered s must not verify"
    );
}

#[test]
fn tampered_e_component_rejected() {
    let (sk, pk) = random_keypair();
    let msg = [0u8; 40];
    let mut sig = sign(&sk, &msg).unwrap();
    sig[40] ^= 0xFF; // corrupt first byte of e
    assert!(
        !verify_signature(&sig, &msg, &pk).unwrap(),
        "tampered e must not verify"
    );
}

#[test]
fn all_zero_signature_rejected() {
    let (_, pk) = random_keypair();
    let msg = [0u8; 40];
    let sig = [0u8; 80];
    // Should either return Ok(false) or Err — either is acceptable, not Ok(true)
    let result = verify_signature(&sig, &msg, &pk);
    assert!(
        result.map(|v| !v).unwrap_or(true),
        "all-zero signature must not verify"
    );
}

#[test]
fn modified_message_rejected() {
    let (sk, pk) = random_keypair();
    let msg = [0x22u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    let mut bad_msg = msg;
    bad_msg[20] ^= 0x01;
    assert!(
        !verify_signature(&sig, &bad_msg, &pk).unwrap(),
        "signature over different message must not verify"
    );
}

#[test]
fn wrong_public_key_rejected() {
    let (sk, _right_pk) = random_keypair();
    let (_, wrong_pk) = random_keypair();
    let msg = [0x33u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    assert!(
        !verify_signature(&sig, &msg, &wrong_pk).unwrap(),
        "signature must not verify under wrong public key"
    );
}

// ─── 3. Input length validation ──────────────────────────────────────────────

#[test]
fn sign_rejects_short_private_key() {
    let msg = [0u8; 40];
    assert!(sign(&[0u8; 20], &msg).is_err(), "20-byte key must be rejected");
}

#[test]
fn sign_rejects_long_private_key() {
    let msg = [0u8; 40];
    assert!(sign(&[0u8; 41], &msg).is_err(), "41-byte key must be rejected");
}

#[test]
fn sign_rejects_empty_private_key() {
    let msg = [0u8; 40];
    assert!(sign(&[], &msg).is_err(), "empty key must be rejected");
}

#[test]
fn verify_rejects_short_signature() {
    let (_, pk) = random_keypair();
    let msg = [0u8; 40];
    assert!(
        verify_signature(&[0u8; 40], &msg, &pk).is_err(),
        "40-byte sig must be rejected"
    );
}

#[test]
fn verify_rejects_empty_signature() {
    let (_, pk) = random_keypair();
    let msg = [0u8; 40];
    assert!(verify_signature(&[], &msg, &pk).is_err());
}

#[test]
fn verify_rejects_short_message() {
    let (sk, pk) = random_keypair();
    let msg = [0u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    assert!(
        verify_signature(&sig, &msg[..20], &pk).is_err(),
        "20-byte message must be rejected"
    );
}

#[test]
fn verify_rejects_empty_message() {
    let (sk, pk) = random_keypair();
    let msg = [0u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    assert!(verify_signature(&sig, &[], &pk).is_err());
}

#[test]
fn scalar_from_bytes_rejects_wrong_length() {
    assert!(ScalarField::from_bytes_le(&[0u8; 20]).is_err());
    assert!(ScalarField::from_bytes_le(&[0u8; 41]).is_err());
    assert!(ScalarField::from_bytes_le(&[]).is_err());
}

// ─── 4. Determinism and uniqueness ──────────────────────────────────────────

#[test]
fn sign_with_nonce_is_deterministic() {
    let sk = fixed_sk_bytes();
    let msg = [0x55u8; 40];
    let nonce = fixed_nonce();
    let sig1 = sign_with_nonce(&sk, &msg, &nonce).unwrap();
    let sig2 = sign_with_nonce(&sk, &msg, &nonce).unwrap();
    assert_eq!(sig1, sig2, "identical inputs must produce identical signatures");
}

#[test]
fn different_nonces_produce_different_signatures() {
    let sk = fixed_sk_bytes();
    let pk = fixed_pk_bytes();
    let msg = [0u8; 40];
    let mut nonce_a = [0u8; 40];
    nonce_a[0] = 1;
    let mut nonce_b = [0u8; 40];
    nonce_b[0] = 2;

    let sig_a = sign_with_nonce(&sk, &msg, &nonce_a).unwrap();
    let sig_b = sign_with_nonce(&sk, &msg, &nonce_b).unwrap();

    assert_ne!(sig_a, sig_b, "different nonces must produce different signatures");
    // Both must still verify
    assert!(verify_signature(&sig_a, &msg, &pk).unwrap());
    assert!(verify_signature(&sig_b, &msg, &pk).unwrap());
}

#[test]
fn random_sign_produces_unique_signatures() {
    let sk = fixed_sk_bytes();
    let msg = [0u8; 40];
    let sigs: Vec<_> = (0..10).map(|_| sign(&sk, &msg).unwrap()).collect();
    for i in 0..sigs.len() {
        for j in (i + 1)..sigs.len() {
            assert_ne!(sigs[i], sigs[j], "random sigs [{i}] and [{j}] must differ");
        }
    }
}

// ─── 5. Batch verification ───────────────────────────────────────────────────

#[test]
fn batch_verify_all_valid() {
    let pairs: Vec<_> = (0u8..5).map(|i| {
        let (sk, pk) = random_keypair();
        let mut msg = [0u8; 40];
        msg[0] = i;
        let sig = sign(&sk, &msg).unwrap();
        (sig, msg, pk)
    }).collect();

    let sigs: Vec<Vec<u8>> = pairs.iter().map(|(s, _, _)| s.clone()).collect();
    let msgs: Vec<[u8; 40]> = pairs.iter().map(|(_, m, _)| *m).collect();
    let pks: Vec<[u8; 40]> = pairs.iter().map(|(_, _, p)| *p).collect();

    assert!(batch_verify(&sigs, &msgs, &pks).unwrap());
}

#[test]
fn batch_verify_one_bad_sig_fails() {
    let (sk, pk) = random_keypair();
    let msg = [0u8; 40];
    let mut sig = sign(&sk, &msg).unwrap();
    sig[5] ^= 0xFF; // corrupt one byte
    assert!(!batch_verify(&[sig], &[msg], &[pk]).unwrap());
}

#[test]
fn batch_verify_mismatched_lengths_error() {
    let (sk, _pk) = random_keypair();
    let msg = [0u8; 40];
    let sig = sign(&sk, &msg).unwrap();
    let (_, pk2) = random_keypair();
    // 1 sig, 1 msg, 2 public keys — must error
    assert!(batch_verify(&[sig], &[msg], &[_pk, pk2]).is_err());
}

// ─── 6. Public-key validation ────────────────────────────────────────────────

#[test]
fn valid_public_key_accepted() {
    let pk = fixed_pk_bytes();
    assert!(validate_public_key(&pk).is_ok());
}

#[test]
fn all_zero_public_key_rejected() {
    // The zero Fp5 element cannot be decoded as a valid curve point
    let result = validate_public_key(&[0u8; 40]);
    assert!(result.is_err(), "zero public key must be rejected");
}

#[test]
fn random_bytes_public_key_rejected_or_accepted_gracefully() {
    // Random bytes might or might not decode to a valid point, but must not panic
    let garbage = [0xDEu8; 40];
    let _ = validate_public_key(&garbage); // must not panic
}

// ─── 7. Scalar field arithmetic sanity ──────────────────────────────────────

#[test]
fn scalar_add_is_commutative() {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    assert_eq!(a.add(b).to_bytes_le(), b.add(a).to_bytes_le());
}

#[test]
fn scalar_mul_identity() {
    let a = ScalarField::sample_crypto();
    let result = a.mul(&ScalarField::ONE);
    assert_eq!(a.to_bytes_le(), result.to_bytes_le());
}

#[test]
fn scalar_add_zero_identity() {
    let a = ScalarField::sample_crypto();
    let result = a.add(ScalarField::ZERO);
    assert_eq!(a.to_bytes_le(), result.to_bytes_le());
}

#[test]
fn scalar_sub_self_is_zero() {
    let a = ScalarField::sample_crypto();
    let result = a.sub(a);
    assert_eq!(result.to_bytes_le(), ScalarField::ZERO.to_bytes_le());
}

#[test]
fn scalar_neg_double_is_identity() {
    let a = ScalarField::sample_crypto();
    let neg_neg_a = a.neg().neg();
    assert_eq!(a.to_bytes_le(), neg_neg_a.to_bytes_le());
}

// ─── 8. ScalarField — is_canonical ──────────────────────────────────────────

#[test]
fn scalar_fresh_sample_is_canonical() {
    // Every scalar coming from the public API must be < N.
    for _ in 0..20 {
        let s = ScalarField::sample_crypto();
        assert!(s.is_canonical(), "sample_crypto must always return a canonical scalar");
    }
}

#[test]
fn scalar_constants_are_canonical() {
    assert!(ScalarField::ZERO.is_canonical(), "ZERO must be canonical");
    assert!(ScalarField::ONE.is_canonical(), "ONE must be canonical");
    assert!(ScalarField::TWO.is_canonical(), "TWO must be canonical");
}

#[test]
fn scalar_n_is_not_canonical() {
    // N is the modulus — it is not a valid element in [0, N).
    assert!(
        !ScalarField::N.is_canonical(),
        "N itself must NOT be canonical (it is the modulus)"
    );
}

#[test]
fn scalar_arithmetic_results_are_canonical() {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    assert!(a.add(b).is_canonical(),  "add result must be canonical");
    assert!(a.sub(b).is_canonical(),  "sub result must be canonical");
    assert!(a.mul(&b).is_canonical(), "mul result must be canonical");
    assert!(a.neg().is_canonical(),   "neg result must be canonical");
}

// ─── 9. ScalarField — inverse ────────────────────────────────────────────────

#[test]
fn scalar_inverse_times_self_is_one() {
    for _ in 0..5 {
        let a = ScalarField::sample_crypto();
        let inv = a.inverse().expect("non-zero scalar must have an inverse");
        let product = a.mul(&inv);
        assert_eq!(
            product.to_bytes_le(),
            ScalarField::ONE.to_bytes_le(),
            "a * a⁻¹ must equal ONE"
        );
    }
}

#[test]
fn scalar_inverse_of_one_is_one() {
    let inv = ScalarField::ONE.inverse().unwrap();
    assert_eq!(inv.to_bytes_le(), ScalarField::ONE.to_bytes_le(), "1⁻¹ must be 1");
}

#[test]
fn scalar_inverse_of_two() {
    // 2 * inv(2) == 1
    let inv2 = ScalarField::TWO.inverse().unwrap();
    let product = ScalarField::TWO.mul(&inv2);
    assert_eq!(product.to_bytes_le(), ScalarField::ONE.to_bytes_le(), "2 * 2⁻¹ must be 1");
}

#[test]
fn scalar_inverse_of_zero_is_none() {
    assert!(ScalarField::ZERO.inverse().is_none(), "zero has no inverse");
}

#[test]
fn scalar_inverse_is_canonical() {
    let a = ScalarField::sample_crypto();
    let inv = a.inverse().unwrap();
    assert!(inv.is_canonical(), "inverse must be canonical");
}

// ─── 10. ScalarField — hex and bytes round-trips ─────────────────────────────

#[test]
fn scalar_hex_roundtrip() {
    let original = ScalarField::sample_crypto();
    let hex = original.to_hex();
    assert_eq!(hex.len(), 80, "hex string must be exactly 80 chars");
    let restored = ScalarField::from_hex(&hex).unwrap();
    assert_eq!(original.to_bytes_le(), restored.to_bytes_le(), "hex round-trip failed");
}

#[test]
fn scalar_hex_roundtrip_with_0x_prefix() {
    let original = ScalarField::sample_crypto();
    let hex = format!("0x{}", original.to_hex());
    let restored = ScalarField::from_hex(&hex).unwrap();
    assert_eq!(original.to_bytes_le(), restored.to_bytes_le(), "0x-prefixed hex round-trip failed");
}

#[test]
fn scalar_bytes_roundtrip() {
    let original = ScalarField::sample_crypto();
    let bytes = original.to_bytes_le();
    let restored = ScalarField::from_bytes_le(&bytes).unwrap();
    assert_eq!(original.to_bytes_le(), restored.to_bytes_le(), "bytes round-trip failed");
}

#[test]
fn scalar_from_hex_wrong_length_is_error() {
    // 78 hex chars (too short by 2)
    assert!(ScalarField::from_hex(&"a".repeat(78)).is_err());
    // 82 hex chars (too long by 2)
    assert!(ScalarField::from_hex(&"a".repeat(82)).is_err());
}

// ─── 11. ScalarField — distributivity ───────────────────────────────────────

#[test]
fn scalar_distributivity() {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    let c = ScalarField::sample_crypto();
    // a * (b + c) == a*b + a*c
    let lhs = a.mul(&b.add(c));
    let rhs = a.mul(&b).add(a.mul(&c));
    assert_eq!(lhs.to_bytes_le(), rhs.to_bytes_le(), "distributivity a*(b+c) == a*b+a*c failed");
}

#[test]
fn scalar_mul_commutative() {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    assert_eq!(
        a.mul(&b).to_bytes_le(),
        b.mul(&a).to_bytes_le(),
        "scalar multiplication must be commutative"
    );
}

// ─── 12. Point — equality ────────────────────────────────────────────────────

#[test]
fn point_generator_equals_itself() {
    let g = Point::generator();
    assert!(g == g, "generator must equal itself");
}

#[test]
fn point_neutral_equals_itself() {
    let n = Point::neutral();
    assert!(n == n, "neutral point must equal itself");
}

#[test]
fn point_g_and_2g_differ() {
    let g = Point::generator();
    let sk2 = ScalarField::TWO;
    let two_g = g.mul(&sk2);
    assert!(g != two_g, "G and 2·G must not be equal");
}

#[test]
fn point_scalar_mul_additive() {
    // (a+b)*G == a*G + b*G
    let g = Point::generator();
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    let ab_g = g.mul(&a.add(b));
    let ag_plus_bg = g.mul(&a).add(&g.mul(&b));
    assert!(ab_g == ag_plus_bg, "(a+b)*G must equal a*G + b*G");
}

// ─── 13. Production-hardening: zeroize ──────────────────────────────────────

#[test]
fn scalar_zeroize_clears_all_bytes() {
    let mut secret = ScalarField::sample_crypto();
    // Confirm it is non-zero before we wipe it.
    assert_ne!(secret.to_bytes_le(), [0u8; 40], "sample must be non-zero");
    secret.zeroize();
    assert_eq!(
        secret.to_bytes_le(),
        [0u8; 40],
        "zeroize must overwrite every byte with 0"
    );
}

#[test]
fn scalar_zeroize_multiple_times_is_idempotent() {
    let mut s = ScalarField::sample_crypto();
    s.zeroize();
    s.zeroize(); // must not panic or corrupt
    assert_eq!(s.to_bytes_le(), [0u8; 40]);
}

// ─── 14. Production-hardening: constant-time select ─────────────────────────

#[test]
fn select_zero_mask_returns_a0() {
    let a0 = ScalarField::ONE;
    let a1 = ScalarField::TWO;
    let result = ScalarField::select(0, &a0, &a1);
    assert_eq!(result, a0, "select(0, a0, a1) must return a0");
}

#[test]
fn select_full_mask_returns_a1() {
    let a0 = ScalarField::ONE;
    let a1 = ScalarField::TWO;
    let result = ScalarField::select(0xFFFF_FFFF_FFFF_FFFF, &a0, &a1);
    assert_eq!(result, a1, "select(all-ones, a0, a1) must return a1");
}

#[test]
fn select_produces_correct_values_for_many_pairs() {
    for _ in 0..20 {
        let a0 = ScalarField::sample_crypto();
        let a1 = ScalarField::sample_crypto();
        assert_eq!(ScalarField::select(0, &a0, &a1), a0);
        assert_eq!(ScalarField::select(0xFFFF_FFFF_FFFF_FFFF, &a0, &a1), a1);
    }
}

// ─── 15. Production-hardening: constant-time equality ───────────────────────

#[test]
fn scalar_ct_eq_reflexive() {
    let a = ScalarField::sample_crypto();
    assert_eq!(a, a, "a scalar must equal itself");
}

#[test]
fn scalar_ct_eq_symmetric() {
    let a = ScalarField::sample_crypto();
    let b = a; // Copy
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn scalar_ct_ne_different_values() {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    // Vanishingly unlikely to collide; treat as guaranteed different.
    // Even in the extremely improbable collision the test is vacuously OK.
    if a.to_bytes_le() != b.to_bytes_le() {
        assert_ne!(a, b, "two independently generated scalars must not be equal");
    }
}

#[test]
fn scalar_ct_eq_zero_constants() {
    assert_eq!(ScalarField::ZERO, ScalarField::ZERO);
    assert_ne!(ScalarField::ZERO, ScalarField::ONE);
    assert_ne!(ScalarField::ONE,  ScalarField::TWO);
}

// ─── 16. Production-hardening: inverse is None for zero, canonical for rest ──

#[test]
fn inverse_of_zero_is_none() {
    assert!(ScalarField::ZERO.inverse().is_none());
}

#[test]
fn inverse_canonical_for_100_samples() {
    for _ in 0..100 {
        let a = ScalarField::sample_crypto();
        let inv = a.inverse().expect("non-zero sample must have inverse");
        assert!(inv.is_canonical(), "inverse must be canonical");
        assert_eq!(a.mul(&inv), ScalarField::ONE, "a * a⁻¹ must be ONE");
    }
}

