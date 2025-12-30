use api_client::LighterClient;
use dotenv::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    // Benchmark parameters (can override with env vars)
    let order_count: usize = env::var("STRESS_COUNT").unwrap_or_else(|_| "100".into()).parse()?;
    let delay_ms: u64 = env::var("STRESS_DELAY_MS").unwrap_or_else(|_| "100".into()).parse()?;
    let order_book_index: u8 = env::var("ORDER_BOOK_INDEX").unwrap_or_else(|_| "0".into()).parse()?;
    let base_amount: i64 = env::var("BASE_AMOUNT").unwrap_or_else(|_| "1000".into()).parse()?;
    let avg_execution_price: i64 = env::var("AVG_EXECUTION_PRICE").unwrap_or_else(|_| "350000".into()).parse()?;

    let base_client_order_index: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           Stress Test Benchmark - Invalid Signature Issue   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    println!("Configuration:");
    println!("  • Orders:        {}", order_count);
    println!("  • Delay:         {}ms between orders", delay_ms);
    println!("  • Market:        {}", order_book_index);
    println!("  • Amount:        {}", base_amount);
    println!("  • Price:         {}", avg_execution_price);
    println!("  • Base URL:      {}\n", base_url);

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    let mut stats = BenchmarkStats::new();
    let start_time = Instant::now();

    for i in 0..order_count {
        let client_order_index = base_client_order_index + i as u64;
        let order_start = Instant::now();

        let resp = client
            .create_market_order(
                order_book_index,
                client_order_index,
                base_amount,
                avg_execution_price,
                false, // BID
            )
            .await;

        let elapsed = order_start.elapsed();

        match resp {
            Ok(json) => {
                let code = json["code"].as_i64().unwrap_or_default();
                let msg = json["message"].as_str().unwrap_or("").to_string();

                if code == 200 {
                    stats.success += 1;
                } else if code == 21120 {
                    stats.sig_fail += 1;
                    if stats.sig_fail <= 3 {
                        stats.sample_sig_errors.push((i, code, msg.clone()));
                    }
                } else if msg.contains("nonce") {
                    stats.nonce_fail += 1;
                    if stats.nonce_fail <= 3 {
                        stats.sample_nonce_errors.push((i, code, msg.clone()));
                    }
                } else {
                    stats.other_fail += 1;
                    if stats.other_fail <= 3 {
                        stats.sample_other_errors.push((i, code, msg.clone()));
                    }
                }

                stats.record_latency(elapsed);
            }
            Err(e) => {
                stats.transport_fail += 1;
                stats.record_latency(elapsed);
                if stats.transport_fail <= 3 {
                    stats.sample_transport_errors.push((i, format!("{}", e)));
                }
            }
        }

        // Progress every 25 orders
        if (i + 1) % 25 == 0 {
            println!(
                "Progress: {:>3}/{:>3} | success={:<3} sig_fail={:<3} nonce_fail={:<3} other={:<3}",
                i + 1,
                order_count,
                stats.success,
                stats.sig_fail,
                stats.nonce_fail,
                stats.other_fail
            );
        }

        if i < order_count - 1 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    let total_elapsed = start_time.elapsed();
    stats.print_summary(order_count, total_elapsed);

    Ok(())
}

struct BenchmarkStats {
    success: usize,
    sig_fail: usize,
    nonce_fail: usize,
    other_fail: usize,
    transport_fail: usize,
    total_time: Duration,
    min_latency: Duration,
    max_latency: Duration,
    latencies: Vec<Duration>,
    sample_sig_errors: Vec<(usize, i64, String)>,
    sample_nonce_errors: Vec<(usize, i64, String)>,
    sample_other_errors: Vec<(usize, i64, String)>,
    sample_transport_errors: Vec<(usize, String)>,
}

impl BenchmarkStats {
    fn new() -> Self {
        Self {
            success: 0,
            sig_fail: 0,
            nonce_fail: 0,
            other_fail: 0,
            transport_fail: 0,
            total_time: Duration::ZERO,
            min_latency: Duration::MAX,
            max_latency: Duration::ZERO,
            latencies: Vec::new(),
            sample_sig_errors: Vec::new(),
            sample_nonce_errors: Vec::new(),
            sample_other_errors: Vec::new(),
            sample_transport_errors: Vec::new(),
        }
    }

    fn record_latency(&mut self, d: Duration) {
        self.total_time += d;
        self.latencies.push(d);
        if d < self.min_latency {
            self.min_latency = d;
        }
        if d > self.max_latency {
            self.max_latency = d;
        }
    }

