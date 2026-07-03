# Lighter Exchange Rust SDK

Production-ready async Rust client for Lighter Exchange with integrated signing, WebSocket support, and optimized HTTP/2.0 connection pooling.

## Features

- **Three Client Types**: Choose based on your needs
  - `LighterClient`: REST API + signing (primary client)
  - `SignerClient`: Standalone signer for auth tokens and message signing
  - `CombinedClient`: REST + WebSocket bundled together
  - `WebSocketClient`: Standalone real-time market data stream

- **Performance**: HTTP/2.0 multiplexing, 1000 per-host connection pool, 30s idle timeout
- **Crypto**: Native Rust implementation (Schnorr signatures, Poseidon2 hashing - no external binaries)
- **Nonce Management**: Optimistic caching (fetch once, increment locally)
- **Type Safety**: Strong typing with serde serialization, compile-time error detection
- **Async-First**: Built on Tokio, 100% async/await

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
api-client = { path = "../api-client" }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
```

## Quick Start

### 1. Read Market Data

```rust
use api_client::LighterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(
        "https://mainnet.zklighter.elliot.ai".to_string(),
        "0x...", // 80-char hex private key
        361816,  // account index
        0,       // api key index
    )?;

    // Fetch account info
    let account = client.get_my_account().await?;
    println!("Account: {:?}", account);

    // Fetch nonce for transaction
    let nonce = client.get_nonce().await?;
    println!("Next nonce: {}", nonce);

    // Get order book
    let order_book = client.get_order_book(0).await?;
    println!("Best bid: {:?}", order_book["bids"].get(0));

    Ok(())
}
```

### 2. Create an Order

```rust
use api_client::{LighterClient, CreateOrderRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(
        "https://mainnet.zklighter.elliot.ai".to_string(),
        "0x...",
        361816,
        0,
    )?;

    let order = CreateOrderRequest {
        account_index: 361816,
        order_book_index: 0,     // ETH market
        client_order_index: 12345,
        base_amount: 1000,       // 0.001 ETH
        price: 203900,           // $2039.00
        is_ask: false,           // Buy order
        order_type: 0,           // Limit
        time_in_force: 1,        // Good Till Time
        reduce_only: false,
        trigger_price: 0,
    };

    let response = client.create_order(order).await?;
    println!("Order created: {:?}", response);

    Ok(())
}
```

### 3. Use WebSocket for Real-Time Data

```rust
use api_client::CombinedClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CombinedClient::new(
        "https://mainnet.zklighter.elliot.ai".to_string(),
        "0x...",
        361816,
        0,
    )?;

    // REST client
    let account = client.rest().get_my_account().await?;
    println!("Account: {:?}", account);

    // WebSocket client for streaming
    let ws = client.ws();
    let mut rx = ws.connect().await?;
    ws.subscribe_order_book(0).await?;

    while let Some(msg) = rx.recv().await {
        match msg {
            api_client::websocket::WsMessage::Connected(data) => {
                println!("Connected: {:?}", data.session_id);
            }
            api_client::websocket::WsMessage::OrderBook(data) => {
                println!("Order book update: market {}", data.market_id);
            }
            api_client::websocket::WsMessage::Account(data) => {
                println!("Account update: {}", data.account_id);
            }
            api_client::websocket::WsMessage::AccountAssets(data) => {
                println!("Account assets update: {}", data.account_id);
            }
            api_client::websocket::WsMessage::Ping => {
                println!("Ping received");
            }
            api_client::websocket::WsMessage::Error(err) => {
                eprintln!("WebSocket error: {}", err);
            }
            api_client::websocket::WsMessage::Unknown(raw) => {
                println!("Unhandled payload: {:?}", raw);
            }
            _ => {}
        }
    }

    Ok(())
}
```

## API Methods

### Read-Only (Market Data)

```rust
// Nonce for next transaction
let nonce = client.get_nonce().await?;

// Account info
let account = client.get_my_account().await?;
let limits = client.get_account_limits(account_index).await?;
let metadata = client.get_account_metadata(account_index).await?;

// Orders
let active = client.get_account_active_orders(account_index, None, Some(10), None).await?;
let inactive = client.get_account_inactive_orders(account_index, None, Some(10), None).await?;

// Market data
let order_book = client.get_order_book(market_index).await?;
let trades = client.get_recent_trades(market_index, Some(100)).await?;
let candles = client.get_candles(market_index, resolution, None, None, Some(100), None).await?;
let funding = client.get_funding_rates(market_index, Some(100), None).await?;

