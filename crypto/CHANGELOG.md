# Changelog

All notable changes to `goldilocks-crypto` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.1] - 2026-03-01

### Added
- **`ScalarField::from_hex` / `to_hex`** — parse and serialise scalars as 80-character
  lowercase hex strings (optional `0x` prefix supported). Both methods include doctests.
- **`AffinePoint` re-export** — `AffinePoint` is now publicly re-exported from the crate
  root (`use goldilocks_crypto::AffinePoint`). It was referenced in documentation but was
  previously unreachable.

### Fixed
- Updated `README.md` to use the correct crate name `goldilocks_crypto` (was `crypto`)
  in all `use` statements, `[dependencies]` snippets, and the docs.rs link.
- Corrected git dependency paths in `README.md` (removed stale `rust-signer/` prefix).

### Changed
- Added `homepage` and `documentation` fields to `Cargo.toml`.

---

## [0.1.0] - 2025-06-01

### Added
- Initial release.
- `ScalarField` — 5-limb (320-bit) scalar arithmetic over the ECgFp5 scalar order; includes
  `add`, `sub`, `mul`, `neg`, Montgomery ladder, `sample_crypto` (CSPRNG), `from_bytes_le`,
  `to_bytes_le`.
- `Point` — extended projective coordinates for ECgFp5 over `Fp5`; includes `add`, `double`,
  `mul` (windowed), `encode`, `decode`.
- `AffinePoint` — affine representation with `encode` / `decode`.
- `sign` / `sign_with_nonce` / `verify_signature` / `sign_hashed_message` — Schnorr signature
  scheme using Poseidon2 as the hash function.
- `validate_public_key` — checks that a byte-encoded public key lies on the curve.
- `batch_verify` — verify multiple signatures in a single call.
- `CryptoError` enum with `thiserror` integration.
- Optional `serde` feature flag.
