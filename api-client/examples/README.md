# Lighter Rust SDK Examples

This directory contains comprehensive examples demonstrating how to use the Lighter Rust SDK for both **Perpetual Futures** and **Spot Trading**.

## 📋 Table of Contents

- [Setup](#setup)
- [Perpetual Futures Trading](#perpetual-futures-trading)
- [Spot Trading](#spot-trading)
- [Order Management](#order-management)
- [Advanced Features](#advanced-features)
- [High-Frequency Trading](#high-frequency-trading)

## 🔧 Setup

### 1. Create `.env` File

All examples require a `.env` file in the project root with your credentials:

```bash
# API Configuration
BASE_URL=https://testnet.zklighter.elliot.ai
# For mainnet: BASE_URL=https://mainnet.zklighter.elliot.ai

# Account Details
ACCOUNT_INDEX=271
API_KEY_INDEX=4

# Private Key (40 bytes, hex format - NO 0x prefix)
API_PRIVATE_KEY=8ed02d04e41d7fbf39c1ef2afd64b9655f4862fac9b4fe551984be85b3e3e6efd3d1cd046026d132
```

**⚠️ Security Note:** Never commit your `.env` file or expose your private key. All examples now require environment variables - no hardcoded secrets.

### 2. Run an Example

```bash
# From the api-client directory
cargo run --example create_limit_order

# Or from the lighter-rust root
cargo run --example create_limit_order --manifest-path api-client/Cargo.toml
```

## 📈 Perpetual Futures Trading

Perpetual futures allow you to trade with leverage and hold positions without expiration dates.

### Basic Order Examples

#### Create Limit Order (Perpetual)
```bash
cargo run --example create_limit_order
```
- Creates a limit order on perpetual futures markets
- Demonstrates basic order creation with price and size
- Uses automatic nonce management

#### Create Market Order (Perpetual)
```bash
cargo run --example create_market_order
```
- Creates a market order that executes immediately
- Useful for quick entry/exit from positions
- Executes at best available price

### Advanced Perpetual Trading

#### Stop Loss & Take Profit
```bash
cargo run --example create_sl_tp
```
- Creates Stop Loss (SL) orders to limit losses
- Creates Take Profit (TP) orders to secure profits
- Demonstrates both Market and Limit SL/TP orders

#### Position-Tied SL/TP
```bash
cargo run --example create_position_tied_sl_tp
```
- Creates SL/TP orders tied to your entire position
- Orders automatically adjust as position size changes
- Perfect for protecting existing positions

#### IOC Order with Attached SL/TP
```bash
cargo run --example create_grouped_ioc_with_attached_sl_tp
```
- Creates an Immediate-or-Cancel (IOC) order
- Automatically attaches SL/TP orders based on execution size
- Demonstrates grouped order functionality

### Order Lifecycle

#### Create, Modify, Cancel
```bash
cargo run --example create_modify_cancel_order
```
- Complete order lifecycle demonstration
- Shows how to modify order price and size
- Demonstrates order cancellation

### Advanced Order Types

#### OCO (One-Cancels-the-Other)
```bash
cargo run --example create_limit_order_otoco
cargo run --example create_market_order_otoco
```
- Creates entry order with attached SL/TP
- If one order executes, the other is canceled
- Perfect for risk management

#### TWAP (Time-Weighted Average Price)
```bash
cargo run --example create_twap_order
```
- Splits large orders over time
- Reduces market impact
- Useful for large position entries/exits

### Position Management

#### Close Position
```bash
cargo run --example close_position_otoco
```
- Closes a position with re-entry protection
- Uses grouped orders for safety

#### Close All Positions
```bash
cargo run --example close_all_positions
```
- Closes all open positions across markets
- Useful for risk management

## 🪙 Spot Trading

Spot trading allows you to trade assets directly without leverage.

### Basic Spot Trading

#### Spot Limit Order
```bash
cargo run --example create_spot_limit_order
```
- Creates a limit order on spot markets
- Buy or sell assets at a specific price
- No leverage - direct asset exchange

#### Spot Market Order
```bash
cargo run --example create_spot_market_order
```
- Creates a market order on spot markets
- Executes immediately at best available price
- Useful for quick asset swaps

#### Spot Trading Basics
```bash
cargo run --example spot_trading_basics
```
- Comprehensive spot trading demonstration
- Shows buy/sell limit orders
- Demonstrates market orders
- Explains differences from perpetual trading

### Key Differences: Spot vs Perpetual

| Feature | Spot Trading | Perpetual Futures |
|---------|-------------|-------------------|
| Leverage | ❌ No leverage | ✅ Up to 100x+ |
| Funding Rates | ❌ None | ✅ Periodic funding |
| Position Direction | Buy/Sell assets | Long/Short positions |
| `reduce_only` | ⚠️ Rarely used | ✅ Commonly used |
| Market Indices | Different indices | Different indices |

**Important:** Spot markets use different market indices than perpetual markets. Check the API documentation for the correct spot market indices.

## 🔄 Order Management

### Cancel Orders

#### Cancel Single Order
```bash
cargo run --example cancel_order
```
- Cancels a specific order by order index
- Useful for order management

#### Cancel All Orders
```bash
cargo run --example cancel_all_orders
```
- Cancels all open orders
- Can filter by time in force

#### Cancel Order and Replace (OTOC)
```bash
cargo run --example cancel_order_otoco
```
- Cancels existing order and creates new one
- Atomic operation - both succeed or both fail

## 🚀 Advanced Features

### High-Frequency Trading

#### Multi-Client HFT
```bash
cargo run --example hft_multi_client
```
- Demonstrates high-frequency trading with multiple API keys
- Shows automatic and manual nonce management
- Parallel order execution
- Performance benchmarking

**Key Features:**
- Round-robin client selection
- Automatic nonce caching (lock-free)
- Manual nonce management with mutex optimization
- Sequential and parallel order execution
- Comprehensive performance metrics

### Batch Operations

#### Send Transaction Batch
```bash
cargo run --example send_tx_batch
```
- Sends multiple transactions in a batch
- Efficient for bulk operations

### Account Operations

#### Transfer Funds
```bash
cargo run --example transfer_update_leverage
```
- Transfers USDC between accounts
- Updates leverage settings
- Demonstrates account management

#### Withdraw from L2
```bash
cargo run --example withdraw_l2
```
- Withdraws USDC from Layer 2
- Demonstrates withdrawal process

#### Setup API Key
```bash
cargo run --example setup_api_key
```
- Sets up a new API key
- Demonstrates key management

#### Create Auth Token
```bash
cargo run --example create_auth_token
```
- Generates authentication tokens
- For API access without private key exposure

## 📊 Example Categories

### By Trading Type

**Perpetual Futures:**
- `create_limit_order.rs` - Basic limit orders
- `create_market_order.rs` - Market orders
- `create_sl_tp.rs` - Stop loss & take profit
- `create_position_tied_sl_tp.rs` - Position-tied orders
- `create_grouped_ioc_with_attached_sl_tp.rs` - IOC with SL/TP
- `create_limit_order_otoco.rs` - OCO orders
- `create_market_order_otoco.rs` - Market OCO
- `create_twap_order.rs` - TWAP orders
- `close_position_otoco.rs` - Close with protection
- `close_all_positions.rs` - Close all positions

**Spot Trading:**
- `create_spot_limit_order.rs` - Spot limit orders
- `create_spot_market_order.rs` - Spot market orders
- `spot_trading_basics.rs` - Comprehensive spot guide

### By Functionality

**Order Management:**
- `create_modify_cancel_order.rs` - Full lifecycle
- `cancel_order.rs` - Cancel single
- `cancel_all_orders.rs` - Cancel all
- `cancel_order_otoco.rs` - Cancel and replace

**Advanced:**
- `hft_multi_client.rs` - High-frequency trading
- `send_tx_batch.rs` - Batch operations
- `create_auth_token.rs` - Authentication
- `setup_api_key.rs` - Key management
- `transfer_update_leverage.rs` - Account operations
- `withdraw_l2.rs` - Withdrawals

## 🔍 Understanding Order Parameters

### Common Parameters

```rust
CreateOrderRequest {
    account_index: 271,           // Your account index
    order_book_index: 0,          // Market index (0 = default)
    client_order_index: 12345,    // Unique order ID
    base_amount: 1000,            // Order size (smallest unit)
    price: 349659,                // Limit price (cents)
    is_ask: false,                // false = buy, true = sell
    order_type: 0,                // 0 = Limit, 1 = Market
    time_in_force: 1,             // 0 = IOC, 1 = GTT
    reduce_only: false,           // Only reduce position?
    trigger_price: 0,              // Trigger price for SL/TP
}
```

### Order Types

- `0` = **Limit Order** - Execute at specified price or better
- `1` = **Market Order** - Execute at current market price
- `2` = **Stop Loss (Market)** - Market order triggered at price
- `3` = **Stop Loss (Limit)** - Limit order triggered at price
- `4` = **Take Profit (Market)** - Market order triggered at price
- `5` = **Take Profit (Limit)** - Limit order triggered at price

### Time in Force

- `0` = **IOC** (Immediate or Cancel) - Execute immediately, cancel remaining
- `1` = **GTT** (Good Till Time) - Valid until order expiry

### Market Indices

**Perpetual Markets:**
- `0` = ETH/USDC Perpetual
- `1` = BTC/USDC Perpetual
- Check API docs for more markets

**Spot Markets:**
- Different indices than perpetuals
- Check API documentation for spot market indices

## 🎯 Best Practices

### 1. Use Automatic Nonce (Recommended)

```rust
// ✅ Fast and safe - lock-free atomic operations
let response = client.create_order(order).await?;
```

### 2. Handle Errors Properly

```rust
match client.create_order(order).await {
    Ok(response) => {
        let code = response["code"].as_i64().unwrap_or_default();
        if code == 200 {
            println!("Success!");
        } else {
            eprintln!("Error {}: {}", code, response["message"]);
        }
    }
    Err(e) => eprintln!("Request failed: {}", e),
}
```

### 3. Use Unique Client Order Index

```rust
// Use timestamp or counter
let client_order_index = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_millis() as u64;
```

### 4. Parallel Execution for HFT

```rust
let tasks: Vec<_> = orders.iter().map(|order| {
    tokio::spawn(async move {
        client.create_order(order).await
    })
}).collect();
let results = futures::future::join_all(tasks).await;
```

## 🐛 Troubleshooting

### "Environment variable is required" Error

**Solution:** Create a `.env` file with all required variables:
- `BASE_URL`
- `ACCOUNT_INDEX`
- `API_KEY_INDEX`
- `API_PRIVATE_KEY`

### "Invalid signature" Error

**Possible causes:**
- Wrong `BASE_URL` (testnet vs mainnet)
- Incorrect private key format
- Wrong `account_index` or `api_key_index`

**Solution:** Verify all credentials match your account.

### "Account not found" Error

**Possible causes:**
- Account not initialized on the exchange
- Wrong `account_index`

**Solution:** Ensure your account is set up on the exchange.

### "Invalid nonce" Error

**Solution:** The client automatically handles nonce management. If you see this error:
1. Wait a moment and retry
2. Call `client.refresh_nonce().await?` to reset the cache

## 📚 Additional Resources

- [Getting Started Guide](../docs/getting-started.md)
- [Architecture Overview](../docs/architecture.md)

## 🔐 Security Reminders

1. **Never commit `.env` files** - Add to `.gitignore`
2. **Never expose private keys** - Use environment variables
3. **Use testnet for development** - Test with testnet first
4. **Rotate API keys regularly** - For production use

## 💡 Tips

- Start with `create_limit_order` or `create_spot_limit_order` for basics
- Use `spot_trading_basics` to understand spot vs perpetual differences
- Check `hft_multi_client` for high-performance patterns
- All examples are production-ready and can be adapted for your use case

---

**Happy Trading! 🚀**

