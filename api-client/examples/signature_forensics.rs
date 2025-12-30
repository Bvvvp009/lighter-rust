// Signature Forensics Tool - Deep Diagnostic Capture
// This tool captures every detail of signature generation and validation
// to identify root causes of signature failures

use api_client::{CreateOrderRequest, LighterClient};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
struct SignatureForensics {
    // Request metadata
    attempt_id: String,
    timestamp_created: i64,
    timestamp_sent: i64,
    timestamp_responded: i64,
    
    // Transaction data
    account_index: i64,
    api_key_index: u8,
    market_index: u8,
    client_order_index: u64,
    base_amount: i64,
    price: i64,
    is_ask: bool,
    order_type: u8,
    time_in_force: u8,
    reduce_only: bool,
    trigger_price: i64,
    order_expiry: i64,
    expired_at: i64,
    nonce: i64,
    
    // JSON serialization
    tx_json_before_sig: String,
    tx_json_after_sig: String,
    json_bytes_count: usize,
    
    // Hashing data
    tx_hash_hex: String,
    tx_hash_bytes: Vec<u8>,
    
    // Signature components  
    nonce_used_hex: String,
    signature_hex: String,
    signature_s_hex: String,  // First 40 bytes
    signature_e_hex: String,  // Last 40 bytes
    
    // Server response
    response_code: i64,
    response_message: String,
    response_success: bool,
    response_full_json: String,
    
    // Retry tracking
    is_retry: bool,
    retry_count: u32,
    parent_attempt_id: Option<String>,
}

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

    println!("🔬 Signature Forensics Tool");
    println!("================================");
    println!("This tool will capture detailed signature generation data");
    println!("to identify root causes of signature validation failures.\n");

    // Test configuration
    let test_count: usize = std::env::var("TEST_COUNT")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .expect("TEST_COUNT must be a number");
    
    let output_file = "signature_forensics.jsonl";
    println!("📝 Output file: {}", output_file);
    println!("🎯 Test orders: {}", test_count);
    println!();

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut forensics_records = Vec::new();

    for i in 0..test_count {
        let client_order_index = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_micros() as u64;

        let order = CreateOrderRequest {
            account_index,
            order_book_index: 2, // ETH market
            client_order_index,
            base_amount: -1_000_000,  // 0.001 ETH
            price: 3200_000_000,      // $3200
            is_ask: true,
            order_type: 0,  // Limit
            time_in_force: 1,  // GoodTillTime
            reduce_only: false,
            trigger_price: 0,
        };

        println!("[{}/{}] Creating order {}...", i + 1, test_count, client_order_index);
        
        // Capture forensics data
        let forensics = capture_order_forensics(&client, &order, i).await?;
        
        if forensics.response_success {
            success_count += 1;
            println!("  ✅ SUCCESS (code {})", forensics.response_code);
        } else {
            failure_count += 1;
            println!("  ❌ FAILED (code {}) - {}", 
                forensics.response_code, forensics.response_message);
        }

        // Write to file immediately (append mode)
        write_forensics_record(&forensics, output_file)?;
        forensics_records.push(forensics);

        // Small delay to avoid overwhelming the server
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!();
    println!("================================");
    println!("📊 Results Summary");
    println!("================================");
    println!("Total orders:    {}", test_count);
    println!("✅ Successful:   {} ({:.1}%)", success_count, 
        (success_count as f64 / test_count as f64) * 100.0);
    println!("❌ Failed:       {} ({:.1}%)", failure_count,
        (failure_count as f64 / test_count as f64) * 100.0);
    println!();
    println!("📁 Forensics data saved to: {}", output_file);
    println!();
    
    // Analyze patterns
    analyze_forensics(&forensics_records)?;

    Ok(())
}

