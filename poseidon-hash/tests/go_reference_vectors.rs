use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

// Generated from the local Go reference implementation in `../poseidon_crypto`
// using `HashToQuinticExtension([]GoldilocksField{1,2,3,4,5,6,7,8})`.
const GO_POSEIDON_1_TO_8_HEX: &str =
    "3d34119729533099cf1859ace0140479a30ca4cb077efcd37fe6ca00d942b76b5e2db386fd348ae7";

#[test]
fn hash_to_quintic_extension_matches_go_reference_vector_for_1_to_8() {
    let input: Vec<Goldilocks> = (1u64..=8).map(Goldilocks::from_canonical_u64).collect();
    let got = hex::encode(hash_to_quintic_extension(&input).to_bytes_le());
    assert_eq!(
        got, GO_POSEIDON_1_TO_8_HEX,
        "Poseidon2 output drifted from the Go reference vector for inputs 1..=8"
    );
}
