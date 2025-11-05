# Poseidon Hash Library

The `poseidon-hash` crate provides the Poseidon2 hash function implementation along with Goldilocks field arithmetic and Fp5Element operations.

## Quick Start

```rust
use poseidon_hash::{Goldilocks, Fp5Element, hash_to_quintic_extension};

// Field arithmetic
let a = Goldilocks::from_canonical_u64(42);
let b = Goldilocks::from_canonical_u64(10);
let sum = a.add(&b);
let product = a.mul(&b);

// Poseidon2 hashing
let elements = vec![
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
];
let hash = hash_to_quintic_extension(&elements);

// Fp5 extension field
let fp5 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
let bytes = fp5.to_bytes_le(); // [u8; 40]
```

See also: [Getting Started Guide](./getting-started.md) | [Crypto Library](./crypto.md) | [Examples](./examples.md)

## Overview

This library implements:
- **Poseidon2 Hash**: A cryptographic hash function optimized for zero-knowledge proofs
- **Goldilocks Field**: Prime field with p = 2^64 - 2^32 + 1
- **Fp5Element**: Quintic extension field (GF(p^5)) operations

## Installation

```toml
[dependencies]
poseidon-hash = "0.1"
```

Or from git:

```toml
[dependencies]
poseidon-hash = { git = "https://github.com/elliottech/lighter-rust", path = "rust-signer/poseidon-hash" }
```

## Basic Usage

### Goldilocks Field Element

```rust
use poseidon_hash::Goldilocks;

// Create a Goldilocks field element
let element = Goldilocks::from_canonical_u64(42);

// Or use From trait
let element: Goldilocks = 42u64.into();

// Field arithmetic
let a = Goldilocks::from_canonical_u64(10);
let b = Goldilocks::from_canonical_u64(5);
let sum = a.add(&b);
let product = a.mul(&b);
let inverse = a.inverse();

// Constants
let zero = Goldilocks::zero();
let one = Goldilocks::one();
```

### Fp5Element (Quintic Extension)

```rust
use poseidon_hash::{Fp5Element, Goldilocks};

// Create an Fp5Element from u64 array
let fp5 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);

// Or from Goldilocks elements
let fp5 = Fp5Element([
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
    Goldilocks::from_canonical_u64(4),
    Goldilocks::from_canonical_u64(5),
]);

// Fp5 arithmetic
let a = Fp5Element::zero();
let b = Fp5Element::one();
let sum = a.add(&b);
let product = a.mul(&b);
let inverse = a.inverse();

// Convert to bytes
let bytes: [u8; 40] = fp5.to_bytes_le();
```

### Poseidon2 Hash

```rust
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

// Hash Goldilocks elements to Fp5Element (40 bytes)
let elements = vec![
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
];
let hash = hash_to_quintic_extension(&elements);

// Convert hash to bytes
let hash_bytes: [u8; 40] = hash.to_bytes_le();
```

## API Reference

### Exported Types and Functions

The main exports from the `poseidon-hash` crate are:

- **`Goldilocks`**: Goldilocks field element (base field)
- **`Fp5Element`**: Quintic extension field element (40 bytes)
- **`hash_to_quintic_extension`**: Poseidon2 hash function
- **`permute`**: Poseidon2 permutation function (advanced use)

### Goldilocks

The Goldilocks field uses prime modulus p = 2^64 - 2^32 + 1, optimized for 64-bit CPU operations.

#### Creating Elements

```rust
use poseidon_hash::Goldilocks;

// From u64 (canonical form)
let x = Goldilocks::from_canonical_u64(42);

// From i64 (handles negatives)
let neg = Goldilocks::from_i64(-10);

// Using From trait
let x: Goldilocks = 42u64.into();

// Constants
let zero = Goldilocks::zero();
let one = Goldilocks::one();

// Field modulus
let modulus = Goldilocks::MODULUS; // 0xffffffff00000001
```

#### Arithmetic Operations

```rust
use poseidon_hash::Goldilocks;

let a = Goldilocks::from_canonical_u64(10);
let b = Goldilocks::from_canonical_u64(5);

// Addition
let sum = a.add(&b);

// Subtraction
let diff = a.sub(&b);

// Multiplication
let product = a.mul(&b);

// Square
let square = a.square();

// Double
let doubled = a.double();

// Multiplicative inverse (panics if zero)
let inverse = a.inverse();

// Check if zero
let is_zero = a.is_zero();
```

#### Conversion

```rust
use poseidon_hash::Goldilocks;

let element = Goldilocks::from_canonical_u64(42);

// To canonical u64
let value: u64 = element.to_canonical_u64();

// Access raw value
let raw: u64 = element.0;
```

### Fp5Element