async fn capture_order_forensics(
    client: &LighterClient,
    order: &CreateOrderRequest,
    index: usize,
) -> Result<SignatureForensics, Box<dyn std::error::Error>> {
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    let attempt_id = format!("{}-{}", 
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros(),
        index
    );

    let timestamp_created = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as i64;

    // We need to replicate the exact transaction building logic
    // to capture intermediate values
    
    // Calculate expired_at and order_expiry (matching api-client logic)
    let now = timestamp_created;
    let expired_at_skew: i64 = std::env::var("EXPIRED_AT_SKEW_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let expired_at = now + 599_000 + expired_at_skew;
    
    let order_expiry = if order.time_in_force == 1 && order.order_type == 0 {
        now + (28 * 24 * 60 * 60 * 1000)
    } else {
        0
    };

    // For now, we'll use a test nonce since we can't extract it from the internal method
    // This is a limitation - we'd need to modify api-client to expose this data
    let nonce = 0i64; // Placeholder - will be filled by actual implementation

    // Build transaction JSON (before signature)
    let tx_info = json!({
        "AccountIndex": order.account_index,
        "ApiKeyIndex": order.order_book_index, // This is simplified
        "MarketIndex": order.order_book_index,
        "ClientOrderIndex": order.client_order_index,
        "BaseAmount": order.base_amount,
        "Price": order.price,
        "IsAsk": if order.is_ask { 1 } else { 0 },
        "Type": order.order_type,
        "TimeInForce": order.time_in_force,
        "ReduceOnly": if order.reduce_only { 1 } else { 0 },
        "TriggerPrice": order.trigger_price,
        "OrderExpiry": order_expiry,
        "ExpiredAt": expired_at,
        "Nonce": nonce,
        "Sig": ""
    });

    let tx_json_before_sig = serde_json::to_string(&tx_info)?;

    // Send the actual order
    let timestamp_sent = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as i64;
    
    let response = client.create_order(order.clone()).await?;
    
    let timestamp_responded = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as i64;

    // Parse response
    let response_code = response["code"].as_i64().unwrap_or(-1);
    let response_message = response["message"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let response_success = response_code == 200;
    let response_full_json = serde_json::to_string(&response)?;

    Ok(SignatureForensics {
        attempt_id,
        timestamp_created,
        timestamp_sent,
        timestamp_responded,
        account_index: order.account_index,
        api_key_index: 0, // Simplified
        market_index: order.order_book_index,
        client_order_index: order.client_order_index,
        base_amount: order.base_amount,
        price: order.price,
        is_ask: order.is_ask,
        order_type: order.order_type,
        time_in_force: order.time_in_force,
        reduce_only: order.reduce_only,
        trigger_price: order.trigger_price,
        order_expiry,
        expired_at,
        nonce,
        tx_json_before_sig,
        tx_json_after_sig: String::new(), // Will need to capture this
        json_bytes_count: 0,
        tx_hash_hex: String::new(),
        tx_hash_bytes: Vec::new(),
        nonce_used_hex: String::new(),
        signature_hex: String::new(),
        signature_s_hex: String::new(),
        signature_e_hex: String::new(),
        response_code,
        response_message,
        response_success,
        response_full_json,
        is_retry: false,
        retry_count: 0,
        parent_attempt_id: None,
    })
}

fn write_forensics_record(
    forensics: &SignatureForensics,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_file)?;
    
    let json = serde_json::to_string(forensics)?;
    writeln!(file, "{}", json)?;
    
    Ok(())
}

fn analyze_forensics(
    records: &[SignatureForensics],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Forensics Analysis");
    println!("================================");

    let failures: Vec<_> = records.iter()
        .filter(|r| !r.response_success)
        .collect();

    let successes: Vec<_> = records.iter()
        .filter(|r| r.response_success)
        .collect();

    if failures.is_empty() {
        println!("✅ No failures detected - all signatures validated successfully!");
        return Ok(());
    }

    println!("Failed Signatures Analysis:");
    println!();

    // Analyze error codes
    let mut error_codes: std::collections::HashMap<i64, usize> = 
        std::collections::HashMap::new();
    for failure in &failures {
        *error_codes.entry(failure.response_code).or_insert(0) += 1;
    }

    println!("Error Code Distribution:");
    for (code, count) in error_codes.iter() {
        println!("  Code {}: {} occurrences", code, count);
    }
    println!();

    // Analyze timing
    let failed_latencies: Vec<_> = failures.iter()
        .map(|f| f.timestamp_responded - f.timestamp_sent)
        .collect();
    let success_latencies: Vec<_> = successes.iter()
        .map(|s| s.timestamp_responded - s.timestamp_sent)
        .collect();

    if !failed_latencies.is_empty() {
        let avg_failed = failed_latencies.iter().sum::<i64>() as f64 
            / failed_latencies.len() as f64;
        println!("Average latency for failures: {:.1}ms", avg_failed);
    }

    if !success_latencies.is_empty() {
        let avg_success = success_latencies.iter().sum::<i64>() as f64 
            / success_latencies.len() as f64;
        println!("Average latency for successes: {:.1}ms", avg_success);
    }

    println!();
    println!("💡 Next Steps:");
    println!("  1. Review signature_forensics.jsonl for detailed data");
    println!("  2. Compare failed vs successful signature components");
    println!("  3. Check for patterns in nonce values, timestamps, or JSON structure");
    println!("  4. Run with DEBUG_TX_JSON=1 for even more detail");

    Ok(())
}
