# Troubleshooting Guide

Common issues and solutions when using the Lighter Rust SDK.

## Environment Setup

### Missing `.env` file

**Error**: `API_PRIVATE_KEY not found` or any `env var not set` panic.

**Fix**: Copy `.env.example` to `.env` and fill in your credentials:

```bash
cp .env.example .env
```

Required variables:
```
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=<your account index>
API_KEY_INDEX=<your api key index>
API_PRIVATE_KEY=<your 40-byte private key hex, no 0x prefix>
```

### Invalid private key length

**Error**: `CryptoError::InvalidPrivateKeyLength`

The Lighter SDK uses a 40-byte (80 hex character) Goldilocks scalar private key — not a standard 32-byte Ethereum key. Ensure your `API_PRIVATE_KEY` is exactly 80 hex characters.

---

## API Errors

### 401 Unauthorized / Auth token rejected

- Confirm your `API_KEY_INDEX` matches the key registered on-chain for your account.
- Auth tokens expire; generate a fresh token if the deadline has passed.
- Check that `ACCOUNT_INDEX` matches the account that owns the API key.

### `insufficient available collateral`

Your account does not have enough USDC margin to open the requested position. Deposit collateral or reduce the order size.

### Nonce conflict / `invalid nonce`

The SDK fetches a fresh nonce automatically. If you submit multiple transactions in rapid succession without nonce management, use the manual nonce methods (`sign_create_order_with_nonce`) to avoid races. See `send_tx_batch.rs` for an example.

### `order book index out of range`

Pass a valid `order_book_index`. Use `client.get_order_books()` to list active markets and their indices.

---

## Compilation Errors

### `error[E0560]: struct … has no field 'order_expiry'`

Add `order_expiry: 0` to your `CreateOrderRequest`. This field is required even if unused.

### `cannot find value 'X' in scope` inside examples

Examples require a `.env` file present at the repo root; ensure `dotenv::dotenv().ok()` is called before `env::var(...)`.

---

## WebSocket Issues

### Connection closes immediately

- Ensure your auth token is fresh (not expired).
- Some endpoints require authentication; check the subscription message format in `websocket_stream.rs`.

### Messages arrive as `WsMessage::Unknown`

Add handling for new message types: the exchange may add new message type strings. Log `raw_type` to identify new variants.

---

## Runtime Panics

### `called unwrap() on None` in examples

Most examples assume that the account is in a specific state (e.g., has open orders, has a position). Read the example's comments to understand preconditions before running it.

---

## Benchmark / Stress Tests

Do **not** run `stress_market_orders`, `benchmark_stress`, or `benchmark_stress_parallel` on mainnet without understanding the cost. These examples fire many real orders in rapid succession and will consume significant gas/fees.

---

## See Also

- [Getting Started](./getting-started.md)
- [Running Examples](./running-examples.md)
- [API Methods Reference](./api-methods.md)
