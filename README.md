# Rust Signer for Lighter Protocol

A fast, secure, and reliable Rust implementation of the Lighter Protocol signer. This crate provides everything you need to interact with the Lighter Protocol API, including transaction signing, order management, and authentication.

## Features

- 🔐 **Secure Signing**: Schnorr signatures with Goldilocks field arithmetic
- ⚡ **High Performance**: Native Rust implementation with optimized cryptographic operations
- 🛡️ **Type Safe**: Strong compile-time guarantees for API correctness
- 📦 **Modular**: Use individual libraries or the complete solution
- 🔑 **Key Management**: Built-in secure key generation and management

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api-client = { path = "rust-signer/api-client" }
signer = { path = "rust-signer/signer" }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
serde_json = "1.0"
```

### Basic Usage

Create a market order:

```rust
use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize client
    let client = LighterClient::new(
        env::var("BASE_URL")?,
        &env::var("API_PRIVATE_KEY")?,
        env::var("ACCOUNT_INDEX")?.parse()?,
        env::var("API_KEY_INDEX")?.parse()?,
    )?;
    
    // Create a market buy order
    let response = client.create_market_order(
        0,           // market_index (0 = default market)
        12345,       // client_order_index (unique ID)
        1000,        // base_amount (order size)
        450000,      // avg_execution_price (max price)
        false,       // is_ask (false = buy, true = sell)
    ).await?;
    
    println!("Order submitted: {:?}", response);
    Ok(())
}
```

### Environment Setup

Create a `.env` file:

```bash
BASE_URL=https://testnet.zklighter.elliot.ai
API_PRIVATE_KEY=0x0123456789abcdef...
ACCOUNT_INDEX=1
API_KEY_INDEX=0
```

## Common Operations

### Create a Limit Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

let order = CreateOrderRequest {
    account_index: 1,
    order_book_index: 0,
    client_order_index: 12345,
    base_amount: 1000,
    price: 450000,
    is_ask: false,      // false = buy order
    order_type: 0,      // 0 = LIMIT
    time_in_force: 1,   // 1 = GOOD_TILL_TIME
    reduce_only: false,
    trigger_price: 0,
};

let response = client.create_order(order).await?;
```

### Cancel an Order

```rust
let response = client.cancel_order(
    0,      // market_index
    12345,  // order_index
).await?;
```

### Cancel All Orders

```rust
let response = client.cancel_all_orders(
    0,  // time_in_force (0 = immediate)
    0,  // time parameter
).await?;
```

### Generate Authentication Token

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(&env::var("API_PRIVATE_KEY")?)?;

let deadline = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)?
    .as_secs() as i64 + 600; // 10 minutes from now

let token = key_manager.create_auth_token(
    deadline,
    env::var("ACCOUNT_INDEX")?.parse()?,
    env::var("API_KEY_INDEX")?.parse()?,
)?;

println!("Auth token: {}", token);
```

### Generate New Key Pair

```rust
use signer::KeyManager;

// Generate a new random key pair
let key_manager = KeyManager::generate();

let private_key = hex::encode(key_manager.private_key_bytes());
let public_key = hex::encode(key_manager.public_key_bytes());

println!("Private key: 0x{}", private_key);
println!("Public key: 0x{}", public_key);
```

## Library Structure

The Rust signer is organized into four libraries:

### 1. `poseidon-hash`
Cryptographic hash function implementation for Poseidon2.

```rust
use poseidon_hash::{hash_to_quintic_extension, Goldilocks};

let elements = vec![Goldilocks::from(1), Goldilocks::from(2)];
let hash = hash_to_quintic_extension(&elements);
```

### 2. `crypto`
Low-level cryptographic primitives (field arithmetic, elliptic curves, Schnorr signatures).

```rust
use crypto::schnorr::sign_with_nonce;

