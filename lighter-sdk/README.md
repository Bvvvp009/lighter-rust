# `lighter-sdk`

Unified umbrella crate for the Lighter Rust SDK.

## What it exports

- `api_client` — async REST + WebSocket client types
- `signer` — key management and transaction signing
- `goldilocks_crypto` — Schnorr / ECgFp5 primitives
- `poseidon_hash` — Poseidon2 and Goldilocks field utilities

## Quick start

A safe mainnet smoke-check example is available at `examples/mainnet_readonly_smoke.rs`. It validates the broader account/market/history/token/metrics surface and sign-only flows without sending live orders.

A curated one-command validation matrix is available at `examples/mainnet_validation_matrix.rs`. It prints a table covering auth, API-key verification, read-only REST checks, and—when explicitly enabled—safe live trading checks.

A separate live but self-cleaning example is available at `examples/mainnet_live_order_cycle.rs` for placing a tiny real order, cancelling/closing it immediately, and ending with no open position.

```toml
[dependencies]
lighter-sdk = { path = "../lighter-rust/lighter-sdk" }
```

```rust,no_run
use lighter_sdk::{LighterClient, KeyManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_private_key = "<API_PRIVATE_KEY_HEX>";

    let _key_manager = KeyManager::from_hex(api_private_key)?;

    let client = LighterClient::new(
        "https://mainnet.zklighter.elliot.ai".to_string(),
        api_private_key,
        361816,
        4,
    )?;

    let _status = client.get_status().await?;
    Ok(())
}
```
