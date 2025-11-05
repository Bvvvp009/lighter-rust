# Getting Started

This guide will help you integrate the Rust signer into your project and start using it right away.

## Installation

### Add Dependencies

Add the following to your `Cargo.toml`:

```toml
[dependencies]
api-client = { path = "../rust-signer/api-client" }
signer = { path = "../rust-signer/signer" }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
serde_json = "1.0"
hex = "0.4"
```

Or if using from a published crate:

```toml
[dependencies]
api-client = "0.1"
signer = "0.1"
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
serde_json = "1.0"
```

### Build the Project

```bash
cargo build --release
```

## Configuration

### 1. Create Environment File

Create a `.env` file in your project root:

```bash
# API Configuration
BASE_URL=https://testnet.zklighter.elliot.ai
# For mainnet: BASE_URL=https://mainnet.zklighter.elliot.ai

# Account Details
ACCOUNT_INDEX=1
API_KEY_INDEX=0

# Private Key (40 bytes, hex format)
API_PRIVATE_KEY=0x0123456789abcdef0123456789abcdef01234567
```

### 2. Load Environment Variables

The examples use `dotenv`, but you can load variables any way you prefer:

```rust
dotenv::dotenv().ok();
```

## Your First Transaction

### Step 1: Initialize the Client

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
    
    // Your code here
    Ok(())
}
```

### Step 2: Create Your First Order

**Market Order Example:**

```rust
// Create a market buy order
let response = client.create_market_order(
    0,           // market_index (0 = default market)
    12345,       // client_order_index (your unique order ID)
    1000,        // base_amount (order size in smallest unit)
    450000,      // avg_execution_price (maximum price you'll accept)
    false,       // is_ask (false = buy, true = sell)
).await?;

println!("Order response: {:?}", response);
```

**Limit Order Example:**

```rust
use api_client::CreateOrderRequest;

let order = CreateOrderRequest {
    account_index: env::var("ACCOUNT_INDEX")?.parse()?,
    order_book_index: 0,        // Market index
    client_order_index: 12345,  // Unique order ID
    base_amount: 1000,          // Order size
    price: 450000,              // Limit price
    is_ask: false,              // false = buy, true = sell
    order_type: 0,              // 0 = LIMIT order
    time_in_force: 1,           // 1 = GOOD_TILL_TIME
    reduce_only: false,         // false = can increase position
    trigger_price: 0,           // 0 = no trigger price
};

let response = client.create_order(order).await?;
```

## Understanding Order Parameters

### Market Index
- `0` = Default market (usually ETH/USDC or BTC/USDC)
- Check API documentation for other market indices

### Client Order Index
A unique identifier for your order. Use any number you want, but ensure it's unique per account. Common approaches:
- Timestamp: `SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64`
- Sequential counter: `1, 2, 3, ...`
- Random: `rand::random::<u64>()`

### Base Amount
Order size in the smallest denomination of the base token. For example:
- If 1 ETH = 1,000,000,000,000,000,000 wei
- To buy 0.001 ETH, use: `1_000_000_000_000_000` (15 zeros)

### Price
For limit orders, the price in the smallest denomination of the quote token (usually USDC).

### Is Ask
- `false` = Buy order (you want to buy the base token)
- `true` = Sell order (you want to sell the base token)

### Order Type
- `0` = LIMIT - Execute at specified price or better
- `1` = MARKET - Execute at current market price

### Time in Force
- `0` = IMMEDIATE_OR_CANCEL - Execute immediately, cancel remaining
- `1` = GOOD_TILL_TIME - Valid until order expiry
- `2` = FILL_OR_KILL - Execute fully or cancel entirely
- `3` = POST_ONLY - Only add to order book (maker order)

## Key Management

### Loading an Existing Key

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex("0x0123...")?;
// or
let key_manager = KeyManager::from_hex(&env::var("API_PRIVATE_KEY")?)?;
```

### Generating a New Key Pair

```rust
use signer::KeyManager;

// Generate cryptographically secure random key
let key_manager = KeyManager::generate();

// Get keys as hex strings
let private_key_hex = format!("0x{}", hex::encode(key_manager.private_key_bytes()));
let public_key_hex = format!("0x{}", hex::encode(key_manager.public_key_bytes()));

println!("Save this private key securely: {}", private_key_hex);
println!("Public key for registration: {}", public_key_hex);
```

