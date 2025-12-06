# Signer Library

The `signer` crate provides high-level key management and transaction signing functionality for the Lighter Exchange.

## Overview

This library provides:
- **KeyManager**: Manages private keys and generates public keys (40-byte format)
- **Transaction Signing**: Signs 40-byte message hashes using Schnorr signatures
- **Auth Token Generation**: Creates authentication tokens for API access
- **Message Signing**: Signs arbitrary 40-byte messages

## Installation

```toml
[dependencies]
signer = { path = "../signer" }
crypto = { path = "../crypto" }
poseidon-hash = { path = "../poseidon-hash" }
```

## Basic Usage

### Key Management

```rust
use signer::KeyManager;

// Create KeyManager from private key hex string (80 hex chars = 40 bytes)
let private_key_hex = "6227989d19d906db99e5da73c3ce4c2e41d80854cecce7618a1e45978a604c7c8fac5d6cc3eb315b";
let key_manager = KeyManager::from_hex(private_key_hex)?;

// Or from bytes directly
let private_key_bytes: [u8; 40] = [0u8; 40]; // Your 40-byte private key
let key_manager = KeyManager::new(&private_key_bytes)?;

// Get public key (40 bytes)
let public_key = key_manager.public_key_bytes();
println!("Public key: {}", hex::encode(&public_key));

// Get private key bytes
let private_key = key_manager.private_key_bytes();
```

### Signing Messages

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Sign a 40-byte message hash
let message: [u8; 40] = [0u8; 40]; // Your 40-byte message hash
let signature = key_manager.sign(&message)?;

// Signature is 80 bytes (s || e format: 40 bytes s + 40 bytes e)
println!("Signature: {}", hex::encode(&signature));
```

### Creating Auth Tokens

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Calculate deadline (Unix timestamp in seconds)
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as i64 + 600; // 10 minutes from now

// Generate auth token
// Format: "deadline:account_index:api_key_index:signature_hex"
let auth_token = key_manager.create_auth_token(
    deadline,
    account_index,  // Your account index
    api_key_index,  // Your API key index
)?;

println!("Auth token: {}", auth_token);
```

## API Reference

### KeyManager

The main struct for managing keys and signing operations.

#### Creating KeyManager

```rust
use signer::KeyManager;

// From private key hex string (40 bytes = 80 hex characters)
// Supports both "0x..." and plain hex formats
let private_key_hex = "6227989d19d906db99e5da73c3ce4c2e41d80854cecce7618a1e45978a604c7c8fac5d6cc3eb315b";
let key_manager = KeyManager::from_hex(private_key_hex)?;

// Or with 0x prefix
let key_manager = KeyManager::from_hex("0x6227989d19d906db99e5da73c3ce4c2e41d80854cecce7618a1e45978a604c7c8fac5d6cc3eb315b")?;

// From private key bytes (40 bytes)
let private_key_bytes: [u8; 40] = [0u8; 40]; // Your 40-byte private key
let key_manager = KeyManager::new(&private_key_bytes)?;

// Generate a new random key pair
let key_manager = KeyManager::generate();
```

#### Getting Public Key

```rust
let key_manager = KeyManager::from_hex(private_key_hex)?;

// As bytes (40 bytes)
let public_key_bytes: [u8; 40] = key_manager.public_key_bytes();

// As hex string (encode manually)
let public_key_hex = hex::encode(public_key_bytes);
println!("Public key: {}", public_key_hex);

// Get private key bytes (40 bytes)
let private_key_bytes: [u8; 40] = key_manager.private_key_bytes();
```

#### Signing

```rust
let key_manager = KeyManager::from_hex(private_key_hex)?;

// Sign a 40-byte message hash
// Message should be a 40-byte hash (e.g., from Poseidon2)
let message: [u8; 40] = [0u8; 40]; // Your 40-byte message hash

// Sign message (returns 80-byte signature: s || e)
let signature: [u8; 80] = key_manager.sign(&message)?;

// Signature format: 40 bytes s || 40 bytes e
println!("Signature: {}", hex::encode(&signature));
```

#### Auth Tokens

```rust
let key_manager = KeyManager::from_hex(private_key_hex)?;

// Create auth token
// Format: "deadline:account_index:api_key_index:signature_hex"
let deadline = 1234567890i64; // Unix timestamp in seconds
let account_index = 271i64;
let api_key_index = 4u8;

let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;

// Token format: "deadline:account_index:api_key_index:signature_hex"
println!("Token: {}", auth_token);
```

