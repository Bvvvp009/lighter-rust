# Crypto Library

The `crypto` crate provides cryptographic primitives including Schnorr signatures, elliptic curve operations, and scalar field arithmetic over the ECgFp5 curve.

## Quick Start

```rust
use crypto::{ScalarField, Point, sign_with_nonce, verify_signature};

// Generate a random private key
let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le(); // [u8; 40]

// Derive public key
let public_key = Point::generator().mul(&private_key);
let public_key_bytes = public_key.encode().to_bytes_le(); // [u8; 40]

// Sign a message (40 bytes)
let message = [0u8; 40];
let nonce = ScalarField::sample_crypto();
let nonce_bytes = nonce.to_bytes_le();
let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();

// Verify signature
let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
assert!(is_valid);
```

See also: [Getting Started Guide](./getting-started.md) | [Examples](./examples.md) | [Poseidon Hash](./poseidon-hash.md)

## Overview

This library implements:
- **ECgFp5 Curve**: Elliptic curve defined over the quintic extension field
- **Schnorr Signatures**: Signature scheme over ECgFp5 using Poseidon2 hashing
- **Scalar Field Operations**: Operations on the curve's scalar field (320-bit)
- **Point Arithmetic**: Elliptic curve point addition and multiplication

## Installation

```toml
[dependencies]
crypto = "0.1"
poseidon-hash = "0.1"  # Required dependency
```

Or from git:

```toml
[dependencies]
crypto = { git = "https://github.com/elliottech/lighter-rust", path = "rust-signer/crypto" }
poseidon-hash = { git = "https://github.com/elliottech/lighter-rust", path = "rust-signer/poseidon-hash" }
```

## Basic Usage

### Schnorr Signatures

```rust
use crypto::{ScalarField, Point, sign_with_nonce, verify_signature};

// Generate a random scalar (private key)
let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

// Derive public key
let public_key = Point::generator().mul(&private_key);
let public_key_bytes = public_key.encode().to_bytes_le();

// Sign a message (must be 40 bytes)
let message = [0u8; 40];
let nonce = ScalarField::sample_crypto();
let nonce_bytes = nonce.to_bytes_le();
let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();

// Verify the signature
let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
assert!(is_valid);
```

### Scalar Field Operations

```rust
use crypto::ScalarField;

// Generate a random scalar (cryptographically secure)
let scalar = ScalarField::sample_crypto();

// Create scalar from bytes (40 bytes, little-endian)
let bytes: [u8; 40] = [0u8; 40];
let scalar = ScalarField::from_bytes_le(&bytes).unwrap();

// Create from hex string (80 hex characters = 40 bytes)
let hex_str = "0".repeat(80);
let scalar = ScalarField::from_hex(&hex_str).unwrap();

// Arithmetic operations
let a = ScalarField::sample_crypto();
let b = ScalarField::sample_crypto();
let sum = a.add(b);
let product = a.mul(b);
let negated = a.neg();

// Constants
let zero = ScalarField::ZERO;
let one = ScalarField::ONE;
let two = ScalarField::TWO;

// Convert to bytes
let bytes: [u8; 40] = scalar.to_bytes_le();
```

### Elliptic Curve Points

```rust
use crypto::{Point, ScalarField};

// Get the generator point
let generator = Point::generator();

// Get neutral (identity) point
let neutral = Point::neutral();

// Scalar multiplication (generate public key from private key)
let private_key = ScalarField::sample_crypto();
let public_key = generator.mul(&private_key);

// Point addition
let point1 = generator.mul(&ScalarField::ONE);
let point2 = generator.mul(&ScalarField::TWO);
let sum = point1.add(&point2);

// Point doubling
let doubled = point1.double();

// Encode point to Fp5Element (40 bytes)
let encoded = public_key.encode();
let point_bytes: [u8; 40] = encoded.to_bytes_le();

// Decode Fp5Element back to point
let decoded = Point::decode(&encoded);
```

## API Reference

### Exported Types and Functions

