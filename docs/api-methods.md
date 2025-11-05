# API Methods Reference

Complete reference for all available methods in the Rust signer.

## Client Initialization

```rust
use api_client::LighterClient;

let client = LighterClient::new(
    base_url: String,        // "https://testnet.zklighter.elliot.ai"
    private_key_hex: &str,   // "0x0123..." (hex string, 40 bytes)
    account_index: i64,      // Your account index
    api_key_index: u8,       // Your API key index
)?;
```

## Transaction Methods

### 1. Create Market Order

Creates a market order that executes immediately at the current market price.

```rust
let response = client.create_market_order(
    market_index: u8,           // Market index (0 = default)
    client_order_index: u64,    // Unique order ID
    base_amount: i64,           // Order size in smallest unit
    avg_execution_price: i64,   // Maximum average execution price
    is_ask: bool,               // true = sell, false = buy
).await?;
```

**Parameters:**
- `market_index` (u8): Market identifier (0 = default market)
- `client_order_index` (u64): Unique identifier for your order
- `base_amount` (i64): Order size in smallest denomination
- `avg_execution_price` (i64): Maximum average price for execution
- `is_ask` (bool): `true` for sell orders, `false` for buy orders

**Returns:**
- `Result<serde_json::Value>` - API response JSON

### 2. Create Limit Order

Creates a limit order at a specific price.

```rust
use api_client::CreateOrderRequest;

let order = CreateOrderRequest {
    account_index: 1,
    order_book_index: 0,
    client_order_index: 12345,
    base_amount: 1000,
    price: 450000,
    is_ask: false,              // false = buy
    order_type: 0,              // 0 = LIMIT
    time_in_force: 1,           // 1 = GOOD_TILL_TIME
    reduce_only: false,
    trigger_price: 0,
};

let response = client.create_order(order).await?;
```

**Order Types:**
- `0` = LIMIT
- `1` = MARKET
- `2` = STOP_LOSS
- `3` = STOP_LOSS_LIMIT
- `4` = TAKE_PROFIT
- `5` = TAKE_PROFIT_LIMIT
- `6` = TWAP

**Time in Force:**
- `0` = IMMEDIATE_OR_CANCEL (IOC)
- `1` = GOOD_TILL_TIME (GTT)
- `2` = FILL_OR_KILL (FOK)
- `3` = POST_ONLY

**Parameters:**
- `order_type` (u8): Type of order (see above)
- `time_in_force` (u8): Order time in force (see above)
- `reduce_only` (bool): If `true`, order only reduces position
- `trigger_price` (i64): Trigger price for conditional orders (0 = none)
- `order_expiry` (i64): Order expiry timestamp (-1 = default 28 days)

**Returns:**
- `Result<serde_json::Value>` - API response JSON

### 3. Cancel Order

Cancels a specific order by its order index.

```rust
let response = client.cancel_order(
    order_book_index: u8,   // Market index
    order_index: i64,       // Order index to cancel
).await?;
```

**Parameters:**
- `order_book_index` (u8): Market index where the order exists
- `order_index` (i64): Index of the order to cancel

**Returns:**
- `Result<serde_json::Value>` - API response

### 4. Cancel All Orders

Cancels all orders for your account.

```rust
let response = client.cancel_all_orders(
    time_in_force: u8,      // 0 = IMMEDIATE, 1 = SCHEDULED, 2 = ABORT
    time: i64,              // Time parameter
).await?;
```

**Cancel All Time in Force:**
- `0` = IMMEDIATE - Cancel immediately
- `1` = SCHEDULED - Schedule cancellation
- `2` = ABORT - Abort scheduled cancellations

**Parameters:**
- `time_in_force` (u8): Cancellation type
- `time` (i64): Time parameter (usually 0 for immediate)

**Returns:**
- `Result<serde_json::Value>` - API response

### 5. Change API Key

Registers a new public key (API key setup).

```rust
// First get the public key bytes (40 bytes)
let new_public_key: [u8; 40] = key_manager.public_key_bytes();

let response = client.change_api_key(
    &new_public_key,   // New 40-byte public key
).await?;
```

**Parameters:**
- `new_public_key` (&[u8; 40]): New public key (40 bytes)

**Returns:**
- `Result<serde_json::Value>` - API response

## Authentication Methods

### Create Auth Token

Generates an authentication token for API access.

```rust
use signer::KeyManager;

let key_manager = KeyManager::from_hex("0x...")?;
let token = key_manager.create_auth_token(
    deadline: i64,          // Unix timestamp (seconds) when token expires
    account_index: i64,     // Account index
    api_key_index: u8,      // API key index
)?;

// Token format: "deadline:account_index:api_key_index:signature_hex"
```

