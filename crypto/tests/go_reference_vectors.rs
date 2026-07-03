use goldilocks_crypto::{sign_hashed_message, verify_signature, KeyPair};

const PRIVATE_KEY_HEX: &str =
    "825ed9fde4a049e5eb4a0a31dd3cc53ac657e4e0171f44ae1224ad301f8e51af5c4bbcafa28e1b55";
const HASHED_MESSAGE_HEX: &str =
    "1f1507bc68e6328fdd4a5d205159851b97f95feb7630874366e6862275a2d4bf8bd7f41b65612a26";
const GO_PUBLIC_KEY_HEX: &str =
    "99f3473027655c41eebb21afd06b516b438b42ad70c27ac8208cdb56b60be7d5c9ddfb05e3cf9518";
const GO_SIGNATURE_HEX: &str =
    "c455748865b3b44b94d4e912a252989a638f140241c34e339ae9c8e27f29089ee73cda68277f5d3c6ca1ed461bd60f3a2e764eed9c1a0271eb3b1f21fc4d4fe6a08dc0b363eb546e467ed1c1dd509f1d";

#[test]
fn public_key_derivation_matches_go_reference_vector() {
    let private_key = hex::decode(PRIVATE_KEY_HEX).unwrap();
    let keypair = KeyPair::from_private_key_bytes(&private_key).unwrap();

    assert_eq!(
        hex::encode(keypair.public_key_bytes()),
        GO_PUBLIC_KEY_HEX,
        "public key derivation drifted from the Go reference implementation"
    );
}

#[test]
fn deterministic_signature_matches_go_reference_vector() {
    let private_key = hex::decode(PRIVATE_KEY_HEX).unwrap();
    let message = hex::decode(HASHED_MESSAGE_HEX).unwrap();
    let public_key = hex::decode(GO_PUBLIC_KEY_HEX).unwrap();

    let mut nonce = [0u8; 40];
    nonce[..8].copy_from_slice(&0x0123_4567_89ab_cdefu64.to_le_bytes());

    let signature = sign_hashed_message(&private_key, &message, &nonce).unwrap();

    assert_eq!(
        hex::encode(&signature),
        GO_SIGNATURE_HEX,
        "deterministic Schnorr signature drifted from the Go reference implementation"
    );
    assert!(
        verify_signature(&signature, &message, &public_key).unwrap(),
        "reference signature should still verify under the derived public key"
    );
}
