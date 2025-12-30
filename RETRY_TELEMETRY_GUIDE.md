# Retry Telemetry System

## Overview

The retry telemetry system monitors signature and nonce failures in real-time, providing detailed statistics to track the health of order submission and identify patterns in signature validation issues.

## What It Tracks

The telemetry system monitors:

1. **Signature Retry Rate**: Percentage of orders requiring signature retry due to server validation failures
2. **Nonce Retry Rate**: Percentage of orders requiring nonce retry due to synchronization issues  
3. **Retry Distribution**: How many retries each failed order requires
4. **Error Codes**: Distribution of specific error codes (21120 for signature, 21104 for nonce)
5. **Timing**: How long orders take with and without retries
6. **Final Outcomes**: Success rate after retries vs. exhausted retries

## Log Format

The system adds structured log entries prefixed with `[RETRY TELEMETRY]`:

### Retry Attempt
```
[RETRY TELEMETRY] Signature validation failed - Attempt 1/3 | Nonce: 1250 | Code: 21120 | Msg: invalid signature
[RETRY TELEMETRY] Nonce mismatch - Attempt 2/3 | Used: 1251 | Code: 21104 | Msg: nonce mismatch
```

### Successful Order (After Retries)
```
[RETRY TELEMETRY] Order successful after retries | Sig retries: 1 | Nonce retries: 0 | Total time: 234ms | Final nonce: 1251
```

### Failed Order (Exhausted Retries)
```
[RETRY TELEMETRY] All retries exhausted | Sig retries: 2 | Nonce retries: 1 | Total time: 567ms | Last nonce: 1252
```

## Analysis Tools

### 1. Post-Processing Analysis (`analyze_retry_telemetry.py`)

Analyzes log files to generate comprehensive statistics.

**Usage:**
```bash
# Analyze a log file
python scripts/analyze_retry_telemetry.py app.log

# Analyze from piped input
cargo run | python scripts/analyze_retry_telemetry.py --stdin
```

**Output Example:**
```
📊 OVERALL STATISTICS
Total orders processed: 50
Successful orders: 47
Failed orders (exhausted retries): 3
Success rate: 94.00%

🔄 RETRY RATES
Orders requiring signature retry: 3 (6.00%)
Orders requiring nonce retry: 1 (2.00%)
Total signature retries: 4
Total nonce retries: 1

📈 SIGNATURE RETRY DISTRIBUTION
  1 retry(ies): 2 orders (4.00%)
  2 retry(ies): 1 orders (2.00%)

🏥 HEALTH ASSESSMENT
✅ OK: Signature retry rate < 5% - Within acceptable range
```

### 2. Real-Time Monitoring (`monitor_retries.py`)

Watches logs in real-time and displays live statistics.

**Usage:**
```bash
# Monitor application output
cargo run 2>&1 | python scripts/monitor_retries.py

# Monitor existing log file
python scripts/monitor_retries.py app.log
```

**Live Display:**
```
===============================================================================
LIVE RETRY MONITOR - 2025-12-30 15:30:45
===============================================================================

📊 Total Orders: 47
✅ Success Rate: 94.0%
❌ Failure Rate: 6.0%

🔄 RETRY RATES:
   Signature: 3 orders (6.4%) 🟡
   Nonce:     1 orders (2.1%)

📝 RECENT ACTIVITY (last 10 orders):
   ✅ 15:30:44 [no retries]
   ✅ 15:30:44 [sig:1]
   ✅ 15:30:45 [no retries]
   ...

🏥 HEALTH:
   ✅ OK: Signature retry rate within normal range
```

## Health Thresholds

The system uses these thresholds to assess health:

| Signature Retry Rate | Assessment | Action |
|---------------------|-----------|---------|
| 0% | 🟢 Excellent | None needed |
| 0-5% | 🟡 OK | Monitor trends |
| 5-10% | 🟠 Caution | Investigate patterns |
| >10% | 🔴 Critical | Server-side investigation required |

## Interpreting Results

