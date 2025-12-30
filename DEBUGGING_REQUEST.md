# Debugging Request for Lighter Team - Signature Validation Issue

## Problem Statement

The Rust SDK successfully generates valid Schnorr signatures (verified locally), but the Lighter Exchange server rejects approximately 6% of orders with **code 21120 - "invalid signature"** error.

- **Test Size:** 200 orders
- **Failure Rate:** 6% (12 failures)
- **Pattern:** Non-deterministic (different orders fail each run)
- **Nonce Errors:** 0 (perfect nonce management)
- **Local Verification:** ✅ Passed (signatures ARE cryptographically valid)

## What We Know

### Client-Side (✅ Verified Correct)
- ✅ Schnorr signature generation: Working correctly
- ✅ Poseidon2 hashing: Working correctly
- ✅ Elliptic curve arithmetic: Working correctly
- ✅ Nonce management: Perfect (0 duplicates, proper incrementing)
- ✅ Public key derivation: Consistent across all orders

### Server-Side (❓ Unknown)
- ❓ Which transaction fields are included in hash?
- ❓ What is the expected field order for Poseidon input?
- ❓ Are there any timestamp/expiration validations?
- ❓ Is there a race condition in order validation?

## Data We Can Provide

### From Each Failed Order (Captured in Debug Logs)

```
[SIG_DEBUG] tx_type=14 nonce=1237 expired_at=1767031271542 
            account_index=361816 api_key_index=6

[SIG_DEBUG] elements=[304, 14, 1237, 1767031271542, 361816, 6, 0, 
            1767030652, 1000, 294000, 0, 1, 0, 0, 0, 0]

[SIG_DEBUG] hash_bytes=8951f74437ca0d85ea9a32081708c589d4a860772ef55e427afa96f2a84fdc8f
            2a0cc5d75e63022e

[SIG_DEBUG] pubkey=99f3473027655c41eebb21afd06b516b438b42ad70c27ac8208cdb56b60be7d5c
            9ddfb05e3cf9518

[SIG_DEBUG] sig_hex=e6c9ba635986170ff6720d6f410706cd20846be438da26a2b96fecdd685bde4e
            67e243d38b3e713cad6f22603d5f3b56f8d5ee576617c99de68859381bee64a2cc3adb5714f
            791103eb0364609fa424a

[SIG_DEBUG] sig_b64=5sm6Y1mGFw/2cg1vQQcGzSCEa+Q42iaiuW/s3Whb3k5n4kPTiz5xPK1vImA9XztW+NXuV2YXyZ3miFk4G+5kosw621cU95EQPrA2Rgn6Qko=
```

### Transaction Elements (16 Goldilocks Fields)

| Index | Value | Field |
|-------|-------|-------|
| 0 | 304 | Chain ID |
| 1 | 14 | Tx Type (CREATE_ORDER) |
| 2 | 1237 | Nonce |
| 3 | 1767031271542 | Expired At (timestamp) |
| 4 | 361816 | Account Index |
| 5 | 6 | API Key Index |
| 6 | 0 | ? |
| 7 | 1767030652 | ? (possibly ClientOrderIndex) |
| 8 | 1000 | Base Amount |
| 9 | 294000 | Price |
| 10 | 0 | Reduce Only |
| 11 | 1 | Is Ask |
| 12-15 | 0 | Reserved/unused |

**Request to Lighter Team:**
1. Confirm field order and names
2. Are there additional fields in server-side hash input?
3. Are fields 6, 7 named correctly?

## Specific Questions for Server Validation

1. **Which fields are included in transaction hash?**
   - Our implementation includes 16 Goldilocks fields (see above)
   - Are there additional fields?
   - Are the fields in different order?

2. **What is the expected public key?**
   - We derive from: `sha256(private_key)` as scalar
   - Is this the same derivation you use?
   - Can you confirm public key format (40 bytes, little-endian)?

3. **How is the expired_at field used?**
   - We pass current timestamp as-is
   - Do you validate it's not too far in past/future?
   - Could clock skew cause intermittent failures?

4. **Is account 361816 with api_key_index 6 valid?**
   - Can you confirm this account exists in your DB?
   - Is the public key correctly mapped?
   - Are there any account-level restrictions?

5. **Have you experienced similar issues with other SDKs?**
   - Does Go SDK work flawlessly?
   - Do you have test vectors we can use?

## How to Reproduce Issues

### Prerequisites
```bash
export LIGHTER_CHAIN_ID=304  # Force mainnet
export LIGHTER_BASE_URL=https://mainnet.zklighter.elliot.ai
export API_PRIVATE_KEY=<your_key>
export LIGHTER_ACCOUNT_INDEX=361816
export LIGHTER_API_KEY_INDEX=6
```

### Run Tests
```bash
# Test 1: Verify signatures are valid locally
cargo run --release --package goldilocks-crypto --example verify_captured_sig

# Test 2: Stress test with full debug output
export SIG_DEBUG_DUMP=1
cargo run --release --package api-client --example benchmark_stress > stress_test.log 2>&1

# Inspect failed orders
grep "Order.*code=21120" stress_test.log
```

## Files with Complete Data

- `stress_test_output.txt` - Full test output with [SIG_DEBUG] logs for all 200 orders
  - ~5500 lines
  - Every signature attempt logged
  - Hash bytes, pubkey, signature hex and base64

## Expected Resolution

Once we understand:
1. Server-side field validation logic
2. Public key derivation method
3. Any timestamp/expiration checks

We can:
1. Adjust transaction field order if needed
2. Update public key derivation if required
3. Add timestamp validation if needed
4. Fix any other mismatches

## Technical Contact Points

- Rust SDK: `api-client/src/lib.rs` - `sign_transaction_internal()`
- Signature generation: `crypto/src/schnorr.rs`
- Hashing: Uses Poseidon2 from `poseidon-hash` crate
- Public key: Derived from ED25519-style private key in `signer/src/lib.rs`

---

**Status:** Ready to provide additional data or make code changes once server expectations are clarified.
