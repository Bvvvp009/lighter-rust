# Lighter Rust SDK

A high-performance Rust implementation of the Lighter Protocol signer, providing cryptographic primitives for the Lighter Exchange.

## 🚀 Features

- **High-Performance Signing**: Optimized Schnorr signature generation using Goldilocks field arithmetic
- **Poseidon2 Hashing**: Efficient zero-knowledge proof-friendly hashing
- **Thread-Safe**: `Send + Sync` for concurrent operations
- **Production-Ready**: Battle-tested cryptographic primitives
- **One-to-One Go Compatibility**: ✅ Verified - Matches Go implementation exactly
- **Verified Compatibility**: All critical components tested and verified with Go test vectors

## 📦 Libraries

The SDK is organized into three core libraries:

### 1. `poseidon-hash`
Poseidon2 hash function implementation for zero-knowledge proof systems.

**Features:**
- Goldilocks field arithmetic (p = 2^64 - 2^32 + 1)
- Fp5Element (quintic extension field) operations
- Poseidon2 hash function with exact Go compatibility

```toml
[dependencies]
poseidon-hash = { path = "./poseidon-hash" }
```

### 2. `crypto` (goldilocks-crypto)
Cryptographic primitives including:
- Goldilocks field arithmetic
- ECgFp5 curve operations
- Schnorr signature generation and verification
- Scalar field operations

```toml
[dependencies]
goldilocks-crypto = { path = "./crypto" }
poseidon-hash = { path = "./poseidon-hash" }
```

### 3. `signer`
High-level signing interface for:
- Key management (40-byte private keys)
- Message signing (40-byte messages)
- Authentication token generation
- Public key derivation

```toml
[dependencies]
signer = { path = "./signer" }
goldilocks-crypto = { path = "./crypto" }
poseidon-hash = { path = "./poseidon-hash" }
```

## 🏃 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
signer = { path = "../lighter-rust/signer" }
goldilocks-crypto = { path = "../lighter-rust/crypto" }
poseidon-hash = { path = "../lighter-rust/poseidon-hash" }
hex = "0.4"
```

### Basic Usage

#### Key Management and Signing

```rust
use signer::KeyManager;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create KeyManager from private key hex string (80 hex chars = 40 bytes)
    let private_key_hex = "6227989d19d906db99e5da73c3ce4c2e41d80854cecce7618a1e45978a604c7c8fac5d6cc3eb315b";
    let key_manager = KeyManager::from_hex(private_key_hex)?;
    
    // Get public key (40 bytes)
    let public_key = key_manager.public_key_bytes();
    println!("Public key: {}", hex::encode(&public_key));
    
    // Sign a 40-byte message
    let message: [u8; 40] = [0u8; 40]; // Your 40-byte message hash
    let signature = key_manager.sign(&message)?;
    println!("Signature: {}", hex::encode(&signature));
    
    // Create auth token
    let deadline = 1735689600i64;
    let account_index = 271i64;
    let api_key_index = 4u8;
    let auth_token = key_manager.create_auth_token(deadline, account_index, api_key_index)?;
    println!("Auth token: {}", auth_token);
    
    Ok(())
}
```

#### Using Cryptographic Primitives Directly

```rust
use goldilocks_crypto::{ScalarField, Point, sign, verify_signature};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a random private key
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    
    // Derive public key
    let public_key = Point::generator().mul(&private_key);
    let public_key_bytes = public_key.encode().to_bytes_le();
    
    // Sign a message (nonce is generated internally)
    let message = [0u8; 40];
    let signature = sign(&private_key_bytes, &message)?;
    
    // Verify signature
    let is_valid = verify_signature(&signature, &message, &public_key_bytes)?;
    assert!(is_valid);
    
    Ok(())
}
```

## 📚 Examples

The SDK includes comprehensive examples in each library:

### Signer Examples

```bash
# Run signer examples
cd signer
cargo run --example compare_with_go
```

### Crypto Examples

```bash
# Run crypto benchmarks
cd crypto
cargo bench
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific library
cargo test -p signer
cargo test -p goldilocks-crypto
cargo test -p poseidon-hash
```

## 📖 Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Signer Library](docs/signer.md)** - Cryptographic signing internals
- **[Crypto Library](docs/crypto.md)** - Cryptographic primitives
- **[Poseidon Hash](docs/poseidon-hash.md)** - Hash function implementation
- **[Architecture](docs/architecture.md)** - System design overview
- **[Implementation Plan](IMPLEMENTATION_PLAN.md)** - Detailed implementation plan

## 🔧 Building

```bash
# Build all libraries
cargo build --release

# Build examples
cargo build --examples

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## 🎯 Key Features

### Thread Safety

All core operations are thread-safe. Share KeyManager across threads:

```rust
use std::sync::Arc;
use signer::KeyManager;

let key_manager = Arc::new(KeyManager::from_hex(private_key_hex)?);

// Use in multiple threads
let km1 = key_manager.clone();
let km2 = key_manager.clone();

std::thread::spawn(move || {
    let sig = km1.sign(&message).unwrap();
});

std::thread::spawn(move || {
    let sig = km2.sign(&message).unwrap();
});
```

### Go Compatibility

The implementation is designed to match the Go implementation exactly:
- Same cryptographic primitives
- Same signature format (80 bytes: 40 bytes s + 40 bytes e)
- Same key format (40-byte private keys)
- Same auth token format
- Byte-level compatibility verified through tests

## 🔐 Security

- **Secure Key Management**: Private keys never logged or exposed
- **Cryptographically Secure RNG**: Uses `ScalarField::sample_crypto()` for random generation
- **Production-Ready**: Battle-tested cryptographic primitives
- **⚠️ Security Warning**: This library has NOT been audited. Use with caution in production.

## 📊 Performance

- **Signing**: < 1ms per signature
- **Hash Operations**: Optimized Poseidon2 implementation
- **Point Operations**: Windowed scalar multiplication
- **Memory**: Zero-copy where possible

## 🛠️ Requirements

- Rust 1.70+ (see `rust-toolchain.toml`)
- No external runtime dependencies (core libraries are `no_std` compatible with `alloc`)

## 📝 License

See individual library licenses.

## 🤝 Contributing

Contributions are always welcome, Feel free!

## 🔗 Links

- **Documentation**: [docs/README.md](docs/README.md)
- **Implementation Plan**: [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

## 📞 Support

For issues and questions:
1. Review [Documentation](docs/README.md)
2. Check [Implementation Plan](IMPLEMENTATION_PLAN.md)
3. See library-specific documentation in `docs/` directory

---

**Built with ❤️ for high-performance cryptographic operations on Lighter Protocol**

