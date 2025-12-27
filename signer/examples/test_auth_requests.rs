//! Test Auth Token Generation with Detailed Logging
//!
//! This example creates a NEW auth token for each request, logs full request/response
//! details, and verifies the server actually accepts and returns valid data.
//!
//! Usage:
//!   cargo run --example test_auth_requests --release
//!
//! Environment variables:
//!   API_PRIVATE_KEY - Your API private key (hex, with or without 0x prefix)
//!   API_KEY_INDEX   - API key index (default: 5)
//!   ACCOUNT_INDEX   - Account index (default: 361816)
//!   BASE_URL        - Base URL (default: https://mainnet.zklighter.elliot.ai)

use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::Client;
use serde_json::Value;
use tokio::time::sleep;

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

struct Config {
    api_private_key: String,
    api_key_index: u8,
    account_index: i64,
    base_url: String,
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
        
        Ok(Config {
            api_private_key,
            api_key_index,
            account_index,
            base_url,
        })
    }
}

struct RequestResult {
    request_num: usize,
    endpoint: String,
    auth_token: String,
    status_code: u16,
    success: bool,
    response_body: String,
    error: Option<String>,
}

impl RequestResult {
    fn print(&self) {
        println!("\n{}", "=".repeat(80));
        println!("REQUEST #{}: {}", self.request_num, self.endpoint);
        println!("{}", "=".repeat(80));
        println!("Auth Token: {}...", &self.auth_token[..self.auth_token.len().min(80)]);
        println!("Status Code: {}", self.status_code);
        println!("Success: {}", if self.success { "✅ YES" } else { "❌ NO" });
        
        if let Some(ref error) = self.error {
            println!("Error: {}", error);
        }
        
        println!("\nResponse Body:");
        if self.response_body.len() > 500 {
            println!("{}...", &self.response_body[..500]);
            println!("\n[Response truncated - {} total chars]", self.response_body.len());
        } else {
            println!("{}", self.response_body);
        }
        
        // Parse JSON response to check if we got actual data
        if let Ok(json) = serde_json::from_str::<Value>(&self.response_body) {
            println!("\n📊 Response Analysis:");
            
            // Check for common response fields
            if json.is_object() {
                let obj = json.as_object().unwrap();
                println!("  - JSON Object with {} fields", obj.len());
                
                // Check for error indicators
                if obj.contains_key("code") || obj.contains_key("error") || obj.contains_key("message") {
                    println!("  ⚠️  WARNING: Response contains error fields");
                }
                
                // Check for data indicators
                if obj.contains_key("data") || obj.contains_key("accounts") || obj.contains_key("orders") {
                    println!("  ✅ Response contains data fields");
                }
                
                // List top-level keys
                let keys: Vec<String> = obj.keys().take(10).map(|k| k.clone()).collect();
                if !keys.is_empty() {
                    println!("  - Top-level keys: {}", keys.join(", "));
                }
            } else if json.is_array() {
                let arr = json.as_array().unwrap();
                println!("  - JSON Array with {} elements", arr.len());
                if !arr.is_empty() {
                    println!("  ✅ Response contains array data");
                }
            }
        } else {
            println!("\n⚠️  Response is not valid JSON");
        }
    }
}

