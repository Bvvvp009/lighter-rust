#!/usr/bin/env python3
"""
Analyze retry telemetry from application logs to monitor signature failure patterns.

This script parses log files looking for [RETRY TELEMETRY] entries and generates
statistics about:
- Signature retry rate (% of orders requiring signature retry)
- Nonce retry rate (% of orders requiring nonce retry)
- Average retries per failed order
- Time distribution of retries
- Patterns and trends over time

Usage:
    python analyze_retry_telemetry.py <logfile>
    python analyze_retry_telemetry.py --stdin  # Read from stdin
"""

import re
import sys
from collections import defaultdict
from datetime import datetime
from typing import Dict, List, Tuple

class RetryStats:
    def __init__(self):
        self.total_orders = 0
        self.orders_with_sig_retry = 0
        self.orders_with_nonce_retry = 0
        self.total_sig_retries = 0
        self.total_nonce_retries = 0
        self.successful_after_retry = 0
        self.failed_after_retries = 0
        self.retry_times = []  # Time taken for orders that needed retries
        self.success_times = []  # Time taken for successful orders
        
        # Detailed tracking
        self.sig_retry_distribution = defaultdict(int)  # {retry_count: frequency}
        self.nonce_retry_distribution = defaultdict(int)
        self.error_codes = defaultdict(int)
        self.error_messages = []

    def add_retry(self, retry_type: str, code: int, msg: str, nonce: int):
        """Track a retry attempt"""
        if retry_type == 'sig':
            self.total_sig_retries += 1
        else:
            self.total_nonce_retries += 1
        
        self.error_codes[code] += 1
        self.error_messages.append((retry_type, code, msg, nonce))

    def add_success(self, sig_retries: int, nonce_retries: int, elapsed_ms: float, nonce: int):
        """Track a successful order"""
        self.total_orders += 1
        
        if sig_retries > 0:
            self.orders_with_sig_retry += 1
            self.sig_retry_distribution[sig_retries] += 1
            
        if nonce_retries > 0:
            self.orders_with_nonce_retry += 1
            self.nonce_retry_distribution[nonce_retries] += 1
        
        if sig_retries > 0 or nonce_retries > 0:
            self.successful_after_retry += 1
            self.retry_times.append(elapsed_ms)
        else:
            self.success_times.append(elapsed_ms)

    def add_failure(self, sig_retries: int, nonce_retries: int, elapsed_ms: float, nonce: int):
        """Track a failed order (exhausted retries)"""
        self.total_orders += 1
        self.failed_after_retries += 1
        self.retry_times.append(elapsed_ms)

    def print_report(self):
        """Print comprehensive statistics report"""
        print("\n" + "="*80)
        print("RETRY TELEMETRY ANALYSIS REPORT")
        print("="*80 + "\n")
        
        if self.total_orders == 0:
            print("⚠️  No order data found in logs\n")
            return
        
        # Overall statistics
        print("📊 OVERALL STATISTICS")
        print("-" * 80)
        print(f"Total orders processed: {self.total_orders}")
        print(f"Successful orders: {self.total_orders - self.failed_after_retries}")
        print(f"Failed orders (exhausted retries): {self.failed_after_retries}")
        print(f"Success rate: {((self.total_orders - self.failed_after_retries) / self.total_orders * 100):.2f}%")
        print()
        
        # Retry rates
        print("🔄 RETRY RATES")
        print("-" * 80)
        sig_rate = (self.orders_with_sig_retry / self.total_orders * 100) if self.total_orders > 0 else 0
        nonce_rate = (self.orders_with_nonce_retry / self.total_orders * 100) if self.total_orders > 0 else 0
        
        print(f"Orders requiring signature retry: {self.orders_with_sig_retry} ({sig_rate:.2f}%)")
        print(f"Orders requiring nonce retry: {self.orders_with_nonce_retry} ({nonce_rate:.2f}%)")
        print(f"Total signature retries: {self.total_sig_retries}")
        print(f"Total nonce retries: {self.total_nonce_retries}")
        
        if self.orders_with_sig_retry > 0:
            avg_sig = self.total_sig_retries / self.orders_with_sig_retry
            print(f"Average signature retries per failed order: {avg_sig:.2f}")
        
        if self.orders_with_nonce_retry > 0:
            avg_nonce = self.total_nonce_retries / self.orders_with_nonce_retry
            print(f"Average nonce retries per failed order: {avg_nonce:.2f}")
        print()
        
        # Retry distribution
        if self.sig_retry_distribution:
            print("📈 SIGNATURE RETRY DISTRIBUTION")
            print("-" * 80)
            for count in sorted(self.sig_retry_distribution.keys()):
                freq = self.sig_retry_distribution[count]
                pct = (freq / self.total_orders * 100)
                print(f"  {count} retry(ies): {freq} orders ({pct:.2f}%)")
            print()
        
        if self.nonce_retry_distribution:
            print("📈 NONCE RETRY DISTRIBUTION")
            print("-" * 80)
            for count in sorted(self.nonce_retry_distribution.keys()):
                freq = self.nonce_retry_distribution[count]
                pct = (freq / self.total_orders * 100)
                print(f"  {count} retry(ies): {freq} orders ({pct:.2f}%)")
            print()
        
        # Error code distribution
        if self.error_codes:
            print("❌ ERROR CODE DISTRIBUTION")
            print("-" * 80)
            for code in sorted(self.error_codes.keys(), key=lambda x: self.error_codes[x], reverse=True):
                freq = self.error_codes[code]
                print(f"  Code {code}: {freq} occurrences")
            print()
        
        # Timing statistics
        if self.retry_times:
            print("⏱️  TIMING STATISTICS")
            print("-" * 80)
            avg_retry = sum(self.retry_times) / len(self.retry_times)
            min_retry = min(self.retry_times)
            max_retry = max(self.retry_times)
            print(f"Orders with retries - Avg: {avg_retry:.0f}ms, Min: {min_retry:.0f}ms, Max: {max_retry:.0f}ms")
            
            if self.success_times:
                avg_success = sum(self.success_times) / len(self.success_times)
                print(f"Orders without retries - Avg: {avg_success:.0f}ms")
            print()
        
        # Health assessment
        print("🏥 HEALTH ASSESSMENT")
        print("-" * 80)
        if sig_rate > 10:
            print("⚠️  WARNING: Signature retry rate > 10% - Server validation issues likely")
        elif sig_rate > 5:
            print("⚠️  CAUTION: Signature retry rate 5-10% - Monitor closely")
        elif sig_rate > 0:
            print("✅ OK: Signature retry rate < 5% - Within acceptable range")
        else:
            print("✅ EXCELLENT: No signature retries detected")
        
        if self.failed_after_retries > 0:
            fail_rate = (self.failed_after_retries / self.total_orders * 100)
            print(f"⚠️  {fail_rate:.2f}% of orders failed after exhausting all retries")
        
        print()


