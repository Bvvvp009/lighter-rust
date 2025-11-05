# Signer Library

The `signer` crate provides high-level key management and transaction signing functionality for the Lighter Exchange.

## Overview

This library provides:
- **KeyManager**: Manages private keys and generates public keys
- **Transaction Signing**: Signs transactions for Lighter Exchange
- **Auth Token Generation**: Creates authentication tokens for API access
- **Message Signing**: General-purpose message signing

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

// Create KeyManager from private key (40 bytes hex string)
let private_key_hex = "your_40_byte_hex_private_key";
let key_manager = KeyManager::new(private_key_hex)?;

// Get public key (40 bytes)
let public_key = key_manager.public_key_bytes();
println!("Public key: {}", hex::encode(&public_key));

// Get public key as hex string
let public_key_hex = key_manager.public_key_hex();
```

### Signing Messages

```rust
use signer::KeyManager;
use poseidon_hash::Fp5Element;

let key_manager = KeyManager::new(private_key_hex)?;

// Create a message (40 bytes -> Fp5Element)
let message_bytes = vec![0u8; 40];
let message = Fp5Element::from_bytes_le(&message_bytes);

// Sign the message
let signature = key_manager.sign(&message)?;

// Signature is 80 bytes (R point + scalar)
println!("Signature: {}", hex::encode(&signature));
```

### Creating Auth Tokens

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::new(private_key_hex)?;

// Create auth token message
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;

let message = format!("LIGHTER_AUTH:{}", timestamp);

// Generate auth token
let auth_token = key_manager.create_auth_token(&message)?;
println!("Auth token: {}", auth_token);
```

## API Reference

### KeyManager

The main struct for managing keys and signing operations.

#### Creating KeyManager

```rust
use signer::KeyManager;

// From private key hex string (40 bytes = 80 hex characters)
let private_key_hex = "0123456789abcdef..."; // 80 characters
let key_manager = KeyManager::new(private_key_hex)?;

// From private key bytes (40 bytes)
let private_key_bytes: [u8; 40] = [0u8; 40];
let key_manager = KeyManager::from_bytes(&private_key_bytes)?;
```

#### Getting Public Key

```rust
let key_manager = KeyManager::new(private_key_hex)?;

// As bytes (40 bytes)
let public_key_bytes: Vec<u8> = key_manager.public_key_bytes();

// As hex string
let public_key_hex: String = key_manager.public_key_hex();

// As Point (from crypto crate)
let public_key_point = key_manager.public_key();
```

#### Signing

```rust
use poseidon_hash::Fp5Element;

let key_manager = KeyManager::new(private_key_hex)?;
let message = Fp5Element::one();

// Sign message (returns 80-byte signature)
let signature: Vec<u8> = key_manager.sign(&message)?;

// Debug signing (deterministic, for testing)
let signature_debug = key_manager.sign_debug(&message, nonce)?;
```

#### Auth Tokens

```rust
let key_manager = KeyManager::new(private_key_hex)?;

// Create auth token from message string
let message = "LIGHTER_AUTH:1234567890";
let auth_token = key_manager.create_auth_token(message)?;

// Auth token is base64-encoded signature
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
Message: "LIGHTER_AUTH:{timestamp}"
Signature: Sign(Message) -> base64 encoded
Token: base64(signature)
```

Example implementation:

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_lighter_auth_token(
    key_manager: &KeyManager,
    timestamp: i64,
) -> Result<String, SignerError> {
    let message = format!("LIGHTER_AUTH:{}", timestamp);
    key_manager.create_auth_token(&message)
}

// Usage
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;

let token = create_lighter_auth_token(&key_manager, timestamp)?;
```

### Deterministic Signing (Testing)

For testing purposes, you can use deterministic signing:

```rust
use signer::KeyManager;
use crypto::ScalarField;
use poseidon_hash::Fp5Element;

let key_manager = KeyManager::new(private_key_hex)?;
let message = Fp5Element::one();

// Use fixed nonce for reproducible signatures
let nonce = ScalarField::from_u64(12345);
let signature = key_manager.sign_debug(&message, &nonce)?;
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

### Key Pair Generation

```rust
use crypto::ScalarField;
use signer::KeyManager;

// Generate random private key
let private_scalar = ScalarField::sample_crypto();
let private_bytes: [u8; 32] = private_scalar.to_bytes_array();

// Pad to 40 bytes (for Lighter format)
let mut private_key = [0u8; 40];
private_key[..32].copy_from_slice(&private_bytes);

// Create KeyManager
let key_manager = KeyManager::from_bytes(&private_key)?;
```

### Signing Transaction Hashes

```rust
use signer::KeyManager;
use poseidon_hash::Fp5Element;

fn sign_transaction_hash(
    key_manager: &KeyManager,
    tx_hash: &[u8; 32],
) -> Result<Vec<u8>, SignerError> {
    // Convert 32-byte hash to Fp5Element (pad to 40 bytes)
    let mut message_bytes = [0u8; 40];
    message_bytes[..32].copy_from_slice(tx_hash);
    let message = Fp5Element::from_bytes_le(&message_bytes);
    
    key_manager.sign(&message)
}
```

## Performance

- Key operations are efficient and optimized
- Signing operations use optimized cryptographic primitives
- Auth token generation is fast (< 1ms typical)

## See Also

- [Crypto Library](./crypto.md) - Underlying cryptographic primitives
- [API Client](./api-client.md) - High-level API for transaction signing
- [Getting Started Guide](./getting-started.md) - Quick start tutorial
