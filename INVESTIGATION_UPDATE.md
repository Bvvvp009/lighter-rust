# Investigation Summary: Signature Validation Issue

## Current Status

**Finding:** The Rust SDK implementation **field order is CORRECT** and matches the Go SDK exactly.

### Field Order Verification

Go SDK (`lighter-go/types/txtypes/create_order.go`):
```go
elems = append(elems, g.FromUint32(txInfo.Price))                  // Element 9
elems = append(elems, g.FromUint32(uint32(txInfo.IsAsk)))          // Element 10
elems = append(elems, g.FromUint32(uint32(txInfo.Type)))           // Element 11
elems = append(elems, g.FromUint32(uint32(txInfo.TimeInForce)))    // Element 12
elems = append(elems, g.FromUint32(uint32(txInfo.ReduceOnly)))     // Element 13
elems = append(elems, g.FromUint32(txInfo.TriggerPrice))           // Element 14
elems = append(elems, g.FromInt64(txInfo.OrderExpiry))             // Element 15
```

Rust SDK (`api-client/src/lib.rs`):
```rust
Goldilocks::from_canonical_u64(price as u64),          // Element 9
Goldilocks::from_canonical_u64(is_ask as u64),         // Element 10 ✅
Goldilocks::from_canonical_u64(order_type as u64),     // Element 11
Goldilocks::from_canonical_u64(time_in_force as u64),  // Element 12
Goldilocks::from_canonical_u64(reduce_only as u64),    // Element 13 ✅
Goldilocks::from_canonical_u64(trigger_price as u64),  // Element 14
to_goldi_i64(order_expiry),                            // Element 15
```

**Conclusion:** Field order is identical. This is **NOT** the bug.

## Test Results (After Investigation)

| Test | Signature Failures | Status |
|------|-------------------|--------|
| Before investigation | 12/200 (6.0%) | Baseline |
| After "fix" (swap fields) | 18/200 (9.0%) | ❌ WORSE |
| After revert | Not tested yet | - |

## What We Know For Sure

✅ **Cryptographic algorithms are correct** (verified locally)
✅ **Field order matches Go SDK** (verified by comparison)
✅ **Nonce management is perfect** (0% nonce errors in all tests)
✅ **Signatures verify locally** (prove client signing works)

❌ **~6-9% non-deterministic server rejections** (code 21120)
❌ **Different orders fail each test run** (not consistent)

## Possible Remaining Issues

### 1. Clock Skew / Timestamp Issues
The `expired_at` field is calculated as:
```rust
let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
let expired_at = now + 599_000; // 10 minutes - 1 second
```

**Hypothesis:** If client clock is ahead of server clock, orders might already be expired when server validates them.

**Test:** Add `EXPIRED_AT_SKEW_MS` environment variable to adjust timing.

### 2. Account/Key Credential Mismatch
**Hypothesis:** The public key derived from `API_PRIVATE_KEY` doesn't match what the server expects for account 361816, api_key_index 6.

**Test:** Request server to confirm the public key they have on file for this account/key combination.

### 3. Race Condition in Server Validation
**Hypothesis:** Server has internal race condition where nonce is marked as "used" before signature is fully validated, causing intermittent failures on retry.

**Evidence:** Failures are non-deterministic and affect different orders each run.

### 4. Network/Serialization Issues
**Hypothesis:** JSON serialization differs slightly between what we sign and what server receives (e.g., whitespace, field ordering in JSON).

**Test:** Capture actual HTTP request body and compare with what was signed.

### 5. Order Expiry Field Calculation
The order expiry logic:
```rust
let order_expiry = if order.time_in_force == 1 && order.order_type == 0 {
    now + (28 * 24 * 60 * 60 * 1000)  // 28 days for GoodTillTime limit orders
} else {
    0  // NilOrderExpiry
};
```

In our test, all orders have `time_in_force=0`, so `order_expiry=0` for all.

**Hypothesis:** Server might be validating order_expiry differently.

## Next Steps

1. **Revert to original code** (field order was correct)
2. **Test with clock skew adjustment** (`EXPIRED_AT_SKEW_MS=-5000` to make expiry 5 seconds earlier)
3. **Verify public key** with Lighter team
4. **Capture HTTP request body** to see if JSON differs from what we signed
5. **Test with different order parameters** (vary time_in_force, order_type, is_ask, reduce_only)

## Conclusion

The ~6-9% signature failure rate is **NOT** due to:
- ❌ Incorrect crypto algorithms
- ❌ Wrong field order in hash
- ❌ Nonce management issues

It **IS** likely due to:
- ✅ Server-side validation timing (clock skew, expiry)
- ✅ Account/key credential mismatch
- ✅ Race condition in server order processing
- ✅ Subtle serialization or validation difference

---

**Status:** Investigation continues. Field order verified correct. Focus shifted to timing/credential issues.
