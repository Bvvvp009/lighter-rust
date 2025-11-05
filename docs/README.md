# Rust Signer Documentation

Welcome to the Rust Signer documentation. This documentation covers all aspects of using the Rust implementation of the Lighter Protocol signer.

## Getting Started

- **[Getting Started Guide](./getting-started.md)** - Quick start tutorial for integrating the Rust signer into your project

## Running Examples

- **[Running Examples](./running-examples.md)** - Comprehensive guide on how to run all available examples, including prerequisites, troubleshooting, and best practices

## API Reference

- **[API Methods Reference](./api-methods.md)** - Complete API reference covering all available methods, parameters, return types, and usage examples

## Library Documentation

- **[API Client](./api-client.md)** - High-level HTTP client for interacting with the Lighter Protocol API
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

- **Client Initialization**: See [API Methods Reference - Client Initialization](./api-methods.md#client-initialization)
- **Creating Orders**: See [API Methods Reference - Create Market Order](./api-methods.md#1-create-market-order) and [Create Limit Order](./api-methods.md#2-create-limit-order)
- **Key Management**: See [API Methods Reference - Key Management](./api-methods.md#key-management-methods)
- **Running Examples**: See [Running Examples Guide](./running-examples.md)

## Overview

The Rust signer is organized into four main libraries:

1. **`poseidon-hash`** - Poseidon2 hash function implementation
2. **`crypto`** - Cryptographic primitives (Goldilocks field, ECgFp5 curve, Schnorr signatures)
3. **`signer`** - High-level signing interface (KeyManager, transaction signing, auth tokens)
4. **`api-client`** - HTTP client for API interactions (LighterClient)

Each library can be used independently or together for a complete signing solution.
