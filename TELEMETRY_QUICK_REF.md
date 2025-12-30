# Retry Telemetry Quick Reference

## Quick Start

### Run with Live Monitoring
```bash
cd api-client
RUST_LOG=warn cargo run --example stress_market_orders 2>&1 | python ../scripts/monitor_retries.py
```

### Run and Save for Analysis
```bash
RUST_LOG=warn cargo run > test.log 2>&1
python scripts/analyze_retry_telemetry.py test.log
```

### Analyze Existing Logs
```bash
python scripts/analyze_retry_telemetry.py /path/to/app.log
```

## Log Patterns to Look For

### Signature Retry (Expected occasionally)
```
[RETRY TELEMETRY] Signature validation failed - Attempt 1/3 | Nonce: 1250 | Code: 21120
```

### Success After Retry (Good - system working)
```
[RETRY TELEMETRY] Order successful after retries | Sig retries: 1 | Nonce retries: 0 | Total time: 234ms
```

### All Retries Exhausted (Bad - needs investigation)
```
[RETRY TELEMETRY] All retries exhausted | Sig retries: 2 | Nonce retries: 1 | Total time: 567ms
```

## Health Thresholds

| Retry Rate | Status | Action |
|-----------|--------|---------|
| 0% | 🟢 Excellent | None |
| 0-5% | 🟡 OK | Monitor |
| 5-10% | 🟠 Warning | Investigate |
| >10% | 🔴 Critical | Contact server team |

## Common Commands

### Enable Logging
```bash
export RUST_LOG=warn      # Show retries only
export RUST_LOG=info      # Show successes too
export RUST_LOG=debug     # Everything
```

### Stress Test with Monitoring
```bash
# 50 orders with live monitoring
RUST_LOG=warn STRESS_COUNT=50 cargo run --example stress_market_orders 2>&1 | \
  python ../scripts/monitor_retries.py
```

### Analyze Last N Lines of Log
```bash
tail -1000 app.log | python scripts/analyze_retry_telemetry.py --stdin
```

### Watch Log File in Real-Time
```bash
tail -f /var/log/app.log | grep "RETRY TELEMETRY"
```

## Troubleshooting

### No telemetry logs appearing
- Check `RUST_LOG` is set to at least `warn`
- Verify stderr is being captured (use `2>&1`)
- Ensure code is actually retrying (not all failures trigger retries)

### Signature retry rate > 10%
1. Run full analysis: `python scripts/analyze_retry_telemetry.py app.log`
2. Check error code distribution
3. Verify timing patterns
4. Contact server team with statistics

### Python script errors
```bash
# Install required packages (none needed - uses stdlib only)
python3 scripts/analyze_retry_telemetry.py app.log
```

## Integration Examples

### Prometheus/Grafana
Parse logs and expose metrics:
```
signature_retry_rate{threshold="5%"} 
order_success_rate
retry_latency_ms{quantile="0.95"}
```

### CI/CD Pipeline
```bash
# Fail if retry rate > 10%
RUST_LOG=warn cargo test > test.log 2>&1
python scripts/analyze_retry_telemetry.py test.log | grep "CRITICAL" && exit 1
```

### Automated Reporting
```bash
# Daily report
RUST_LOG=warn cargo run > daily_$(date +%Y%m%d).log 2>&1
python scripts/analyze_retry_telemetry.py daily_*.log > report.txt
```

## Files

- `api-client/src/lib.rs` - Telemetry implementation
- `scripts/analyze_retry_telemetry.py` - Analysis tool
- `scripts/monitor_retries.py` - Live monitoring
- `RETRY_TELEMETRY_GUIDE.md` - Full documentation
- `TELEMETRY_IMPLEMENTATION.md` - Implementation details

## Support

For issues or questions:
1. Check `RETRY_TELEMETRY_GUIDE.md` for detailed documentation
2. Review `TELEMETRY_IMPLEMENTATION.md` for implementation details
3. Examine `SIGNATURE_INVESTIGATION_CONCLUSION.md` for background

---

**Remember**: 0-5% retry rate is normal and expected. The system is working correctly!
