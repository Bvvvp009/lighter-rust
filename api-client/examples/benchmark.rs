use api_client::LighterClient;
use std::env;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use reqwest::Client;
use serde_json::json;
use base64::Engine;

type OrderResult = std::result::Result<(Duration, Duration, bool, Option<String>), String>;

#[derive(Debug)]
struct BenchmarkResult {
    total_time: Duration,
    signing_time: Duration,
    api_time: Duration,
    success_count: usize,
    error_count: usize,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 RUST SIGNER BENCHMARK");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://testnet.zklighter.elliot.ai".to_string());
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    // Benchmark parameters
    let num_orders = 100;
    let num_market = 50;
    let num_limit = 50;

    println!("🔥 Starting benchmark:");
    println!("  Total orders: {}", num_orders);
    println!("  Market orders: {}", num_market);
    println!("  Limit orders: {}", num_limit);
    println!("  Execution: Simultaneous (async)");
    println!();

    let start_time = Instant::now();

    // Create tasks for simultaneous execution
    let mut tasks: JoinSet<OrderResult> = JoinSet::new();

    // Market orders
    for i in 0..num_market {
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let account_index = account_index;
        let api_key_index = api_key_index;

        tasks.spawn(async move {
            let order_start = Instant::now();
            
            // Create client for this order
            let client = match LighterClient::new(
                base_url.clone(),
                &api_key,
                account_index,
                api_key_index,
            ) {
                Ok(c) => c,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        Duration::ZERO,
                        false,
                        Some(format!("Client creation error: {}", e)),
                    ));
                }
            };

            // Get nonce (part of API time)
            let nonce_start = Instant::now();
            let nonce = match client.get_nonce().await {
                Ok(n) => n,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        Duration::ZERO,
                        false,
                        Some(format!("Nonce error: {}", e)),
                    ));
                }
            };
            let nonce_time = nonce_start.elapsed();

            // Sign transaction (signing time)
            use std::time::{SystemTime, UNIX_EPOCH};
            use serde_json::json;
            
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            let expired_at = now + 599_000;

            let tx_info = json!({
                "AccountIndex": account_index,
                "ApiKeyIndex": api_key_index,
                "MarketIndex": 0,
                "ClientOrderIndex": 1000000 + i as u64,
                "BaseAmount": 1000,
                "Price": 349659,
                "IsAsk": if i % 2 == 0 { 1 } else { 0 },
                "Type": 1, // MARKET
                "TimeInForce": 0,
                "ReduceOnly": 0,
                "TriggerPrice": 0,
                "OrderExpiry": 0,
                "ExpiredAt": expired_at,
                "Nonce": nonce,
                "Sig": ""
            });

            let tx_json = serde_json::to_string(&tx_info).unwrap();
            
            let sign_start = Instant::now();
            let signature = match client.sign_transaction(&tx_json) {
                Ok(sig) => sig,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        nonce_time,
                        false,
                        Some(format!("Signing error: {}", e)),
                    ));
                }
            };
            let signing_time = sign_start.elapsed();

            // Prepare final transaction
            let mut final_tx_info = tx_info;
            final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));
            
            // Send to API (API time) - use separate HTTP client
            let api_start = Instant::now();
            let http_client = Client::new();
            let response = http_client
                .post(&format!("{}/api/v1/sendTx", base_url))
                .form(&[
                    ("tx_type", "14"),
                    ("tx_info", &serde_json::to_string(&final_tx_info).unwrap()),
                    ("price_protection", "true"),
                ])
                .send()
                .await;

            let api_time = api_start.elapsed() + nonce_time;

            match response {
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_default();
                    let response_json: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                    let code = response_json["code"].as_i64().unwrap_or(-1);
                    if code == 200 {
                        Ok((signing_time, api_time, true, None))
                    } else {
                        let msg = response_json["message"]
                            .as_str()
                            .unwrap_or("Unknown error")
                            .to_string();
                        Ok((signing_time, api_time, false, Some(msg)))
                    }
                }
                Err(e) => {
                    Ok((signing_time, api_time, false, Some(e.to_string())))
                }
            }
        });
    }

    // Limit orders
    for i in 0..num_limit {
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let account_index = account_index;
        let api_key_index = api_key_index;

        tasks.spawn(async move {
            let order_start = Instant::now();
            
            // Create client for this order
            let client = match LighterClient::new(
                base_url.clone(),
                &api_key,
                account_index,
                api_key_index,
            ) {
                Ok(c) => c,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        Duration::ZERO,
                        false,
                        Some(format!("Client creation error: {}", e)),
                    ));
                }
            };

            // Get nonce (part of API time)
            let nonce_start = Instant::now();
            let nonce = match client.get_nonce().await {
                Ok(n) => n,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        Duration::ZERO,
                        false,
                        Some(format!("Nonce error: {}", e)),
                    ));
                }
            };
            let nonce_time = nonce_start.elapsed();

            // Sign transaction (signing time)
            use std::time::{SystemTime, UNIX_EPOCH};
            use serde_json::json;
            
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            let expired_at = now + 599_000;

            let tx_info = json!({
                "AccountIndex": account_index,
                "ApiKeyIndex": api_key_index,
                "MarketIndex": 0,
                "ClientOrderIndex": 2000000 + i as u64,
                "BaseAmount": 1000,
                "Price": 349659,
                "IsAsk": if i % 2 == 0 { 1 } else { 0 },
                "Type": 0, // LIMIT
                "TimeInForce": 1,
                "ReduceOnly": 0,
                "TriggerPrice": 0,
                "OrderExpiry": 0,
                "ExpiredAt": expired_at,
                "Nonce": nonce,
                "Sig": ""
            });

            let tx_json = serde_json::to_string(&tx_info).unwrap();
            
            let sign_start = Instant::now();
            let signature = match client.sign_transaction(&tx_json) {
                Ok(sig) => sig,
                Err(e) => {
                    return Ok((
                        Duration::ZERO,
                        nonce_time,
                        false,
                        Some(format!("Signing error: {}", e)),
                    ));
                }
            };
            let signing_time = sign_start.elapsed();

            // Prepare final transaction
            let mut final_tx_info = tx_info;
            final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));
            
            // Send to API (API time) - use separate HTTP client
            let api_start = Instant::now();
            let http_client = Client::new();
            let response = http_client
                .post(&format!("{}/api/v1/sendTx", base_url))
                .form(&[
                    ("tx_type", "14"),
                    ("tx_info", &serde_json::to_string(&final_tx_info).unwrap()),
                    ("price_protection", "true"),
                ])
                .send()
                .await;

            let api_time = api_start.elapsed() + nonce_time;

            match response {
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_default();
                    let response_json: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                    let code = response_json["code"].as_i64().unwrap_or(-1);
                    if code == 200 {
                        Ok((signing_time, api_time, true, None))
                    } else {
                        let msg = response_json["message"]
                            .as_str()
                            .unwrap_or("Unknown error")
                            .to_string();
                        Ok((signing_time, api_time, false, Some(msg)))
                    }
                }
                Err(e) => {
                    Ok((signing_time, api_time, false, Some(e.to_string())))
                }
            }
        });
    }

    // Collect results
    let mut results = BenchmarkResult {
        total_time: Duration::ZERO,
        signing_time: Duration::ZERO,
        api_time: Duration::ZERO,
        success_count: 0,
        error_count: 0,
        errors: Vec::new(),
    };

    let mut signing_times = Vec::new();

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok((sign_time, api_time, success, error))) => {
                signing_times.push(sign_time);
                results.signing_time += sign_time;
                results.api_time += api_time;
                if success {
                    results.success_count += 1;
                } else {
                    results.error_count += 1;
                    if let Some(err) = error {
                        results.errors.push(err);
                    }
                }
            }
            Ok(Err(e)) => {
                results.error_count += 1;
                results.errors.push(e.to_string());
            }
            Err(e) => {
                results.error_count += 1;
                results.errors.push(format!("Task join error: {}", e));
            }
        }
    }

    results.total_time = start_time.elapsed();
    
    // Calculate timing statistics
    if !signing_times.is_empty() {
        let total_signing: Duration = signing_times.iter().sum();
        results.signing_time = total_signing;
        results.api_time = results.total_time - total_signing;
    }

    // Print results
    println!("{}", "═".repeat(80));
    println!("📊 BENCHMARK RESULTS");
    println!("{}", "═".repeat(80));
    println!();
    println!("⏱️  Timing:");
    println!("  Total round-trip time:    {:.2} ms", results.total_time.as_secs_f64() * 1000.0);
    println!("  Total signing time:       {:.2} ms", results.signing_time.as_secs_f64() * 1000.0);
    println!("  Total API call time:      {:.2} ms", results.api_time.as_secs_f64() * 1000.0);
    println!("  Average per order:        {:.2} ms", results.total_time.as_secs_f64() * 1000.0 / num_orders as f64);
    println!("  Average signing per order: {:.2} ms", results.signing_time.as_secs_f64() * 1000.0 / num_orders as f64);
    println!("  Average API per order:     {:.2} ms", results.api_time.as_secs_f64() * 1000.0 / num_orders as f64);
    println!();

    if !signing_times.is_empty() {
        signing_times.sort();
        let min = signing_times.first().unwrap();
        let max = signing_times.last().unwrap();
        let median = signing_times[signing_times.len() / 2];
        let p95 = signing_times[(signing_times.len() as f64 * 0.95) as usize];
        let p99 = signing_times[(signing_times.len() as f64 * 0.99) as usize];

        println!("📈 Signing Time Statistics:");
        println!("  Min:     {:.2} ms", min.as_secs_f64() * 1000.0);
        println!("  Max:     {:.2} ms", max.as_secs_f64() * 1000.0);
        println!("  Median:  {:.2} ms", median.as_secs_f64() * 1000.0);
        println!("  P95:     {:.2} ms", p95.as_secs_f64() * 1000.0);
        println!("  P99:     {:.2} ms", p99.as_secs_f64() * 1000.0);
        println!();
    }

    println!("✅ Success: {}", results.success_count);
    println!("❌ Errors:   {}", results.error_count);
    println!("📊 Success Rate: {:.2}%", 
        (results.success_count as f64 / num_orders as f64) * 100.0);
    println!();

    if !results.errors.is_empty() {
        println!("⚠️  Errors (first 10):");
        for (i, error) in results.errors.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        if results.errors.len() > 10 {
            println!("  ... and {} more errors", results.errors.len() - 10);
        }
    }

    println!("{}", "═".repeat(80));

    Ok(())
}

