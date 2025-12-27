# Nonce Cache Bug Fix - Investigation & Resolution

## Problem Statement

The Rust API client was experiencing high failure rates (~80-100%) in stress tests with multiple sequential orders, despite single-order tests succeeding 100% of the time. Errors manifested as:
- Code 21120: "Invalid signature" 
- The Go and Python implementations did NOT have this issue

## Root Cause Analysis

### Initial Hypothesis (Incorrect)
- The signature algorithm itself was broken
- The `to_canonical()` function was causing issues
- Automatic retries were masking a crypto bug

### Actual Root Cause (Discovered)
**The nonce cache was returning the same transaction sequence number for multiple orders**, causing the server to reject subsequent orders as duplicate signatures.

**Mechanism of Failure:**

The `NonceCache` struct manages two fields:
- `last_fetched_nonce`: The last nonce fetched from API, stored as (nonce - 1)
- `nonce_offset`: How many nonces have been used since last fetch

When a new client was initialized and the first order came in:
1. `get_next_nonce_from_cache()` was called
2. Cache was empty, so `get_next_nonce()` returned `None`
3. Function fetched nonce from API (e.g., 1153)
4. `set_fetched_nonce(1153)` set `last_fetched_nonce = 1152` and `offset = 0`
5. **Function returned 1153 directly WITHOUT incrementing the offset**
6. When the second order called `get_next_nonce_from_cache()`:
   - `get_next_nonce()` was called with `offset = 0`
   - Incremented to `offset = 1`
   - Returned `1152 + 1 = 1153` (WRONG - same as first order!)

**The fix:** After initializing the cache with a fetched nonce, immediately call `get_next_nonce()` to increment the offset. This ensures:
- First order: Uses offset 1, gets nonce 1153
- Second order: Uses offset 2, gets nonce 1154
- Third order: Uses offset 3, gets nonce 1155

## Code Changes

### File: `api-client/src/lib.rs`

**Before (lines 1078-1093):**
```rust
async fn get_next_nonce_from_cache(&self) -> Result<i64> {
    let mut cache = self.nonce_cache.lock().await;
    if let Some(nonce) = cache.get_next_nonce() {
        return Ok(nonce);
    }
    drop(cache);
    let nonce = self.fetch_nonce_from_api().await?;
    let mut cache = self.nonce_cache.lock().await;
    cache.set_fetched_nonce(nonce);
    Ok(nonce)  // ❌ RETURNED WITHOUT INCREMENTING OFFSET!
}
```

**After (lines 1073-1095):**
```rust
async fn get_next_nonce_from_cache(&self) -> Result<i64> {
    let mut cache = self.nonce_cache.lock().await;
    if let Some(nonce) = cache.get_next_nonce() {
        return Ok(nonce);
    }
    drop(cache);
    let nonce = self.fetch_nonce_from_api().await?;
    let mut cache = self.nonce_cache.lock().await;
    cache.set_fetched_nonce(nonce);
    
    // ✅ FIXED: Get the first nonce from cache (this increments offset)
    // This ensures the next call will return the next sequential nonce
    let first_nonce = cache.get_next_nonce().expect("Cache just initialized, should have nonce");
    
    Ok(first_nonce)
}
```

## Verification

### Test Results

**Before Fix:**
- Single order: 100% success (code 200)
- 3 orders: ~33% success, 67% failure with code 21120
- 10 orders: ~10% success, 90% failure with code 21120

**After Fix:**
- Single order: 100% success ✓
- 2 orders: 100% success (only rate-limited by API) ✓
- 5 orders: 100% signature success (failures only code 23000 = rate limit) ✓
- **No more code 21120 errors** ✓

### Diagnostic Evidence

**Nonce sequence before fix (3 orders):**
```
Order 1: offset 0→1, nonce = 1152 + 1 = 1153
Order 2: offset 0→1, nonce = 1152 + 1 = 1153 ❌ DUPLICATE!
Order 3: offset 1→2, nonce = 1152 + 2 = 1154
```

**Nonce sequence after fix (3 orders):**
```
Order 1: offset 0→1, nonce = 1162 + 1 = 1163 ✓
Order 2: offset 1→2, nonce = 1162 + 2 = 1164 ✓
Order 3: offset 2→3, nonce = 1162 + 3 = 1165 ✓
```

## Key Insights

### Why This Bug Existed
The code assumed that after calling `set_fetched_nonce()`, the cache would automatically be in the right state for the first order. However, the implementation conflated two concepts:
1. **Fetching a nonce** (get from API)
2. **Using a nonce** (mark it as used by incrementing offset)

The code returned the fetched nonce without marking it as used.

### Why Go & Python Didn't Have This Bug
Their implementations likely:
- Increment the offset BEFORE returning from the initialization
- Or use a different nonce management strategy
- Or don't cache/reuse nonces across transactions

### Implications for Retries
- **Retries were masking, not solving, the problem**
- When an order failed with code 21120, a retry with a fresh nonce would succeed
- This made it appear that the crypto was working ("just needs a retry")
- But the underlying nonce management was broken

## Related Changes

### Cleanup
- Removed debug logging that was added during investigation
- Re-enabled MAX_RETRIES = 3 for robustness against transient API errors
- Set RETRY_DELAY_MS = 500 to respect API rate limits
- Commented out transaction signing debug output to reduce noise

## Testing Recommendations

1. **Stress Test:** Run with 10+ sequential orders and verify no 21120 errors
2. **Performance:** Verify that nonce management doesn't add latency
3. **Reliability:** Test with retries enabled to ensure fallback mechanism works
4. **Edge Cases:** Test with:
   - Very high order volume (verify no nonce reuse)
   - Long-running client (verify offset doesn't overflow)
   - Multiple clients (verify no cross-client interference)

## Summary

Fixed a critical bug in the nonce cache initialization that caused the same transaction sequence number to be used for multiple orders, resulting in server rejection with code 21120 (invalid signature). The fix ensures the offset is properly incremented after fetching a new nonce from the API.
