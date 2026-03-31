//! `KeyPair` — a high-level wrapper combining a private scalar and its public key.
//!
//! The private key is stored as a `ScalarField` which derives `Zeroize`, so
//! calling [`KeyPair::zeroize`] (or dropping a `KeyPair` after calling it)
//! wipes the secret scalar from memory, preventing key material from lingering
//! in heap allocations.
//!
//! # Example
//!
//! ```rust
//! use goldilocks_crypto::keypair::KeyPair;
//!
//! // Generate a fresh keypair.
//! let kp = KeyPair::generate();
//!
//! // Sign and verify.
//! let msg = [0u8; 40];
//! let sig = kp.sign(&msg).unwrap();
//! assert!(kp.verify(&sig, &msg).unwrap());
//!
//! // Derive a keypair deterministically from 32+ bytes of seed material.
//! let seed = b"my very secret seed material 123";
//! let kp2 = KeyPair::from_seed(seed).unwrap();
//! let sig2 = kp2.sign(&msg).unwrap();
//! assert!(kp2.verify(&sig2, &msg).unwrap());
//! ```

use zeroize::Zeroize;
use crate::{
    CryptoError, Result, ScalarField, Point,
    sign, verify_signature,
    signature::Signature,
};

/// A Schnorr keypair: private scalar + derived public key point encoding.
///
/// # Memory safety
///
/// The private scalar is stored as [`ScalarField`] which derives [`Zeroize`].
/// Call [`KeyPair::zeroize`] explicitly when you are done with the keypair to
/// clear the secret from memory.  Alternatively wrap it in `zeroize::Zeroizing`
/// or ensure the value is dropped promptly.
pub struct KeyPair {
    private_key: ScalarField,
    public_key_bytes: [u8; 40],
}

impl KeyPair {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Generates a fresh keypair using cryptographically secure randomness.
    pub fn generate() -> Self {
        let sk = ScalarField::sample_crypto();
        Self::from_scalar(sk)
    }

    /// Constructs a keypair from a 40-byte little-endian private key.
    ///
    /// Returns `Err` if `bytes` is not exactly 40 bytes or if the decoded
    /// scalar is the zero element (which produces the neutral public key).
    pub fn from_private_key_bytes(bytes: &[u8]) -> Result<Self> {
        let sk = ScalarField::from_bytes_le(bytes)
            .map_err(|_| CryptoError::InvalidPrivateKeyLength(bytes.len()))?;
        if sk.is_zero() {
            return Err(CryptoError::InvalidPrivateKeyLength(0));
        }
        Ok(Self::from_scalar(sk))
    }

    /// Derives a keypair deterministically from arbitrary seed bytes.
    ///
    /// The seed is hashed (via the Poseidon2-backed [`ScalarField::from_seed_bytes`])
    /// to produce a canonical private scalar.  The same seed always produces
    /// the same keypair.  The seed must be at least 1 byte long.
    ///
    /// # Security note
    /// Use a high-entropy seed (e.g. 32+ random bytes or an HKDF output).
    /// Do **not** use a low-entropy password directly.
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        if seed.is_empty() {
            return Err(CryptoError::InvalidPrivateKeyLength(0));
        }
        let sk = ScalarField::from_seed_bytes(seed);
        if sk.is_zero() {
            // Astronomically unlikely; defend anyway.
            return Err(CryptoError::InvalidPrivateKeyLength(0));
        }
        Ok(Self::from_scalar(sk))
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    /// Returns the 40-byte little-endian encoding of the private key.
    pub fn private_key_bytes(&self) -> [u8; 40] {
        self.private_key.to_bytes_le()
    }

    /// Returns the 40-byte encoding of the public key point.
    pub fn public_key_bytes(&self) -> [u8; 40] {
        self.public_key_bytes
    }

    /// Returns a reference to the underlying private scalar.
    pub fn private_key(&self) -> &ScalarField {
        &self.private_key
    }

    // ------------------------------------------------------------------
    // Signing
    // ------------------------------------------------------------------

