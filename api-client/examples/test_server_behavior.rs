// Test 3: Server Signature Validation Behavior
// This test sends orders to the real API to understand server-side validation

use api_client::{CreateOrderRequest, LighterClient};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY must be set");
    let account_index: i64 = std::env::var("ACCOUNT_INDEX")
        .expect("ACCOUNT_INDEX must be set")
        .parse()
        .expect("ACCOUNT_INDEX must be a number");
    let api_key_index: u8 = std::env::var("API_KEY_INDEX")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("API_KEY_INDEX must be a number");
    
    let client = LighterClient::new(
        "https://api-testnet.lighter.xyz".to_string(),
        &private_key,
        account_index,
        api_key_index,
    )?;

    println!("🔬 Test 3: Server Signature Validation Behavior");
    println!("================================================");
    println!("This test examines how the server validates signatures\n");

    // Test 3a: Sequential orders
    println!("🧪 Test 3a: Sequential Order Submission");
    println!("Sending 50 orders sequentially with 100ms delay...\n");
    
    let mut sequential_results = Vec::new();
    let mut seq_success = 0;
    let mut seq_sig_failures = 0;
    let mut seq_nonce_failures = 0;
    let mut seq_other_failures = 0;

    for i in 0..50 {
        let client_order_index = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_micros() as u64;

        let order = CreateOrderRequest {
            account_index,
            order_book_index: 2,
            client_order_index,
            base_amount: -1_000_000,
            price: 3200_000_000,
            is_ask: true,
            order_type: 0,
            time_in_force: 1,
            reduce_only: false,
            trigger_price: 0,
        };

        let start = std::time::Instant::now();
        let response = client.create_order(order).await?;
        let elapsed = start.elapsed();

        let code = response["code"].as_i64().unwrap_or(-1);
        let message = response["message"].as_str().unwrap_or("").to_string();
        
        let status = if code == 200 {
            seq_success += 1;
            "✅"
        } else {
            let msg_lower = message.to_lowercase();
            if code == 21120 || msg_lower.contains("invalid signature") {
                seq_sig_failures += 1;
                "❌ SIG"
            } else if code == 21104 || msg_lower.contains("nonce") {
                seq_nonce_failures += 1;
                "❌ NONCE"
            } else {
                seq_other_failures += 1;
                "❌ OTHER"
            }
        };

        println!("[{:2}/50] {} code {} in {:4}ms - {}", 
            i + 1, status, code, elapsed.as_millis(), 
            if message.len() > 50 { &message[..50] } else { &message });

        sequential_results.push((code, message, elapsed));
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!();
    println!("Sequential Results:");
    println!("  Success:         {} ({:.1}%)", seq_success, (seq_success as f64 / 50.0) * 100.0);
    println!("  Sig failures:    {} ({:.1}%)", seq_sig_failures, (seq_sig_failures as f64 / 50.0) * 100.0);
    println!("  Nonce failures:  {} ({:.1}%)", seq_nonce_failures, (seq_nonce_failures as f64 / 50.0) * 100.0);
    println!("  Other failures:  {} ({:.1}%)", seq_other_failures, (seq_other_failures as f64 / 50.0) * 100.0);

    // Calculate average latencies
    let seq_latencies: Vec<_> = sequential_results.iter().map(|(_, _, e)| e.as_millis()).collect();
    let seq_avg_latency = seq_latencies.iter().sum::<u128>() as f64 / seq_latencies.len() as f64;
    println!("  Avg latency:     {:.1}ms", seq_avg_latency);

    // Test 3b: Parallel orders
    println!();
    println!("🧪 Test 3b: Parallel Order Submission");
    println!("Sending 50 orders in parallel (10 at a time)...\n");
    
    let mut parallel_results = Vec::new();
    let mut par_success = 0;
    let mut par_sig_failures = 0;
    let mut par_nonce_failures = 0;
    let mut par_other_failures = 0;

    // Send in batches of 10
    for batch in 0..5 {
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let client_clone = LighterClient::new(
                "https://api-testnet.lighter.xyz".to_string(),
                &private_key,
                account_index,
                api_key_index,
            )?;
            
            let handle = tokio::spawn(async move {
                let client_order_index = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64 + i; // Add offset for uniqueness

                let order = CreateOrderRequest {
                    account_index,
                    order_book_index: 2,
                    client_order_index,
                    base_amount: -1_000_000,
                    price: 3200_000_000,
                    is_ask: true,
                    order_type: 0,
                    time_in_force: 1,
                    reduce_only: false,
                    trigger_price: 0,
                };

                let start = std::time::Instant::now();
                let response = client_clone.create_order(order).await?;
                let elapsed = start.elapsed();

                Ok::<_, Box<dyn std::error::Error + Send + Sync>>((response, elapsed))
            });
            
            handles.push(handle);
        }

        // Wait for batch to complete
        let results = futures::future::join_all(handles).await;
        
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok((response, elapsed))) => {
                    let code = response["code"].as_i64().unwrap_or(-1);
                    let message = response["message"].as_str().unwrap_or("").to_string();
                    
                    let status = if code == 200 {
                        par_success += 1;
                        "✅"
                    } else {
                        let msg_lower = message.to_lowercase();
                        if code == 21120 || msg_lower.contains("invalid signature") {
                            par_sig_failures += 1;
                            "❌ SIG"
                        } else if code == 21104 || msg_lower.contains("nonce") {
                            par_nonce_failures += 1;
                            "❌ NONCE"
                        } else {
                            par_other_failures += 1;
                            "❌ OTHER"
                        }
                    };

                    println!("[B{}-{:2}] {} code {} in {:4}ms - {}", 
                        batch + 1, i + 1, status, code, elapsed.as_millis(), 
                        if message.len() > 40 { &message[..40] } else { &message });

                    parallel_results.push((code, message, elapsed));
                }
                Ok(Err(e)) => {
                    println!("[B{}-{:2}] ❌ ERROR: {}", batch + 1, i + 1, e);
                }
                Err(e) => {
                    println!("[B{}-{:2}] ❌ JOIN ERROR: {}", batch + 1, i + 1, e);
                }
            }
        }
        
        // Small delay between batches
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!();
    println!("Parallel Results:");
    println!("  Success:         {} ({:.1}%)", par_success, (par_success as f64 / 50.0) * 100.0);
    println!("  Sig failures:    {} ({:.1}%)", par_sig_failures, (par_sig_failures as f64 / 50.0) * 100.0);
    println!("  Nonce failures:  {} ({:.1}%)", par_nonce_failures, (par_nonce_failures as f64 / 50.0) * 100.0);
    println!("  Other failures:  {} ({:.1}%)", par_other_failures, (par_other_failures as f64 / 50.0) * 100.0);

    let par_latencies: Vec<_> = parallel_results.iter().map(|(_, _, e)| e.as_millis()).collect();
    let par_avg_latency = par_latencies.iter().sum::<u128>() as f64 / par_latencies.len() as f64;
    println!("  Avg latency:     {:.1}ms", par_avg_latency);

    // Analysis
    println!();
    println!("================================");
    println!("📋 Analysis");
    println!("================================");

    let seq_failure_rate = ((seq_sig_failures + seq_nonce_failures) as f64 / 50.0) * 100.0;
    let par_failure_rate = ((par_sig_failures + par_nonce_failures) as f64 / 50.0) * 100.0;

    println!("Sequential failure rate: {:.1}%", seq_failure_rate);
    println!("Parallel failure rate:   {:.1}%", par_failure_rate);
    println!();

    if par_failure_rate > seq_failure_rate * 1.5 {
        println!("🚨 FINDING: Parallel requests have significantly higher failure rate!");
        println!("   Hypothesis: Race condition or server congestion");
        println!("   Recommendation: Implement request queueing/throttling");
    } else if (par_failure_rate - seq_failure_rate).abs() < 2.0 {
        println!("✅ Failure rates are similar (sequential vs parallel)");
        println!("   → NOT a race condition or concurrency issue");
        println!("   → Failure cause is consistent regardless of request pattern");
    } else {
        println!("⚠️  Sequential has higher failure rate (unusual)");
        println!("   → May indicate server warming up or other factors");
    }

    println!();
    
    if seq_sig_failures > 0 || par_sig_failures > 0 {
        println!("⚠️  Signature validation failures detected:");
        println!("   Sequential: {}", seq_sig_failures);
        println!("   Parallel:   {}", par_sig_failures);
        println!();
        println!("   Possible causes:");
        println!("   1. Server-side validation logic differences");
        println!("   2. Timing/expiry issues");
        println!("   3. JSON serialization differences");
        println!("   4. Hash computation differences");
        println!();
        println!("   👉 Run Test 4 to analyze timing correlation");
    }

    Ok(())
}
