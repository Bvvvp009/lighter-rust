# Changelog

## [Unreleased]

### Fixed
- Fixed "invalid signature" errors (21120)
- Fixed nonce validation (nonce 0 handling)

### Changed
- All transaction types now use `multipart/form-data` encoding
- Automatic nonce management with lock-free atomic operations

### Added
- Optimistic nonce management for high-performance trading
- Spot trading support
- 24 comprehensive examples covering perpetual futures and spot trading
- Comprehensive documentation

### Performance
- Improved HFT performance: orders complete in ~200-500ms
- Lock-free nonce management for maximum throughput
