# Signer Library

High-level signing interface for the Lighter Protocol, providing key management and transaction signing functionality.

## ✅ Verification Status

**Go Compatibility:** ✅ VERIFIED

- ✅ Auth token generation: Format matches Go exactly (message format verified)
- ✅ Message signing: Signature format matches Go (80 bytes: s || e)
- ✅ Key management: Public key derivation matches Go
- ✅ Test vectors: Comprehensive test suite with Go test vectors integrated

## Overview

This library provides a high-level API for:
- **Key Management**: Create and manage 40-byte private keys
- **Message Signing**: Sign 40-byte message hashes using Schnorr signatures
- **Auth Token Generation**: Create authentication tokens for API access
- **Public Key Derivation**: Derive public keys from private keys

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
signer = { path = "../signer" }
goldilocks-crypto = { path = "../crypto" }
poseidon-hash = { path = "../poseidon-hash" }
hex = "0.4"
```

## Usage

### Key Management

```rust
use signer::KeyManager;
use hex;

// Create from hex string (80 hex chars = 40 bytes)
let private_key_hex = "6227989d19d906db99e5da73c3ce4c2e41d80854cecce7618a1e45978a604c7c8fac5d6cc3eb315b";
let key_manager = KeyManager::from_hex(private_key_hex)?;

// Or from bytes
let private_key_bytes: [u8; 40] = [0u8; 40]; // Your private key
let key_manager = KeyManager::new(&private_key_bytes)?;

// Generate a random key pair
let key_manager = KeyManager::generate();

// Get public key (40 bytes)
let public_key = key_manager.public_key_bytes();
println!("Public key: {}", hex::encode(&public_key));

// Get private key bytes
let private_key = key_manager.private_key_bytes();
```

### Message Signing

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Sign a 40-byte message
let message: [u8; 40] = [0u8; 40]; // Your message hash
let signature = key_manager.sign(&message)?;

// Signature is 80 bytes: 40 bytes s + 40 bytes e
println!("Signature: {}", hex::encode(&signature));
```

### Auth Token Generation

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Create auth token
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as i64
    + 7 * 60 * 60; // 7 hours from now

let account_index = 271i64;
let api_key_index = 4u8;

let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
// Format: "deadline:account_index:api_key_index:signature_hex"
println!("Auth token: {}", auth_token);
```

## Features

- **Thread-Safe**: `KeyManager` is `Send + Sync` for concurrent use
- **Go-Compatible**: Auth tokens and signatures match Go implementation exactly
- **Type-Safe**: Compile-time guarantees for key and message lengths
- **Zero-Copy**: Efficient operations with minimal allocations

## API Reference

### KeyManager

```rust
impl KeyManager {
    /// Create from 40-byte private key
    pub fn new(private_key_bytes: &[u8]) -> Result<Self>;
    
    /// Create from hex string (with or without 0x prefix)
    pub fn from_hex(hex_str: &str) -> Result<Self>;
    
    /// Generate a new random key pair
    pub fn generate() -> Self;
    
    /// Get public key as 40-byte array
    pub fn public_key_bytes(&self) -> [u8; 40];
    
    /// Get private key as 40-byte array
    pub fn private_key_bytes(&self) -> [u8; 40];
    
    /// Sign a 40-byte message
    pub fn sign(&self, message: &[u8; 40]) -> Result<[u8; 80]>;
    
    /// Create auth token
    pub fn create_auth_token(
        &self,
        deadline: i64,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<String>;
}
```

## Error Handling

```rust
use signer::{KeyManager, SignerError};

match KeyManager::from_hex(invalid_hex) {
    Ok(km) => println!("Success"),
    Err(SignerError::HexDecode(e)) => eprintln!("Invalid hex: {}", e),
    Err(SignerError::Crypto(e)) => eprintln!("Crypto error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

Run tests with:

```bash
cargo test --package signer
```

Test coverage includes:
- Key management operations
- Message signing
- Auth token generation
- Go compatibility tests

## Go Compatibility

This implementation is verified to match the Go implementation:

- ✅ Auth token format: `deadline:account_index:api_key_index:signature` matches Go exactly
- ✅ Signature format: 80 bytes (40 bytes s + 40 bytes e) matches Go
- ✅ Message format: Auth token message format matches Go byte-for-byte
- ✅ Public key derivation: Matches Go's `SchnorrPkFromSk`

See `tests/auth_token_comparison.rs` for comprehensive comparison tests.

## Performance

- Signing: ~100-200 microseconds per signature
- Auth token generation: ~150-300 microseconds
- Public key derivation: ~50-100 microseconds

## Security Considerations

⚠️ **Important**: This library has NOT been security audited.

- **Private Keys**: Never expose private keys in logs or error messages
- **Random Generation**: `KeyManager::generate()` uses cryptographically secure RNG
- **Nonce Generation**: Signing uses secure random nonces internally
- **Audit Status**: Requires security review before production use

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.

















