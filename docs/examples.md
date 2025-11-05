# Code Examples

Practical code examples for using the Rust Signer libraries.

## Table of Contents

1. [Basic Signing](#basic-signing)
2. [API Client Usage](#api-client-usage)
3. [Key Management](#key-management)
4. [Auth Tokens](#auth-tokens)
5. [Error Handling](#error-handling)

## Basic Signing

### Sign a Message

```rust
use signer::KeyManager;
use poseidon_hash::Fp5Element;

let key_manager = KeyManager::new(private_key_hex)?;

// Create message
let message_bytes = vec![0u8; 40];
let message = Fp5Element::from_bytes_le(&message_bytes);

// Sign
let signature = key_manager.sign(&message)?;
println!("Signature: {}", hex::encode(&signature));
```

### Verify Signature

```rust
use crypto::{SchnorrSignature, Point, ScalarField};
use poseidon_hash::Fp5Element;

let key_manager = KeyManager::new(private_key_hex)?;
let message = Fp5Element::one();
let signature = key_manager.sign(&message)?;

// Verify (requires public key and message)
let public_key = key_manager.public_key();
let sig = SchnorrSignature::from_bytes(&signature)?;
let is_valid = sig.verify(&public_key, &message);
println!("Valid: {}", is_valid);
```

## API Client Usage

### Market Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(base_url, private_key, account_index, api_key_index)?;
    
    let order = CreateOrderRequest {
        account_index: 0,
        order_book_index: 0,
        client_order_index: 12345,
        base_amount: 1000,
        price: 349659,              // Market price
        is_ask: false,              // Buy
        order_type: 0,              // Market
        time_in_force: 0,           // IOC
        reduce_only: false,
        trigger_price: 0,
    };
    
    let response = client.create_order(order).await?;
    println!("Order submitted: {:?}", response);
    Ok(())
}
```

### Limit Order

```rust
let limit_order = CreateOrderRequest {
    account_index: 0,
    order_book_index: 0,
    client_order_index: 67890,
    base_amount: 2000,
    price: 51000_0000,             // Limit price
    is_ask: true,                  // Sell
    order_type: 1,                 // Limit
    time_in_force: 1,              // GTC
    reduce_only: false,
    trigger_price: 0,
};

let response = client.create_order(limit_order).await?;
```

## Key Management

### Generate Key Pair

```rust
use crypto::ScalarField;
use signer::KeyManager;

// Generate random private key
let private_scalar = ScalarField::sample_crypto();
let private_bytes: [u8; 32] = private_scalar.to_bytes_array();

// Pad to 40 bytes for Lighter format
let mut private_key = [0u8; 40];
private_key[..32].copy_from_slice(&private_bytes);

// Create KeyManager
let key_manager = KeyManager::from_bytes(&private_key)?;
let public_key = key_manager.public_key_hex();
println!("Public key: {}", public_key);
```

### Load from Environment

```rust
use std::env;
use signer::KeyManager;

let private_key_hex = env::var("API_PRIVATE_KEY")
    .expect("API_PRIVATE_KEY not set");
let key_manager = KeyManager::new(&private_key_hex)?;
```

## Auth Tokens

### Create Auth Token

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

let key_manager = KeyManager::new(private_key_hex)?;

// Get current timestamp
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;

// Create auth message
let message = format!("LIGHTER_AUTH:{}", timestamp);

// Generate token
let auth_token = key_manager.create_auth_token(&message)?;
println!("Auth token: {}", auth_token);
```

### Auth Token with Expiry

```rust
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

let key_manager = KeyManager::new(private_key_hex)?;

// Token valid for 1 hour
let expires_at = SystemTime::now() + Duration::from_secs(3600);
let timestamp = expires_at
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;

let message = format!("LIGHTER_AUTH:{}:{}", timestamp, expires_at);
let token = key_manager.create_auth_token(&message)?;
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

## Complete Working Example

See `rust-signer/api-client/examples/verify_fixes.rs` for a complete, working example that:
- Loads configuration from environment
- Creates a client
- Constructs a transaction
- Signs and submits an order
- Handles responses and errors

Run it with:

```bash
cd rust-signer/api-client
cargo run --example verify_fixes
```

## See Also

- [Getting Started Guide](./getting-started.md)
- [API Client Documentation](./api-client.md)
- [Signer Documentation](./signer.md)