### Getting Key Bytes

```rust
let private_key_bytes: [u8; 40] = key_manager.private_key_bytes();
let public_key_bytes: [u8; 40] = key_manager.public_key_bytes();
```

## Authentication Tokens

Generate authentication tokens for API access:

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex(&env::var("API_PRIVATE_KEY")?)?;

// Calculate deadline (10 minutes from now)
let deadline = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)?
    .as_secs() as i64 + 600;

// Generate token
let token = key_manager.create_auth_token(
    deadline,
    env::var("ACCOUNT_INDEX")?.parse()?,
    env::var("API_KEY_INDEX")?.parse()?,
)?;

// Token format: "deadline:account_index:api_key_index:signature_hex"
println!("Use this token for API authentication: {}", token);
```

## Managing Orders

### Cancel a Specific Order

```rust
let response = client.cancel_order(
    0,      // market_index where order exists
    12345,  // order_index to cancel
).await?;
```

### Cancel All Orders

```rust
// Cancel all orders immediately
let response = client.cancel_all_orders(
    0,  // time_in_force: 0 = IMMEDIATE
    0,  // time parameter: 0 for immediate
).await?;
```

## Error Handling

Always handle errors appropriately:

```rust
match client.create_market_order(...).await {
    Ok(response) => {
        // Check response code
        if let Some(code) = response.get("code") {
            if let Some(code_num) = code.as_i64() {
                if code_num == 0 {
                    println!("Order accepted!");
                    if let Some(tx_hash) = response.get("tx_hash") {
                        println!("Transaction: {}", tx_hash);
                    }
                } else {
                    println!("Order rejected with code: {}", code_num);
                    if let Some(message) = response.get("message") {
                        println!("Message: {}", message);
                    }
                }
            }
        }
    }
    Err(e) => {
        eprintln!("Failed to submit order: {}", e);
        // Handle network errors, authentication errors, etc.
    }
}
```

## Complete Working Example

Here's a complete example that creates and cancels an order:

```rust
use api_client::{LighterClient, CreateOrderRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Initialize client
    let client = LighterClient::new(
        env::var("BASE_URL")?,
        &env::var("API_PRIVATE_KEY")?,
        env::var("ACCOUNT_INDEX")?.parse()?,
        env::var("API_KEY_INDEX")?.parse()?,
    )?;
    
    // Create order
    let order = CreateOrderRequest {
        account_index: env::var("ACCOUNT_INDEX")?.parse()?,
        order_book_index: 0,
        client_order_index: 12345,
        base_amount: 1000,
        price: 450000,
        is_ask: false,
        order_type: 0,
        time_in_force: 1,
        reduce_only: false,
        trigger_price: 0,
    };
    
    println!("Creating order...");
    let create_response = client.create_order(order).await?;
    println!("Order created: {:?}", create_response);
    
    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Cancel order (using order_index from response)
    if let Some(order_index) = create_response.get("order_index") {
        if let Some(index) = order_index.as_i64() {
            println!("Canceling order {}...", index);
            let cancel_response = client.cancel_order(0, index).await?;
            println!("Order canceled: {:?}", cancel_response);
        }
    }
    
    Ok(())
}
```

## Next Steps

- Explore [API Methods Reference](./api-methods.md) for all available methods
- Check out [Examples](./running-examples.md) for more usage patterns
- Read [Architecture](./architecture.md) to understand the system design

## Common Issues

**"Invalid signature" error:**
- Verify your `BASE_URL` matches testnet/mainnet correctly
- Check that your private key is correct (80 hex characters = 40 bytes)
- Ensure account_index and api_key_index are correct

**"Order price flagged" error:**
- Your price may be too far from market price
- Try a price closer to current market rates

**Connection errors:**
- Verify `BASE_URL` is correct
- Check your internet connection
- Ensure the API endpoint is accessible

For more troubleshooting, see [TROUBLESHOOTING.md](./TROUBLESHOOTING.md).
