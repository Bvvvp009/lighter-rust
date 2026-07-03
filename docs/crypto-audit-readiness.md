# Crypto Audit Readiness

This note captures the current audit-prep status for `goldilocks-crypto` and `poseidon-hash`.

For the current internal findings, fixes, and go/no-go position, see `docs/crypto-internal-audit-report.md`.

## Scope

The recommended external review scope is:

- `crypto/` (`goldilocks-crypto`)
  - scalar arithmetic
  - ECgFp5 point arithmetic and encoding/decoding
  - Schnorr signing and verification
  - secret handling / zeroization
- `poseidon-hash/`
  - Goldilocks field arithmetic
  - Fp5 extension-field arithmetic
  - Poseidon2 hashing
  - Merkle tree helper logic

## Hardening completed in this pass

- strict rejection of malformed or non-canonical secret scalars
- strict 40-byte canonical nonce enforcement for Schnorr signing
- rejection of non-canonical signature scalar encodings during verification
- new `try_inverse()` APIs for callers that want zero inputs to surface explicitly
- canonical `Fp5Element` byte encoding/decoding via `Goldilocks::to_bytes_le` / `from_bytes_le`
- expanded negative/property-style test coverage for inverse round-trips and malformed input rejection

## Added release-readiness scaffolding

- `crypto/fuzz/` with a `parse_and_verify` libFuzzer target for parsing and signature-validation surfaces
- `poseidon-hash/fuzz/` with a `decode_and_merkle` libFuzzer target for field decoding and Merkle proof surfaces
- Go-reference vector tests in `crypto/tests/go_reference_vectors.rs` and `poseidon-hash/tests/go_reference_vectors.rs`
- optional live integration workflow in `.github/workflows/live-integration.yml` for running ignored network tests when secrets are configured

## External audit checklist

A third-party auditor should still review:

1. Montgomery/scalar multiplication correctness and edge cases
2. EC group law completeness and exceptional-case handling
3. point encoding/decoding soundness and subgroup assumptions
4. constant-time behaviour / timing side-channel exposure
5. cross-language differential validation against the Go implementation and protocol test vectors
6. libFuzzer or cargo-fuzz campaigns over public decoding/parsing surfaces

## Reproducible verification commands

```bash
cargo test -p goldilocks-crypto --all-targets
cargo test -p poseidon-hash --all-targets
cargo test --workspace --lib
```

## Sign-off position

These crates are now materially better prepared for an independent audit, but they should still be described as **internally hardened and test-verified**, not **externally audited**.
