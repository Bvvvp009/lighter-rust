# Using Standalone Cryptographic Libraries

The `poseidon-hash` and `goldilocks-crypto` crates can be used independently of the full Lighter Rust SDK. They implement rare Rust primitives for Zero-Knowledge proof systems and are suitable for any project that needs Goldilocks-field cryptography or Poseidon2 hashing.

## `poseidon-hash`

Poseidon2 hash function over the Goldilocks field.

### Add to `Cargo.toml`

```toml
[dependencies]
poseidon-hash = { path = "../lighter-rust/poseidon-hash" }
```

Or from crates.io once published:
```toml
[dependencies]
poseidon-hash = "0.1"
```

### Usage

```rust
use poseidon_hash::{GoldilocksField, poseidon2_hash};

// Hash an array of field elements
let inputs = [GoldilocksField(1), GoldilocksField(2), GoldilocksField(3)];
let digest = poseidon2_hash(&inputs);
println!("Hash: {:?}", digest);
```

### Merkle Tree

```rust
use poseidon_hash::merkle::MerkleTree;

let leaves: Vec<[u8; 40]> = vec![/* your 40-byte leaf data */];
let tree = MerkleTree::new(&leaves);
let root = tree.root();
let proof = tree.proof(0); // proof for leaf at index 0
let valid = MerkleTree::verify(&root, &leaves[0], &proof, 0);
```

---

## `goldilocks-crypto`

Schnorr signatures over the ECgFp5 elliptic curve, defined over the Goldilocks field.

### Add to `Cargo.toml`

```toml
[dependencies]
goldilocks-crypto = { path = "../lighter-rust/crypto" }
```

### Key Generation

```rust
use goldilocks_crypto::{KeyPair, ScalarField};

// Generate a random key pair
let keypair = KeyPair::generate();
let private_key = keypair.private_key(); // &ScalarField
let public_key = keypair.public_key();   // &Point

// From bytes (40-byte little-endian scalar)
let keypair = KeyPair::from_bytes(&private_key_bytes)?;
```

### Signing and Verification

```rust
use goldilocks_crypto::{KeyPair, schnorr};

let keypair = KeyPair::generate();

// Message must be exactly 40 bytes
let message: [u8; 40] = [0u8; 40];

let signature = schnorr::sign(keypair.private_key(), &message)?;
let valid = schnorr::verify(keypair.public_key(), &message, &signature)?;
assert!(valid);
```

### Batch Verification

```rust
use goldilocks_crypto::batch_verify;

// Verify many signatures at once (more efficient than one-by-one)
let pairs: Vec<(&Point, &[u8; 40], &Signature)> = /* ... */;
let all_valid = batch_verify::verify_batch(&pairs)?;
```

---

## Security Notes

- Private keys are 40-byte Goldilocks scalars, **not** standard 32-byte Ethereum private keys.
- These libraries have undergone an internal audit. See [Crypto Internal Audit Report](./crypto-internal-audit-report.md) for details.
- The `crypto` crate uses `zeroize` to clear key material from memory on drop.

## See Also

- [Crypto Documentation](./crypto.md)
- [Poseidon Hash Documentation](./poseidon-hash.md)
- [Crypto Internal Audit Report](./crypto-internal-audit-report.md)
