//! Stress Test: Send 100-1000 Real API Requests with Auth Tokens
//!
//! This test validates that the signature verification fix works correctly
//! by sending many authenticated requests to the Lighter API.
//!
//! Usage:
//!   cargo run --example stress_test_orders --release
//!
//! Environment variables:
//!   API_PRIVATE_KEY - Your API private key (hex, with or without 0x prefix)
//!   API_KEY_INDEX   - API key index (default: 5)
//!   ACCOUNT_INDEX   - Account index (default: 361816)
//!   BASE_URL        - Base URL (default: https://mainnet.zklighter.elliot.ai)
//!   NUM_REQUESTS    - Number of requests to send (default: 100, can be 100-1000)
//!   ENDPOINT        - API endpoint to test (default: /api/v1/accountActiveOrders)

use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::Client;
use tokio::time::sleep;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use signer::KeyManager;

fn load_dotenv() {
    if let Ok(current_dir) = env::current_dir() {
        let env_files = [
            current_dir.join(".env"),
            current_dir.join("..").join(".env"),
            current_dir.join("..").join("..").join(".env"),
        ];
        for env_file in env_files.iter() {
            if env_file.exists() {
                if let Ok(content) = std::fs::read_to_string(env_file) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"').trim_matches('\'');
                            if env::var(key).is_err() {
                                env::set_var(key, value);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
struct Config {
    api_private_key: String,
    api_key_index: u8,
    account_index: i64,
    base_url: String,
    num_requests: usize,
    endpoint: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let api_private_key = env::var("API_PRIVATE_KEY")
            .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
        
        let api_key_index = env::var("API_KEY_INDEX")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u8>()
            .map_err(|_| "API_KEY_INDEX must be a valid u8")?;
        
        let account_index = env::var("ACCOUNT_INDEX")
            .unwrap_or_else(|_| "361816".to_string())
            .parse::<i64>()
            .map_err(|_| "ACCOUNT_INDEX must be a valid i64")?;
        
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
        
        let num_requests = env::var("NUM_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        
        let endpoint = env::var("ENDPOINT")
            .unwrap_or_else(|_| "/api/v1/accountActiveOrders".to_string());
        
        Ok(Config {
            api_private_key,
            api_key_index,
            account_index,
            base_url,
            num_requests: num_requests.min(10000), // Cap at 10k for safety
            endpoint,
        })
    }
}

struct RequestStats {
    total: AtomicUsize,
    successful: AtomicUsize,
    failed: AtomicUsize,
    auth_failures: AtomicUsize,
    other_errors: AtomicUsize,
    total_time_ms: AtomicU64,
}

impl RequestStats {
    fn new() -> Self {
        Self {
            total: AtomicUsize::new(0),
            successful: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            auth_failures: AtomicUsize::new(0),
            other_errors: AtomicUsize::new(0),
            total_time_ms: AtomicU64::new(0),
        }
    }
    
    fn record_success(&self, duration_ms: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.successful.fetch_add(1, Ordering::Relaxed);
        self.total_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }
    
    fn record_auth_failure(&self, duration_ms: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.failed.fetch_add(1, Ordering::Relaxed);
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
        self.total_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }
    
    fn record_other_error(&self, duration_ms: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.failed.fetch_add(1, Ordering::Relaxed);
        self.other_errors.fetch_add(1, Ordering::Relaxed);
        self.total_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }
    
    fn print_summary(&self) {
        let total = self.total.load(Ordering::Relaxed);
        let successful = self.successful.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let auth_failures = self.auth_failures.load(Ordering::Relaxed);
        let other_errors = self.other_errors.load(Ordering::Relaxed);
        let total_time_ms = self.total_time_ms.load(Ordering::Relaxed);
        
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_time_ms = if total > 0 {
            total_time_ms as f64 / total as f64
        } else {
            0.0
        };
        
        println!("\n{}", "=".repeat(80));
        println!("STRESS TEST SUMMARY");
        println!("{}", "=".repeat(80));
        println!("Total Requests:        {}", total);
        println!("Successful:            {} ({:.2}%)", successful, success_rate);
        println!("Failed:                {} ({:.2}%)", failed, 100.0 - success_rate);
        println!("  - Auth Failures:     {} ({:.2}%)", auth_failures, 
                 if total > 0 { (auth_failures as f64 / total as f64) * 100.0 } else { 0.0 });
        println!("  - Other Errors:      {} ({:.2}%)", other_errors,
                 if total > 0 { (other_errors as f64 / total as f64) * 100.0 } else { 0.0 });
        println!("Average Response Time: {:.2} ms", avg_time_ms);
        println!("Total Time:            {:.2} seconds", total_time_ms as f64 / 1000.0);
        
        println!("\n{}", "=".repeat(80));
        if auth_failures == 0 && successful > 0 {
            println!("✅ VERDICT: All auth tokens were accepted by the server!");
            println!("✅ Signature verification fix is working correctly!");
            if success_rate >= 95.0 {
                println!("✅ Excellent success rate (>= 95%)");
            } else if success_rate >= 90.0 {
                println!("⚠️  Good success rate (>= 90%) but some failures");
            } else {
                println!("⚠️  Success rate below 90% - may indicate issues");
            }
        } else if auth_failures > 0 {
            println!("❌ VERDICT: {} auth token(s) were REJECTED by the server", auth_failures);
            println!("❌ This indicates a problem with signature verification");
            let failure_rate = (auth_failures as f64 / total as f64) * 100.0;
            if failure_rate > 10.0 {
                println!("❌ High failure rate (>10%) - signature fix may not be working correctly");
            } else {
                println!("⚠️  Low failure rate (<10%) - may be intermittent edge cases");
            }
        } else {
            println!("⚠️  VERDICT: No successful requests - check network/API status");
        }
        println!("{}", "=".repeat(80));
    }
}

async fn make_request(
    client: &Client,
    config: &Config,
    key_manager: &KeyManager,
    request_id: usize,
    stats: &RequestStats,
) {
    let start_time = SystemTime::now();
    
    // Generate NEW auth token for THIS request
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let auth_token = match key_manager.create_auth_token(deadline, config.account_index, config.api_key_index) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("[Request {}] Failed to generate auth token: {}", request_id, e);
            stats.record_other_error(0);
            return;
        }
    };
    
    let url = format!("{}{}", config.base_url, config.endpoint);
    
    // Build request with auth token
    let mut request = client.get(&url);
    request = request.header("Authorization", &auth_token);
    
    // Add query parameters based on endpoint
    if config.endpoint.contains("accountActiveOrders") {
        request = request.query(&[
            ("account_index", config.account_index.to_string().as_str()),
            ("market_id", "0"),
        ]);
    } else if config.endpoint.contains("accountLimits") {
        request = request.query(&[("account_index", config.account_index.to_string().as_str())]);
    }
    
    // Send request
    let result = request.send().await;
    
    let duration_ms = start_time.elapsed().unwrap().as_millis() as u64;
    
    match result {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            
            if status.is_success() {
                stats.record_success(duration_ms);
                if request_id % 50 == 0 {
                    println!("[Request {}] ✅ Success ({} ms)", request_id, duration_ms);
                }
            } else if status_code == 401 {
                stats.record_auth_failure(duration_ms);
                eprintln!("[Request {}] ❌ Auth failure (401) ({} ms)", request_id, duration_ms);
            } else {
                stats.record_other_error(duration_ms);
                // Always show first few errors for debugging
                if request_id <= 5 {
                    let body_text = response.text().await.unwrap_or_default();
                    eprintln!("[Request {}] ⚠️  Error {}: {} ({} ms)", request_id, status_code, 
                             body_text.chars().take(100).collect::<String>(), duration_ms);
                } else if request_id % 50 == 0 {
                    eprintln!("[Request {}] ⚠️  Error {} ({} ms)", request_id, status_code, duration_ms);
                }
            }
        }
        Err(e) => {
            stats.record_other_error(duration_ms);
            if request_id % 50 == 0 {
                eprintln!("[Request {}] ❌ Request failed: {} ({} ms)", request_id, e, duration_ms);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🚀 Stress Test: Sending Real API Requests with Auth Tokens");
    println!("{}", "=".repeat(80));
    println!("This test validates the signature verification fix by sending");
    println!("many authenticated requests to the Lighter API.\n");
    
    // Load configuration
    let config = Config::from_env()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    println!("Configuration:");
    println!("  API Key Index:  {}", config.api_key_index);
    println!("  Account Index:  {}", config.account_index);
    println!("  Base URL:       {}", config.base_url);
    println!("  Endpoint:       {}", config.endpoint);
    println!("  Number of Requests: {}", config.num_requests);
    println!("  Private Key:    {}...\n", &config.api_private_key[..config.api_private_key.len().min(20)]);
    
    // Initialize key manager
    let key_manager = Arc::new(KeyManager::from_hex(&config.api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?);
    
    // Create HTTP client
    let client = Arc::new(Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?);
    
    let stats = Arc::new(RequestStats::new());
    
    println!("Starting stress test...");
    println!("Each request will use a NEW auth token (fresh signature)\n");
    
    let start_time = SystemTime::now();
    
    // Send requests with controlled concurrency (max 10 concurrent)
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10));
    let mut handles = Vec::new();
    
    for i in 1..=config.num_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client_clone = client.clone();
        let config_clone = config.clone();
        let key_manager_clone = key_manager.clone();
        let stats_clone = stats.clone();
        
        let handle = tokio::spawn(async move {
            make_request(
                &client_clone,
                &config_clone,
                &key_manager_clone,
                i,
                &stats_clone,
            ).await;
            drop(permit);
        });
        
        handles.push(handle);
        
        // Print progress every 100 requests
        if i % 100 == 0 {
            let elapsed = start_time.elapsed().unwrap().as_secs();
            let rate = i as f64 / elapsed as f64;
            println!("Progress: {}/{} requests sent ({:.1} req/s)", i, config.num_requests, rate);
        }
        
        // Small delay to avoid overwhelming the server
        if i % 10 == 0 {
            sleep(Duration::from_millis(50)).await;
        }
    }
    
    // Wait for all requests to complete
    println!("\nWaiting for all requests to complete...");
    for handle in handles {
        handle.await?;
    }
    
    let total_elapsed = start_time.elapsed().unwrap().as_secs_f64();
    
    // Print final summary
    stats.print_summary();
    println!("\nTotal test duration: {:.2} seconds", total_elapsed);
    println!("Average rate: {:.2} requests/second", config.num_requests as f64 / total_elapsed);
    
    Ok(())
}

