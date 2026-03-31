# Changelog

All notable changes to `poseidon-hash` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.4] - 2026-03-01

### Security
- Added `#![forbid(unsafe_code)]` crate-wide — the compiler now statically rejects any
  `unsafe` block, preventing memory-safety regressions in future contributions.

### Documentation
- Annotated all three round-constant arrays with explicit source references:
  - `EXTERNAL_CONSTANTS` and `INTERNAL_CONSTANTS` — verified against and attributed to
    `poseidon_crypto/hash/poseidon2_goldilocks/config.go` (elliottech/poseidon_crypto).
  - `MATRIX_DIAG_12_U64` — verified against and attributed to the Plonky3 Goldilocks
    Poseidon2 implementation
    (`Plonky3/Plonky3@eeb4e37b/goldilocks/src/poseidon2.rs#L28`).

---

## [0.1.3] - 2026-03-01

### Fixed
- **Critical: `PartialEq` for `Goldilocks`** — Removed `#[derive(PartialEq, Eq)]` from both
  `Goldilocks` and `Fp5Element`; replaced with a manual implementation that compares
  `to_canonical_u64()` values. The old derived impl compared raw `u64` bits, so
  `Goldilocks(p + 1) != Goldilocks(1)` even though both represent the value `1` in the field.
  This silently corrupted every `==` comparison on `Fp5Element`.

### Added
- Expanded `fp5_mul_inverse_is_one_for_base_elements` test — now exercises 6 distinct
  base-field elements `[a, 0, 0, 0, 0]` and verifies `a * a⁻¹ = [1, 0, 0, 0, 0]`.

---

## [0.1.2] - 2025-12-01

### Added
- `Fp5Element::inverse()` — multiplicative inverse over the quintic extension field.
- `Fp5Element::square()` — dedicated squaring (faster than `mul`).
- `Fp5Element::from_uint64_array` — convenient array constructor.
- `serde` feature flag for optional serialization support.

### Changed
- Improved doctest coverage across `Goldilocks` and `Fp5Element`.

---

## [0.1.1] - 2025-09-01

### Added
- `Goldilocks::from_i64` — supports negative integer inputs via modular wrapping.
- `Goldilocks::from_canonical_u64` — explicit canonical input constructor.
- `hash_to_quintic_extension` — Poseidon2 hash returning an `Fp5Element`.

### Fixed
- Modular reduction edge case in `Goldilocks::mul` for products near `2·p`.

---

## [0.1.0] - 2025-06-01

### Added
- Initial release.
- `Goldilocks` field (prime `p = 2^64 - 2^32 + 1`) with `add`, `sub`, `mul`, `neg`, `inverse`.
- `Fp5Element` quintic extension field (`GF(p^5)`) with full arithmetic.
- `poseidon2_hash` — 12-element Poseidon2 permutation with Goldilocks-tuned round constants.
- `no_std`-compatible (requires `alloc`).
