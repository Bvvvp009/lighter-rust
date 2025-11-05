# Goldilocks Crypto

Rust port of ECgFp5 elliptic curve and Schnorr signatures over the Goldilocks field, ported from lighter-go (Lighter Protocol).

## ⚠️ Security Warning

**This library has NOT been audited and is provided as-is. Use with caution.**

- This is a **prototype port** from the Go SDK (lighter-go)
- **Not security audited** - do not use in production without proper security review
- While the implementation appears to work correctly, cryptographic software requires careful auditing
- This is an open-source contribution and not an official Lighter Protocol library
- Use at your own risk

## Features

- **ECgFp5 Elliptic Curve**: Point operations over the Goldilocks field extension
- **Schnorr Signatures**: Modern signature scheme with Poseidon2-based hashing
- **Scalar Field Arithmetic**: Efficient scalar operations for cryptographic protocols
- **Windowed Scalar Multiplication**: Optimized point multiplication for performance
- **Type-Safe API**: Strong compile-time guarantees for cryptographic operations

## Overview

This crate provides elliptic curve cryptography primitives specifically designed for the Goldilocks field:

- **ECgFp5 Curve**: Elliptic curve operations over the Fp5 extension field
- **Schnorr Signatures**: Signature generation and verification using Poseidon2 hashing
- **Point Arithmetic**: Addition, multiplication, encoding, and decoding
- **Scalar Field**: Efficient scalar operations for private keys and nonces

## Dependencies

This crate depends on [`poseidon-hash`](../poseidon-hash) for:
- Goldilocks field arithmetic
- Poseidon2 hash function
- Fp5 extension field operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
crypto = "0.1"
poseidon-hash = "0.1"  # Required dependency
```

Or use the latest version from git (until published):

```toml
[dependencies]
crypto = { git = "https://github.com/bvvvp009/lighter-rust", path = "rust-signer/crypto" }
poseidon-hash = { git = "https://github.com/bvvvp009/lighter-rust", path = "rust-signer/poseidon-hash" }
```

## Usage

### Key Generation

```rust
use crypto::{ScalarField, Point};

// Generate a random private key
let private_key = ScalarField::sample_crypto();

// Derive public key
let public_key = Point::generator().mul(&private_key);
```

### Schnorr Signatures

```rust
use crypto::{sign_with_nonce, verify_signature, Point, ScalarField};

// Generate keys
let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

// Derive public key
let public_key = Point::generator().mul(&private_key);
let public_key_bytes = public_key.encode().to_bytes_le();

// Sign a message (40 bytes)
let message = [0u8; 40];
let nonce = ScalarField::sample_crypto();
let nonce_bytes = nonce.to_bytes_le();
let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();

// Verify signature
let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
assert!(is_valid);
```

### Point Operations

```rust
use crypto::{Point, ScalarField};

// Create points
let generator = Point::generator();
let point1 = generator.mul(&ScalarField::from(2));
let point2 = generator.mul(&ScalarField::from(3));

// Point addition
let sum = point1.add(&point2);

// Point encoding/decoding
let encoded = sum.encode();
let decoded = Point::decode(&encoded).unwrap();
```

### Private Key Management

```rust
use crypto::ScalarField;

// Generate a new random private key (recommended)
let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

// Create from bytes (40 bytes, little-endian)
let private_key_bytes = [0u8; 40];
let private_key = ScalarField::from_bytes_le(&private_key_bytes).unwrap();

// Convert to bytes
let bytes = private_key.to_bytes_le(); // Returns [u8; 40]

// Create from hex string (80 hex characters = 40 bytes)
let hex_key = "0".repeat(80);
let private_key = ScalarField::from_hex(&hex_key).unwrap();
```

## Integration Guide

### Complete Signing Example

Here's a complete example of generating keys, signing, and verifying:

```rust
use crypto::{ScalarField, Point, sign_with_nonce, verify_signature};

// Step 1: Generate key pair
let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

