# API Client Library

The `api-client` crate provides an HTTP client for interacting with the Lighter Exchange API, including order creation, transaction signing, and account management.

## Overview

This library provides:
- **LighterClient**: Main client for API interactions
- **Order Management**: Create, cancel, and manage orders
- **Transaction Signing**: Automatic transaction signing and submission
- **Nonce Management**: Automatic nonce fetching and management
- **Error Handling**: Comprehensive error types for API operations

## Installation

```toml
[dependencies]
api-client = { path = "../api-client" }
signer = { path = "../signer" }
tokio = { version = "1.0", features = ["full"] }
```

## Basic Usage

### Creating a Client

```rust
use api_client::LighterClient;

// Initialize client
let client = LighterClient::new(
    "https://mainnet.zklighter.elliot.ai".to_string(), // Base URL
    "your_private_key_hex",                            // 40-byte hex private key
    0,                                                  // Account index
    0,                                                  // API key index
)?;
```

### Creating an Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(
        base_url,
        private_key_hex,
        account_index,
        api_key_index,
    )?;

    // Create order request
    let order = CreateOrderRequest {
        account_index: 0,
        order_book_index: 0,        // 0 = BTC-USD, 1 = ETH-USD, etc.
        client_order_index: 12345,  // Unique client-side order ID
        base_amount: 1000,          // Amount in base token (with decimals)
        price: 50000_0000,          // Price (with 4 decimals)
        is_ask: false,              // false = buy order, true = sell order
        order_type: 0,              // 0 = MarketOrder, 1 = LimitOrder
        time_in_force: 0,           // 0 = ImmediateOrCancel
        reduce_only: false,         // true for closing positions only
        trigger_price: 0,           // For stop orders
    };

    // Submit order
    let response = client.create_order(order).await?;
    println!("Order response: {:?}", response);
    
    Ok(())
}
```

## API Reference

### LighterClient

The main client struct for API interactions.

#### Initialization

```rust
use api_client::LighterClient;

// Create new client
let client = LighterClient::new(
    base_url: String,      // API base URL (mainnet or testnet)
    private_key: &str,     // 40-byte hex private key
    account_index: i64,    // Account index
    api_key_index: u8,     // API key index
) -> Result<LighterClient, ApiError>;
```

#### Create Order

```rust
let response = client.create_order(order: CreateOrderRequest)
    .await
    -> Result<serde_json::Value, ApiError>;
```

#### Get Nonce

```rust
// Get next nonce for account/api_key
let nonce = client.get_nonce()
    .await
    -> Result<i64, ApiError>;
```

### CreateOrderRequest

Structure for order creation requests.

```rust
pub struct CreateOrderRequest {
    pub account_index: i64,       // Account index
    pub order_book_index: u8,     // Market index (0=BTC-USD, etc.)
    pub client_order_index: u64,  // Unique client order ID
    pub base_amount: i64,         // Amount in base token
    pub price: i64,               // Price (with 4 decimals)
    pub is_ask: bool,             // true = sell, false = buy
    pub order_type: u8,           // Order type (0=Market, 1=Limit)
    pub time_in_force: u8,        // Time in force (0=IOC, etc.)
    pub reduce_only: bool,        // Reduce-only flag
    pub trigger_price: i64,       // Trigger price for stop orders
}
```

### Order Types

```rust
// Order Type Constants
const LIMIT_ORDER: u8 = 0;
const MARKET_ORDER: u8 = 1;
const STOP_LOSS_ORDER: u8 = 2;           // Market SL
const STOP_LOSS_LIMIT_ORDER: u8 = 3;     // Limit SL
const TAKE_PROFIT_ORDER: u8 = 4;         // Market TP
const TAKE_PROFIT_LIMIT_ORDER: u8 = 5;   // Limit TP

// Time in Force Constants
const IMMEDIATE_OR_CANCEL: u8 = 0;  // IOC
const GOOD_TILL_TIME: u8 = 1;       // GTT
const FILL_OR_KILL: u8 = 2;         // FOK
const POST_ONLY: u8 = 3;            // Post-only
```

## Advanced Usage

### Environment Configuration

**⚠️ Security:** All examples now require environment variables. Never hardcode secrets.

```rust
use std::env;

// Load from .env file
dotenv::dotenv().ok();

// Required environment variables
let base_url = env::var("BASE_URL")
    .map_err(|_| "BASE_URL environment variable is required")?;
let private_key = env::var("API_PRIVATE_KEY")
    .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
let account_index: i64 = env::var("ACCOUNT_INDEX")
    .map_err(|_| "ACCOUNT_INDEX environment variable is required")?
    .parse()?;
let api_key_index: u8 = env::var("API_KEY_INDEX")
    .map_err(|_| "API_KEY_INDEX environment variable is required")?
    .parse()?;

let client = LighterClient::new(base_url, &private_key, account_index, api_key_index)?;
```

### Error Handling

```rust
use api_client::{LighterClient, ApiError};

