# Changelog

## [Unreleased]

### Security
- Hardened Schnorr signing to require canonical, non-zero private keys and exact 40-byte canonical nonces
- Hardened signature verification to reject non-canonical scalar encodings in `s`/`e`
- Added safer `try_inverse()` APIs for `Goldilocks` and `Fp5Element` while preserving backwards-compatible helpers
- Canonicalized `Fp5Element` byte serialization/deserialization to reject malformed Goldilocks limbs
- Expanded crypto property/negative tests for nonce validation, canonical encodings, and inverse round-trips

### Fixed
- Fixed "invalid signature" errors (21120)
- Fixed nonce validation (nonce 0 handling)
- Fixed `check_api_key()` compatibility with current mainnet `/api/v1/apikeys` responses

### Changed
- All transaction types now use `multipart/form-data` encoding
- Automatic nonce management with lock-free atomic operations

### Added
- Optimistic nonce management for high-performance trading
- Spot trading support
- `get_status`, `get_info`, `get_api_keys`, `get_pnl`, `get_order_books`, and `get_trades` read-only SDK methods
- `get_l1_metadata`, `get_lease_options`, `get_leases`, `get_liquidations`, `get_position_funding`, `get_tokens`, `get_exchange_metrics`, `get_execute_stats`, and `export_data` parity methods
- `create_token`, `revoke_token`, `change_account_tier`, `request_faucet`, and `lit_lease` account-management helpers
- `stake_assets`, `unstake_assets`, `approve_integrator`, and related sign-only parity helpers
- new `lighter-sdk` umbrella crate that re-exports the full Rust SDK surface
- 24 comprehensive examples covering perpetual futures and spot trading
- Comprehensive documentation

### Performance
- Improved HFT performance: orders complete in ~200-500ms
- Lock-free nonce management for maximum throughput
