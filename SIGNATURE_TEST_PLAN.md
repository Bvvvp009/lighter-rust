# Deep Signature Investigation - Test Plan

## Objective
Identify the root cause of 6% signature validation failures through systematic testing.

## Test Suite

### Test 1: JSON Serialization Determinism ⚠️ CRITICAL
**Hypothesis**: JSON field order varies, causing signature inconsistency

**Test Code**:
```rust
// Test if serde_json produces deterministic output
let tx_info = create_tx_info();
let mut json_outputs = Vec::new();

for _ in 0..1000 {
    let json = serde_json::to_string(&tx_info)?;
    json_outputs.push(json);
}

// All outputs must be identical
let first = &json_outputs[0];
assert!(json_outputs.iter().all(|j| j == first), 
    "JSON serialization is NON-DETERMINISTIC!");
```

**Expected Result**: All 1000 serializations should be identical
**If fails**: This is the root cause - fix serialization

---

### Test 2: Deterministic Signature Verification
**Hypothesis**: Same input should always produce same signature

**Test Code**:
```rust
let private_key = /* test key */;
let message = /* fixed message */;
let nonce = [1u8; 40]; // Fixed nonce

let sig1 = sign_with_fixed_nonce(&private_key, &message, &nonce)?;
let sig2 = sign_with_fixed_nonce(&private_key, &message, &nonce)?;

assert_eq!(sig1, sig2, "Signatures must be deterministic with fixed nonce");

// Verify both
assert!(verify_signature(&sig1, &message, &public_key)?);
assert!(verify_signature(&sig2, &message, &public_key)?);
```

**Expected Result**: Identical signatures, both valid
**If fails**: Bug in signing implementation

---

### Test 3: Server Response to Identical Signatures
**Hypothesis**: Server state affects validation

**Test Code**:
```rust
// Create order with fixed nonce (deterministic signature)
let order = /* test order */;

// Send same order twice (same signature)
let response1 = client.create_order_with_fixed_nonce(order.clone(), fixed_nonce).await?;
let response2 = client.create_order_with_fixed_nonce(order.clone(), fixed_nonce).await?;

if response1.code == 200 && response2.code != 200 {
    println!("⚠️  FOUND IT: Server rejects duplicate nonce!");
} else if response1.code != 200 && response2.code == 200 {
    println!("⚠️  FOUND IT: Server state affects validation!");
}
```

**Expected Result**: Both should fail with "duplicate nonce" or both succeed
**If inconsistent**: Server-side validation issue

---

### Test 4: Sequential vs Parallel Failure Rates
**Hypothesis**: Race conditions in parallel requests

**Test Code**:
```rust
// Test 1: Sequential
let mut sequential_failures = 0;
for i in 0..100 {
    let result = client.create_order(order(i)).await?;
    if result.code != 200 {
        sequential_failures += 1;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// Test 2: Parallel
let parallel_results = futures::future::join_all(
    (0..100).map(|i| client.create_order(order(i)))
).await;
let parallel_failures = parallel_results.iter()
    .filter(|r| r.code != 200)
    .count();

println!("Sequential failures: {}%", sequential_failures);
println!("Parallel failures: {}%", parallel_failures);
```

**Expected Result**: Similar failure rates
**If parallel >> sequential**: Race condition or server congestion

---

### Test 5: Nonce Sequence Violation Detection
**Hypothesis**: Out-of-order nonces cause failures

**Test Code**:
```rust
// Send nonces deliberately out of order
let base_nonce = fetch_nonce().await?;

// Send nonce+2 first
let r1 = client.create_order_with_nonce(order1, Some(base_nonce + 2)).await?;
// Then nonce+1
let r2 = client.create_order_with_nonce(order2, Some(base_nonce + 1)).await?;
// Then nonce+0
let r3 = client.create_order_with_nonce(order3, Some(base_nonce + 0)).await?;

// Check which ones fail
```

**Expected Result**: Server should reject out-of-order nonces
**If out-of-order accepted**: Nonce validation is lenient

---

### Test 6: Timestamp Expiry Correlation
**Hypothesis**: Clock skew causes some failures

**Test Code**:
```rust
// Test with various timestamp skews
for skew_ms in [-5000, -1000, 0, 1000, 5000] {
    std::env::set_var("EXPIRED_AT_SKEW_MS", skew_ms.to_string());
    
    let result = client.create_order(order()).await?;
    println!("Skew {}ms: code {}", skew_ms, result.code);
}
```

**Expected Result**: All should succeed (within 10min expiry window)
**If some fail**: Clock skew is relevant

---

### Test 7: Signature Component Analysis
**Hypothesis**: Specific signature values cause issues

**Test Code**:
```rust
// Capture 1000 signatures (both success and fail)
let mut successful_sigs = Vec::new();
let mut failed_sigs = Vec::new();

for i in 0..1000 {
    let (sig, response) = create_and_capture_order(i).await?;
    
    if response.code == 200 {
        successful_sigs.push(sig);
    } else {
        failed_sigs.push(sig);
    }
}

// Analyze patterns
analyze_signature_patterns(&successful_sigs, &failed_sigs);
```

**Analysis**:
- Check if failed signatures have unusual `s` or `e` values
- Check for edge cases (very large/small values)
- Check byte patterns

---

## Implementation Priority

### Phase 1: Quick Tests (1-2 hours)
1. ✅ JSON serialization determinism
2. ✅ Deterministic signature verification
3. ⏸️ Server identical signature response

### Phase 2: Behavioral Tests (2-3 hours)
4. ⏸️ Sequential vs parallel
5. ⏸️ Nonce sequence violation
6. ⏸️ Timestamp skew correlation

### Phase 3: Deep Analysis (3-4 hours)
7. ⏸️ Signature component analysis
8. ⏸️ Statistical correlation analysis
9. ⏸️ Root cause identification

---

## Success Criteria

We've found the root cause when:
1. We can **reproduce failures reliably** (>80% reproduction rate)
2. We can **explain why retries succeed** based on what changed
3. We can **predict failure** before sending the request
4. We have **actionable fix** or mitigation strategy

---

## Likely Outcomes

### Scenario A: JSON Serialization (60% probability)
- **Finding**: serde_json field order varies
- **Fix**: Use ordered maps or custom serializer
- **Impact**: Failures drop to ~0%

### Scenario B: Nonce Race Condition (25% probability)
- **Finding**: Parallel requests cause nonce conflicts
- **Fix**: Implement nonce locking or sequential queueing
- **Impact**: Failures drop to ~1-2%

### Scenario C: Server-Side Issue (10% probability)
- **Finding**: Server validation has bugs or state issues
- **Fix**: Report to Lighter Protocol, implement robust retries
- **Impact**: Mitigation only, not elimination

### Scenario D: Timing/Clock Skew (5% probability)
- **Finding**: Timestamp validation too strict
- **Fix**: Implement clock sync, adjust expiry windows
- **Impact**: Failures drop to ~2-3%

---

## Instrumentation Requirements

To execute this test plan, we need:

1. **Logging Enhancement**: Add detailed signature capture to api-client
2. **Test Framework**: Build test harness with controlled conditions
3. **Data Collection**: JSONL output for all attempts
4. **Analysis Tools**: Python scripts for pattern detection

---

## Next Steps

1. Implement Test 1 & 2 (determinism checks)
2. If determinism confirmed, run Test 3 (server behavior)
3. Based on Test 3 results, prioritize remaining tests
4. Build forensics capture for failing cases
5. Perform statistical analysis on collected data
6. Document findings and implement fix