    /// Signs `message` with this keypair.
    ///
    /// `message` must be exactly 40 bytes of canonical Goldilocks field data.
    /// A random nonce is generated internally for each call.
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let raw = sign(&self.private_key.to_bytes_le(), message)?;
        Signature::from_bytes(&raw)
    }

    /// Verifies that `signature` over `message` was produced by this keypair.
    pub fn verify(&self, signature: &Signature, message: &[u8]) -> Result<bool> {
        verify_signature(signature.as_bytes(), message, &self.public_key_bytes)
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    fn from_scalar(sk: ScalarField) -> Self {
        let pk = Point::generator().mul(&sk).encode().to_bytes_le();
        Self { private_key: sk, public_key_bytes: pk }
    }
}

impl Zeroize for KeyPair {
    fn zeroize(&mut self) {
        self.private_key.zeroize();
        self.public_key_bytes.zeroize();
    }
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // Best-effort: wipe the private key when the keypair is dropped.
        // Callers should *also* call zeroize() explicitly when dealing with
        // secrets to ensure it happens before the memory is potentially reused.
        self.private_key.zeroize();
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &hex::encode(self.public_key_bytes))
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroize;

    #[test]
    fn generate_sign_verify_roundtrip() {
        let kp = KeyPair::generate();
        let msg = [0x01u8; 40];
        let sig = kp.sign(&msg).unwrap();
        assert!(kp.verify(&sig, &msg).unwrap());
    }

    #[test]
    fn from_private_key_bytes_roundtrip() {
        let kp1 = KeyPair::generate();
        let sk_bytes = kp1.private_key_bytes();
        let kp2 = KeyPair::from_private_key_bytes(&sk_bytes).unwrap();
        assert_eq!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }

    #[test]
    fn from_seed_is_deterministic() {
        let seed = b"deterministic seed for testing";
        let kp1 = KeyPair::from_seed(seed).unwrap();
        let kp2 = KeyPair::from_seed(seed).unwrap();
        assert_eq!(kp1.public_key_bytes(), kp2.public_key_bytes());
        assert_eq!(kp1.private_key_bytes(), kp2.private_key_bytes());
    }

    #[test]
    fn different_seeds_give_different_keypairs() {
        let kp1 = KeyPair::from_seed(b"seed one").unwrap();
        let kp2 = KeyPair::from_seed(b"seed two").unwrap();
        assert_ne!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }

    #[test]
    fn from_seed_sign_and_verify() {
        let kp = KeyPair::from_seed(b"test seed 32 bytes padding here!").unwrap();
        let msg = [0xAAu8; 40];
        let sig = kp.sign(&msg).unwrap();
        assert!(kp.verify(&sig, &msg).unwrap());
    }

    #[test]
    fn wrong_key_does_not_verify() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let msg = [0x02u8; 40];
        let sig = kp1.sign(&msg).unwrap();
        // kp2.verify checks under kp2's public key
        assert!(!kp2.verify(&sig, &msg).unwrap());
    }

    #[test]
    fn modified_message_does_not_verify() {
        let kp = KeyPair::generate();
        let msg = [0x03u8; 40];
        let sig = kp.sign(&msg).unwrap();
        let mut bad_msg = msg;
        bad_msg[0] ^= 0xFF;
        assert!(!kp.verify(&sig, &bad_msg).unwrap());
    }

    #[test]
    fn zeroize_clears_private_key() {
        let mut kp = KeyPair::generate();
        let before = kp.private_key_bytes();
        assert_ne!(before, [0u8; 40]);
        kp.zeroize();
        assert_eq!(kp.private_key_bytes(), [0u8; 40]);
    }

    #[test]
    fn from_private_key_bytes_rejects_wrong_length() {
        assert!(KeyPair::from_private_key_bytes(&[0u8; 20]).is_err());
        assert!(KeyPair::from_private_key_bytes(&[0u8; 41]).is_err());
    }

    #[test]
    fn from_seed_rejects_empty() {
        assert!(KeyPair::from_seed(&[]).is_err());
    }

    #[test]
    fn debug_does_not_leak_private_key() {
        let kp = KeyPair::generate();
        let debug_str = format!("{kp:?}");
        assert!(debug_str.contains("[REDACTED]"), "private key must be redacted in Debug output");
        assert!(!debug_str.contains(&hex::encode(kp.private_key_bytes())));
    }
}
