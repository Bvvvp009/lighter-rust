# Running Examples

This guide shows you how to run the included examples to learn how to use the Rust signer.

## Prerequisites

1. **Rust installed** (version 1.70 or higher)
   ```bash
   rustc --version
   ```

2. **Environment file** - Create a `.env` file in the `rust-signer` directory:
   ```bash
   BASE_URL=https://testnet.zklighter.elliot.ai
   API_PRIVATE_KEY=0x0123456789abcdef...
   ACCOUNT_INDEX=1
   API_KEY_INDEX=0
   ```

3. **Build dependencies** (runs automatically):
   ```bash
   cargo build
   ```

## Available Examples

### 1. Simple Test

Basic client initialization and transaction signing.

```bash
cargo run --example simple_test --release
```

Tests basic functionality without submitting to the exchange.

### 2. Create Market Order

Submit a real market order to the exchange.

```bash
cargo run --example create_market_order --release
```

**What it does:**
- Fetches current market price
- Creates a market buy order
- Submits to exchange
- Displays transaction hash

**⚠️ Warning:** This makes a real trade on testnet.

### 3. Create Limit Order

Submit a limit order at a specific price.

```bash
cargo run --example create_limit_order --release
```

**What it does:**
- Creates a limit order with specified price
- Sets order expiry (default 28 days)
- Submits to exchange

**⚠️ Warning:** This makes a real trade on testnet.

### 4. Cancel Order

Cancel a specific order by order index.

```bash
cargo run --example cancel_order --release
```

**Note:** Modify the example to include a valid `order_index` from a previous order.

### 5. Cancel All Orders

Cancel all orders for your account.

```bash
cargo run --example cancel_all_orders --release
```

**⚠️ Warning:** This cancels ALL your open orders.

### 6. Setup API Key

Generate a new key pair for API key registration.

```bash
cargo run --example setup_api_key --release
```

**What it does:**
- Generates new private/public key pair
- Displays key information
- Shows how to use the keys

**Note:** This only generates keys. You'll need to register the public key separately.

### 7. Create Auth Token

Generate an authentication token for API access.

```bash
cargo run --example create_auth_token --release
```

**What it does:**
- Creates auth token with 10-minute expiry
- Shows token format
- Displays signature details

### 8. Verify Fixes

Comprehensive test of transaction signing.

```bash
cargo run --example verify_fixes --release
```

**What it does:**
- Tests transaction signing
- Verifies chain ID handling
- Validates timing calculations
- Submits test order

**⚠️ Warning:** This makes a real trade on testnet.

## Running Examples

All examples follow the same pattern:

```bash
# From the rust-signer directory
cd rust-signer

# Run an example
cargo run --example <example_name> --release

# Example
cargo run --example create_market_order --release
```

The `--release` flag optimizes performance. For faster compilation during development, omit it:

```bash
cargo run --example create_market_order
```

## Environment Variables

All examples read from `.env` file. Required variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `BASE_URL` | API endpoint URL | `https://testnet.zklighter.elliot.ai` |
| `API_PRIVATE_KEY` | Your 40-byte private key (hex) | `0x0123...` (80 hex chars) |
| `ACCOUNT_INDEX` | Your account number | `1` |
| `API_KEY_INDEX` | Your API key slot | `0` |

Optional variables (some examples use these):

| Variable | Description | Example |
|----------|-------------|---------|
| `MARKET_INDEX` | Market identifier | `0` (default market) |
| `BASE_AMOUNT` | Default order size | `1000` |
| `ITERATIONS` | Benchmark iterations | `10` |

## Creating Your Own Example

1. Create a new file in `rust-signer/api-client/examples/`:
   ```rust
   // my_example.rs
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

2. Add to `api-client/Cargo.toml`:
   ```toml
   [[example]]
   name = "my_example"
   path = "examples/my_example.rs"
   ```

3. Run it:
   ```bash
   cargo run --example my_example --release
   ```

## Troubleshooting

### Example Not Found

**Error:** `error: no example target named 'example_name'`

**Solution:** Check the example name matches exactly. List all examples:
```bash
cargo run --example --help
```

### Environment Variable Not Set

**Error:** `BASE_URL must be set in .env file`

**Solution:** 
- Ensure `.env` file exists in `rust-signer` directory
- Check variable names are correct (case-sensitive)
- Verify no extra spaces around `=`

### Invalid Signature Error

**Error:** API returns "invalid signature" (code 21120)

**Solution:**
- Verify `BASE_URL` matches your chain (testnet vs mainnet)
- Check `API_PRIVATE_KEY` is exactly 80 hex characters (40 bytes)
- Ensure `ACCOUNT_INDEX` and `API_KEY_INDEX` are correct
- Try running again (sometimes intermittent)

### Connection Errors

**Error:** `Failed to fetch nonce` or network errors

**Solution:**
- Verify `BASE_URL` is correct and accessible
- Check your internet connection
- Ensure firewall isn't blocking requests
- Try testnet first: `https://testnet.zklighter.elliot.ai`

### Compilation Errors

**Error:** Various Rust compilation errors

**Solution:**
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`
- Check you're in the `rust-signer` directory

## Best Practices

1. **Start with testnet** - Always test on testnet first
2. **Small amounts** - Use small order amounts for testing
3. **Check responses** - Always verify API responses
4. **Handle errors** - Implement proper error handling
5. **Release mode** - Use `--release` for actual trading

## Next Steps

- Explore the [API Methods Reference](./api-methods.md) for all available functions
- Read [Getting Started](./getting-started.md) for integration guide
- Check [Architecture](./architecture.md) for system overview
