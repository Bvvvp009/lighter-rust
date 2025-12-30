# Signature Failure Root Cause Investigation

## Executive Summary

We're investigating why 6% of signatures initially fail validation but succeed on retry. This investigation goes beyond symptom management (retry logic) to identify the fundamental root cause.

## Current Situation

### Observed Behavior
- **Initial failure rate**: ~6% of orders fail signature validation
- **Retry success rate**: 94% succeed on retry
- **Net failure rate**: ~0.36% (6% of 6%)

### Critical Questions
1. **Why do signatures fail initially?** - What makes a "bad" signature?
2. **Why do retries succeed?** - What changes between attempt 1 and attempt 2?
3. **Why do some signatures fail even after retries?** - What makes them permanently invalid?

## Investigation Areas

### 1. Nonce Generation Analysis ✓

**Current Implementation:**
```rust
// In signer/src/lib.rs (line 67)
pub fn sign(&self, message: &[u8; 40]) -> Result<[u8; 80]> {
    let nonce_scalar = ScalarField::sample_crypto();  // Random nonce each call
    let nonce_bytes = nonce_scalar.to_bytes_le();
    self.sign_with_fixed_nonce(message, &nonce_bytes)
}
```

**Analysis:**
- Each signature uses `ScalarField::sample_crypto()` which generates a **new random nonce**
- Nonce generation uses `rand::thread_rng()` - cryptographically secure
- No nonce reuse between signatures (✓ correct)
- **FINDING**: Nonce generation is non-deterministic by design (correct for Schnorr)

**Question**: Why would the same transaction with a different nonce succeed when retried?

### 2. Signature Algorithm Review ✓

**Schnorr Signing Process:**
```
1. Generate random nonce k: k = ScalarField::sample_crypto()
2. Compute R = k * G (nonce times generator point)
3. Encode R as Fp5Element
4. Compute challenge: e = H(R || message)  [Poseidon2 hash]
5. Compute response: s = k - e * sk  [where sk is private key]
6. Signature = (s, e)  [80 bytes total]
```

**Verification Process:**
```
1. Parse signature: (s, e)
2. Recompute R': R' = s*G + e*PublicKey
3. Recompute e': e' = H(R' || message)
4. Verify: e' == e
```

**Analysis:**
- Implementation follows standard Schnorr signature scheme
- All tests pass with 100% verification success
- Cryptographic primitives are correct

**Question**: If the algorithm is correct, why would server reject valid signatures?

### 3. Transaction Data Serialization

**JSON Serialization Order:**
```rust
// api-client/src/lib.rs (lines 569-585)
let tx_info = json!({
    "AccountIndex": self.account_index,
    "ApiKeyIndex": self.api_key_index,
    "MarketIndex": order.order_book_index,
    "ClientOrderIndex": order.client_order_index,
    "BaseAmount": order.base_amount,
    "Price": order.price,
    "IsAsk": if order.is_ask { 1 } else { 0 },
    "Type": order.order_type,
    "TimeInForce": order.time_in_force,
    "ReduceOnly": if order.reduce_only { 1 } else { 0 },
    "TriggerPrice": order.trigger_price,
    "OrderExpiry": order_expiry,
    "ExpiredAt": expired_at,
    "Nonce": nonce,
    "Sig": ""
});
```

**Potential Issue**: JSON field ordering
- JSON spec says objects are unordered
- Signature computed on serialized string
- Different field order → different string → different hash → invalid signature

**Investigation Needed**: 
- Does `serde_json::to_string()` guarantee field order?
- Does the server expect a specific field order?

### 4. Timing Dependencies

**Timestamp Generation:**
```rust
let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
let expired_at = now + 599_000; // 10 minutes - 1 second
```

**Potential Issues:**
1. **Clock Skew**: Client clock vs server clock difference
2. **Network Latency**: Signature created at time T1, validated at server time T2
3. **Nonce Sequence**: Server expects nonces in order, race conditions possible

**Hypothesis**: If server validates timestamp BEFORE signature:
- Expired timestamp → reject immediately
- Valid timestamp + bad signature → signature validation fails
- Retry with fresh timestamp → succeeds

### 5. Server-Side Validation Logic

**What we don't know:**
1. Does server validate timestamp before signature?
2. Does server have any signature caching?
3. Are there rate limits on signature validation?
4. Does server use a signature validation queue?

**Critical Test Needed**: Send same signature twice:
- If both fail: signature is truly invalid
- If first fails, second succeeds: server state issue

### 6. Race Conditions in Parallel Requests

**Current behavior:**
- Stress test sends many orders in parallel
- Each order has independent nonce
- Nonce cache manages sequential nonce allocation

