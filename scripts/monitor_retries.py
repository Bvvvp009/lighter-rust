#!/usr/bin/env python3
"""
Real-time retry telemetry monitor.

Watches application logs in real-time and displays retry statistics as they occur.
Useful for live monitoring of signature failure rates during stress testing.

Usage:
    python monitor_retries.py <logfile>    # Monitor existing log file
    <app> | python monitor_retries.py      # Monitor from stdin
"""

import sys
import re
import time
from collections import deque
from datetime import datetime

class LiveMonitor:
    def __init__(self, window_size=100):
        self.window_size = window_size
        self.recent_orders = deque(maxlen=window_size)
        
        self.total_orders = 0
        self.sig_retries = 0
        self.nonce_retries = 0
        self.failures = 0
        
        self.last_update = time.time()
        self.update_interval = 2.0  # Update display every 2 seconds

    def process_line(self, line):
        """Process a single log line"""
        
        # Success after retries
        if 'Order successful after retries' in line:
            match = re.search(r'Sig retries: (\d+) \| Nonce retries: (\d+)', line)
            if match:
                sig_r = int(match.group(1))
                nonce_r = int(match.group(2))
                self.total_orders += 1
                if sig_r > 0:
                    self.sig_retries += 1
                if nonce_r > 0:
                    self.nonce_retries += 1
                
                self.recent_orders.append({
                    'type': 'success',
                    'sig_retries': sig_r,
                    'nonce_retries': nonce_r,
                    'timestamp': datetime.now()
                })
                self.maybe_update_display()
        
        # Failure after exhausting retries
        elif 'All retries exhausted' in line:
            match = re.search(r'Sig retries: (\d+) \| Nonce retries: (\d+)', line)
            if match:
                sig_r = int(match.group(1))
                nonce_r = int(match.group(2))
                self.total_orders += 1
                self.failures += 1
                if sig_r > 0:
                    self.sig_retries += 1
                if nonce_r > 0:
                    self.nonce_retries += 1
                
                self.recent_orders.append({
                    'type': 'failure',
                    'sig_retries': sig_r,
                    'nonce_retries': nonce_r,
                    'timestamp': datetime.now()
                })
                self.maybe_update_display()
                
        # Individual retry attempts
        elif 'Signature validation failed' in line or 'Nonce mismatch' in line:
            retry_type = 'sig' if 'Signature' in line else 'nonce'
            match = re.search(r'Code: (\d+)', line)
            code = int(match.group(1)) if match else 0
            
            # Real-time alert for signature failures
            if retry_type == 'sig':
                print(f"\n⚠️  SIGNATURE RETRY at {datetime.now().strftime('%H:%M:%S')} - Code: {code}")

    def maybe_update_display(self):
        """Update display if enough time has passed"""
        now = time.time()
        if now - self.last_update >= self.update_interval:
            self.update_display()
            self.last_update = now

    def update_display(self):
        """Update the live statistics display"""
        # Clear screen (works on Unix and Windows)
        print("\033[2J\033[H", end='')
        
        print("=" * 80)
        print(f"LIVE RETRY MONITOR - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("=" * 80)
        print()
        
        if self.total_orders == 0:
            print("Waiting for order data...")
            return
        
        # Overall stats
        sig_rate = (self.sig_retries / self.total_orders * 100) if self.total_orders > 0 else 0
        nonce_rate = (self.nonce_retries / self.total_orders * 100) if self.total_orders > 0 else 0
        fail_rate = (self.failures / self.total_orders * 100) if self.total_orders > 0 else 0
        
        print(f"📊 Total Orders: {self.total_orders}")
        print(f"✅ Success Rate: {100 - fail_rate:.1f}%")
        print(f"❌ Failure Rate: {fail_rate:.1f}%")
        print()
        
        # Retry rates with visual indicators
        print("🔄 RETRY RATES:")
        print(f"   Signature: {self.sig_retries} orders ({sig_rate:.1f}%) {self._get_health_indicator(sig_rate)}")
        print(f"   Nonce:     {self.nonce_retries} orders ({nonce_rate:.1f}%)")
        print()
        
        # Recent activity (last 10 orders)
        print(f"📝 RECENT ACTIVITY (last {min(10, len(self.recent_orders))} orders):")
        for order in list(self.recent_orders)[-10:]:
            timestamp = order['timestamp'].strftime('%H:%M:%S')
            status = '✅' if order['type'] == 'success' else '❌'
            sig = f"sig:{order['sig_retries']}" if order['sig_retries'] > 0 else ""
            nonce = f"nonce:{order['nonce_retries']}" if order['nonce_retries'] > 0 else ""
            retries = f"[{sig} {nonce}]".strip() if sig or nonce else "[no retries]"
            print(f"   {status} {timestamp} {retries}")
        print()
        
        # Health assessment
        print("🏥 HEALTH:")
        if sig_rate > 10:
            print("   ⚠️  CRITICAL: Signature retry rate > 10%")
        elif sig_rate > 5:
            print("   ⚠️  WARNING: Signature retry rate > 5%")
        elif sig_rate > 0:
            print("   ✅ OK: Signature retry rate within normal range")
        else:
            print("   ✅ EXCELLENT: No signature retries")
        
        if fail_rate > 5:
            print(f"   ⚠️  WARNING: {fail_rate:.1f}% of orders exhausting retries")
        
        print()
        print("Press Ctrl+C to exit...")

    def _get_health_indicator(self, rate):
        """Get visual health indicator based on retry rate"""
        if rate == 0:
            return "🟢"
        elif rate < 5:
            return "🟡"
        elif rate < 10:
            return "🟠"
        else:
            return "🔴"

    def run(self, input_stream):
        """Main monitoring loop"""
        try:
            print("Starting live monitor... Waiting for log data...")
            for line in input_stream:
                line = line.strip()
                if '[RETRY TELEMETRY]' in line:
                    self.process_line(line)
            
            # Final update when stream ends
            self.update_display()
            
        except KeyboardInterrupt:
            print("\n\nMonitoring stopped by user")
            self.update_display()


def follow_file(filename):
    """Generator that yields new lines from a file as they're written"""
    try:
        with open(filename, 'r') as f:
            # Start at end of file
            f.seek(0, 2)
            while True:
                line = f.readline()
                if line:
                    yield line
                else:
                    time.sleep(0.1)  # Wait for new data
    except FileNotFoundError:
        print(f"Error: File '{filename}' not found")
        sys.exit(1)


def main():
    monitor = LiveMonitor()
    
    if len(sys.argv) > 1:
        # Follow a log file
        filename = sys.argv[1]
        print(f"Monitoring log file: {filename}")
        monitor.run(follow_file(filename))
    else:
        # Read from stdin (piped input)
        print("Reading from stdin (pipe application output here)...")
        monitor.run(sys.stdin)


if __name__ == '__main__':
    main()