// Step 2: Derive public key
let generator = Point::generator();
let public_key_point = generator.mul(&private_key);
let public_key_bytes = public_key_point.encode().to_bytes_le();

// Step 3: Prepare message (must be 40 bytes)
let message = b"Hello, World! This is a 40-byte message!!"; // 40 bytes
assert_eq!(message.len(), 40);

// Step 4: Generate nonce (CRITICAL: must be cryptographically secure and unique)
let nonce = ScalarField::sample_crypto();
let nonce_bytes = nonce.to_bytes_le();

// Step 5: Sign the message
let signature = sign_with_nonce(&private_key_bytes, message, &nonce_bytes)
    .expect("Signing failed");

// Step 6: Verify the signature
let is_valid = verify_signature(&signature, message, &public_key_bytes)
    .expect("Verification failed");
assert!(is_valid, "Signature should be valid");
```

### Key Derivation Pattern

```rust
use crypto::{ScalarField, Point};

fn derive_key_pair(seed: &[u8]) -> ([u8; 40], [u8; 40]) {
    // Hash seed to get private key (simplified - use proper KDF in production)
    use poseidon_hash::{hash_to_quintic_extension, Goldilocks};
    
    let seed_elements: Vec<Goldilocks> = seed.chunks(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            arr[..chunk.len()].copy_from_slice(chunk);
            Goldilocks::from_canonical_u64(u64::from_le_bytes(arr))
        })
        .collect();
    
    let hash = hash_to_quintic_extension(&seed_elements);
    let private_key_bytes = hash.to_bytes_le();
    
    // Derive public key
    let private_key = ScalarField::from_bytes_le(&private_key_bytes).unwrap();
    let public_key = Point::generator().mul(&private_key);
    let public_key_bytes = public_key.encode().to_bytes_le();
    
    (private_key_bytes, public_key_bytes)
}
```

### Batch Signature Verification

```rust
use crypto::verify_signature;

fn verify_batch(
    signatures: &[&[u8]],
    messages: &[&[u8]],
    public_keys: &[&[u8]],
) -> Vec<bool> {
    signatures
        .iter()
        .zip(messages.iter())
        .zip(public_keys.iter())
        .map(|((sig, msg), pk)| {
            verify_signature(sig, msg, pk).unwrap_or(false)
        })
        .collect()
}
```

### Error Handling

```rust
use crypto::{sign_with_nonce, CryptoError};

fn sign_message_safe(
    private_key: &[u8],
    message: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    // Validate inputs
    if private_key.len() != 40 {
        return Err(CryptoError::InvalidPrivateKeyLength(private_key.len()));
    }
    
    if message.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(message.len()));
    }
    
    // Sign
    sign_with_nonce(private_key, message, nonce)
}
```

## Optional Features

- **`serde`**: Enable serialization/deserialization support

```toml
[dependencies]
crypto = { version = "0.1", features = ["serde"] }
poseidon-hash = { version = "0.1", features = ["serde"] }
```

## Use Cases

- Transaction signing for blockchain protocols
- Cryptographic signature schemes
- Key exchange protocols
- Zero-Knowledge proof systems requiring curve operations
- Secure authentication mechanisms

## Performance

The implementation is optimized for:
- Fast point multiplication using windowed algorithm
- Efficient signature generation and verification
- Low memory allocations
- Production-grade cryptographic security

## Security Considerations

⚠️ **Important**: This library has NOT been security audited. Use with caution in production systems.

- **Private Keys**: Never expose private keys in logs or error messages
- **Nonces**: Always use cryptographically secure random nonces (`ScalarField::sample_crypto()`)
- **Side Channels**: The implementation is designed to be constant-time where possible
- **Audit Status**: This is a prototype port from lighter-go and requires security review before production use

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/crypto).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

This library is ported from the Lighter-Go of the [Lighter Protocol](https://github.com/elliottech/lighter-go) project. Feel free to contribute or request any changes.

