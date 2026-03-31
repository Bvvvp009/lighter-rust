//! `Signature` — a typed wrapper around the raw 80-byte Schnorr signature.
//!
//! Using this type instead of a plain `[u8; 80]` or `Vec<u8>` prevents
//! accidental swapping of signature and message/key arguments and provides
//! convenient serialisation helpers.
//!
//! # Example
//!
//! ```rust
//! use goldilocks_crypto::{ScalarField, Point, Signature, sign, verify_signature};
//!
//! let sk = ScalarField::sample_crypto();
//! let sk_bytes = sk.to_bytes_le();
//! let pk_bytes = Point::generator().mul(&sk).encode().to_bytes_le();
//! let msg = [0u8; 40];
//!
//! let raw = sign(&sk_bytes, &msg).unwrap();
//! let sig = Signature::from_bytes(&raw).unwrap();
//!
//! assert!(verify_signature(sig.as_bytes(), &msg, &pk_bytes).unwrap());
//! ```

use crate::{CryptoError, Result};

/// A typed 80-byte Schnorr signature.
///
/// The layout is `[s (40 bytes) || e (40 bytes)]` matching the byte format
/// returned by [`crate::sign`] and consumed by [`crate::verify_signature`].
#[derive(Clone, PartialEq, Eq)]
pub struct Signature([u8; 80]);

impl Signature {
    /// Expected byte length of a serialised signature.
    pub const BYTE_LEN: usize = 80;

    /// Constructs a `Signature` from a 80-byte array.
    pub fn from_array(bytes: [u8; 80]) -> Self {
        Signature(bytes)
    }

    /// Parses a `Signature` from a byte slice.
    ///
    /// Returns `Err` if `bytes` is not exactly 80 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::BYTE_LEN {
            return Err(CryptoError::InvalidSignatureLength(bytes.len()));
        }
        let mut arr = [0u8; 80];
        arr.copy_from_slice(bytes);
        Ok(Signature(arr))
    }

    /// Returns a reference to the raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Copies the raw bytes into a fixed-size array.
    pub fn to_array(&self) -> [u8; 80] {
        self.0
    }

    /// Encodes this signature as a 160-character lowercase hex string.
    ///
    /// The first 80 chars encode `s`; the last 80 chars encode `e`.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Decodes a `Signature` from a 160-character hex string.
    ///
    /// Accepts an optional `0x` prefix.
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 160 {
            return Err(CryptoError::InvalidSignatureLength(s.len() / 2));
        }
        let bytes = hex::decode(s).map_err(CryptoError::HexDecode)?;
        Self::from_bytes(&bytes)
    }

    /// Splits the signature into its two 40-byte components `(s, e)`.
    ///
    /// `s` is the scalar commitment; `e` is the hash challenge.
    pub fn split(&self) -> ([u8; 40], [u8; 40]) {
        let mut s = [0u8; 40];
        let mut e = [0u8; 40];
        s.copy_from_slice(&self.0[..40]);
        e.copy_from_slice(&self.0[40..]);
        (s, e)
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature({})", self.to_hex())
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl TryFrom<Vec<u8>> for Signature {
    type Error = CryptoError;
    fn try_from(v: Vec<u8>) -> Result<Self> {
        Self::from_bytes(&v)
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = CryptoError;
    fn try_from(v: &[u8]) -> Result<Self> {
        Self::from_bytes(v)
    }
}

impl From<Signature> for Vec<u8> {
    fn from(sig: Signature) -> Vec<u8> {
        sig.0.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScalarField, sign};

    fn make_sig() -> (Vec<u8>, [u8; 40]) {
        let sk = ScalarField::sample_crypto();
        let sk_bytes = sk.to_bytes_le();
        let msg = [0x42u8; 40];
        (sign(&sk_bytes, &msg).unwrap(), msg)
    }

    #[test]
    fn from_bytes_roundtrip() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        assert_eq!(sig.as_bytes(), raw.as_slice());
    }

    #[test]
    fn to_hex_from_hex_roundtrip() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        let hex = sig.to_hex();
        assert_eq!(hex.len(), 160);
        let restored = Signature::from_hex(&hex).unwrap();
        assert_eq!(sig, restored);
    }

    #[test]
    fn from_hex_0x_prefix() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        let hex_0x = format!("0x{}", sig.to_hex());
        assert_eq!(Signature::from_hex(&hex_0x).unwrap(), sig);
    }

    #[test]
    fn from_bytes_wrong_length_errors() {
        assert!(Signature::from_bytes(&[0u8; 40]).is_err());
        assert!(Signature::from_bytes(&[0u8; 81]).is_err());
        assert!(Signature::from_bytes(&[]).is_err());
    }

    #[test]
    fn split_gives_correct_halves() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        let (s, e) = sig.split();
        assert_eq!(&s, &raw[..40]);
        assert_eq!(&e, &raw[40..]);
    }

    #[test]
    fn display_equals_to_hex() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        assert_eq!(format!("{sig}"), sig.to_hex());
    }

    #[test]
    fn try_from_vec_u8() {
        let (raw, _) = make_sig();
        let sig: Signature = raw.clone().try_into().unwrap();
        assert_eq!(sig.as_bytes(), raw.as_slice());
    }

    #[test]
    fn into_vec_u8() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        let v: Vec<u8> = sig.into();
        assert_eq!(v, raw);
    }

    #[test]
    fn to_array_roundtrip() {
        let (raw, _) = make_sig();
        let sig = Signature::from_bytes(&raw).unwrap();
        let arr = sig.to_array();
        assert_eq!(&arr[..], raw.as_slice());
    }
}
