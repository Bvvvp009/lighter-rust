# Nonce Management Comparison: Python SDK vs TypeScript SDK vs Rust Implementation

## Python SDK (`lighter-python`)

### 1. OptimisticNonceManager (Default)
```python
# Initialization
self.nonce = {
    api_key_index: get_nonce_from_api(...) - 1  # Store as nonce - 1
    for api_key_index in range(start_api_key, end_api_key + 1)
}

# Get next nonce
def next_nonce(self) -> Tuple[int, int]:
    self.current_api_key = increment_circular(...)
    self.nonce[self.current_api_key] += 1  # Increment first
    return (self.current_api_key, self.nonce[self.current_api_key])

# On failure
def acknowledge_failure(self, api_key_index: int) -> None:
    self.nonce[api_key_index] -= 1  # Decrement to allow retry
```

**Key Points:**
- Stores `nonce - 1` initially (matches our Rust implementation)
- Increments locally without fetching from API
- Decrements on failure to allow retry
- No delay between nonce fetches (optimistic)

### 2. ApiNonceManager (Conservative)
```python
def next_nonce(self) -> Tuple[int, int]:
    """
    It is recommended to wait at least 350ms before using the same api key.
    Please be mindful of your transaction frequency when using this nonce manager.
    """
    self.current_api_key = increment_circular(...)
    self.nonce[self.current_api_key] = get_nonce_from_api(...)  # Always fetch
    return (self.current_api_key, self.nonce[self.current_api_key])
```

**Key Points:**
- Always fetches from API (no optimistic increments)
- **Recommends 350ms wait** (not 100ms!)
- Slower but more reliable

### 3. Error Handling
```python
# In signer_client.py
if "invalid nonce" in str(e):
    self.nonce_manager.hard_refresh_nonce(api_key_index)  # Hard refresh
    return None, None, trim_exc(str(e))
else:
    self.nonce_manager.acknowledge_failure(api_key_index)  # Decrement
```

**Key Points:**
- On "invalid nonce" error: Hard refresh (fetch from API)
- On other errors: Acknowledge failure (decrement)

---

## TypeScript SDK (`lighter-ts`)

### 1. NonceCache (Batch Pre-fetching)
```typescript
class NonceCache {
  private batchSize = 20;  // Pre-fetch 20 nonces at a time
  private maxCacheAge = 30000;  // 30 seconds

  async getNextNonce(apiKeyIndex: number): Promise<number> {
    // If cache empty or expired, fetch new batch
    if (!nonces || nonces.length === 0 || this.isCacheExpired()) {
      await this.refreshNonces(apiKeyIndex);
    }
    
    // Return first nonce and remove from cache
    const nonceInfo = cachedNonces.shift()!;
    
    // Pre-fetch more when cache gets low (<= 2 remaining)
    if (cachedNonces.length <= 2) {
      this.refreshNonces(apiKeyIndex).catch(() => {});
    }
    
    return nonceInfo.nonce;
  }

  // Fetch callback calculates sequential nonces
  async (apiKeyIndex: number, count: number) => {
    const firstNonceResult = await this.transactionApi.getNextNonce(...);
    const nonces: number[] = [];
    for (let i = 0; i < count; i++) {
      nonces.push(firstNonceResult.nonce + i);  // Optimistic calculation
    }
    return nonces;
  }
}
```

**Key Points:**
- Pre-fetches batches of 20 nonces
- Calculates sequential nonces optimistically: `firstNonce + i`
- Cache expires after 30 seconds
- Pre-fetches more when cache gets low (<= 2 remaining)
- Very efficient for high-frequency trading

### 2. SingleKeyNonceManager (Simple Optimistic)
```typescript
class SingleKeyNonceManager {
  private nonceOffset: number = 0;
  private lastFetchedNonce: number = -1;

  async getNextNonce(): Promise<number> {
    if (this.lastFetchedNonce === -1) {
      return await this.getCurrentNonce();
    }
    this.nonceOffset++;
    return this.lastFetchedNonce + this.nonceOffset;  // Optimistic increment
  }
}
```

**Key Points:**
- Similar to Python's OptimisticNonceManager
- Uses `lastFetchedNonce + nonceOffset` pattern
- No delay between increments

---

## Rust Implementation (Current)

### Current Implementation
```rust
struct NonceState {
    last_fetched_nonce: i64,  // Stored as nonce - 1
    nonce_offset: i64,
    last_fetch_time: Option<Instant>,
    used_nonces: HashSet<i64>,
}

fn get_next_nonce(&mut self) -> i64 {
    self.nonce_offset += 1;
    self.last_fetched_nonce + self.nonce_offset  // Matches Python
}

fn set_fetched_nonce(&mut self, nonce: i64) {
    self.last_fetched_nonce = nonce - 1;  // Matches Python
    self.nonce_offset = 0;
}

fn acknowledge_failure(&mut self) {
    if self.nonce_offset > 0 {
        self.nonce_offset -= 1;  // Matches Python
    }
}
```

**Current Behavior:**
- Matches Python's OptimisticNonceManager pattern
- Fetches from API with 100ms delay (matching Go signer)
- Tracks used nonces to prevent duplicates
- Only marks nonces as used on success (code 200)

---

## Key Differences & Recommendations

### 1. Delay Recommendations
- **Go Signer**: 100ms delay between transactions
- **Python ApiNonceManager**: **350ms recommended** wait
- **TypeScript**: No explicit delay (relies on batch pre-fetching)
- **Our Rust**: Currently 100ms (matching Go)

**Recommendation**: Consider increasing to 200-350ms for better reliability, or implement batch pre-fetching like TypeScript.

### 2. Batch Pre-fetching (TypeScript Approach)
TypeScript's NonceCache is very efficient:
- Pre-fetches 20 nonces at once
- Calculates sequential nonces optimistically
- Pre-fetches more when cache gets low

**Recommendation**: Consider implementing batch pre-fetching for high-frequency trading scenarios.

### 3. Error Handling
- **Python**: Hard refresh on "invalid nonce", acknowledge failure on others
- **TypeScript**: Relies on cache refresh
- **Our Rust**: Currently refreshes on invalid nonce, removes from used set on invalid signature

**Recommendation**: Our current approach is good, but we should ensure we're handling all error cases correctly.

### 4. Nonce Tracking
- **Python**: No duplicate tracking (relies on API)
- **TypeScript**: Cache prevents duplicates within batch
- **Our Rust**: Tracks used nonces in HashSet

**Recommendation**: Our duplicate tracking is good, but we should consider batch pre-fetching to reduce API calls.

---

## Suggested Improvements for Rust Implementation

1. **Implement Batch Pre-fetching** (like TypeScript):
   - Pre-fetch 10-20 nonces at once
   - Calculate sequential nonces optimistically
   - Pre-fetch more when cache gets low

2. **Increase Delay** (like Python ApiNonceManager):
   - Consider 200-350ms delay instead of 100ms
   - Or make it configurable

3. **Add Cache Expiration** (like TypeScript):
   - Expire cache after 30 seconds
   - Refresh if cache is stale

4. **Better Error Recovery**:
   - Hard refresh on "invalid nonce" errors (like Python)
   - Keep current behavior for invalid signature errors

