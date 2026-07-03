# Lighter Rust SDK Documentation

Welcome to the Lighter Rust SDK documentation. This section covers the public crates in the workspace and the recommended entry points for integrating the SDK.

## Getting Started

- **[Getting Started Guide](./getting-started.md)** - Quick start tutorial for integrating the Lighter Rust SDK into your project

## Running Examples

- **[Running Examples](./running-examples.md)** - Guide on how to run all available examples, including prerequisites, troubleshooting, and best practices

## API Reference

- **[API Methods Reference](./api-methods.md)** - Complete API reference covering all available methods, parameters, return types, and usage examples

## Library Documentation

- **[Signer](./signer.md)** - Cryptographic signer for transaction signing and key management
- **[Crypto](./crypto.md)** - Low-level cryptographic primitives (Schnorr signatures, field arithmetic)
- **[Poseidon Hash](./poseidon-hash.md)** - Poseidon2 hash function implementation
- **[Crypto Internal Audit Report](./crypto-internal-audit-report.md)** - Formal internal audit report for `goldilocks-crypto` and `poseidon-hash`

## Architecture & Examples

- **[Architecture](./architecture.md)** - System architecture, design decisions, and component overview
- **[Code Examples](./examples.md)** - Practical code examples and usage patterns

## Troubleshooting

- **[Troubleshooting Guide](./TROUBLESHOOTING.md)** - Common issues and their solutions

## Standalone Libraries

The cryptographic libraries (`poseidon-hash` and `crypto`) can be used independently:

- **[Standalone Libraries Guide](./STANDALONE_LIBRARIES.md)** - Using libraries outside the signer

These libraries implement rare Rust primitives for Zero-Knowledge proof systems.

## Quick Links

- **Trading SDK**: See [lighter-sdk README](../lighter-sdk/README.md)
- **API Client**: See [API Client Documentation](./api-client.md)
- **Key Management**: See [Signer Documentation](./signer.md)
- **Cryptographic Primitives**: See [Crypto Documentation](./crypto.md)
- **Hash Functions**: See [Poseidon Hash Documentation](./poseidon-hash.md)

## Overview

The SDK is organized into five crates arranged in a dependency chain:

1. **`poseidon-hash`** - Poseidon2 hash function and Goldilocks field arithmetic
2. **`crypto`** - Cryptographic primitives (ECgFp5 curve, Schnorr signatures)
3. **`signer`** - Key management, auth tokens, and transaction signing
4. **`api-client`** - HTTP + WebSocket client for all Lighter Exchange endpoints
5. **`lighter-sdk`** - High-level trading SDK (recommended entry point for new integrations)

The lower three libraries (`poseidon-hash`, `crypto`, `signer`) can also be used independently. See [Standalone Libraries Guide](./STANDALONE_LIBRARIES.md).