async fn make_request(
    client: &Client,
    config: &Config,
    key_manager: &KeyManager,
    request_num: usize,
    endpoint: &str,
    query_params: &[(&str, &str)],
) -> RequestResult {
    // Generate NEW auth token for THIS request
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let auth_token = match key_manager.create_auth_token(deadline, config.account_index, config.api_key_index) {
        Ok(token) => token,
        Err(e) => {
            return RequestResult {
                request_num,
                endpoint: endpoint.to_string(),
                auth_token: String::new(),
                status_code: 0,
                success: false,
                response_body: String::new(),
                error: Some(format!("Failed to generate auth token: {}", e)),
            };
        }
    };
    
    let url = format!("{}{}", config.base_url, endpoint);
    
    // Build request with auth token
    let mut request = client.get(&url);
    request = request.header("Authorization", &auth_token);
    
    // Add query parameters
    for (key, value) in query_params {
        request = request.query(&[(key, value)]);
    }
    
    // Also add auth as query param (for compatibility)
    request = request.query(&[("auth", &auth_token)]);
    
    // Send request
    let result = match request.send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            
            // Get response body
            let body_text = match response.text().await {
                Ok(text) => text,
                Err(e) => format!("Failed to read response body: {}", e),
            };
            
            let success = status.is_success();
            let error = if success { None } else { Some(format!("HTTP {}", status_code)) };
            
            RequestResult {
                request_num,
                endpoint: endpoint.to_string(),
                auth_token,
                status_code,
                success,
                response_body: body_text,
                error,
            }
        }
        Err(e) => RequestResult {
            request_num,
            endpoint: endpoint.to_string(),
            auth_token,
            status_code: 0,
            success: false,
            response_body: String::new(),
            error: Some(format!("Request failed: {}", e)),
        },
    };
    
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Authenticated Request Test with Detailed Logging");
    println!("{}", "=".repeat(80));
    println!("This test creates a NEW auth token for each request");
    println!("and logs full request/response details for verification.\n");
    
    // Load configuration
    let config = Config::from_env()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    println!("Configuration:");
    println!("  API Key Index:  {}", config.api_key_index);
    println!("  Account Index:  {}", config.account_index);
    println!("  Base URL:       {}", config.base_url);
    println!("  Private Key:    {}...\n", &config.api_private_key[..config.api_private_key.len().min(20)]);
    
    // Initialize key manager
    let key_manager = KeyManager::from_hex(&config.api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Define endpoints to test
    let endpoints = vec![
        ("/api/v1/accountActiveOrders", vec![
            ("account_index", config.account_index.to_string()),
            ("market_id", "0".to_string()),
        ]),
        ("/api/v1/accountLimits", vec![
            ("account_index", config.account_index.to_string()),
        ]),
        ("/api/v1/accountMetadata", vec![
            ("by", "index".to_string()),
            ("value", config.account_index.to_string()),
        ]),
    ];
    
    let mut results = Vec::new();
    let num_requests_per_endpoint = 5; // Test 5 requests per endpoint
    
    println!("📝 Testing {} endpoints with {} requests each ({} total requests)", 
             endpoints.len(), num_requests_per_endpoint, endpoints.len() * num_requests_per_endpoint);
    println!("   Each request uses a NEW auth token\n");
    
    let mut request_num = 1;
    
    for (endpoint, params) in endpoints.iter() {
        println!("\n🔗 Testing endpoint: {}", endpoint);
        println!("{}", "-".repeat(80));
        
        for i in 1..=num_requests_per_endpoint {
            println!("\n[Request {}/{} for {}]", i, num_requests_per_endpoint, endpoint);
            
            // Convert params to query format
            let query_params: Vec<(&str, &str)> = params.iter()
                .map(|(k, v)| {
                    let k_str: &str = k;
                    let v_str: &str = v;
                    (k_str, v_str)
                })
                .collect();
            
            // Make request with NEW auth token
            let result = make_request(
                &client,
                &config,
                &key_manager,
                request_num,
                endpoint,
                &query_params,
            ).await;
            
            // Print detailed result
            result.print();
            
            results.push(result);
            request_num += 1;
            
            // Wait 200ms before next request (except for last request)
            if i < num_requests_per_endpoint || request_num <= endpoints.len() * num_requests_per_endpoint {
                sleep(Duration::from_millis(200)).await;
            }
        }
    }
    
    // Print summary
    println!("\n\n{}", "=".repeat(80));
    println!("TEST SUMMARY");
    println!("{}", "=".repeat(80));
    
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();
    let auth_failures = results.iter()
        .filter(|r| !r.success && r.status_code == 401)
        .count();
    let other_errors = results.iter()
        .filter(|r| !r.success && r.status_code != 401)
        .count();
    
    println!("Total Requests:     {}", total);
    println!("Successful:         {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("Failed:             {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!("  - Auth Failures:  {} ({:.1}%)", auth_failures, (auth_failures as f64 / total as f64) * 100.0);
    println!("  - Other Errors:   {} ({:.1}%)", other_errors, (other_errors as f64 / total as f64) * 100.0);
    
    // Status code breakdown
    let mut status_codes: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for result in &results {
        *status_codes.entry(result.status_code).or_insert(0) += 1;
    }
    
    if !status_codes.is_empty() {
        println!("\nStatus Code Breakdown:");
        let mut codes: Vec<_> = status_codes.iter().collect();
        codes.sort();
        for (code, count) in codes {
            println!("  {}: {} requests", code, count);
        }
    }
    
    // Check if responses contain actual data
    println!("\n📊 Response Analysis:");
    let responses_with_data = results.iter()
        .filter(|r| r.success)
        .filter(|r| {
            if let Ok(json) = serde_json::from_str::<Value>(&r.response_body) {
                if json.is_object() {
                    let obj = json.as_object().unwrap();
                    obj.contains_key("data") || obj.contains_key("accounts") || 
                    obj.contains_key("orders") || obj.len() > 3
                } else if json.is_array() {
                    let arr = json.as_array().unwrap();
                    !arr.is_empty()
                } else {
                    false
                }
            } else {
                false
            }
        })
        .count();
    
    println!("  Requests with actual data: {}/{}", responses_with_data, successful);
    
    if successful > 0 {
        let data_rate = (responses_with_data as f64 / successful as f64) * 100.0;
        println!("  Data rate: {:.1}%", data_rate);
        
        if data_rate < 50.0 {
            println!("  ⚠️  WARNING: Low data rate - many responses may be empty or error messages");
        } else {
            println!("  ✅ Good data rate - responses contain actual data");
        }
    }
    
    // Final verdict
    println!("\n{}", "=".repeat(80));
    if auth_failures == 0 && successful > 0 {
        println!("✅ VERDICT: All auth tokens were accepted by the server");
        if responses_with_data == successful {
            println!("✅ All successful requests returned actual data");
        } else {
            println!("⚠️  Some requests succeeded but returned empty/error responses");
        }
    } else if auth_failures > 0 {
        println!("❌ VERDICT: {} auth token(s) were REJECTED by the server", auth_failures);
        println!("   This indicates a problem with auth token generation or signature verification");
    } else {
        println!("⚠️  VERDICT: No successful requests - check network connectivity and API status");
    }
    println!("{}", "=".repeat(80));
    
    Ok(())
}