## Advanced Usage

### Transaction Signing

For signing Lighter Exchange transactions, use the `api-client` library which handles transaction construction. The signer library is used internally:

```rust
// See api-client documentation for transaction signing
use api_client::{LighterClient, CreateOrderRequest};

let client = LighterClient::new(base_url, private_key_hex, account_index, api_key_index)?;
let order = CreateOrderRequest { /* ... */ };
let response = client.create_order(order).await?;
```

### Message Formatting

When signing custom messages, convert them to Fp5Element format:

```rust
use signer::KeyManager;
use poseidon_hash::{Fp5Element, GoldilocksField};

fn sign_string_message(key_manager: &KeyManager, message: &str) -> Result<Vec<u8>, SignerError> {
    // Convert string to bytes
    let message_bytes = message.as_bytes();
    
    // Pad or chunk to 40-byte multiples
    let mut chunks = Vec::new();
    for chunk in message_bytes.chunks(40) {
        let mut padded = vec![0u8; 40];
        padded[..chunk.len()].copy_from_slice(chunk);
        chunks.push(Fp5Element::from_bytes_le(&padded));
    }
    
    // Hash the chunks to get single Fp5Element
    let message_hash = poseidon_hash::poseidon2_hash(&chunks);
    
    // Sign
    key_manager.sign(&message_hash)
}
```

### Auth Token Format

Auth tokens follow this format:

```
Message: "deadline:account_index:api_key_index"
Hash: Poseidon2(Message bytes) -> 40-byte hash
Signature: Sign(Hash) -> 80-byte signature (s || e)
Token: "deadline:account_index:api_key_index:signature_hex"
```

Example implementation:

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Calculate deadline (Unix timestamp in seconds)
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as i64 + 600; // 10 minutes from now

// Generate token
let token = key_manager.create_auth_token(
    deadline,
    account_index,
    api_key_index,
)?;

// Token: "deadline:account_index:api_key_index:signature_hex"
println!("Use this token: {}", token);
```

### Key Generation

Generate a new random key pair:

```rust
use signer::KeyManager;

// Generate cryptographically secure random key
let key_manager = KeyManager::generate();

// Get keys
let private_key = key_manager.private_key_bytes();
let public_key = key_manager.public_key_bytes();

println!("Private key: {}", hex::encode(&private_key));
println!("Public key: {}", hex::encode(&public_key));
```

## Error Handling

```rust
use signer::{KeyManager, SignerError};

match KeyManager::new(invalid_key) {
    Ok(km) => {
        // Use key manager
    }
    Err(SignerError::InvalidPrivateKeyLength(len)) => {
        eprintln!("Invalid key length: {} (expected 40 bytes)", len);
    }
    Err(SignerError::InvalidPrivateKeyFormat) => {
        eprintln!("Invalid key format (must be hex string)");
    }
    Err(SignerError::CryptoError(e)) => {
        eprintln!("Crypto error: {:?}", e);
    }
}
```

## Security Best Practices

1. **Private Key Storage**: Never hardcode private keys. Use environment variables or secure key management.
2. **Key Generation**: In production, generate keys using secure random number generators.
3. **Message Validation**: Always validate messages before signing to prevent signing malicious data.
4. **Auth Tokens**: Include timestamps and expiration in auth token messages to prevent replay attacks.
5. **Error Messages**: Don't expose sensitive information in error messages.

## Common Patterns

### Signing Transaction Hashes

The `api-client` library handles transaction signing internally. For custom use cases:

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Transaction hash from Poseidon2 (40 bytes)
let tx_hash: [u8; 40] = [0u8; 40]; // Your 40-byte transaction hash

// Sign the hash
let signature = key_manager.sign(&tx_hash)?;

// Signature is 80 bytes (s || e format)
println!("Signature: {}", hex::encode(&signature));
```

## Performance

- Key operations are efficient and optimized
- Signing operations use optimized cryptographic primitives
- Auth token generation is fast (< 1ms typical)

## See Also

- [Crypto Library](./crypto.md) - Underlying cryptographic primitives
- [API Client](./api-client.md) - High-level API for transaction signing
- [Getting Started Guide](./getting-started.md) - Quick start tutorial
