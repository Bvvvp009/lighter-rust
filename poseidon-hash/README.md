# Poseidon Hash (Goldilocks)

Rust port of Poseidon2 hash function and Goldilocks field arithmetic, ported from lighter-go (Lighter Protocol).

## ⚠️ Security Warning

**This library has NOT been audited and is provided as-is. Use with caution.**

- This is a **prototype port** from the Go SDK (lighter-go)
- **Not security audited** - do not use in production without proper security review
- While the implementation appears to work correctly, cryptographic software requires careful auditing
- This is an open-source contribution and not an official Lighter Protocol library
- Use at your own risk

## Features

- **Goldilocks Field Arithmetic**: Fast field operations with prime `p = 2^64 - 2^32 + 1`
- **Poseidon2 Hash Function**: ZK-friendly hash function optimized for Zero-Knowledge proof systems
- **Fp5 Quintic Extension Field**: 40-byte field elements for cryptographic operations
- **Optimized Performance**: Efficient implementations for production use
- **No Standard Library**: Can be used in `no_std` environments (with `alloc`)

## Overview

This crate provides essential cryptographic primitives for Zero-Knowledge proof systems:

- **Goldilocks Field**: A special prime field optimized for 64-bit CPU operations and ZK systems like Plonky2
- **Poseidon2**: A hash function designed specifically for ZK circuits with low constraint counts
- **Fp5 Extension Field**: Quintic extension field (GF(p^5)) for elliptic curve operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
poseidon-hash = "0.1"
```

Or use the latest version from git (until published):

```toml
[dependencies]
poseidon-hash = { git = "https://github.com/bvvvp009/lighter-rust", path = "rust-signer/poseidon-hash" }
```

## Usage

### Basic Field Arithmetic

```rust
use poseidon_hash::Goldilocks;

let a = Goldilocks::from(42);
let b = Goldilocks::from(10);
let sum = a.add(&b);
let product = a.mul(&b);
```

### Poseidon2 Hashing

```rust
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

let elements = vec![
    Goldilocks::from(1),
    Goldilocks::from(2),
    Goldilocks::from(3),
];
let hash = hash_to_quintic_extension(&elements);
```

### Fp5 Extension Field

```rust
use poseidon_hash::Fp5Element;

let a = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
let b = Fp5Element::one();
let product = a.mul(&b);
let bytes = product.to_bytes_le(); // Returns [u8; 40]
```

## Optional Features

- **`serde`**: Enable serialization/deserialization support

```toml
[dependencies]
poseidon-hash = { version = "0.1", features = ["serde"] }
```

## Integration Guide

### Using in Your Project

1. **Add the dependency** to your `Cargo.toml` (see Installation above)

2. **Import the types** you need:

```rust
use poseidon_hash::{Goldilocks, Fp5Element, hash_to_quintic_extension};
```

3. **Use field arithmetic** for ZK circuit operations:

```rust
// Create field elements
let a = Goldilocks::from_canonical_u64(42);
let b = Goldilocks::from_canonical_u64(10);

// Perform operations
let sum = a.add(&b);
let product = a.mul(&b);
let inverse = a.inverse();

// Check properties
assert!(!a.is_zero());
assert_eq!(Goldilocks::zero().is_zero(), true);
```

4. **Hash data** for ZK proofs:

```rust
// Prepare input elements
let inputs = vec![
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
];

// Hash to Fp5Element (40 bytes)
let hash = hash_to_quintic_extension(&inputs);
let hash_bytes = hash.to_bytes_le();
```

5. **Work with extension fields**:

```rust
// Create Fp5 elements
let elem1 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
let elem2 = Fp5Element::one();

// Operations
let sum = elem1.add(&elem2);
let product = elem1.mul(&elem2);
let square = elem1.square();
let inverse = elem1.inverse();

// Serialization
let bytes = elem1.to_bytes_le(); // [u8; 40]
```

### Common Patterns

**Merkle Tree Construction:**
```rust
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};

fn merkle_hash(left: &[u8; 40], right: &[u8; 40]) -> [u8; 40] {
    // Convert bytes to Goldilocks elements
    let left_elems: Vec<Goldilocks> = left.chunks(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(chunk);
            Goldilocks::from_canonical_u64(u64::from_le_bytes(arr))
        })
        .collect();
    
    let right_elems: Vec<Goldilocks> = right.chunks(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(chunk);
            Goldilocks::from_canonical_u64(u64::from_le_bytes(arr))
        })
        .collect();
    
    // Combine and hash
    let combined: Vec<Goldilocks> = left_elems.into_iter()
        .chain(right_elems.into_iter())
        .collect();
    
    hash_to_quintic_extension(&combined).to_bytes_le()
}
```

**Converting Integers to Field Elements:**
```rust
use poseidon_hash::Goldilocks;

// From u64
let elem = Goldilocks::from_canonical_u64(12345);

// From i64 (handles negatives)
let neg_elem = Goldilocks::from_i64(-10);

// From u64 using From trait
let elem: Goldilocks = 42u64.into();
```

## Use Cases

- Zero-Knowledge proof systems (Plonky2, STARKs)
- Cryptographic research and protocol development
- Blockchain protocols requiring ZK-friendly hashing
- Merkle tree construction for ZK systems
- Commitment schemes and hash-based signatures
- Elliptic curve cryptography over extension fields

## Performance

The implementation is optimized for:
- Fast modular reduction using Goldilocks prime properties
- Efficient field arithmetic operations
- Low memory allocations
- Production-grade performance

## Security Considerations

⚠️ **Important**: This library has NOT been security audited. Use with caution in production systems.

- **Audit Status**: This is a prototype port from lighter-go and requires security review before production use
- **Hash Function**: Poseidon2 is designed for ZK-proof systems but this implementation needs auditing
- **Field Operations**: Ensure proper input validation and bounds checking in your application

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/poseidon-hash).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

This library is ported from GO-SDK of the [Lighter Protocol](https://github.com/elliottech/lighter-go) project. Feel free to open a PR or an issue.