def parse_log_line(line: str) -> Tuple[str, Dict]:
    """Parse a telemetry log line and extract structured data"""
    
    # Match retry warning: [RETRY TELEMETRY] Signature validation failed - Attempt 1/3 | Nonce: 1250 | Code: 21120 | Msg: ...
    retry_pattern = r'\[RETRY TELEMETRY\] (Signature validation failed|Nonce mismatch) - Attempt (\d+)/(\d+) \| (?:Nonce|Used): (\d+) \| Code: (\d+) \| Msg: (.+)'
    match = re.search(retry_pattern, line)
    if match:
        retry_type = 'sig' if 'Signature' in match.group(1) else 'nonce'
        return 'retry', {
            'type': retry_type,
            'attempt': int(match.group(2)),
            'max_attempts': int(match.group(3)),
            'nonce': int(match.group(4)),
            'code': int(match.group(5)),
            'msg': match.group(6)
        }
    
    # Match success: [RETRY TELEMETRY] Order successful after retries | Sig retries: 1 | Nonce retries: 0 | Total time: 234ms | Final nonce: 1251
    success_pattern = r'\[RETRY TELEMETRY\] Order successful after retries \| Sig retries: (\d+) \| Nonce retries: (\d+) \| Total time: ([0-9.]+)(?:ms|s) \| Final nonce: (\d+)'
    match = re.search(success_pattern, line)
    if match:
        time_val = float(match.group(3))
        # Convert seconds to ms if needed
        if 's' in line and 'ms' not in line:
            time_val *= 1000
        return 'success', {
            'sig_retries': int(match.group(1)),
            'nonce_retries': int(match.group(2)),
            'elapsed_ms': time_val,
            'nonce': int(match.group(4))
        }
    
    # Match failure: [RETRY TELEMETRY] All retries exhausted | Sig retries: 2 | Nonce retries: 1 | Total time: 567ms | Last nonce: 1252
    failure_pattern = r'\[RETRY TELEMETRY\] All retries exhausted \| Sig retries: (\d+) \| Nonce retries: (\d+) \| Total time: ([0-9.]+)(?:ms|s) \| Last nonce: (\d+)'
    match = re.search(failure_pattern, line)
    if match:
        time_val = float(match.group(3))
        if 's' in line and 'ms' not in line:
            time_val *= 1000
        return 'failure', {
            'sig_retries': int(match.group(1)),
            'nonce_retries': int(match.group(2)),
            'elapsed_ms': time_val,
            'nonce': int(match.group(4))
        }
    
    return None, {}


def analyze_logs(logfile):
    """Analyze log file for retry telemetry"""
    stats = RetryStats()
    
    for line in logfile:
        event_type, data = parse_log_line(line)
        
        if event_type == 'retry':
            stats.add_retry(data['type'], data['code'], data['msg'], data['nonce'])
        elif event_type == 'success':
            stats.add_success(data['sig_retries'], data['nonce_retries'], 
                            data['elapsed_ms'], data['nonce'])
        elif event_type == 'failure':
            stats.add_failure(data['sig_retries'], data['nonce_retries'],
                            data['elapsed_ms'], data['nonce'])
    
    stats.print_report()


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_retry_telemetry.py <logfile>")
        print("       python analyze_retry_telemetry.py --stdin")
        sys.exit(1)
    
    if sys.argv[1] == '--stdin':
        analyze_logs(sys.stdin)
    else:
        try:
            with open(sys.argv[1], 'r', encoding='utf-8') as f:
                analyze_logs(f)
        except FileNotFoundError:
            print(f"Error: File '{sys.argv[1]}' not found")
            sys.exit(1)
        except Exception as e:
            print(f"Error reading file: {e}")
            sys.exit(1)


if __name__ == '__main__':
    main()
