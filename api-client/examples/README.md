# API Client Examples

This directory contains example programs demonstrating how to use the Lighter Client API.

## Setup

1. Copy `.env.example` to `.env` in the api-client directory:
```bash
cp .env.example .env
```

2. Edit `.env` with your credentials:
```bash
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=<your_account_index>
API_KEY_INDEX=<your_api_key_index>
API_PRIVATE_KEY=<your_private_key_hex>
```

## Examples

### test_single_order
Basic example that submits a single market order.

```bash
cargo run --example test_single_order --release
```

Uses environment variables or falls back to defaults. Output shows the order response code and transaction hash on success.

### create_market_order
Creates a market order with configurable parameters.

```bash
cargo run --example create_market_order --release
```

### create_limit_order
Creates a limit order with a specified price level.

```bash
cargo run --example create_limit_order --release
```

### cancel_order
Cancels a previously created order.

```bash
cargo run --example cancel_order --release
```

### stress_market_orders
Stress test that submits multiple market orders sequentially.

```bash
STRESS_COUNT=100 STRESS_DELAY_MS=500 cargo run --example stress_market_orders --release
```

Configuration:
- `STRESS_COUNT`: Number of orders to submit (default: 1000)
- `STRESS_DELAY_MS`: Milliseconds between orders (default: 300)

### transfer_update_leverage
Examples for transferring funds and updating leverage.

```bash
cargo run --example transfer_update_leverage --release
```

### check_api_key
Validates your API key configuration.

```bash
cargo run --example check_api_key --release
```

## Signature Fix (December 2025)

All examples now use the corrected Schnorr signature implementation. The bug fix removed an unnecessary `to_canonical()` call that was corrupting scalar values during signature computation.

**Key points:**
- ✅ All signatures are now valid
- ✅ Single order tests: 100% success rate
- ✅ Public key matches Go implementation exactly
- ⚠️ Rate limiting may apply with 1000+ orders - use `STRESS_DELAY_MS` to control submission rate

## Error Codes

- `200`: Success
- `21120`: Invalid signature (now eliminated by fix)
- `23000`: Rate limit exceeded (use delays between orders)
- Other codes: Check API documentation

## Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BASE_URL` | string | https://mainnet.zklighter.elliot.ai | API endpoint |
| `ACCOUNT_INDEX` | i64 | 361816 | Your account index |
| `API_KEY_INDEX` | u8 | 6 | Your API key index |
| `API_PRIVATE_KEY` | hex string | - | Your private key (required) |
| `STRESS_COUNT` | usize | 1000 | Orders to submit in stress test |
| `STRESS_DELAY_MS` | u64 | 300 | Milliseconds between orders |
| `ORDER_BOOK_INDEX` | u8 | 0 | Market to trade on |
| `BASE_AMOUNT` | i64 | 1000 | Order amount |
| `AVG_EXECUTION_PRICE` | i64 | 350000 | Order price |
| `IS_ASK` | bool | false | Buy (0) or sell (1) |

## Troubleshooting

### "Invalid signature" errors
The signature implementation has been fixed. If you see code 21120, ensure you're using the latest code.

### Rate limiting (code 23000)
Increase `STRESS_DELAY_MS` to space out requests. Recommended: 500ms or more for stress tests.

### Connection errors
Check that `BASE_URL` is correct and the server is accessible.

### Authentication errors
Verify `API_KEY_INDEX` and `API_PRIVATE_KEY` are correct and correspond to your account.