**Potential race condition:**
```
Thread 1: Get nonce 100, create signature, send
Thread 2: Get nonce 101, create signature, send
Network: Request 2 arrives before Request 1
Server: Expects nonce 100, got 101 → reject
Server: Then gets nonce 100 → accept
```

**Test**: Send orders sequentially vs parallel and compare failure rates

## Proposed Investigation Steps

### Step 1: Signature Forensics Tool ✓ (Next)
Create a tool that captures:
- Input data (transaction JSON before hashing)
- Hash output (Poseidon2 hash)
- Nonce value used
- Signature components (s, e)
- Server response (success/fail)

Save all data to allow analysis of failed vs successful signatures.

### Step 2: Deterministic Signature Testing
Test with **fixed nonce**:
```rust
// Use same nonce for same transaction
let nonce_bytes = [1u8; 40]; // Fixed nonce
let sig1 = sign_with_fixed_nonce(msg, &nonce_bytes);
let sig2 = sign_with_fixed_nonce(msg, &nonce_bytes);
assert_eq!(sig1, sig2); // Should be identical
```

If identical signatures still fail/succeed non-deterministically → server-side issue

### Step 3: JSON Serialization Consistency
Compare multiple serializations:
```rust
for _ in 0..100 {
    let json1 = serde_json::to_string(&tx_info)?;
    let json2 = serde_json::to_string(&tx_info)?;
    assert_eq!(json1, json2); // Must be deterministic
}
```

### Step 4: Timing Analysis
Add microsecond timestamps to telemetry:
- Time signature created
- Time request sent
- Time server responded
- Correlate timing with success/failure

### Step 5: Sequential vs Parallel Testing
```rust
// Test 1: Sequential orders (no concurrency)
for i in 0..100 {
    create_order(i).await?;
}

// Test 2: Parallel orders (concurrency)
let handles: Vec<_> = (0..100)
    .map(|i| tokio::spawn(create_order(i)))
    .collect();
```

Compare failure rates: If parallel has higher failure → race condition

### Step 6: Server Response Analysis
Capture exact error messages:
- "invalid signature" → crypto validation failed
- "invalid nonce" → nonce sequence issue  
- "expired" → timestamp issue
- Other errors?

## Hypotheses Ranked by Likelihood

### Hypothesis 1: JSON Serialization Non-Determinism (HIGH)
**Symptoms**: Random failures, retries with same data succeed
**Cause**: JSON field order varies, causing different hashes
**Test**: Lock down serialization order, verify determinism
**Fix**: Use deterministic serialization (ordered maps)

### Hypothesis 2: Nonce Race Condition (MEDIUM-HIGH)
**Symptoms**: Failures in parallel requests, sequential works better
**Cause**: Nonces arrive out of order at server
**Test**: Sequential vs parallel comparison
**Fix**: Implement nonce synchronization or server-side out-of-order handling

### Hypothesis 3: Server Validation Queue Backup (MEDIUM)
**Symptoms**: Random failures under load, retries succeed
**Cause**: Server drops/rejects requests under load
**Test**: Monitor failure rate vs request rate
**Fix**: Client-side rate limiting and backoff

### Hypothesis 4: Timestamp Clock Skew (LOW-MEDIUM)
**Symptoms**: Failures at specific times of day
**Cause**: Client/server clock difference
**Test**: Add clock skew adjustment, monitor correlation
**Fix**: Implement clock synchronization

### Hypothesis 5: Signature Algorithm Bug (LOW)
**Symptoms**: Tests pass, but production fails
**Cause**: Edge case in arithmetic or encoding
**Test**: Comprehensive test vectors from other SDKs
**Fix**: Match reference implementation exactly

## Expected Outcomes

### If Root Cause is Client-Side:
- Fix the issue (e.g., deterministic serialization)
- Failure rate drops to near-zero
- No retries needed

### If Root Cause is Server-Side:
- Document the issue with evidence
- Implement robust retry strategy (current approach)
- Consider reporting to Lighter Protocol team

### If Root Cause is Network/Timing:
- Implement adaptive timing strategies
- Add clock synchronization
- Optimize request pacing

## Success Criteria

Investigation is complete when we can:
1. **Reproduce failures deterministically** - Know exactly when/why a signature fails
2. **Explain retry success** - Understand what makes the second attempt work
3. **Predict failures** - Given transaction data, predict if signature will fail
4. **Fix or mitigate** - Either eliminate failures or implement optimal mitigation

## Next Actions

1. ✅ Build signature forensics diagnostic tool
2. ⏸️ Run controlled experiments (deterministic nonces)
3. ⏸️ Analyze serialization consistency
4. ⏸️ Compare sequential vs parallel failure rates
5. ⏸️ Correlate timing data with failures
6. ⏸️ Document findings and root cause
