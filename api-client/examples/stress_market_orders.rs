use api_client::LighterClient;
use dotenv::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use signer::KeyManager;

#[derive(Debug, Clone)]
struct OrderTiming {
    signing_micros: u128,
    network_micros: u128,
    total_micros: u128,
}

fn calculate_percentiles(mut timings: Vec<u128>) -> (u128, u128, u128, u128, f64) {
    if timings.is_empty() {
        return (0, 0, 0, 0, 0.0);
    }
    timings.sort_unstable();
    let len = timings.len();
    let p50_idx = ((len * 50) / 100).min(len - 1);
    let p95_idx = ((len * 95) / 100).min(len - 1);
    let p99_idx = ((len * 99) / 100).min(len - 1);
    let p50 = timings[p50_idx];
    let p95 = timings[p95_idx];
    let p99 = timings[p99_idx];
    let p100 = timings[len - 1];
    let avg = timings.iter().sum::<u128>() as f64 / len as f64;
    (p50, p95, p99, p100, avg)
}

fn measure_signing_time(key_manager: &KeyManager) -> u128 {
    // Measure time to sign a typical order message (40 bytes)
    let test_msg = [0u8; 40];
    let start = Instant::now();
    let _ = key_manager.sign(&test_msg);
    start.elapsed().as_micros()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger for telemetry
    env_logger::init();
    
    dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    // Tunables
    let iterations: usize = env::var("STRESS_COUNT").unwrap_or_else(|_| "1000".into()).parse()?;
    let delay_ms: u64 = env::var("STRESS_DELAY_MS").unwrap_or_else(|_| "300".into()).parse()?;
    let order_book_index: u8 = env::var("ORDER_BOOK_INDEX").unwrap_or_else(|_| "0".into()).parse()?;
    let base_amount: i64 = env::var("BASE_AMOUNT").unwrap_or_else(|_| "1000".into()).parse()?;
    let avg_execution_price: i64 = env::var("AVG_EXECUTION_PRICE").unwrap_or_else(|_| "350000".into()).parse()?;
    let is_ask: bool = env::var("IS_ASK").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

    let base_client_order_index: u64 = env::var("CLIENT_ORDER_INDEX_BASE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            // Use seconds since epoch to avoid collisions across runs
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

    // Optional: supply an external starting nonce to bypass cache/refetch entirely.
    // When set, nonces will be deterministic: external_nonce_start + i
    let external_nonce_start: Option<i64> = env::var("EXTERNAL_NONCE_START").ok().and_then(|v| v.parse().ok());

    println!("Starting stress: {} orders, {} ms spacing", iterations, delay_ms);
    println!("Market: {}, base_amount: {}, price: {}, side: {}", order_book_index, base_amount, avg_execution_price, if is_ask { "ASK" } else { "BID" });

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Measure average signing time with 10 samples
    let api_key_bytes = hex::decode(&api_key)?;
    let key_manager = KeyManager::new(&api_key_bytes)?;
    let mut sign_samples = Vec::new();
    for _ in 0..10 {
        sign_samples.push(measure_signing_time(&key_manager));
    }
    let avg_signing_micros = sign_samples.iter().sum::<u128>() / sign_samples.len() as u128;
    println!("Average signing time: {:.2} μs ({:.3} ms)\n", avg_signing_micros, avg_signing_micros as f64 / 1000.0);

    let mut success = 0usize;
    let mut sig_fail = 0usize;
    let mut other_fail = 0usize;
    let mut sample_errors: Vec<String> = Vec::new();
    let mut timings: Vec<OrderTiming> = Vec::new();

    for i in 0..iterations {
        let order_start = Instant::now();
        
        let client_order_index = base_client_order_index + i as u64;
        
        let resp = if let Some(start) = external_nonce_start {
            // Deterministic external nonce sequence
            let nonce = start + i as i64;
            client
                .create_market_order_with_nonce(
                    order_book_index,
                    client_order_index,
                    base_amount,
                    avg_execution_price,
                    is_ask,
                    Some(nonce),
                )
                .await
        } else {
            client
                .create_market_order(
                    order_book_index,
                    client_order_index,
                    base_amount,
                    avg_execution_price,
                    is_ask,
                )
                .await
        };

        let total_elapsed = order_start.elapsed();

        match resp {
            Ok(json) => {
                let code = json["code"].as_i64().unwrap_or_default();
                if code == 200 {
                    success += 1;
                    // Use measured average signing time, network time is total minus signing
                    let network_micros = total_elapsed.as_micros().saturating_sub(avg_signing_micros);
                    timings.push(OrderTiming {
                        signing_micros: avg_signing_micros,
                        network_micros,
                        total_micros: total_elapsed.as_micros(),
                    });
                } else {
                    let msg = json["message"].as_str().unwrap_or("").to_string();
                    if code == 21120 {
                        sig_fail += 1;
                    } else {
                        other_fail += 1;
                    }
                    if sample_errors.len() < 10 {
                        sample_errors.push(format!("code={} msg={}", code, msg));
                    }
                }
            }
            Err(e) => {
                other_fail += 1;
                if sample_errors.len() < 10 {
                    sample_errors.push(format!("transport_err={}" , e));
                }
            }
        }

        if (i + 1) % 50 == 0 {
            println!(
                "Progress {:>4}/{} | ok={} sig_fail={} other_fail={}",
                i + 1,
                iterations,
                success,
                sig_fail,
                other_fail
            );
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    let total = success + sig_fail + other_fail;
    let fail = sig_fail + other_fail;
    let fail_rate = if total == 0 {
        0.0
    } else {
        (fail as f64 / total as f64) * 100.0
    };

    println!("\nRun complete:");
    println!("  total={} success={} sig_fail={} other_fail={} fail_rate={:.2}%", total, success, sig_fail, other_fail, fail_rate);
    if !sample_errors.is_empty() {
        println!("  sample errors (up to 10):");
        for e in sample_errors {
            println!("    - {}", e);
        }
    }

    // Display timing statistics
    if !timings.is_empty() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║                 PERFORMANCE BENCHMARKS                     ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        
        let signing_times: Vec<u128> = timings.iter().map(|t| t.signing_micros).collect();
        let network_times: Vec<u128> = timings.iter().map(|t| t.network_micros).collect();
        let total_times: Vec<u128> = timings.iter().map(|t| t.total_micros).collect();
        
        let (sign_p50, sign_p95, sign_p99, sign_p100, sign_avg) = calculate_percentiles(signing_times);
        let (net_p50, net_p95, net_p99, net_p100, net_avg) = calculate_percentiles(network_times);
        let (tot_p50, tot_p95, tot_p99, tot_p100, tot_avg) = calculate_percentiles(total_times);
        
        println!("║ Signing Time (signature generation):                      ║");
        println!("║   avg: {:>8.2} μs  p50: {:>8} μs  p95: {:>8} μs       ║", sign_avg, sign_p50, sign_p95);
        println!("║   p99: {:>8} μs  max: {:>8} μs                        ║", sign_p99, sign_p100);
        println!("║                                                            ║");
        println!("║ Network Time (send + receive):                            ║");
        println!("║   avg: {:>8.2} μs  p50: {:>8} μs  p95: {:>8} μs       ║", net_avg, net_p50, net_p95);
        println!("║   p99: {:>8} μs  max: {:>8} μs                        ║", net_p99, net_p100);
        println!("║                                                            ║");
        println!("║ Total Time (end-to-end):                                  ║");
        println!("║   avg: {:>8.2} μs  p50: {:>8} μs  p95: {:>8} μs       ║", tot_avg, tot_p50, tot_p95);
        println!("║   p99: {:>8} μs  max: {:>8} μs                        ║", tot_p99, tot_p100);
        println!("╚════════════════════════════════════════════════════════════╝");
        
        // Convert to milliseconds for readability
        println!("\n(In milliseconds):");
        println!("  Signing:  avg={:.2}ms  p50={:.2}ms  p95={:.2}ms  p99={:.2}ms  max={:.2}ms", 
            sign_avg / 1000.0, sign_p50 as f64 / 1000.0, sign_p95 as f64 / 1000.0, sign_p99 as f64 / 1000.0, sign_p100 as f64 / 1000.0);
        println!("  Network:  avg={:.2}ms  p50={:.2}ms  p95={:.2}ms  p99={:.2}ms  max={:.2}ms", 
            net_avg / 1000.0, net_p50 as f64 / 1000.0, net_p95 as f64 / 1000.0, net_p99 as f64 / 1000.0, net_p100 as f64 / 1000.0);
        println!("  Total:    avg={:.2}ms  p50={:.2}ms  p95={:.2}ms  p99={:.2}ms  max={:.2}ms", 
            tot_avg / 1000.0, tot_p50 as f64 / 1000.0, tot_p95 as f64 / 1000.0, tot_p99 as f64 / 1000.0, tot_p100 as f64 / 1000.0);
    }

    Ok(())
}