**Parameters:**
- `deadline` (i64): Unix timestamp in seconds when token expires
- `account_index` (i64): Your account index
- `api_key_index` (u8): Your API key index

**Returns:**
- `Result<String>` - Token string

**Token Format:**
```
{deadline}:{account_index}:{api_key_index}:{signature_hex}
```

## Utility Methods

### Get Nonce

Retrieves the next nonce from the API.

```rust
let nonce = client.get_nonce().await?;
```

**Returns:**
- `Result<i64>` - Next nonce value

### Sign Transaction

Signs a transaction JSON string (low-level method).

```rust
let signature = client.sign_transaction(&tx_json)?;
```

**Parameters:**
- `tx_json` (&str): JSON string of transaction fields

**Returns:**
- `Result<[u8; 80]>` - 80-byte signature array (s || e format)

**Note:** This is an internal method but is exposed for advanced use cases.

## Key Management Methods

### Generate Key Pair

Generates a new random key pair.

```rust
use signer::KeyManager;

let key_manager = KeyManager::generate();
let private_key = key_manager.private_key_bytes();
let public_key = key_manager.public_key_bytes();
```

**Returns:**
- `KeyManager` instance with randomly generated keys

### Get Public Key

Retrieves the public key from a KeyManager.

```rust
let public_key = key_manager.public_key_bytes(); // [u8; 40]
```

**Returns:**
- `[u8; 40]` - 40-byte public key array

### Get Private Key

Retrieves the private key from a KeyManager.

```rust
let private_key = key_manager.private_key_bytes(); // [u8; 40]
```

**Returns:**
- `[u8; 40]` - 40-byte private key array

## Constants

### Order Types

| Constant | Value | Description |
|----------|-------|-------------|
| `ORDER_TYPE_LIMIT` | 0 | Limit order |
| `ORDER_TYPE_MARKET` | 1 | Market order |
| `ORDER_TYPE_STOP_LOSS` | 2 | Stop loss order |
| `ORDER_TYPE_STOP_LOSS_LIMIT` | 3 | Stop loss limit order |
| `ORDER_TYPE_TAKE_PROFIT` | 4 | Take profit order |
| `ORDER_TYPE_TAKE_PROFIT_LIMIT` | 5 | Take profit limit order |
| `ORDER_TYPE_TWAP` | 6 | TWAP order |

### Time in Force

| Constant | Value | Description |
|----------|-------|-------------|
| `ORDER_TIME_IN_FORCE_IOC` | 0 | Immediate or Cancel |
| `ORDER_TIME_IN_FORCE_GOOD_TILL_TIME` | 1 | Good Till Time |
| `ORDER_TIME_IN_FORCE_FOK` | 2 | Fill or Kill |
| `ORDER_TIME_IN_FORCE_POST_ONLY` | 3 | Post Only |

### Transaction Types

| Constant | Value | Description |
|----------|-------|-------------|
| `TX_TYPE_CHANGE_PUB_KEY` | 8 | Change public key |
| `TX_TYPE_CREATE_SUB_ACCOUNT` | 9 | Create sub-account |
| `TX_TYPE_CREATE_ORDER` | 14 | Create order |
| `TX_TYPE_CANCEL_ORDER` | 15 | Cancel order |
| `TX_TYPE_CANCEL_ALL_ORDERS` | 16 | Cancel all orders |

### Cancel All Time in Force

| Constant | Value | Description |
|----------|-------|-------------|
| `CANCEL_ALL_TIF_IMMEDIATE` | 0 | Cancel immediately |
| `CANCEL_ALL_TIF_SCHEDULED` | 1 | Schedule cancellation |
| `CANCEL_ALL_TIF_ABORT` | 2 | Abort scheduled cancellation |

## Error Handling

```rust
match client.create_market_order(...).await {
    Ok(response) => {
        println!("Success: {:?}", response);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Handle error
    }
}
```

## Complete Example

```rust
use api_client::{LighterClient, CreateOrderRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let base_url = env::var("BASE_URL")?;
    let private_key = env::var("API_PRIVATE_KEY")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    
    // Create client
    let client = LighterClient::new(
        base_url,
        &private_key,
        account_index,
        api_key_index,
    )?;
    
    // Create market order
    let response = client.create_market_order(
        0,           // market_index
        12345,       // client_order_index
        1000,        // base_amount
        450000,      // avg_execution_price
        false,       // is_ask (buy)
    ).await?;
    
    println!("Order submitted: {:?}", response);
    
    Ok(())
}
```

## See Also

- [Getting Started Guide](./getting-started.md) - Quick start tutorial
- [Examples Guide](./running-examples.md) - How to run examples
- [Architecture Documentation](./architecture.md) - System design details
