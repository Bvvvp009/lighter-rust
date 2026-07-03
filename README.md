# `lighter-rust`

Rust SDK and native crypto stack for the **Lighter** exchange.

It includes:
- a typed async REST client
- WebSocket support
- native signing and auth-token generation
- Goldilocks / Poseidon2 / ECgFp5 crypto primitives
- verified mainnet examples for safe read-only and self-cleaning live flows

> ⚠️ **Security note:** the crypto stack has an internal hardening report in `docs/crypto-internal-audit-report.md`, but there is **no external independent cryptography audit sign-off yet**.

---

## Workspace crates

| Crate | Folder | Purpose |
|---|---|---|
| `poseidon-hash` | `poseidon-hash/` | Goldilocks field arithmetic, Poseidon2, Fp5, Merkle utilities |
| `goldilocks-crypto` | `crypto/` | ECgFp5 curve operations, Schnorr signing, batch verification |
| `signer` | `signer/` | key management, auth-token generation, transaction signing |
| `api-client` | `api-client/` | async REST client and order-management examples |
| `lighter-sdk` | `lighter-sdk/` | umbrella crate re-exporting the full Rust SDK surface |

---

## Current status

As of **Apr 10, 2026**, the SDK is in a strong release-ready state for internal/public pushing:

| Verification | Command | Result |
|---|---|---|
| Full workspace tests | `cargo test -q --workspace --all-targets` | ✅ Passed |
| Strict lint | `cargo clippy -q --workspace --all-targets -- -D warnings` | ✅ Passed |
| Safe mainnet validation | `cargo run -q -p lighter-sdk --example mainnet_validation_matrix` | ✅ Passed |
| Leveraged live order cycle | `RUN_LIVE_ORDERS=true ... cargo run -q -p lighter-sdk --example mainnet_validation_matrix` | ✅ Passed with cleanup |

Recent live validation confirmed:
- real mainnet auth works
- REST/account/market endpoints respond correctly
- live create/cancel and open/close flows succeed under leverage
- test runs finish with **`open_orders_after=0`** and **no residual test position**

---

## Quick start

### Add the umbrella crate

```toml
[dependencies]
lighter-sdk = { path = "./lighter-sdk" }
```

### Minimal example

```rust,no_run
use lighter_sdk::LighterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::new(
        "https://mainnet.zklighter.elliot.ai".to_string(),
        "<API_PRIVATE_KEY_HEX>",
        361816,
        4,
    )?;

    let status = client.get_status().await?;
    println!("network_id={:?} timestamp={:?}", status.network_id, status.timestamp);
    Ok(())
}
```

---

## Environment

Most live examples read the following from `.env` or process env:

```env
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=<your_account_index>
API_KEY_INDEX=<your_api_key_index>
API_PRIVATE_KEY=<your_private_key_hex>
ORDER_BOOK_INDEX=0
```

Useful optional flags for the mature live examples:

| Variable | Description |
|---|---|
| `RUN_LIVE_ORDERS=true` | enables live order steps in the validation matrix |
| `LIVE_PRECHECK_LEVERAGE_MULTIPLIER=20` | adjusts the local collateral precheck |
| `FORCE_LIVE_ORDER_ATTEMPT=true` | forces a live attempt past the conservative precheck |

---

## Recommended validation flow

If you want the cleanest professional path, run these in order:

### 1. Validate credentials
```bash
cargo run -q -p api-client --example check_api_key
cargo run -q -p api-client --example create_auth_token
```

### 2. Safe read-only smoke check
```bash
cargo run -q -p lighter-sdk --example mainnet_readonly_smoke
```

### 3. One-command validation matrix
```bash
cargo run -q -p lighter-sdk --example mainnet_validation_matrix
```

### 4. Leveraged live validation
```bash
RUN_LIVE_ORDERS=true \
LIVE_PRECHECK_LEVERAGE_MULTIPLIER=20 \
FORCE_LIVE_ORDER_ATTEMPT=true \
cargo run -q -p lighter-sdk --example mainnet_validation_matrix
```

This last command verifies:
- auth token generation
- API key verification
- core read-only REST coverage
- leverage update
- limit create/cancel
- tiny market open/close
- cleanup back to a neutral state

---

## Example maturity highlights

The example suite now includes:

| Example | Purpose |
|---|---|
| `api-client/examples/check_api_key.rs` | mainnet-safe API key verification |
| `api-client/examples/create_auth_token.rs` | ready-to-use auth token generation |
| `api-client/examples/create_modify_cancel_flow.rs` | order lifecycle demo with leverage-aware safety checks |
| `lighter-sdk/examples/mainnet_readonly_smoke.rs` | broad read-only + sign-only validation |
| `lighter-sdk/examples/mainnet_live_order_cycle.rs` | self-cleaning live order cycle |
| `lighter-sdk/examples/mainnet_validation_matrix.rs` | polished one-command table-style validation |

---

## Security and audit posture

- `#![forbid(unsafe_code)]` is used in the crypto crates
- private keys are zeroized with `zeroize`
- Go-reference vectors and fuzz scaffolding were added for the crypto stack
- the internal report is available at `docs/crypto-internal-audit-report.md`
- **external independent cryptography review is still recommended before broad production promotion**

---

## Development commands

```bash
# Full verification
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings

# Safe SDK validation
cargo run -p lighter-sdk --example mainnet_validation_matrix

# Live validation with leverage
RUN_LIVE_ORDERS=true LIVE_PRECHECK_LEVERAGE_MULTIPLIER=20 FORCE_LIVE_ORDER_ATTEMPT=true \
  cargo run -p lighter-sdk --example mainnet_validation_matrix
```

---

## Documentation

- Root docs index: [`docs/README.md`](./docs/README.md)
- Internal crypto hardening report: [`docs/crypto-internal-audit-report.md`](./docs/crypto-internal-audit-report.md)
- Audit-readiness note: [`docs/crypto-audit-readiness.md`](./docs/crypto-audit-readiness.md)
- Umbrella crate readme: [`lighter-sdk/README.md`](./lighter-sdk/README.md)
- API examples readme: [`api-client/examples/README.md`](./api-client/examples/README.md)

---

## License

See the crate-level license files in the workspace for the applicable terms.