let signature = sign_with_nonce(&private_key, &message, &nonce)?;
```

### 3. `signer`
High-level key management and signing interface.

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex("0x...")?;
let signature = key_manager.sign(&message_hash)?;
```

### 4. `api-client`
HTTP client for Lighter Protocol API interactions.

```rust
use api_client::LighterClient;

let client = LighterClient::new(base_url, private_key, account_index, api_key_index)?;
let response = client.create_market_order(...).await?;
```

## Examples

Run the included examples:

```bash
# Create a market order
cargo run --example create_market_order --release

# Create a limit order
cargo run --example create_limit_order --release

# Cancel an order
cargo run --example cancel_order --release

# Generate auth token
cargo run --example create_auth_token --release

# Setup new API key
cargo run --example setup_api_key --release
```

See [docs/running-examples.md](docs/running-examples.md) for all available examples.

## Documentation

- **[Getting Started](docs/getting-started.md)** - Integration guide
- **[API Reference](docs/api-methods.md)** - Complete API documentation
- **[Running Examples](docs/running-examples.md)** - How to run examples
- **[Architecture](docs/architecture.md)** - System design overview

## Order Types

| Type | Value | Description |
|------|-------|-------------|
| LIMIT | 0 | Limit order at specific price |
| MARKET | 1 | Market order at current price |
| STOP_LOSS | 2 | Stop loss order |
| STOP_LOSS_LIMIT | 3 | Stop loss limit order |
| TAKE_PROFIT | 4 | Take profit order |
| TAKE_PROFIT_LIMIT | 5 | Take profit limit order |
| TWAP | 6 | Time-weighted average price order |

## Time in Force

| Type | Value | Description |
|------|-------|-------------|
| IMMEDIATE_OR_CANCEL | 0 | Execute immediately or cancel |
| GOOD_TILL_TIME | 1 | Valid until expiry |
| FILL_OR_KILL | 2 | Execute fully or cancel |
| POST_ONLY | 3 | Only add liquidity |

## Error Handling

All API methods return `Result<T, E>`:

```rust
match client.create_market_order(...).await {
    Ok(response) => {
        println!("Success: {:?}", response);
        // Extract transaction hash
        if let Some(tx_hash) = response.get("tx_hash") {
            println!("Transaction: {}", tx_hash);
        }
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Handle error appropriately
    }
}
```

## Security Best Practices

1. **Never commit private keys**: Use environment variables or secure key storage
2. **Use testnet first**: Always test on testnet before using mainnet
3. **Validate inputs**: Check all order parameters before submission
4. **Handle errors**: Always handle API errors gracefully
5. **Key rotation**: Regularly rotate API keys for security

## Requirements

- Rust 1.70 or higher
- Tokio runtime (for async operations)
- Valid Lighter Protocol account and API key

## License

[Your License Here]

## Contributing

[Contributing Guidelines]

## Standalone Libraries

The `poseidon-hash` and `crypto` crates are **valuable standalone libraries** that can be used independently:

### `poseidon-hash` - Goldilocks Field + Poseidon2 Hash
- **Goldilocks field arithmetic** (p = 2^64 - 2^32 + 1)
- **Poseidon2 hash function** (ZK-friendly)
- **Fp5 quintic extension field**
- **Use cases**: ZK proof systems, blockchain L2s, privacy applications

### `crypto` - ECgFp5 Curve + Schnorr Signatures
- **ECgFp5 elliptic curve** operations
- **Schnorr signatures** (Poseidon2-based)
- **Scalar field arithmetic**
- **Use cases**: Digital signatures, authentication, wallets

**These are rare Rust implementations** - Most ZK libraries are in Go/C++/Python. This is production-ready Rust code used in live systems.

See [Standalone Libraries Guide](docs/STANDALONE_LIBRARIES.md) for details on using these independently.

## Support

For issues and questions:
- Check the [documentation](docs/)
- Review [troubleshooting guide](docs/TROUBLESHOOTING.md)
- Open an issue on GitHub

