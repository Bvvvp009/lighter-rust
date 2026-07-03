#![forbid(unsafe_code)]
//! # Goldilocks Crypto
//!
//! Rust implementation of ECgFp5 elliptic curve and Schnorr signatures over the Goldilocks field.
//!
//! ## ⚠️ Security Warning
//!
//! **This library has NOT been audited and is provided as-is. Use with caution.**
//!
//! - Prototype implementation focused on correctness
//! - **Not security audited** - do not use in production without proper security review
//! - While the implementation appears to work correctly, cryptographic software requires careful auditing
//! - This is an open-source contribution and not an official Lighter Protocol library
//! - Use at your own risk
//!
//! ## Overview
//!
//! This crate provides elliptic curve cryptography primitives specifically designed for
//! the Goldilocks field, including:
//!
//! - **ECgFp5 Elliptic Curve**: Point operations over the Fp5 extension field
//! - **Schnorr Signatures**: Signature generation and verification using Poseidon2 hashing
//! - **Scalar Field**: Efficient scalar operations for private keys and nonces
//! - **Point Arithmetic**: Addition, multiplication, encoding, and decoding
//!
//! ## Dependencies
//!
//! This crate depends on [`poseidon-hash`] for:
//! - Goldilocks field arithmetic
//! - Poseidon2 hash function
//! - Fp5 extension field operations
//!
//! ## Example
//!
//! ```rust
//! use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};
//!
//! // Generate a random private key
//! let private_key = ScalarField::sample_crypto();
//! let private_key_bytes = private_key.to_bytes_le();
//!
//! // Derive public key
//! let public_key = Point::generator().mul(&private_key);
//! let public_key_bytes = public_key.encode().to_bytes_le();
//!
//! // Sign a message (nonce is generated internally)
//! let message = [0u8; 40];
//! let signature = sign(&private_key_bytes, &message).unwrap();
//!
//! // Verify signature
//! let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
//! assert!(is_valid);
//! ```
//!
//! [`poseidon-hash`]: https://crates.io/crates/poseidon-hash

pub mod batch_verify;
pub mod keypair;
pub mod scalar_field;
pub mod schnorr;
pub mod signature;

pub use scalar_field::ScalarField;

pub use poseidon_hash::{Fp5Element, Goldilocks};

// Re-export Schnorr functions
pub use batch_verify::batch_verify;
pub use keypair::KeyPair;
pub use schnorr::{
    sign, sign_hashed_message, sign_with_nonce, validate_public_key, verify_signature, AffinePoint,
    Point,
};
pub use signature::Signature;
// WeierstrassPoint will be added when needed - for now using Point::mul_add2 for verification
pub type WeierstrassPoint = Point;

use thiserror::Error;

/// Errors that can occur during cryptographic operations.
#[derive(Error, Debug)]
pub enum CryptoError {
    /// The private key has an invalid length.
    #[error("Invalid private key length: expected 40 bytes, got {0}")]
    InvalidPrivateKeyLength(usize),
    /// The signature format is invalid.
    #[error("Invalid signature format")]
    InvalidSignature,
    /// The signature has an invalid length.
    #[error("Invalid signature length: expected 80 bytes, got {0}")]
    InvalidSignatureLength(usize),
    /// The message has an invalid length.
    #[error("Invalid message length: expected 40 bytes, got {0}")]
    InvalidMessageLength(usize),
    /// The nonce has an invalid length.
    #[error("Invalid nonce length: expected 40 bytes, got {0}")]
    InvalidNonceLength(usize),
    /// The public key has an invalid length.
    #[error("Invalid public key length: expected 40 bytes, got {0}")]
    InvalidPublicKeyLength(usize),
    /// The public key is invalid or cannot be decoded.
    #[error("Invalid public key: cannot decode as encoded point")]
    InvalidPublicKey,
    /// A scalar input uses a non-canonical byte encoding.
    #[error("Non-canonical scalar encoding for {field}")]
    NonCanonicalScalar { field: &'static str },
    /// A secret scalar input is zero when a non-zero value is required.
    #[error("Zero scalar is not allowed for {field}")]
    ZeroScalar { field: &'static str },
    /// Hex decoding failed.
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    /// A message 8-byte chunk represents a value ≥ the Goldilocks field modulus.
    /// Messages must be formed from canonical Goldilocks field elements
    /// (each 8-byte little-endian chunk must be in [0, 2¶⁴ − 2³² + 1)).
    #[error("Non-canonical message: chunk {index} value 0x{value:016x} is >= Goldilocks modulus")]
    NonCanonicalMessage { index: usize, value: u64 },
}

/// Result type for cryptographic operations.
pub type Result<T> = std::result::Result<T, CryptoError>;
