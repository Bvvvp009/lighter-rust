# Lighter Rust SDK

A high-performance Rust implementation of the Lighter Protocol signer, providing cryptographic primitives and API client functionality for trading on the Lighter Exchange.

## 🚀 Features

- **High-Performance Signing**: Optimized Schnorr signature generation using Goldilocks field arithmetic
- **Poseidon2 Hashing**: Efficient zero-knowledge proof-friendly hashing
- **Automatic Nonce Management**: Lock-free atomic operations for optimal HFT performance
- **Comprehensive API Client**: Full support for perpetual futures and spot trading
- **Thread-Safe**: `Send + Sync` for concurrent operations
- **Production-Ready**: Battle-tested with comprehensive examples

## 📦 Libraries

The SDK is organized into four main libraries:

### 1. `poseidon-hash`
Poseidon2 hash function implementation for zero-knowledge proof systems.

```toml
[dependencies]
poseidon-hash = { path = "./poseidon-hash" }
```

### 2. `crypto`
Cryptographic primitives including:
- Goldilocks field arithmetic
- ECgFp5 curve operations
- Schnorr signature generation

```toml
[dependencies]
crypto = { path = "./crypto" }
```

### 3. `signer`
High-level signing interface for:
- Key management (40-byte private keys)
- Transaction signing
- Authentication token generation

```toml
[dependencies]
signer = { path = "./signer" }
```

### 4. `api-client`
HTTP client for Lighter Exchange API:
- Order management (create, modify, cancel)
- Account operations (transfer, withdraw, leverage)
- Automatic nonce management
- Transaction signing and submission

```toml
[dependencies]
api-client = { path = "./api-client" }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
```

## 🏃 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api-client = { path = "../lighter-rust/api-client" }
signer = { path = "../lighter-rust/signer" }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
```

### Configuration

Create a `.env` file:

```bash
BASE_URL=https://testnet.zklighter.elliot.ai
ACCOUNT_INDEX=271
API_KEY_INDEX=4
API_PRIVATE_KEY=your_40_byte_hex_private_key
```

### Basic Usage

```rust
use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let client = LighterClient::new(
        env::var("BASE_URL")?,
        &env::var("API_PRIVATE_KEY")?,
        env::var("ACCOUNT_INDEX")?.parse()?,
        env::var("API_KEY_INDEX")?.parse()?,
    )?;
    
    // Create a limit order
    let order = api_client::CreateOrderRequest {
        account_index: env::var("ACCOUNT_INDEX")?.parse()?,
        order_book_index: 0,
        client_order_index: 12345,
        base_amount: 1000,
        price: 349659,
        is_ask: false,
        order_type: 0,
        time_in_force: 1,
        reduce_only: false,
        trigger_price: 0,
    };
    
    let response = client.create_order(order).await?;
    println!("Order created: {:?}", response);
    
    Ok(())
}
```

## 📚 Examples

The SDK includes **24 comprehensive examples** covering:

### Perpetual Futures Trading
- `create_limit_order` - Basic limit orders
- `create_market_order` - Market orders
- `create_sl_tp` - Stop loss & take profit orders
- `create_position_tied_sl_tp` - Position-tied orders
- `create_grouped_ioc_with_attached_sl_tp` - IOC with attached SL/TP
- `create_limit_order_otoco` - One-triggers-one-cancels-other
- `create_market_order_otoco` - Market OCO orders
- `create_twap_order` - Time-weighted average price orders
- `close_position_otoco` - Close with protection
- `close_all_positions` - Close all positions

### Spot Trading
- `create_spot_limit_order` - Spot limit orders
- `create_spot_market_order` - Spot market orders
- `spot_trading_basics` - Comprehensive spot guide

### Order Management
- `create_modify_cancel_order` - Full order lifecycle
- `cancel_order` - Cancel single order
- `cancel_all_orders` - Cancel all orders
- `cancel_order_otoco` - Cancel and replace

### Advanced Features
- `hft_multi_client` - High-frequency trading with multiple API keys
- `send_tx_batch` - Batch transaction submission
- `create_auth_token` - Authentication token generation
- `setup_api_key` - API key management
- `transfer_update_leverage` - Account operations
- `withdraw_l2` - Layer 2 withdrawals

### Running Examples

```bash
# From the api-client directory
cargo run --example create_limit_order

# Or from the lighter-rust root
cargo run --example create_limit_order --manifest-path api-client/Cargo.toml
```

**See [Examples README](api-client/examples/README.md) for complete documentation.**

## 📖 Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Getting Started](docs/getting-started.md)** - Quick start tutorial
- **[Examples Guide](api-client/examples/README.md)** - All 24 examples documented
- **[Signer Library](docs/signer.md)** - Cryptographic signing internals
- **[Architecture](docs/architecture.md)** - System design overview

## 🔧 Building

```bash
# Build all libraries
cargo build --release

# Build examples
cargo build --examples

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## 🎯 Key Features

### Nonce Management

The SDK provides two nonce management modes:

**Automatic (Recommended)**: Lock-free atomic operations for maximum performance
```rust
let response = client.create_order(order).await?;
```

**Manual**: Explicit nonce control for advanced use cases
```rust
let nonce = client.get_nonce().await?;
let response = client.create_order_direct(order, nonce).await?;
```

### Thread Safety

All client operations are thread-safe. Share clients across threads:

```rust
use std::sync::Arc;

let client = Arc::new(LighterClient::new(...)?);

// Use in multiple threads
let client1 = client.clone();
let client2 = client.clone();

tokio::spawn(async move {
    client1.create_order(order1).await
});

tokio::spawn(async move {
    client2.create_order(order2).await
});
```

### High-Frequency Trading

The `hft_multi_client` example demonstrates:
- Parallel order execution
- Multiple API key management
- Performance benchmarking
- Automatic and manual nonce modes

```bash
cargo run --example hft_multi_client
```

## 🔐 Security

- **No Hardcoded Secrets**: All examples use environment variables
- **Secure Key Management**: Private keys never logged or exposed
- **Production-Ready**: Battle-tested error handling

## 📊 Performance

- **Signing**: < 1ms per transaction
- **Order Submission**: ~130-200ms (network dependent)
- **Throughput**: 100+ orders/second with parallel execution
- **Nonce Management**: Lock-free atomic operations

## 🛠️ Requirements

- Rust 1.70+ (see `rust-toolchain.toml`)
- Tokio async runtime
- Network access to Lighter Exchange API

## 📝 License

See individual library licenses.

## 🤝 Contributing

Contributions are always welcome, Feel free!

## 🔗 Links

- **Examples**: [api-client/examples/README.md](api-client/examples/README.md)
- **Documentation**: [docs/README.md](docs/README.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

## 📞 Support

For issues and questions:
1. Check the [Examples README](api-client/examples/README.md)
2. Review [Documentation](docs/README.md)
3. See [Troubleshooting Guide](docs/TROUBLESHOOTING.md)

---

**Built with ❤️ for high-performance trading on Lighter Exchange**

