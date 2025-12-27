use api_client::LighterClient;
use dotenv::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Starting stress: {} orders, {} ms spacing", iterations, delay_ms);
    println!("Market: {}, base_amount: {}, price: {}, side: {}", order_book_index, base_amount, avg_execution_price, if is_ask { "ASK" } else { "BID" });

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    let mut success = 0usize;
    let mut sig_fail = 0usize;
    let mut other_fail = 0usize;
    let mut sample_errors: Vec<String> = Vec::new();

    for i in 0..iterations {
        let client_order_index = base_client_order_index + i as u64;
        let resp = client
            .create_market_order(
                order_book_index,
                client_order_index,
                base_amount,
                avg_execution_price,
                is_ask,
            )
            .await;

        match resp {
            Ok(json) => {
                let code = json["code"].as_i64().unwrap_or_default();
                if code == 200 {
                    success += 1;
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

    Ok(())
}