The main exports from the `crypto` crate are:

- **`ScalarField`**: Scalar field element (320-bit, 5 limbs)
- **`Point`**: Elliptic curve point on ECgFp5
- **`AffinePoint`**: Affine representation of a point
- **`sign_with_nonce`**: Sign a message with a given nonce
- **`verify_signature`**: Verify a Schnorr signature
- **`Goldilocks`**: Re-exported from `poseidon-hash`
- **`Fp5Element`**: Re-exported from `poseidon-hash`
- **`CryptoError`**: Error type for cryptographic operations
- **`Result<T>`**: Result type alias for `Result<T, CryptoError>`

### ScalarField

Represents an element in the scalar field of the elliptic curve (320 bits, represented as 5 u64 limbs).

#### Creating Scalars

```rust
use crypto::ScalarField;

// Random scalar (cryptographically secure)
let scalar = ScalarField::sample_crypto();

// From bytes (40 bytes, little-endian)
let bytes: [u8; 40] = [0u8; 40];
let scalar = ScalarField::from_bytes_le(&bytes).unwrap();

// From hex string (80 hex characters = 40 bytes)
let hex_str = "0".repeat(80);
let scalar = ScalarField::from_hex(&hex_str).unwrap();

// From Fp5Element
use crypto::Fp5Element;
let fp5 = Fp5Element::one();
let scalar = ScalarField::from_fp5_element(&fp5);

// Constants
let zero = ScalarField::ZERO;
let one = ScalarField::ONE;
let two = ScalarField::TWO;
```

#### Operations

```rust
let a = ScalarField::sample_crypto();
let b = ScalarField::sample_crypto();

// Addition
let sum = a.add(b);

// Subtraction
let diff = a.sub(b);

// Multiplication
let product = a.mul(b);

// Negation
let negated = a.neg();

// Comparison
let is_equal = a.equals(&b);
let is_zero = a.is_zero();
```

#### Conversion

```rust
let scalar = ScalarField::sample_crypto();

// To bytes (40 bytes, little-endian)
let bytes: [u8; 40] = scalar.to_bytes_le();

// To bytes (40 bytes, with padding to 40 bytes)
let bytes: [u8; 40] = scalar.to_bytes();
```

### Point

Represents a point on the ECgFp5 elliptic curve in projective coordinates (x, z, u, t).

#### Creating Points

```rust
use crypto::{Point, ScalarField, Fp5Element};

// Generator point (base point)
let generator = Point::generator();

// Neutral (identity) element
let neutral = Point::neutral();

// From scalar (public key from private key)
let private_key = ScalarField::sample_crypto();
let public_key = Point::generator().mul(&private_key);

// From projective coordinates
let x = Fp5Element::zero();
let z = Fp5Element::one();
let u = Fp5Element::zero();
let t = Fp5Element::one();
let point = Point::new(x, z, u, t);

// Decode from Fp5Element
let encoded = Fp5Element::one();
let point = Point::decode(&encoded);
```

#### Operations

```rust
use crypto::{Point, ScalarField};

let generator = Point::generator();
let scalar = ScalarField::sample_crypto();

// Scalar multiplication
let point = generator.mul(&scalar);

// Point addition
let p1 = generator.mul(&ScalarField::ONE);
let p2 = generator.mul(&ScalarField::TWO);
let sum = p1.add(&p2);

// Point doubling
let doubled = p1.double();

// Add affine point
use crypto::AffinePoint;
let affine = AffinePoint::neutral();
let result = p1.add_affine(&affine);

// Multiple doublings
let result = p1.set_m_double(5); // 2^5 * p1

// Check if neutral
let is_neutral = point.is_neutral();

// Equality check
let are_equal = point1.equals(&point2);
```

#### Conversion

```rust
use crypto::{Point, Fp5Element};

let point = Point::generator();

// Encode to Fp5Element (40 bytes)
let encoded: Fp5Element = point.encode();
let bytes: [u8; 40] = encoded.to_bytes_le();

// Convert to affine coordinates
let affine = point.to_affine_single();
```

