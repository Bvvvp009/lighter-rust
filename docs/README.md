# Rust Signer Documentation

Welcome to the Rust Signer documentation. This documentation covers all aspects of using the Rust implementation of the Lighter Protocol signer.

## Getting Started

- **[Getting Started Guide](./getting-started.md)** - Quick start tutorial for integrating the Rust signer into your project

## Running Examples

- **[Running Examples](./running-examples.md)** - Guide on how to run all available examples, including prerequisites, troubleshooting, and best practices

## API Reference

- **[API Methods Reference](./api-methods.md)** - Complete API reference covering all available methods, parameters, return types, and usage examples

## Library Documentation

- **[Signer](./signer.md)** - Cryptographic signer for transaction signing and key management
- **[Crypto](./crypto.md)** - Low-level cryptographic primitives (Schnorr signatures, field arithmetic)
- **[Poseidon Hash](./poseidon-hash.md)** - Poseidon2 hash function implementation

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

- **Key Management**: See [Signer Documentation](./signer.md)
- **Cryptographic Primitives**: See [Crypto Documentation](./crypto.md)
- **Hash Functions**: See [Poseidon Hash Documentation](./poseidon-hash.md)

## Overview

The Rust signer is organized into three core libraries:

1. **`poseidon-hash`** - Poseidon2 hash function implementation
2. **`crypto`** - Cryptographic primitives (Goldilocks field, ECgFp5 curve, Schnorr signatures)
3. **`signer`** - High-level signing interface (KeyManager, transaction signing, auth tokens)

Each library can be used independently or together for a complete signing solution.