// History
let deposits = client.get_deposit_history(account_index, Some(100), None).await?;
let withdrawals = client.get_withdraw_history(account_index, Some(100), None).await?;
let transfers = client.get_transfer_history(account_index, Some(100), None).await?;
let txs = client.get_account_transactions(account_index, Some(100), None).await?;

// Exchange info
let stats = client.get_exchange_stats().await?;
let assets = client.get_asset_details(asset_index).await?;
```

### Write Operations (Signing Required)

```rust
// Create order
let response = client.create_order(order_request).await?;
let tx_hash = response["tx_hash"].as_str();

// Cancel order
let response = client.cancel_order(market_index, order_index).await?;

// Modify order
let response = client.modify_order(modify_request).await?;

// Transfer USDC
let response = client.transfer(to_account, usdc_amount, fee, memo).await?;

// Withdraw
let response = client.withdraw(usdc_amount).await?;

// Setup API key
let response = client.setup_api_key().await?;

// Create grouped orders (batch)
let response = client.create_grouped_orders(grouping_type, orders).await?;

// Update leverage
let response = client.update_leverage(market_index, leverage_ratio).await?;
```

## Environment Setup

Create `.env` file in project root:

```env
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=361816
API_KEY_INDEX=0
API_PRIVATE_KEY=0x...64char hex key...
```

## Configuration

### HTTP Client Settings

The SDK uses optimized defaults:
- **HTTP Version**: HTTP/2.0 with multiplexing
- **Connection Pool**: 1000 per-host, 30s idle timeout
- **Timeouts**: 10s connection, 30s total, 60s keep-alive
- **Serialization**: Standard JSON for all request/response

### Custom Configuration

```rust
use api_client::LighterClient;

let client = LighterClient::new(
    "https://testnet.zklighter.elliot.ai".to_string(),
    private_key,
    account_index,
    api_key_index,
)?;
```

## Examples

Run included examples:

```bash
# Read-only operations
cargo run --example read_only_get_nonce
cargo run --example read_only_check_api_key

# Create/cancel orders
cargo run --example create_limit_order
cargo run --example cancel_order

# WebSocket streaming
cargo run --example websocket_stream

# Batch operations
cargo run --example send_tx_batch

# Spot trading
cargo run --example spot_buy

# Generate auth token
cargo run --example create_auth_token
```

## Signing & Auth

### Auth Token

For operations requiring authentication (like API key setup), use:

```rust
let token = client.create_auth_token(3600)?;  // 1-hour token
println!("Auth token: {}", token);
```

### Message Signing

For advanced use cases:

```rust
let signer = SignerClient::new(
    private_key_hex.to_string(),
    account_index,
    api_key_index,
)?;

let message: [u8; 40] = [...];  // Pre-hashed message
let signature = signer.sign(&message)?;
println!("Signature: {:x?}", signature);
```

## Testing

Run the full test suite:

```bash
# Unit tests
cargo test --package api-client

# Integration tests (requires .env and mainnet access)
cargo test --package api-client -- --ignored --nocapture

# With explicit mainnet
BASE_URL=https://mainnet.zklighter.elliot.ai cargo test --package api-client -- --ignored --nocapture
```

## Performance

- **Nonce management**: Optimistic caching (1 API call per session)
- **Connection pooling**: HTTP/2.0 multiplexing for 20-30% latency reduction
- **Batch operations**: `create_grouped_orders` for multiple orders in one call
- **Real-time data**: WebSocket streaming for market updates

## Error Handling

All operations return `Result<T>`:

```rust
match client.create_order(order).await {
    Ok(response) => println!("Success: {:?}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

Error types:
- `ApiError::Signer`: Key management/signing error
- `ApiError::Http`: Network/HTTP error
- `ApiError::Json`: Response parsing error
- `ApiError::Api`: Server-returned error

## Resources

- [API Documentation](https://docs.zklighter.elliot.ai)
- [Examples Directory](./examples/)
- [GitHub Repository](https://github.com/bvvvp009/lighter-rust)

## License

Licensed under MIT or Apache 2.0 at your option.

## Support

For issues, questions, or contributions:
- GitHub Issues: [lighter-rust/issues](https://github.com/bvvvp009/lighter-rust/issues)
- Discord: [Lighter Discord](https://discord.gg/lighter)
- Email: support@lighter.exchange
