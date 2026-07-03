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

### Recommended start order

If you want the most polished path, run these first:

1. `check_api_key` — verifies your configured credentials against mainnet
2. `create_auth_token` — generates ready-to-use auth tokens
3. `lighter-sdk/examples/mainnet_readonly_smoke.rs` — broad safe read-only validation
4. `lighter-sdk/examples/mainnet_validation_matrix.rs` — table-style validation with optional live order checks
5. `create_modify_cancel_flow` — targeted order lifecycle demo

### Order Management

#### Basic Orders

**create_market_buy** - Buy order at market price
```bash
cargo run --example create_market_buy --release
```

**create_market_sell** - Sell order at market price
```bash
cargo run --example create_market_sell --release
```

**create_market_order** - Market order with configurable parameters
```bash
cargo run --example create_market_order --release
```

**create_market_with_slippage** - Market order with slippage protection
```bash
cargo run --example create_market_with_slippage --release
```

**create_limit_order** - Limit order at specific price
```bash
cargo run --example create_limit_order --release
```

**create_modify_cancel_flow** - Full lifecycle: create → modify → cancel
```bash
cargo run --example create_modify_cancel_flow --release
```

**modify_order** - Modify an existing order
```bash
cargo run --example modify_order --release
```

**cancel_order** - Cancel a single order
```bash
cargo run --example cancel_order --release
```

**cancel_all_orders** - Cancel all open orders
```bash
cargo run --example cancel_all_orders --release
```

#### Advanced Orders

**create_sl_tp** - Create Stop Loss and Take Profit orders
```bash
cargo run --example create_sl_tp --release
```

**grouped_order_sl_tp** - Grouped order structure with attached SL/TP
```bash
cargo run --example grouped_order_sl_tp --release
```

**close_position** - Close an entire position with market order
```bash
cargo run --example close_position --release
```

### Margin & Leverage

**update_leverage_cross_20x** - Set 20x cross margin leverage
```bash
cargo run --example update_leverage_cross_20x --release
```

**update_leverage_isolated_50x** - Set 50x isolated margin leverage
```bash
cargo run --example update_leverage_isolated_50x --release
```

### Sub-Accounts

**create_sub_account** - Create a new sub-account
```bash
cargo run --example create_sub_account --release
```

### Authentication & Setup

**create_auth_token** - Generate authentication token
```bash
cargo run --example create_auth_token --release
```

**setup_api_key** - Setup and validate API key
```bash
cargo run --example setup_api_key --release
```

**check_api_key** - Validate API key configuration
```bash
cargo run --example check_api_key --release
```

### Performance & Utilities

**send_tx_batch** - Batch multiple transactions together
```bash
cargo run --example send_tx_batch --release
```

**stress_market_orders** - Stress test with multiple orders
```bash
STRESS_COUNT=100 STRESS_DELAY_MS=500 cargo run --example stress_market_orders --release
```

**benchmark_stress** - Performance benchmark
```bash
cargo run --example benchmark_stress --release
```

## Signature Compatibility Notes

All examples use the corrected Schnorr signature implementation. The bug fix removed an unnecessary `to_canonical()` call that was corrupting scalar values during signature computation.

**Key points:**
- All signatures are now valid.
- Single order tests pass consistently.
- Public key derivation matches the Go implementation.
- Rate limiting may apply with 1000+ orders; use `STRESS_DELAY_MS` to control submission rate.

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
| `LIVE_PRECHECK_LEVERAGE_MULTIPLIER` | f64 | 3 / 20 | Collateral precheck multiplier for live examples |
| `FORCE_LIVE_ORDER_ATTEMPT` | bool | false | Force a live attempt even if the conservative local precheck would skip |
| `RUN_LIVE_ORDERS` | bool | false | Enable live order steps in `mainnet_validation_matrix` |

## Troubleshooting

### "Invalid signature" errors
The signature implementation has been fixed. If you see code 21120, ensure you're using the latest code.

### Rate limiting (code 23000)
Increase `STRESS_DELAY_MS` to space out requests. Recommended: 500ms or more for stress tests.

### Connection errors
Check that `BASE_URL` is correct and the server is accessible.

### Authentication errors
Verify `API_KEY_INDEX` and `API_PRIVATE_KEY` are correct and correspond to your account.
