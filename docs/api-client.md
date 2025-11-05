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
const MARKET_ORDER: u8 = 0;
const LIMIT_ORDER: u8 = 1;

// Time in Force Constants
const IMMEDIATE_OR_CANCEL: u8 = 0;
const GOOD_TILL_CANCEL: u8 = 1;
const FILL_OR_KILL: u8 = 2;
const POST_ONLY: u8 = 3;
```

## Advanced Usage

### Environment Configuration

Use environment variables for configuration:

```rust
use std::env;

let base_url = env::var("BASE_URL")
    .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
let private_key = env::var("API_PRIVATE_KEY")?;
let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;

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
- Mainnet URLs → Chain ID: 304
- Testnet URLs → Chain ID: 300

```rust
// Mainnet
let client = LighterClient::new(
    "https://mainnet.zklighter.elliot.ai".to_string(),
    private_key,
    account_index,
    api_key_index,
)?; // Uses chain ID 304

// Testnet
let client = LighterClient::new(
    "https://testnet.zklighter.elliot.ai".to_string(),
    private_key,
    account_index,
    api_key_index,
)?; // Uses chain ID 300
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

### Market Buy Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

let client = LighterClient::new(base_url, private_key, account_index, api_key_index)?;

let buy_order = CreateOrderRequest {
    account_index,
    order_book_index: 0,        // BTC-USD
    client_order_index: 12345,
    base_amount: 1000,          // 0.001 BTC
    price: 50000_0000,          // $50,000 (market price)
    is_ask: false,              // Buy order
    order_type: 0,              // Market order
    time_in_force: 0,           // Immediate or cancel
    reduce_only: false,
    trigger_price: 0,
};

let response = client.create_order(buy_order).await?;
```

### Limit Sell Order

```rust
let sell_order = CreateOrderRequest {
    account_index,
    order_book_index: 0,
    client_order_index: 67890,
    base_amount: 2000,          // 0.002 BTC
    price: 51000_0000,          // $51,000 limit price
    is_ask: true,               // Sell order
    order_type: 1,              // Limit order
    time_in_force: 1,           // Good till cancel
    reduce_only: false,
    trigger_price: 0,
};

let response = client.create_order(sell_order).await?;
```

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

See the examples directory for working examples:

```bash
# Run verification example
cargo run --example verify_fixes

# Run simple test
cargo run --example simple_test
```

## Best Practices

1. **Nonce Management**: The client automatically manages nonces. Don't reuse nonces manually.
2. **Error Handling**: Always handle `ApiError` appropriately for production code.
3. **Rate Limiting**: Implement backoff strategies for rate limit errors (429).
4. **Private Keys**: Never expose private keys. Use environment variables or secure storage.
5. **Order IDs**: Use unique `client_order_index` values to track orders.
6. **Price Precision**: Prices use 4 decimal places (multiply by 10,000).
7. **Amount Precision**: Check the base token decimals for correct amount formatting.

## See Also

- [Signer Library](./signer.md) - Transaction signing internals
- [Getting Started Guide](./getting-started.md) - Quick start tutorial
- [Examples](./examples.md) - Code examples