### Schnorr Signatures

The library provides two main functions for Schnorr signatures:

- **`sign_with_nonce`**: Sign a message using a private key and nonce
- **`verify_signature`**: Verify a signature against a message and public key

#### Signing

```rust
use crypto::{ScalarField, sign_with_nonce};

let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

let message = [0u8; 40]; // Must be exactly 40 bytes

// Generate nonce (CRITICAL: must be cryptographically secure and unique per message)
let nonce = ScalarField::sample_crypto();
let nonce_bytes = nonce.to_bytes_le();

// Sign (returns 80 bytes: 40 bytes s + 40 bytes e)
let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
```

#### Verification

```rust
use crypto::{ScalarField, Point, verify_signature};

let private_key = ScalarField::sample_crypto();
let private_key_bytes = private_key.to_bytes_le();

// Derive public key
let public_key = Point::generator().mul(&private_key);
let public_key_bytes = public_key.encode().to_bytes_le();

let message = [0u8; 40];
let signature = /* ... */;

// Verify signature (returns bool)
let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
```

#### Signature Format

Signatures are 80 bytes total:
- First 40 bytes: response scalar `s` (little-endian)
- Last 40 bytes: challenge scalar `e` (little-endian)

## Advanced Usage

### Affine Points

For efficient batch operations, use `AffinePoint`:

```rust
use crypto::{Point, AffinePoint};

let point = Point::generator();

// Convert to affine
let affine = point.to_affine_single();

// Create window of affine points for efficient scalar multiplication
let window = point.make_window_affine();

// Batch convert multiple points to affine
let points = vec![Point::generator(), Point::neutral()];
let affine_points = Point::batch_to_affine(&points);
```

### Curve Constants

The library exports curve constants for advanced use:

```rust
use crypto::schnorr::{B_ECG_FP5_POINT, B_MUL2_ECG_FP5_POINT, B_MUL4_ECG_FP5_POINT, B_MUL16_ECG_FP5_POINT};

// These are precomputed multiples of the curve parameter B
// Used internally for efficient point operations
```

### Error Handling

```rust
use crypto::{sign_with_nonce, CryptoError};

match sign_with_nonce(&private_key, &message, &nonce) {
    Ok(signature) => {
        // Success
    }
    Err(CryptoError::InvalidPrivateKeyLength(len)) => {
        eprintln!("Invalid key length: {}", len);
    }
    Err(CryptoError::InvalidMessageLength(len)) => {
        eprintln!("Invalid message length: {}", len);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Error Handling

```rust
use crypto::CryptoError;

// Most operations return Result types for error handling
match Point::from_bytes(&invalid_bytes) {
    Ok(point) => {
        // Use point
    }
    Err(CryptoError::InvalidPoint) => {
        // Handle invalid point
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Security Considerations

1. **Private Keys**: Never expose private keys. Use `ScalarField::sample_crypto()` for generating secure random keys.
2. **Nonces**: In production, always use cryptographically secure random nonces. **Never reuse nonces** - each signature must have a unique nonce.
3. **Message Format**: Messages must be exactly 40 bytes (5 Goldilocks field elements).
4. **Verification**: Always verify signatures before trusting messages.
5. **Key Storage**: Store private keys securely. Never commit them to version control.

## Performance

- Point operations are optimized for the ECgFp5 curve
- Scalar multiplication uses windowed method (window size 5) for efficiency
- Signature operations are designed for high throughput
- Batch affine conversion is optimized for multiple points

## Related Documentation

- **[Poseidon Hash Library](./poseidon-hash.md)** - Underlying hash function used for signatures
- **[Signer Library](./signer.md)** - High-level signing API using this library
- **[API Client](./api-client.md)** - Integration with Lighter Exchange API
- **[Getting Started](./getting-started.md)** - Quick start integration guide
- **[Examples](./examples.md)** - Code examples and usage patterns
