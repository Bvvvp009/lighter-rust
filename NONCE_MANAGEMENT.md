# Nonce Management System

## Overview

The `LighterClient` now implements **optimistic nonce management** to support high-frequency trading without waiting for API responses. This prevents "invalid signature" errors caused by nonce reuse.

## How It Works

### Optimistic Nonce Management

1. **Initial Fetch**: On first use, fetch nonce from API
2. **Local Increment**: Increment nonce locally for subsequent transactions
3. **Failure Handling**: On signature errors, decrement offset to reuse nonce
4. **Periodic Sync**: Optionally refresh from API periodically

### Nonce State

```rust
struct NonceState {
    last_fetched_nonce: i64,  // Last nonce fetched from API (-1 = not initialized)
    nonce_offset: i64,         // How many nonces we've used since last fetch
}
```

**Next Nonce Calculation**: `next_nonce = last_fetched_nonce + nonce_offset + 1`

## API Methods

### 1. Automatic Nonce Management (Default)

```rust
// Uses optimistic nonce management automatically
client.create_market_order(...).await?;
```

### 2. Provide Nonce Explicitly

```rust
// Use specific nonce
client.create_market_order_with_nonce(..., Some(42)).await?;

// Fetch from API (use -1)
client.create_market_order_with_nonce(..., Some(-1)).await?;

// Use optimistic management (None)
client.create_market_order_with_nonce(..., None).await?;
```

### 3. Manual Nonce Management

```rust
// Get next nonce (optimistic)
let nonce = client.get_nonce_or_use(None).await?;

// Get nonce from API
let nonce = client.get_nonce().await?;

// Refresh nonce from API (for periodic sync)
let nonce = client.refresh_nonce().await?;

// Acknowledge failure (decrements offset)
client.acknowledge_failure();
```

## Usage Patterns

### High-Frequency Trading

```rust
// No delays needed - nonce is managed internally
for i in 0..100 {
    client.create_market_order(...).await?;
    // Optional: small delay for rate limiting only
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

### With Periodic Sync

```rust
// Refresh nonce every 50 transactions
for i in 0..100 {
    if i % 50 == 0 {
        client.refresh_nonce().await?;
    }
    client.create_market_order(...).await?;
}
```

### Manual Nonce Control

```rust
// When you need to use a specific nonce (e.g., from external source)
let external_nonce = get_nonce_from_external_source();
client.create_market_order_with_nonce(..., Some(external_nonce)).await?;
```

## Error Handling

The client automatically handles signature errors (code 21120) by:
1. Acknowledging the failure
2. Decrementing the nonce offset
3. Allowing the nonce to be reused

This means you don't need retry logic for nonce-related errors.

## Benefits

1. **No Waiting**: No need to wait for API to process transactions
2. **High Frequency**: Supports very high transaction rates
3. **Automatic**: Works transparently without manual intervention
4. **Flexible**: Can still provide nonces explicitly when needed
5. **Resilient**: Automatically handles failures and nonce reuse

## Implementation Details

- Uses `Arc<Mutex<NonceState>>` for thread-safe nonce management
- Nonce state is shared across all method calls on the same client
- On signature errors, automatically decrements offset to allow retry
- Initial nonce fetch happens automatically on first use

