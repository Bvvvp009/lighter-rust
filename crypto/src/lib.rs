//! # Goldilocks Crypto
//!
//! Rust port of ECgFp5 elliptic curve and Schnorr signatures over the Goldilocks field, ported from lighter-go (Lighter Protocol).
//!
//! ## ⚠️ Security Warning
//!
//! **This library has NOT been audited and is provided as-is. Use with caution.**
//!
//! - This is a **prototype port** from the Go SDK (lighter-go)
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
//! use goldilocks_crypto::{ScalarField, Point, sign_with_nonce, verify_signature};
//!
//! // Generate a random private key
//! let private_key = ScalarField::sample_crypto();
//! let private_key_bytes = private_key.to_bytes_le();
//!
//! // Derive public key
//! let public_key = Point::generator().mul(&private_key);
//! let public_key_bytes = public_key.encode().to_bytes_le();
//!
//! // Sign a message
//! let message = [0u8; 40];
//! let nonce = ScalarField::sample_crypto();
//! let nonce_bytes = nonce.to_bytes_le();
//! let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
//!
//! // Verify signature
//! let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
//! assert!(is_valid);
//! ```
//!
//! [`poseidon-hash`]: https://crates.io/crates/poseidon-hash

pub mod schnorr;
pub mod scalar_field;

pub use scalar_field::ScalarField;

pub use poseidon_hash::{Goldilocks, Fp5Element};

// Re-export Schnorr functions
pub use schnorr::{sign_with_nonce, verify_signature, Point};

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
    /// Hex decoding failed.
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}

/// Result type for cryptographic operations.
pub type Result<T> = std::result::Result<T, CryptoError>;

