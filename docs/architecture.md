# Architecture Overview

High-level architecture and design of the Rust Signer implementation.

## System Architecture

The Rust signer is organized into four layers, each building on the previous:

```
┌─────────────────────────────────────┐
│        API Client Layer             │  HTTP client, transaction building
│      (api-client crate)             │
├─────────────────────────────────────┤
│         Signer Layer                │  Key management, message signing
│        (signer crate)               │
├─────────────────────────────────────┤
│        Crypto Layer                 │  Schnorr signatures, elliptic curves
│        (crypto crate)               │
├─────────────────────────────────────┤
│      Poseidon Hash Layer            │  Hash function, field arithmetic
│     (poseidon-hash crate)           │
└─────────────────────────────────────┘
```

## Layer Descriptions

### 1. Poseidon Hash Layer (`poseidon-hash`)

**Purpose**: Cryptographic hash function and field arithmetic

**Responsibilities**:
- Goldilocks field operations (p = 2^64 - 2^32 + 1)
- Fp5Element (quintic extension field) operations
- Poseidon2 hash function implementation

**Key Types**:
- `GoldilocksField`: Field element operations
- `Fp5Element`: Extension field element
- `poseidon2_hash()`: Hash function

**Dependencies**: None (lowest layer)

### 2. Crypto Layer (`crypto`)

**Purpose**: Cryptographic primitives and signature schemes

**Responsibilities**:
- ECgFp5 elliptic curve operations
- Scalar field arithmetic
- Schnorr signature generation and verification
- Point arithmetic (addition, multiplication)

**Key Types**:
- `ScalarField`: Scalar field element (private keys)
- `Point`: Elliptic curve point (public keys)
- `SchnorrSignature`: Signature structure

**Dependencies**: `poseidon-hash`

### 3. Signer Layer (`signer`)

**Purpose**: High-level key management and signing

**Responsibilities**:
- KeyManager for private/public key pairs
- Message signing with proper formatting
- Auth token generation
- Key serialization/deserialization

**Key Types**:
- `KeyManager`: Main key management struct

**Dependencies**: `crypto`, `poseidon-hash`

### 4. API Client Layer (`api-client`)

**Purpose**: HTTP client for Lighter Exchange API

**Responsibilities**:
- HTTP request handling
- Transaction construction and signing
- Nonce management
- Order submission
- Error handling and retries

**Key Types**:
- `LighterClient`: Main API client
- `CreateOrderRequest`: Order structure

**Dependencies**: `signer`, `crypto`, `poseidon-hash`

## Data Flow

### Transaction Signing Flow

```
1. User creates order request
   ↓
2. API Client constructs transaction JSON
   ↓
3. Transaction hash computed (Poseidon2)
   ↓
4. Signer signs hash (Schnorr signature)
   ↓
5. Signed transaction submitted to API
   ↓
6. Exchange verifies signature
```

### Key Generation Flow

```
1. Generate random ScalarField (private key)
   ↓
2. Multiply by generator point (public key)
   ↓
3. Serialize to bytes/hex format
   ↓
4. Store in KeyManager
```

## Cryptographic Primitives

### Goldilocks Field

Prime field with special properties:
- p = 2^64 - 2^32 + 1
- Optimized for 64-bit operations
- Used as base field for extension

### Fp5Element

Quintic extension field (GF(p^5)):
- Represents 5-tuple of Goldilocks elements
- 40 bytes total (5 × 8 bytes)
- Used for curve operations and hashing

### ECgFp5 Curve

Elliptic curve defined over Fp5:
- Used for Schnorr signatures
- Generator point for key generation
- Point compression (40 bytes)

### Schnorr Signatures

Signature scheme:
- Nonce: Random scalar (r)
- R point: r × Generator
- Challenge: Hash(R || PublicKey || Message)
- Response: r + challenge × private_key
- Signature: (R, response)

## Error Handling

Error types flow upward through layers:

```
PoseidonHashError (poseidon-hash)
    ↓
CryptoError (crypto)
    ↓
SignerError (signer)
    ↓
ApiError (api-client)
```

Each layer wraps errors from lower layers.

## Design Principles

1. **Layering**: Clear separation of concerns
2. **Dependencies**: Lower layers don't depend on higher layers
3. **Type Safety**: Strong typing throughout
4. **Error Handling**: Result types for recoverable errors
5. **Performance**: Optimized operations at each layer
6. **Compatibility**: Follows standard cryptographic specifications

## Security Considerations

1. **Private Keys**: Never exposed outside KeyManager
2. **Random Generation**: Cryptographically secure RNG
3. **Nonce Reuse**: Prevented by design
4. **Signature Verification**: Always verify before trusting
5. **Key Storage**: Application responsibility

## Performance Optimizations

- Windowed scalar multiplication
- Optimized field arithmetic
- Point compression
- Batch operations where possible

## Future Extensibility

The layered design allows for:
- Additional signature schemes
- Different hash functions
- Alternative API clients
- Different key storage backends

## See Also

- [Poseidon Hash Documentation](./poseidon-hash.md)
- [Crypto Documentation](./crypto.md)
- [Signer Documentation](./signer.md)
- [API Client Documentation](./api-client.md)
