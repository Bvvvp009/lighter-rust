# Code Examples

Practical code examples for using the Rust Signer libraries.

**📚 For comprehensive examples, see [Examples README](../api-client/examples/README.md)**

## Table of Contents

1. [Basic Signing](#basic-signing)
2. [Perpetual Futures Trading](#perpetual-futures-trading)
3. [Spot Trading](#spot-trading)
4. [API Client Usage](#api-client-usage)
5. [Key Management](#key-management)
6. [Auth Tokens](#auth-tokens)
7. [Error Handling](#error-handling)

## Basic Signing

### Sign a Message

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Sign a 40-byte message hash
let message: [u8; 40] = [0u8; 40]; // Your 40-byte message hash

// Sign (returns 80-byte signature: s || e)
let signature = key_manager.sign(&message)?;
println!("Signature: {}", hex::encode(&signature));
```

## Perpetual Futures Trading

### Market Order (Perpetual)

```rust
use api_client::LighterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(base_url, private_key, account_index, api_key_index)?;
    
    // Create market buy order on perpetual futures
    let response = client.create_market_order(
        0,           // order_book_index (0 = ETH/USDC or BTC/USDC perpetual)
        12345,       // client_order_index
        1000,        // base_amount
        349659,      // avg_execution_price (max price)
        false,       // is_ask (false = buy)
    ).await?;
    
    println!("Order submitted: {:?}", response);
    Ok(())
}
```

### Limit Order (Perpetual)

```rust
use api_client::{LighterClient, CreateOrderRequest};

let order = CreateOrderRequest {
    account_index,
    order_book_index: 0,        // Perpetual market index
    client_order_index: 12345,
    base_amount: 1000,
    price: 349659,              // Limit price (cents)
    is_ask: false,              // Buy order
    order_type: 0,              // 0 = Limit order
    time_in_force: 1,           // 1 = Good Till Time
    reduce_only: false,         // Can increase position
    trigger_price: 0,           // No trigger
};

let response = client.create_order(order).await?;
```

### Stop Loss & Take Profit

```rust
// Stop Loss Limit Order
let sl_order = CreateOrderRequest {
    account_index,
    order_book_index: 0,
    client_order_index: 20001,
    base_amount: 1000,
    price: 450000,              // Limit price
    is_ask: false,              // Buy to close short
    order_type: 3,              // 3 = StopLossLimitOrder
    time_in_force: 1,           // Good Till Time
    reduce_only: true,          // Only reduce position
    trigger_price: 450000,      // Trigger at this price
};

let response = client.create_order(sl_order).await?;
```

## Spot Trading

### Spot Limit Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

let order = CreateOrderRequest {
    account_index,
    order_book_index: 0,        // Spot market index (different from perpetuals)
    client_order_index: 12345,
    base_amount: 1000,          // Amount in smallest unit
    price: 349659,              // Limit price
    is_ask: false,              // Buy order
    order_type: 0,              // Limit order
    time_in_force: 1,           // Good Till Time
    reduce_only: false,         // Spot: typically false
    trigger_price: 0,
};

let response = client.create_order(order).await?;
```

### Spot Market Order

```rust
// Spot market order (use spot market index)
let response = client.create_market_order(
    spot_market_index,  // Use spot market index, not perpetual
    12345,
    1000,
    349659,
    false,
).await?;
```

**See [Examples README](../api-client/examples/README.md) for comprehensive spot and perpetual trading examples.**

## Key Management

### Generate Key Pair

```rust
use signer::KeyManager;

// Generate cryptographically secure random key
let key_manager = KeyManager::generate();

// Get keys
let private_key = key_manager.private_key_bytes();
let public_key = key_manager.public_key_bytes();

println!("Private key: {}", hex::encode(&private_key));
println!("Public key: {}", hex::encode(&public_key));
```

### Load from Environment

```rust
use std::env;
use signer::KeyManager;

dotenv::dotenv().ok();

let private_key_hex = env::var("API_PRIVATE_KEY")
    .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
let key_manager = KeyManager::from_hex(&private_key_hex)?;
```

## Auth Tokens

### Create Auth Token

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Calculate deadline (Unix timestamp in seconds)
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as i64 + 600; // 10 minutes from now

// Generate token
// Format: "deadline:account_index:api_key_index:signature_hex"
let auth_token = key_manager.create_auth_token(
    deadline,
    account_index,
    api_key_index,
)?;

println!("Auth token: {}", auth_token);
```

### Auth Token with Custom Expiry

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::from_hex(private_key_hex)?;

// Token valid for 1 hour (3600 seconds)
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as i64 + 3600;

let token = key_manager.create_auth_token(
    deadline,
    account_index,
    api_key_index,
)?;
```

## Error Handling

### Comprehensive Error Handling

```rust
use api_client::{LighterClient, ApiError};

async fn submit_order_safely(
    client: &LighterClient,
    order: CreateOrderRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.create_order(order).await {
        Ok(response) => {
            println!("✅ Success: {:?}", response);
            Ok(())
        }
        Err(ApiError::Http(e)) => {
            eprintln!("❌ HTTP error: {}", e);
            Err(e.into())
        }
        Err(ApiError::Api(msg)) => {
            eprintln!("❌ API error: {}", msg);
            Err(msg.into())
        }
        Err(ApiError::Signer(e)) => {
            eprintln!("❌ Signing error: {:?}", e);
            Err(format!("Signing failed: {:?}", e).into())
        }
        Err(e) => {
            eprintln!("❌ Unexpected error: {}", e);
            Err(e.into())
        }
    }
}
```

### Retry Logic

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn submit_with_retry(
    client: &LighterClient,
    order: CreateOrderRequest,
    max_retries: u32,
) -> Result<serde_json::Value, ApiError> {
    for attempt in 1..=max_retries {
        match client.create_order(order.clone()).await {
            Ok(response) => return Ok(response),
            Err(ApiError::Http(e)) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}. Retrying...", attempt, e);
                sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(ApiError::Api("Max retries exceeded".to_string()))
}
```

## Advanced Examples

### Batch Order Submission

```rust
async fn submit_batch_orders(
    client: &LighterClient,
    orders: Vec<CreateOrderRequest>,
) -> Vec<Result<serde_json::Value, ApiError>> {
    let mut results = Vec::new();
    
    for order in orders {
        let result = client.create_order(order).await;
        results.push(result);
        
        // Small delay between orders
        sleep(Duration::from_millis(100)).await;
    }
    
    results
}
```

### Transaction Monitoring

```rust
async fn monitor_order(
    client: &LighterClient,
    tx_hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Poll for transaction status
    // (This would require additional API endpoints)
    
    println!("Monitoring transaction: {}", tx_hash);
    
    for _ in 0..10 {
        sleep(Duration::from_secs(5)).await;
        // Check transaction status
        println!("Checking status...");
    }
    
    Ok(())
}
```

## Running Examples

All examples are in the `api-client/examples/` directory:

```bash
# Perpetual futures
cargo run --example create_limit_order
cargo run --example create_market_order
cargo run --example create_sl_tp

# Spot trading
cargo run --example create_spot_limit_order
cargo run --example create_spot_market_order
cargo run --example spot_trading_basics

# Advanced
cargo run --example hft_multi_client
cargo run --example create_grouped_ioc_with_attached_sl_tp
```

**⚠️ All examples require a `.env` file with:**
- `BASE_URL`
- `ACCOUNT_INDEX`
- `API_KEY_INDEX`
- `API_PRIVATE_KEY`

## See Also

- [Examples README](../api-client/examples/README.md) - Comprehensive examples guide
- [Getting Started Guide](./getting-started.md) - Quick start tutorial
- [API Client Documentation](./api-client.md) - API reference
- [Signer Documentation](./signer.md) - Signing internals
