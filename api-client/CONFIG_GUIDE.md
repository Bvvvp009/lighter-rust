# API Client Configuration Guide

## Quick Start

### 1. Set up environment variables

Create a `.env` file in the `api-client` directory:

```bash
cd lighter-rust/api-client
cp .env.example .env
```

### 2. Edit your credentials

```env
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=361816
API_KEY_INDEX=6
API_PRIVATE_KEY=c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51
```

### 3. Run examples

```bash
# Test single order
cargo run --example test_single_order --release

# Stress test with 100 orders
STRESS_COUNT=100 cargo run --example stress_market_orders --release

# Create market order
cargo run --example create_market_order --release
```

## Configuration Details

### Required Variables

| Variable | Format | Example | Description |
|----------|--------|---------|-------------|
| `BASE_URL` | URL | `https://mainnet.zklighter.elliot.ai` | API endpoint |
| `ACCOUNT_INDEX` | integer | `361816` | Your account index on the exchange |
| `API_KEY_INDEX` | 0-255 | `6` | Which API key to use from your account |
| `API_PRIVATE_KEY` | 64-char hex | `c5230d52...f72f51` | Private key for signing (40 bytes in hex) |

### Optional Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `STRESS_COUNT` | integer | 1000 | How many orders to submit in stress test |
| `STRESS_DELAY_MS` | integer | 300 | Milliseconds to wait between orders |
| `ORDER_BOOK_INDEX` | 0-255 | 0 | Which market/pair to trade on |
| `BASE_AMOUNT` | integer | 1000 | Order size (in base asset units) |
| `AVG_EXECUTION_PRICE` | integer | 350000 | Price for limit orders |
| `IS_ASK` | 0 or 1 | 0 | Buy (0) or Sell (1) |
| `CLIENT_ORDER_INDEX_BASE` | integer | epoch seconds | Starting order index |

## Obtaining Your Credentials

### Account Index
Found on your account page in the web interface.

### API Key Index & Private Key
Create an API key through:
1. Web interface → Settings → API Keys
2. Generate new key
3. Copy the private key (keep it secret!)

Example private key formats:
- **Hex string**: `c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51`
- **Base64**: (if your provider uses this, convert to hex)

## Example Configurations

### Production (Mainnet)
```env
BASE_URL=https://mainnet.zklighter.elliot.ai
ACCOUNT_INDEX=123456
API_KEY_INDEX=0
API_PRIVATE_KEY=your_key_here_40_bytes_hex
```

### Testnet
```env
BASE_URL=https://testnet.zklighter.elliot.ai
ACCOUNT_INDEX=123456
API_KEY_INDEX=0
API_PRIVATE_KEY=your_key_here_40_bytes_hex
```

### Stress Testing
```env
STRESS_COUNT=1000
STRESS_DELAY_MS=500
```

## Cryptography & Signatures

### Key Information
- **Algorithm**: Schnorr signature over Goldilocks field
- **Key Size**: 40 bytes (5 limbs × 8 bytes)
- **Signature Size**: 80 bytes (40 bytes s + 40 bytes e)
- **Hash Function**: Poseidon2

### Signature Verification
All signatures are now validated correctly with the December 2025 fix:
- ✅ Point multiplication (G*private_key)
- ✅ Hash computation (Poseidon2)
- ✅ Signature response (s = nonce - e*privkey)

No `to_canonical()` transformations are applied to already-canonical scalars.

## Error Handling

### Common Errors

**Code 200 - Success**
```
✅ Order submitted successfully
```

**Code 21120 - Invalid Signature** (Now Fixed)
```
This error has been eliminated by the cryptographic fix.
If you see this, ensure you're using the latest code:
  git pull && cargo build --release
```

**Code 23000 - Rate Limit**
```
Too many requests. Solution:
  - Increase STRESS_DELAY_MS to 1000+ ms
  - Reduce STRESS_COUNT
  - Batch requests over time
```

**Code 40000-40999 - Bad Request**
```
Invalid order parameters. Check:
  - order_book_index is valid
  - base_amount > 0
  - price is reasonable
```

## Performance Tips

### Single Orders
- Typically: **100% success** with 200ms response time
- Use `test_single_order.rs` as a baseline

### Batch Orders
- Recommended: 500ms+ delay between orders
- Rate limit: Varies by account tier
- Typical limit: 100-1000 orders/minute

### Stress Testing
```bash
# Conservative (safe)
STRESS_COUNT=100 STRESS_DELAY_MS=1000 cargo run --example stress_market_orders --release

# Moderate (balanced)
STRESS_COUNT=500 STRESS_DELAY_MS=500 cargo run --example stress_market_orders --release

# Aggressive (fast, may hit rate limits)
STRESS_COUNT=1000 STRESS_DELAY_MS=300 cargo run --example stress_market_orders --release
```

## Troubleshooting

### Q: "Invalid signature" errors (code 21120)
**A**: This has been fixed. Update your code:
```bash
git pull
cargo clean
cargo build --release
```

### Q: "Too many requests" (code 23000)
**A**: Increase delay between orders:
```bash
STRESS_DELAY_MS=1000 cargo run --example stress_market_orders --release
```

### Q: Connection timeout
**A**: Check BASE_URL is correct and internet connectivity:
```bash
curl https://mainnet.zklighter.elliot.ai/status
```

### Q: Authentication failed
**A**: Verify API key:
```bash
cargo run --example check_api_key --release
```

### Q: Wrong order amounts
**A**: Check your BASE_AMOUNT and AVG_EXECUTION_PRICE environment variables match intended values.

## Development vs Production

### Development
```env
BASE_URL=https://testnet.zklighter.elliot.ai
STRESS_DELAY_MS=100
```

### Production  
```env
BASE_URL=https://mainnet.zklighter.elliot.ai
STRESS_DELAY_MS=500
```

## Security Notes

⚠️ **Keep your `API_PRIVATE_KEY` secret!**

1. Never commit `.env` to git
2. Use `.gitignore` entry:
   ```
   .env
   .env.local
   *.key
   ```
3. Consider using environment variables from your shell instead:
   ```bash
   export API_PRIVATE_KEY="your_key_here"
   ```

## Support

For issues or questions:
1. Check example output/logs
2. Verify environment variables are set
3. Try `test_single_order.rs` first
4. Check API documentation
