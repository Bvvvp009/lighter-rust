use goldilocks_crypto::ScalarField;
use goldilocks_crypto::Point;
use num_bigint::BigUint;
use poseidon_hash::{Goldilocks, hash_to_quintic_extension, Fp5Element};

fn limbs_to_big(limbs: &[u64; 5]) -> BigUint {
    let mut bytes = Vec::with_capacity(40);
    for limb in limbs.iter() {
        bytes.extend_from_slice(&limb.to_le_bytes());
    }
    BigUint::from_bytes_le(&bytes)
}

fn big_to_limbs(n: &BigUint) -> [u64; 5] {
    let mut out = [0u64; 5];
    let bytes = n.to_bytes_le();
    for (i, chunk) in bytes.chunks(8).enumerate().take(5) {
        let mut limb_bytes = [0u8; 8];
        limb_bytes[..chunk.len()].copy_from_slice(chunk);
        out[i] = u64::from_le_bytes(limb_bytes);
    }
    out
}

#[test]
fn signature_equation_holds_for_random_nonces() {
    // This test asserts the Schnorr algebra: s = k - e*sk (mod N)
    // and that reconstructing R via s*G + e*P matches k*G.
    let generator = Point::generator();

    // Fixed message
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let hashed = hash_to_quintic_extension(&data);
    let message = hashed.to_bytes_le();
    let message_fp5 = Fp5Element::from_bytes_le(&message).expect("message fp5");

    // Precompute modulus as BigUint
    let n_big = limbs_to_big(&ScalarField::N.0);

    for _ in 0..200 {
        let sk = ScalarField::sample_crypto();
        let public_point = generator.mul(&sk);

        let k = ScalarField::sample_crypto();

        // R = k*G
        let r_point = generator.mul(&k);
        let r_encoded = r_point.encode();

        // e = H(R || m)
        let mut pre_image = [Goldilocks::zero(); 10];
        pre_image[..5].copy_from_slice(&r_encoded.0);
        pre_image[5..].copy_from_slice(&message_fp5.0);
        let e = ScalarField::from_fp5_element(&hash_to_quintic_extension(&pre_image));

        // s = k - e*sk (mod N)
        let s = k.sub(e.mul(&sk));

        // Reference using BigUint
        let k_big = limbs_to_big(&k.0);
        let e_big = limbs_to_big(&e.0);
        let sk_big = limbs_to_big(&sk.0);
        let e_sk_big = (&e_big * &sk_big) % &n_big;
        let s_big = (&k_big + &n_big - e_sk_big) % &n_big;
        let s_big_limbs = big_to_limbs(&s_big);

        // Check s matches reference
        assert_eq!(s.0, s_big_limbs, "s mismatch vs BigUint ref");

        // Check algebra: s + e*sk == k (mod N)
        let lhs = s.add(e.mul(&sk));
        let lhs_big = (limbs_to_big(&s.0) + (&e_big * &sk_big)) % &n_big;
        let lhs_big_limbs = big_to_limbs(&lhs_big);
        assert_eq!(lhs.0, lhs_big_limbs, "lhs mismatch vs BigUint");
        assert!(lhs.equals(&k), "s + e*sk != k: lhs={:?}, k={:?}", lhs.to_bytes_le(), k.to_bytes_le());

        // Check points: s*G + e*P == k*G
        let r_reconstructed = generator.mul(&s).add(&public_point.mul(&e));
        assert_eq!(r_reconstructed.encode().to_bytes_le(), r_encoded.to_bytes_le(), "R mismatch");
    }
}

#[test]
fn scalar_mul_and_sub_match_biguint() {
    use rand::{rngs::StdRng, RngCore, SeedableRng};

    let mut rng = StdRng::seed_from_u64(12345);
    let n_big = limbs_to_big(&ScalarField::N.0);

    for _ in 0..200 {
        // random canonical scalars
        let a = ScalarField::sample_crypto();
        let b = ScalarField::sample_crypto();

        // mul check
        let prod = a.mul(&b);
        let prod_big = (limbs_to_big(&a.0) * limbs_to_big(&b.0)) % &n_big;
        let prod_big_limbs = big_to_limbs(&prod_big);
        assert_eq!(prod.0, prod_big_limbs, "mul mismatch: a={:?} b={:?}", a.to_bytes_le(), b.to_bytes_le());

        // sub check (add extra modulus to avoid underflow in BigUint subtraction)
        let sub = a.sub(b);
        let sub_big = (limbs_to_big(&a.0) + &n_big + &n_big - limbs_to_big(&b.0)) % &n_big;
        let sub_big_limbs = big_to_limbs(&sub_big);
        assert_eq!(sub.0, sub_big_limbs, "sub mismatch: a={:?} b={:?}", a.to_bytes_le(), b.to_bytes_le());
    }
}