match client.create_order(order).await {
    Ok(response) => {
        println!("Success: {:?}", response);
    }
    Err(ApiError::Http(e)) => {
        eprintln!("HTTP error: {}", e);
    }
    Err(ApiError::Api(msg)) => {
        eprintln!("API error: {}", msg);
    }
    Err(ApiError::Signer(e)) => {
        eprintln!("Signing error: {:?}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Chain ID Configuration

The client automatically determines the chain ID based on the base URL:
- URLs containing `elliot.ai` or `testnet` → Chain ID: 300 (Testnet)
- URLs containing `zklighter.com` (but not `elliot.ai`) → Chain ID: 304 (Mainnet)
- Default: 300 (Testnet) for safety

```rust
// Testnet (Chain ID 300)
let client = LighterClient::new(
    "https://testnet.zklighter.elliot.ai".to_string(),
    private_key,
    account_index,
    api_key_index,
)?;

// Mainnet (Chain ID 304)
let client = LighterClient::new(
    "https://zklighter.com".to_string(),
    private_key,
    account_index,
    api_key_index,
)?;
```

### Transaction Expiry

Transactions automatically expire 10 minutes after creation:

```rust
// ExpiredAt is automatically set to: now + 599 seconds (10 minutes - 1 second)
// Default transaction expiry is 10 minutes
```

### Custom Transaction Signing

For advanced use cases, you can manually construct and sign transactions:

```rust
use api_client::LighterClient;
use serde_json::json;

// Get nonce
let nonce = client.get_nonce().await?;

// Build transaction JSON
let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;
    
let tx = json!({
    "LighterChainId": 304,
    "AccountIndex": account_index,
    "ExpiredAt": now + 599_000,
    "Nonce": nonce,
    // ... other fields
});

// Sign and submit (internal method, see source for details)
```

## Examples

### Perpetual Futures Trading

#### Market Buy Order (Perpetual)

```rust
use api_client::LighterClient;

let client = LighterClient::new(base_url, private_key, account_index, api_key_index)?;

// Create market buy order on perpetual futures
let response = client.create_market_order(
    0,           // order_book_index (0 = ETH/USDC or BTC/USDC perpetual)
    12345,       // client_order_index
    1000,        // base_amount (0.001 tokens)
    50000_0000,  // avg_execution_price (max price in cents)
    false,       // is_ask (false = buy)
).await?;
```

#### Limit Order (Perpetual)

```rust
use api_client::{LighterClient, CreateOrderRequest};

let order = CreateOrderRequest {
    account_index,
    order_book_index: 0,        // Perpetual market index
    client_order_index: 12345,
    base_amount: 1000,          // Order size
    price: 50000_0000,          // Limit price (cents)
    is_ask: false,              // Buy order
    order_type: 0,              // 0 = Limit order
    time_in_force: 1,           // 1 = Good Till Time
    reduce_only: false,         // Can increase position
    trigger_price: 0,           // No trigger
};

let response = client.create_order(order).await?;
```

### Spot Trading

#### Spot Limit Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

let order = CreateOrderRequest {
    account_index,
    order_book_index: 0,        // Spot market index (different from perpetuals)
    client_order_index: 12345,
    base_amount: 1000,          // Amount in smallest unit
    price: 50000_0000,          // Limit price
    is_ask: false,              // Buy order
    order_type: 0,              // Limit order
    time_in_force: 1,           // Good Till Time
    reduce_only: false,         // Spot: typically false
    trigger_price: 0,
};

let response = client.create_order(order).await?;
```

#### Spot Market Order

```rust
// Spot market orders work the same way
// Just use the correct spot market index
let response = client.create_market_order(
    spot_market_index,  // Use spot market index, not perpetual
    12345,
    1000,
    50000_0000,
    false,
).await?;
```

**See [Examples README](../api-client/examples/README.md) for comprehensive spot and perpetual trading examples.**

## Response Format

Order submission returns a JSON response:

```json
{
    "code": 200,
    "message": "{\"ratelimit\": \"didn't use volume quota\"}",
    "predicted_execution_time_ms": 1762241985117,
    "tx_hash": "45bf0ca74fec3d37f26355ea50f92e3247afb574ad08031eeacc90f0e5dc8ba5a89a1d6a537b3dff"
}
```

## Error Codes

Common API error codes:

- `200`: Success
- `21733`: Order price flagged (suspicious price)
- `400`: Bad request
- `401`: Unauthorized (invalid signature)
- `429`: Rate limited

## Testing

See the examples directory for comprehensive examples:

```bash
# Perpetual futures trading
cargo run --example create_limit_order
cargo run --example create_market_order
cargo run --example create_sl_tp

# Spot trading
cargo run --example create_spot_limit_order
cargo run --example create_spot_market_order
cargo run --example spot_trading_basics

# High-frequency trading
cargo run --example hft_multi_client

# See examples/README.md for full list
```

## Best Practices

1. **Nonce Management**: The client automatically manages nonces using lock-free atomic operations. Use automatic nonce mode for best performance.
2. **Error Handling**: Always handle `ApiError` appropriately for production code.
3. **Rate Limiting**: Implement backoff strategies for rate limit errors (429).
4. **Private Keys**: Never expose private keys. Use environment variables (`.env` file) - all examples require this.
5. **Order IDs**: Use unique `client_order_index` values to track orders (timestamp or counter).
6. **Price Precision**: Prices are in cents (multiply dollar amount by 100).
7. **Amount Precision**: Check the base token decimals for correct amount formatting.
8. **Thread Safety**: `LighterClient` is `Send + Sync` - safe to share across threads with `Arc`.
9. **Parallel Execution**: Use `tokio::spawn` for parallel order execution in HFT scenarios.
10. **Spot vs Perpetual**: Use correct market indices - spot markets have different indices than perpetuals.

## See Also

- [Signer Library](./signer.md) - Transaction signing internals
- [Getting Started Guide](./getting-started.md) - Quick start tutorial
- [Examples README](../api-client/examples/README.md) - Comprehensive examples guide

