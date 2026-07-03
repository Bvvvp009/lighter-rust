use goldilocks_crypto::{validate_public_key, verify_signature};
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
/// SDK-level integration tests for the `signer` crate.
///
/// These exercise the full SDK call-path: key construction → hashing → signing → verification,
/// matching the exact flows used by `api-client` consumers.
use signer::KeyManager;

fn canonical_message(pattern: u8) -> [u8; 40] {
    let elements = [Goldilocks::from_canonical_u64(pattern as u64); 5];
    hash_to_quintic_extension(&elements).to_bytes_le()
}

// ─── 1. Key construction ─────────────────────────────────────────────────────

#[test]
fn key_manager_from_hex_roundtrip() {
    let km = KeyManager::generate();
    let pk = km.public_key_bytes();
    let sk_hex = hex::encode(km.private_key_bytes());
    let km2 = KeyManager::from_hex(&sk_hex).unwrap();
    assert_eq!(
        pk,
        km2.public_key_bytes(),
        "pk must match after hex roundtrip"
    );
}

#[test]
fn key_manager_from_hex_with_0x_prefix() {
    let km = KeyManager::generate();
    let sk_hex = format!("0x{}", hex::encode(km.private_key_bytes()));
    let km2 = KeyManager::from_hex(&sk_hex).unwrap();
    assert_eq!(km.public_key_bytes(), km2.public_key_bytes());
}

#[test]
fn key_manager_rejects_too_short_hex() {
    assert!(KeyManager::from_hex("deadbeef").is_err());
}

#[test]
fn key_manager_rejects_too_long_hex() {
    let long_hex = "aa".repeat(41); // 41 bytes = 82 hex chars
    assert!(KeyManager::from_hex(&long_hex).is_err());
}

#[test]
fn key_manager_rejects_invalid_hex_chars() {
    let bad_hex = "zz".repeat(40);
    assert!(KeyManager::from_hex(&bad_hex).is_err());
}

// ─── 2. Public key validity ───────────────────────────────────────────────────

#[test]
fn generated_public_key_is_valid_curve_point() {
    let km = KeyManager::generate();
    let pk = km.public_key_bytes();
    validate_public_key(&pk).expect("generated public key must be a valid curve point");
}

// ─── 3. Sign → verify round-trips ────────────────────────────────────────────

#[test]
fn sign_verify_sdk_roundtrip() {
    let km = KeyManager::generate();
    let pk = km.public_key_bytes();
    let msg = [0xABu8; 40];
    let sig = km.sign(&msg).unwrap();
    assert!(
        verify_signature(&sig, &msg, &pk).unwrap(),
        "SDK-signed message must verify"
    );
}

#[test]
fn sign_verify_various_message_patterns() {
    let km = KeyManager::generate();
    let pk = km.public_key_bytes();
    for pattern in [0x00u8, 0x01, 0x7F, 0x80, 0xFF] {
        let msg = canonical_message(pattern);
        let sig = km.sign(&msg).unwrap();
        assert!(
            verify_signature(&sig, &msg, &pk).unwrap(),
            "failed for pattern 0x{:02x}",
            pattern
        );
    }
}

#[test]
fn signature_is_80_bytes() {
    let km = KeyManager::generate();
    let sig = km.sign(&[0u8; 40]).unwrap();
    assert_eq!(sig.len(), 80);
}

#[test]
fn signatures_are_unique_per_call() {
    let km = KeyManager::generate();
    let msg = [0u8; 40];
    let sigs: Vec<_> = (0..10).map(|_| km.sign(&msg).unwrap()).collect();
    for i in 0..sigs.len() {
        for j in (i + 1)..sigs.len() {
            assert_ne!(
                sigs[i], sigs[j],
                "each sign() call must produce a unique sig"
            );
        }
    }
}

// ─── 4. Cross-key isolation ───────────────────────────────────────────────────

#[test]
fn sig_from_key_a_does_not_verify_under_key_b() {
    let km_a = KeyManager::generate();
    let km_b = KeyManager::generate();
    let pk_b = km_b.public_key_bytes();
    let msg = [0x42u8; 40];
    let sig = km_a.sign(&msg).unwrap();
    assert!(
        !verify_signature(&sig, &msg, &pk_b).unwrap(),
        "sig from key A must not verify under key B"
    );
}

// ─── 5. Auth token ────────────────────────────────────────────────────────────

#[test]
fn auth_token_has_correct_format() {
    let km = KeyManager::generate();
    let deadline = 1735689600i64;
    let account_index = 271i64;
    let api_key_index = 4u8;
    let token = km
        .create_auth_token(deadline, account_index, api_key_index)
        .unwrap();

    // Expected: "deadline:account:api_key_index:signature_hex"
    let parts: Vec<&str> = token.splitn(4, ':').collect();
    assert_eq!(
        parts.len(),
        4,
        "auth token must have 4 colon-separated parts"
    );
    assert_eq!(parts[0], deadline.to_string());
    assert_eq!(parts[1], account_index.to_string());
    assert_eq!(parts[2], api_key_index.to_string());
    assert_eq!(
        parts[3].len(),
        160,
        "signature part must be 80 bytes = 160 hex chars"
    );
}

#[test]
fn auth_token_signature_is_valid() {
    let km = KeyManager::generate();
    let pk = km.public_key_bytes();
    let deadline = 9999999999i64;
    let account_index = 1i64;
    let api_key_index = 0u8;
    let token = km
        .create_auth_token(deadline, account_index, api_key_index)
        .unwrap();

    // Re-derive message hash using the exact same logic as signer::KeyManager::create_auth_token
    let auth_data = format!("{}:{}:{}", deadline, account_index, api_key_index);
    let auth_bytes = auth_data.as_bytes();

    let mut elements = Vec::new();
    let mut i = 0;
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let val = u64::from_le_bytes(bytes);
        elements.push(Goldilocks::from_canonical_u64(val));
        i = next_start;
    }

    let hash_fp5 = hash_to_quintic_extension(&elements);
    let msg: [u8; 40] = hash_fp5.to_bytes_le();

    // Extract signature hex from token (last colon-separated part)
    let sig_hex = token.rsplit(':').next().unwrap();
    let sig_bytes = hex::decode(sig_hex).unwrap();

    assert!(
        verify_signature(&sig_bytes, &msg, &pk).unwrap(),
        "auth token signature must verify against derived message hash"
    );
}

#[test]
fn different_deadlines_produce_different_tokens() {
    let km = KeyManager::generate();
    let t1 = km.create_auth_token(1000, 1, 0).unwrap();
    let t2 = km.create_auth_token(2000, 1, 0).unwrap();
    assert_ne!(t1, t2);
}

// ─── 6. KeyManager from raw bytes ─────────────────────────────────────────────

#[test]
fn key_manager_new_rejects_wrong_length() {
    assert!(KeyManager::new(&[0u8; 20]).is_err());
    assert!(KeyManager::new(&[0u8; 41]).is_err());
    assert!(KeyManager::new(&[]).is_err());
}

#[test]
fn key_manager_new_accepts_40_bytes() {
    let km_orig = KeyManager::generate();
    let km_new = KeyManager::new(&km_orig.private_key_bytes()).unwrap();
    assert_eq!(km_orig.public_key_bytes(), km_new.public_key_bytes());
}
