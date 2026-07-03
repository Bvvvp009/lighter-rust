#![no_main]

use goldilocks_crypto::{validate_public_key, verify_signature, KeyPair, ScalarField, Signature};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let pk = data.get(0..40).unwrap_or(&[]);
    let sig = data.get(40..120).unwrap_or(&[]);
    let sk = data.get(120..160).unwrap_or(&[]);
    let nonce = data.get(160..200).unwrap_or(&[]);

    let mut message = [0u8; 40];
    let message_src = data.get(200..240).unwrap_or(&[]);
    message[..message_src.len().min(40)].copy_from_slice(&message_src[..message_src.len().min(40)]);

    let _ = validate_public_key(pk);
    let _ = verify_signature(sig, &message, pk);
    let _ = Signature::from_bytes(sig);

    if sk.len() == 40 {
        let _ = KeyPair::from_private_key_bytes(sk);
        let _ = ScalarField::from_canonical_bytes_le(sk);
        let _ = goldilocks_crypto::sign_hashed_message(sk, &message, nonce);
    }

    let hex_candidate = hex::encode(&data[..data.len().min(40)]);
    let _ = ScalarField::from_hex(&hex_candidate);
});
