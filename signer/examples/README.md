# Signer Examples

Simple examples demonstrating how to use the signer library.

## Examples

### 1. Generate API Key Pair

Generate a new random API key pair (private/public key).

```bash
cargo run --example generate_api_key
```

**Output:**
- Private key (hex): 40-byte private key
- Public key (hex): 40-byte public key

---

### 2. Create Auth Token

Create an authentication token for API access.

```bash
# Set environment variables
export API_PRIVATE_KEY="your_private_key_hex"
export API_KEY_INDEX=5
export ACCOUNT_INDEX=361816

# Or use .env file
cargo run --example create_auth_token
```

**Required:**
- `API_PRIVATE_KEY` - Your API private key (hex, with or without 0x prefix)

**Optional:**
- `API_KEY_INDEX` - API key index (default: 5)
- `ACCOUNT_INDEX` - Account index (default: 361816)

**Output:**
- Auth token: `deadline:account_index:api_key_index:signature_hex`
- Token expires 7 hours from generation time

---

### 3. Sign Message

Sign a 40-byte message hash using Schnorr signatures.

```bash
# Set environment variable (optional, uses example key if not set)
export API_PRIVATE_KEY="your_private_key_hex"

cargo run --example sign_message
```

**Output:**
- Signature (80 bytes): 40 bytes `s` + 40 bytes `e`
- Public key (for verification)

---

### 4. Test Auth Requests (Detailed Logging)

Test auth token generation by creating a NEW token for each request and logging full request/response details.

```bash
# Set environment variables
export API_PRIVATE_KEY="your_private_key_hex"
export API_KEY_INDEX=5
export ACCOUNT_INDEX=361816
export BASE_URL="https://mainnet.zklighter.elliot.ai"

cargo run --example test_auth_requests --release
```

**Features:**
- Creates a NEW auth token for each request (5 requests per endpoint)
- Logs full request/response details
- Verifies responses contain actual data
- 200ms delay between requests
- Comprehensive analysis of results

---

### 5. Stress Test Auth

Send 100s of requests using auth tokens to test authentication.

```bash
# Set environment variables
export API_PRIVATE_KEY="your_private_key_hex"
export API_KEY_INDEX=5
export ACCOUNT_INDEX=361816
export BASE_URL="https://mainnet.zklighter.elliot.ai"

cargo run --example stress_test_auth --release
```

**Tests:**
- Generates auth token
- Sends 100 requests to each endpoint:
  - `/api/v1/accountActiveOrders`
  - `/api/v1/accountLimits`
  - `/api/v1/accountMetadata`
- Tracks success/failure rates
- Reports statistics

---

## Environment Variables

All examples support loading from `.env` file. Place `.env` in:
- Current directory
- Parent directory (`../.env`)
- Grandparent directory (`../../.env`)

Example `.env` file:
```
API_PRIVATE_KEY=a5cec6f7709b5661...
API_KEY_INDEX=5
ACCOUNT_INDEX=361816
BASE_URL=https://mainnet.zklighter.elliot.ai
```

## Quick Reference

| Example | Command | Purpose |
|---------|---------|---------|
| Generate Key | `cargo run --example generate_api_key` | Create new key pair |
| Create Token | `cargo run --example create_auth_token` | Generate auth token |
| Sign Message | `cargo run --example sign_message` | Sign 40-byte message |
| Test Auth Requests | `cargo run --example test_auth_requests --release` | Test with detailed logging |
| Stress Test | `cargo run --example stress_test_auth --release` | Test auth with API |

