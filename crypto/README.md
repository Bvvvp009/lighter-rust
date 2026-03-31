# goldilocks-crypto

[![Crates.io](https://img.shields.io/crates/v/goldilocks-crypto.svg)](https://crates.io/crates/goldilocks-crypto)
[![docs.rs](https://docs.rs/goldilocks-crypto/badge.svg)](https://docs.rs/goldilocks-crypto)
[![License: MIT / Apache-2.0](https://img.shields.io/crates/l/goldilocks-crypto.svg)](LICENSE-MIT)

> ### ⚠️ NOT SECURITY AUDITED
> This library has **not** been independently security-audited. Do not use in
> production without a professional cryptographic review. See [Security](#-security).

Rust implementation of **ECgFp5 elliptic curve** operations and **Schnorr
signatures** over the Goldilocks prime field. Uses [`poseidon-hash`](../poseidon-hash)
for Poseidon2-based hashing.

---

## What's Inside

| Type | Description |
|------|-------------|
| `KeyPair` | Random or seed-derived key pair with Schnorr signing |
| `Signature` | Schnorr signature with point-encoded `R` and scalar `s` |
| `ScalarField` | 255-bit scalar arithmetic for private keys and nonces |
| `Point` / `AffinePoint` | ECgFp5 curve point: `add`, `double`, scalar `mul`, `encode` |
| `WeierstrassPoint` | Alternative Weierstrass representation |
| `batch_verify` | Verify multiple `(Signature, message, public_key)` triples |

---

## ⚠️ Security

**This library has NOT been independently security-audited.**

- **Do not deploy to production** without a professional cryptographic review.
- Signature verification passes 84 internal tests but the code has not been
  formally audited.
- Private keys are zeroized on drop via the [`zeroize`](https://crates.io/crates/zeroize)
  crate, but side-channel resistance has not been formally evaluated.
- `#![forbid(unsafe_code)]` is set — no unsafe Rust is used.
- This is an unofficial open-source port and is **not affiliated with or endorsed
  by the Lighter Protocol team**.
- Use at your own risk.

---

## Installation

```toml
[dependencies]
goldilocks-crypto = "0.1"
poseidon-hash     = "0.1"  # pulled in transitively; explicit for clarity
```

With optional serde:

```toml
[dependencies]
goldilocks-crypto = { version = "0.1", features = ["serde"] }
```

---

## Usage

### Generate a key pair

```rust
use goldilocks_crypto::KeyPair;

// Random key pair
let kp = KeyPair::generate();
println!("Public key (40 bytes): {:?}", kp.public_key_bytes());
```

### Deterministic key from seed

```rust
use goldilocks_crypto::KeyPair;

let seed = [0x42u8; 32];
let kp = KeyPair::from_seed_bytes(&seed).unwrap();
```

### Sign and verify

```rust
use goldilocks_crypto::{KeyPair, Signature};

let kp  = KeyPair::generate();
let msg = [0u8; 40]; // 40-byte message (Fp5 digest)

let sig: Signature = kp.sign(&msg).unwrap();
sig.verify(&msg, &kp.public_key_bytes()).unwrap();
```

### Batch verification

```rust
use goldilocks_crypto::{KeyPair, batch_verify};

let triples: Vec<_> = (0u8..8).map(|i| {
    let kp  = KeyPair::generate();
    let mut msg = [0u8; 40];
    msg[0] = i;
    let sig = kp.sign(&msg).unwrap();
    (sig, msg, kp.public_key_bytes())
}).collect();

batch_verify(&triples).unwrap();
```

### Low-level scalar / point operations

```rust
use goldilocks_crypto::{ScalarField, Point};

let sk = ScalarField::sample_crypto();
let pk = Point::generator().mul(&sk);
let enc = pk.encode(); // Fp5Element (40 bytes when serialised)
```

---

## Algorithm

| Property | Value |
|----------|-------|
| Curve | ECgFp5 over GF(p⁵), `p = 2⁶⁴ − 2³² + 1` |
| Signature scheme | Schnorr |
| Hash-to-scalar | Poseidon2 (via `poseidon-hash`) |
| Scalar field order | 255-bit prime |

---

## Running Tests

```bash
cargo test -p goldilocks-crypto --all-targets
```

Expected: **84 tests, 0 failures** (21 unit + 2 integration + 61 security).

---

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | no | `Serialize` / `Deserialize` for public types |

---

## License

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Contributing

Issues and pull requests are welcome.