### Normal Behavior (0-5% retry rate)

- A small percentage of signature retries is expected
- Retry mechanism successfully handles intermittent server issues
- No action required

### Elevated Rate (5-10%)

**Possible causes:**
- Increased server load causing intermittent validation delays
- Network latency affecting signature timing
- Recent server deployment with validation changes

**Actions:**
- Monitor trends over time
- Check server logs for validation errors
- Verify retry delay settings are appropriate

### Critical Rate (>10%)

**Possible causes:**
- Server-side signature validation bugs
- Cryptographic implementation mismatch
- Clock synchronization issues affecting timestamps
- Server overload

**Actions:**
1. Collect detailed logs with `analyze_retry_telemetry.py`
2. Review error code distribution
3. Contact server team with telemetry data
4. Verify cryptographic implementation matches spec
5. Check for recent server changes

## Configuration

### Environment Variables

- `RETRY_DELAY_MS`: Milliseconds to wait between retry attempts (default: 100ms)
- `RUST_LOG`: Set to `warn` or `info` to see telemetry logs

### Enabling Telemetry

Telemetry is always enabled. To see logs:

```bash
# Set log level to see warnings (retries)
export RUST_LOG=warn

# Set log level to see info (successes after retry)
export RUST_LOG=info

# Run your application
cargo run
```

## Integration with Monitoring Systems

The structured log format makes it easy to integrate with monitoring systems:

### Prometheus/Grafana

Use a log aggregator to parse telemetry and expose metrics:

```
retry_signature_total{type="signature"} 
retry_signature_total{type="nonce"}
retry_success_after_retry_total
retry_exhausted_total
```

### Datadog/New Relic

Configure log parsing rules:
```
[RETRY TELEMETRY] Signature validation failed
→ increment counter: signature_retry_count
→ tag: {code: 21120, type: signature}
```

## Best Practices

1. **Always monitor signature retry rate** during stress tests
2. **Run analysis after significant changes** to cryptographic or API code  
3. **Set up alerts** if retry rate exceeds 10%
4. **Keep historical data** to identify trends
5. **Include telemetry output** in bug reports related to signature failures

## Example Workflow

### During Development
```bash
# Run with real-time monitoring
RUST_LOG=warn cargo run 2>&1 | python scripts/monitor_retries.py
```

### Stress Testing
```bash
# Run stress test and save logs
RUST_LOG=warn cargo run > stress_test.log 2>&1

# Analyze results
python scripts/analyze_retry_telemetry.py stress_test.log
```

### Production Monitoring
```bash
# Set up log rotation and continuous analysis
RUST_LOG=warn cargo run >> /var/log/lighter-rust/app.log 2>&1 &

# Periodically analyze
python scripts/analyze_retry_telemetry.py /var/log/lighter-rust/app.log
```

## Troubleshooting

### No telemetry appearing in logs

**Check:**
- `RUST_LOG` environment variable is set to at least `warn`
- Application is actually hitting retry logic (some failures don't trigger retries)
- Logs are being captured (check stderr, not just stdout)

### High retry rate (>10%)

**Investigation steps:**
1. Run `analyze_retry_telemetry.py` to get detailed statistics
2. Check error code distribution - all same code or varied?
3. Look at timing - do retries correlate with time of day?
4. Compare with server logs - are validation errors logged?
5. Test with a single order repeatedly - does it always fail or intermittent?

### Retries not helping

If retry rate is high AND many orders still fail after retries:
- This suggests a deeper issue than intermittent failures
- Review Schnorr implementation for edge cases
- Verify field order matches between client and server
- Check for data serialization issues

## Related Documentation

- [SIGNATURE_INVESTIGATION_CONCLUSION.md](../SIGNATURE_INVESTIGATION_CONCLUSION.md) - Initial investigation findings
- [api-client/src/lib.rs](../api-client/src/lib.rs) - Retry implementation
- [crypto/src/schnorr.rs](../crypto/src/schnorr.rs) - Schnorr signature implementation
