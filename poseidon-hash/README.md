# poseidon-hash

[![Crates.io](https://img.shields.io/crates/v/poseidon-hash.svg)](https://crates.io/crates/poseidon-hash)
[![docs.rs](https://docs.rs/poseidon-hash/badge.svg)](https://docs.rs/poseidon-hash)
[![License: MIT / Apache-2.0](https://img.shields.io/crates/l/poseidon-hash.svg)](LICENSE-MIT)

> ### ⚠️ NOT SECURITY AUDITED
> This library has **not** been independently security-audited. Do not use in
> production without a professional cryptographic review. See [Security](#-security).

Rust implementation of **Goldilocks field arithmetic** and the **Poseidon2 hash
function** — the ZK-proof-friendly primitives used by Plonky2 and the Lighter
Protocol.

---

## What's Inside

| Item | Description |
|------|-------------|
| `Goldilocks` | 64-bit prime field `p = 2⁶⁴ − 2³² + 1`: `add`, `sub`, `mul`, `neg`, `inverse`, constant-time compare |
| `Fp5Element` | Quintic extension field GF(p⁵): `add`, `sub`, `mul`, `square`, `inverse`, byte serialisation |
| `poseidon2_hash` | Poseidon2 sponge, width 12 / rate 8, 4+4 full rounds, 22 partial rounds, S-box x⁷ |
| `hash_to_quintic_extension` | Hash a `&[Goldilocks]` slice to a single `Fp5Element` (40 bytes) |
| `merkle::MerkleTree` | Binary Merkle tree built from Poseidon2-hashed leaves |

---

## ⚠️ Security

**This library has NOT been independently security-audited.**

- **Do not deploy to production** without a professional cryptographic review.
- Constants are cross-verified against [`elliottech/poseidon_crypto`](https://github.com/elliottech/poseidon_crypto/blob/main/hash/poseidon2_goldilocks/config.go)
  and [Plonky3](https://github.com/Plonky3/Plonky3/blob/eeb4e37b/goldilocks/src/poseidon2.rs#L28),
  but the Rust code itself has not been formally audited.
- `#![forbid(unsafe_code)]` is set — no unsafe Rust is used.
- This is an unofficial open-source port and is **not affiliated with or endorsed
  by the Lighter Protocol team**.
- Use at your own risk.

---

## Installation

```toml
[dependencies]
poseidon-hash = "0.1"
```

With optional serde support:

```toml
[dependencies]
poseidon-hash = { version = "0.1", features = ["serde"] }
```

---

## Usage

### Goldilocks field arithmetic

```rust
use poseidon_hash::Goldilocks;

let a = Goldilocks::from_canonical_u64(12);
let b = Goldilocks::from_canonical_u64(5);

let sum     = a.add(&b);
let product = a.mul(&b);
let inv_a   = a.inverse();

assert!(a.mul(&inv_a) == Goldilocks::one());
```

### Poseidon2 hash

```rust
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};

let inputs = vec![
    Goldilocks::from_canonical_u64(1),
    Goldilocks::from_canonical_u64(2),
    Goldilocks::from_canonical_u64(3),
];

let digest = hash_to_quintic_extension(&inputs);
let bytes: [u8; 40] = digest.to_bytes_le();
```

### Fp5 extension field

```rust
use poseidon_hash::Fp5Element;

let a = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
let inv = a.inverse();
assert!(a.mul(&inv) == Fp5Element::one());

// Serialise / deserialise
let bytes: [u8; 40] = a.to_bytes_le();
let recovered = Fp5Element::from_bytes_le(&bytes).unwrap();
assert_eq!(a, recovered);
```

### Merkle tree

```rust
use poseidon_hash::{
    Goldilocks, hash_to_quintic_extension,
    merkle::MerkleTree,
};

let leaves: Vec<[u8; 40]> = (0u64..8)
    .map(|i| hash_to_quintic_extension(&[Goldilocks::from_canonical_u64(i)]).to_bytes_le())
    .collect();

let tree  = MerkleTree::new(leaves);
let root  = tree.root();
let proof = tree.proof(3);
assert!(tree.verify(&proof, &tree.leaf(3), 3));
```

---

## Algorithm Parameters

| Parameter | Value |
|-----------|-------|
| Field prime | `2⁶⁴ − 2³² + 1` |
| Sponge width / rate | 12 / 8 |
| Full rounds | 8 (4 + 4) |
| Partial rounds | 22 |
| S-box degree | 7 |

**Constant provenance**
- `EXTERNAL_CONSTANTS`, `INTERNAL_CONSTANTS` —
  [`elliottech/poseidon_crypto` → `hash/poseidon2_goldilocks/config.go`](https://github.com/elliottech/poseidon_crypto/blob/main/hash/poseidon2_goldilocks/config.go)
- `MATRIX_DIAG_12_U64` —
  [Plonky3 `goldilocks/src/poseidon2.rs#L28`](https://github.com/Plonky3/Plonky3/blob/eeb4e37b/goldilocks/src/poseidon2.rs#L28)

---

## Running Tests

```bash
cargo test -p poseidon-hash --all-targets
```

Expected: **59 tests, 0 failures.**

---

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | no | `Serialize` / `Deserialize` for all public types |

---

## `no_std`

This crate is `no_std`-compatible when `alloc` is available. No feature flag
needed — it works out of the box in embedded and WASM targets.

---

## License

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Contributing

Issues and pull requests are welcome.