    fn print_summary(&self, total_orders: usize, elapsed: Duration) {
        let total_fail = self.sig_fail + self.nonce_fail + self.other_fail + self.transport_fail;
        let fail_rate = if total_orders > 0 {
            (total_fail as f64 / total_orders as f64) * 100.0
        } else {
            0.0
        };
        let success_rate = if total_orders > 0 {
            (self.success as f64 / total_orders as f64) * 100.0
        } else {
            0.0
        };

        let avg_latency = if !self.latencies.is_empty() {
            self.total_time.as_millis() as f64 / self.latencies.len() as f64
        } else {
            0.0
        };

        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.cmp(b));
        let p95 = percentile(&sorted, 0.95);
        let p99 = percentile(&sorted, 0.99);
        let p100 = sorted.last().cloned().unwrap_or(Duration::ZERO);

        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║                      Benchmark Results                      ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        println!("Overall Results:");
        println!("  ├─ Total Orders:      {}", total_orders);
        println!("  ├─ Success:           {} ({:.2}%)", self.success, success_rate);
        println!("  ├─ Failed:            {} ({:.2}%)", total_fail, fail_rate);
        println!("  └─ Total Time:        {:.2}s\n", elapsed.as_secs_f64());

        println!("Failure Breakdown:");
        println!("  ├─ Invalid Signature: {} ({:.2}%)", 
            self.sig_fail, 
            (self.sig_fail as f64 / total_orders as f64) * 100.0
        );
        println!("  ├─ Invalid Nonce:     {} ({:.2}%)", 
            self.nonce_fail, 
            (self.nonce_fail as f64 / total_orders as f64) * 100.0
        );
        println!("  ├─ Other API Errors:  {} ({:.2}%)", 
            self.other_fail, 
            (self.other_fail as f64 / total_orders as f64) * 100.0
        );
        println!("  └─ Transport Errors:  {} ({:.2}%)\n", 
            self.transport_fail, 
            (self.transport_fail as f64 / total_orders as f64) * 100.0
        );

        println!("Latency Metrics:");
        println!("  ├─ Min:               {:.0}ms", if self.min_latency == Duration::MAX { 0 } else { self.min_latency.as_millis() });
        println!("  ├─ p95:               {:.0}ms", p95.as_millis());
        println!("  ├─ p99:               {:.0}ms", p99.as_millis());
        println!("  ├─ p100:              {:.0}ms", p100.as_millis());
        println!("  ├─ Max:               {:.0}ms", self.max_latency.as_millis());
        println!("  ├─ Avg:               {:.1}ms", avg_latency);
        println!("  └─ Throughput:        {:.2} ord/s\n", 
            self.success as f64 / elapsed.as_secs_f64()
        );

        if !self.sample_sig_errors.is_empty() {
            println!("Sample Invalid Signature Errors (code 21120):");
            for (order_idx, code, msg) in &self.sample_sig_errors {
                println!("  • Order {}: code={} msg='{}'", order_idx, code, msg);
            }
            println!();
        }

        if !self.sample_nonce_errors.is_empty() {
            println!("Sample Invalid Nonce Errors:");
            for (order_idx, code, msg) in &self.sample_nonce_errors {
                println!("  • Order {}: code={} msg='{}'", order_idx, code, msg);
            }
            println!();
        }

        if !self.sample_other_errors.is_empty() {
            println!("Sample Other Errors:");
            for (order_idx, code, msg) in &self.sample_other_errors {
                println!("  • Order {}: code={} msg='{}'", order_idx, code, msg);
            }
            println!();
        }

        if !self.sample_transport_errors.is_empty() {
            println!("Sample Transport Errors:");
            for (order_idx, err) in &self.sample_transport_errors {
                println!("  • Order {}: {}", order_idx, err);
            }
            println!();
        }

        // Summary diagnosis
        println!("╔════════════════════════════════════════════════════════════╗");
        if self.sig_fail > 0 {
            println!("║ ⚠️  {} invalid signature errors detected                   ║", self.sig_fail);
            println!("║    (Retry logic with sequential nonces in effect)         ║");
        } else {
            println!("║ ✅ No invalid signature errors!                            ║");
        }
        if self.nonce_fail > 0 {
            println!("║ ⚠️  {} invalid nonce errors detected                       ║", self.nonce_fail);
        } else {
            println!("║ ✅ No invalid nonce errors!                                ║");
        }
        println!("╚════════════════════════════════════════════════════════════╝");
    }
}

fn percentile(sorted: &[Duration], pct: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let n = sorted.len() as f64;
    let rank = (pct * (n - 1.0)).round() as usize;
    let idx = rank.min(sorted.len() - 1);
    sorted[idx]
}