Represents an element of the quintic extension field GF(p^5) where p is the Goldilocks prime. Each element is 40 bytes (5 Goldilocks field elements).

#### Creating Elements

```rust
use poseidon_hash::{Fp5Element, Goldilocks};

// From u64 array
let fp5 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);

// From Goldilocks array
let fp5 = Fp5Element([
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
    Goldilocks::from_canonical_u64(4),
    Goldilocks::from_canonical_u64(5),
]);

// Constants
let zero = Fp5Element::zero();
let one = Fp5Element::one();

// Access coefficients
let coeffs = fp5.0; // [Goldilocks; 5]
```

#### Arithmetic Operations

```rust
use poseidon_hash::Fp5Element;

let a = Fp5Element::one();
let b = Fp5Element::zero();

// Addition
let sum = a.add(&b);

// Subtraction
let diff = a.sub(&b);

// Multiplication
let product = a.mul(&b);

// Square (optimized)
let square = a.square();

// Double
let doubled = a.double();

// Multiplicative inverse (returns zero if input is zero)
let inverse = a.inverse();

// Scalar multiplication (multiply by base field element)
use poseidon_hash::Goldilocks;
let scalar = Goldilocks::from_canonical_u64(5);
let scaled = a.scalar_mul(&scalar);

// Negation
let negated = a.neg();

// Check if zero
let is_zero = a.is_zero();
```

#### Conversion

```rust
use poseidon_hash::Fp5Element;

let fp5 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);

// To bytes (40 bytes, little-endian)
let bytes: [u8; 40] = fp5.to_bytes_le();

// Access coefficients
let coeffs = fp5.0; // [Goldilocks; 5]
```

### Poseidon2 Hash Function

```rust
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

// Hash Goldilocks elements to Fp5Element
let inputs = vec![
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
];
let hash: Fp5Element = hash_to_quintic_extension(&inputs);

// Convert to bytes
let hash_bytes: [u8; 40] = hash.to_bytes_le();
```

**Note**: The hash function takes a slice of `Goldilocks` elements and returns an `Fp5Element` (40 bytes). This is the standard way to hash data for use in cryptographic operations.

## Advanced Usage

### Frobenius Automorphism

For advanced field operations, you can use the Frobenius automorphism:

```rust
use poseidon_hash::Fp5Element;

let elem = Fp5Element::one();

// Apply Frobenius once
let frob = elem.frobenius();

// Apply Frobenius multiple times
let frob_n = elem.repeated_frobenius(3);
```

### Converting Bytes to Field Elements

When hashing arbitrary data, convert bytes to Goldilocks elements:

```rust
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

fn hash_bytes(data: &[u8]) -> [u8; 40] {
    // Convert bytes to Goldilocks elements (8 bytes each)
    let elements: Vec<Goldilocks> = data
        .chunks(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            arr[..chunk.len()].copy_from_slice(chunk);
            Goldilocks::from_canonical_u64(u64::from_le_bytes(arr))
        })
        .collect();
    
    // Hash to Fp5Element
    let hash = hash_to_quintic_extension(&elements);
    hash.to_bytes_le()
}
```

### Poseidon2 Permutation

For advanced use cases, you can directly use the permutation function:

```rust
use poseidon_hash::{permute, Goldilocks};

// Permutation operates on 12-element state
let mut state = [Goldilocks::zero(); 12];
state[0] = Goldilocks::from_canonical_u64(1);
state[1] = Goldilocks::from_canonical_u64(2);
// ... set other elements

// Apply permutation
permute(&mut state);
```

## Performance Considerations

- Goldilocks field operations are optimized for the specific prime (p = 2^64 - 2^32 + 1)
- Fast modular reduction using epsilon optimization
- Fp5Element operations use optimized extension field arithmetic
- Poseidon2 is designed for efficiency in zero-knowledge proof systems
- Low memory allocations for production use

## Error Handling

Most operations are infallible for valid inputs. When converting from bytes or external formats, ensure:
- Byte arrays are the correct length (8 bytes for Goldilocks, 40 bytes for Fp5)
- Values are within the field range (0 to MODULUS for Goldilocks)
- Inputs to hash functions are properly formatted

## Security Considerations

1. **Field Modulus**: The Goldilocks prime is carefully chosen for cryptographic security
2. **Hash Function**: Poseidon2 is designed for ZK-proof systems and provides strong security guarantees
3. **Extension Field**: The quintic extension field is used for elliptic curve operations

## Related Documentation

- **[Crypto Library](./crypto.md)** - Uses poseidon-hash for Schnorr signatures and elliptic curve operations
- **[Signer Library](./signer.md)** - High-level signing API using these primitives
- **[Getting Started](./getting-started.md)** - Quick start integration guide
- **[Examples](./examples.md)** - Code examples and usage patterns
