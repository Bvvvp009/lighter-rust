//! Test multiple auth endpoints to verify auth tokens work and return data
//!
//! This example tests various endpoints that require authentication and should return
//! actual data from the server to verify auth is working correctly.
//!
//! Usage:
//!   cargo run --example test_auth_endpoints --release
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

struct EndpointResult {
    endpoint: String,
    status_code: u16,
    success: bool,
    has_data: bool,
    response_preview: String,
    error: Option<String>,
}

impl EndpointResult {
    fn print(&self) {
        let status_icon = if self.success { "✅" } else { "❌" };
        let data_icon = if self.has_data { "📊" } else { "⚠️" };
        
        println!("\n{} {} {}", status_icon, data_icon, self.endpoint);
        println!("  Status: {}", self.status_code);
        
        if let Some(ref error) = self.error {
            println!("  Error: {}", error);
        }
        
        if self.has_data {
            println!("  Response: {}...", &self.response_preview[..self.response_preview.len().min(200)]);
        } else if self.success {
            println!("  Response: (empty or no data)");
        }
    }
}

async fn test_endpoint(
    client: &Client,
    config: &Config,
    key_manager: &KeyManager,
    endpoint: &str,
    query_params: &[(&str, &str)],
) -> EndpointResult {
    // Generate auth token
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (7 * 3600);
    
    let auth_token = match key_manager.create_auth_token(deadline, config.account_index, config.api_key_index) {
        Ok(token) => token,
        Err(e) => {
            return EndpointResult {
                endpoint: endpoint.to_string(),
                status_code: 0,
                success: false,
                has_data: false,
                response_preview: String::new(),
                error: Some(format!("Failed to generate auth token: {}", e)),
            };
        }
    };
    
    let url = format!("{}{}", config.base_url, endpoint);
    
    // Build request with auth token
    let mut request = client.get(&url);
    request = request.header("Authorization", &auth_token);
    request = request.query(&[("auth", &auth_token)]);
    
    // Add query parameters
    for (key, value) in query_params {
        request = request.query(&[(key, value)]);
    }
    
    // Send request
    let result = match request.send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            let success = status.is_success();
            
            let body_text = match response.text().await {
                Ok(text) => text,
                Err(e) => format!("Failed to read response: {}", e),
            };
            
            // Check if response contains actual data
            let has_data = if let Ok(json) = serde_json::from_str::<Value>(&body_text) {
                if json.is_object() {
                    let obj = json.as_object().unwrap();
                    // Check for data fields or meaningful content
                    obj.contains_key("data") || 
                    obj.contains_key("accounts") || 
                    obj.contains_key("orders") ||
                    obj.contains_key("markets") ||
                    obj.contains_key("balances") ||
                    obj.len() > 2
                } else if json.is_array() {
                    let arr = json.as_array().unwrap();
                    !arr.is_empty()
                } else {
                    false
                }
            } else {
                body_text.len() > 50 // Non-JSON but substantial content
            };
            
            let error = if success { None } else { 
                Some(format!("HTTP {}: {}", status_code, &body_text[..body_text.len().min(200)]))
            };
            
            EndpointResult {
                endpoint: endpoint.to_string(),
                status_code,
                success,
                has_data,
                response_preview: body_text,
                error,
            }
        }
        Err(e) => EndpointResult {
            endpoint: endpoint.to_string(),
            status_code: 0,
            success: false,
            has_data: false,
            response_preview: String::new(),
            error: Some(format!("Request failed: {}", e)),
        },
    };
    
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    println!("🔍 Testing Auth Endpoints - Verifying Auth Tokens Return Data");
    println!("{}", "=".repeat(80));
    
    // Load configuration
    let config = Config::from_env()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    println!("\nConfiguration:");
    println!("  API Key Index:  {}", config.api_key_index);
    println!("  Account Index:  {}", config.account_index);
    println!("  Base URL:       {}", config.base_url);
    println!();
    
    // Initialize key manager
    let key_manager = KeyManager::from_hex(&config.api_private_key)
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Test the auth-required endpoint 100 times
    let endpoint = "/api/v1/accountMetadata";
    let account_index_str = config.account_index.to_string();
    let query_params = vec![
        ("by", "index"),
        ("value", account_index_str.as_str()),
    ];
    
    println!("Testing endpoint: {}", endpoint);
    println!("Sending 100 requests...\n");
    
    let mut results = Vec::new();
    let num_requests = 100;
    
    for i in 1..=num_requests {
        if i % 10 == 0 {
            print!("Progress: {}/{} requests...\r", i, num_requests);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
        
        let result = test_endpoint(
            &client,
            &config,
            &key_manager,
            endpoint,
            &query_params,
        ).await;
        
        results.push(result);
        
        // Small delay between requests to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    println!("\nCompleted {} requests\n", num_requests);
    
    // Print summary
    println!("\n{}", "=".repeat(80));
    println!("SUMMARY - 100 Requests to {}", endpoint);
    println!("{}", "=".repeat(80));
    
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let with_data = results.iter().filter(|r| r.success && r.has_data).count();
    let auth_failures = results.iter()
        .filter(|r| !r.success && r.status_code == 401)
        .count();
    let other_errors = results.iter()
        .filter(|r| !r.success && r.status_code != 401)
        .count();
    
    println!("Total Requests:        {}", total);
    println!("Successful Responses:  {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("Responses with Data:   {} ({:.1}%)", with_data, (with_data as f64 / total as f64) * 100.0);
    println!("Auth Failures (401):   {} ({:.1}%)", auth_failures, (auth_failures as f64 / total as f64) * 100.0);
    println!("Other Errors:          {} ({:.1}%)", other_errors, (other_errors as f64 / total as f64) * 100.0);
    
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
            println!("  {}: {} request(s) ({:.1}%)", code, count, (*count as f64 / total as f64) * 100.0);
        }
    }
    
    // Show sample responses
    if let Some(success_result) = results.iter().find(|r| r.success && r.has_data) {
        println!("\nSample Successful Response:");
        println!("  {}", &success_result.response_preview[..success_result.response_preview.len().min(300)]);
    }
    
    if let Some(error_result) = results.iter().find(|r| !r.success) {
        println!("\nSample Error Response:");
        if let Some(ref error) = error_result.error {
            println!("  {}", &error[..error.len().min(300)]);
        }
    }
    
    // Final verdict
    println!("\n{}", "=".repeat(80));
    if auth_failures == 0 && successful == total {
        println!("✅ VERDICT: Auth is PERFECTLY WORKING!");
        println!("   - All {} auth tokens accepted (0 auth failures)", total);
        println!("   - 100% success rate");
        println!("   - Auth token generation and verification: ✅ WORKING");
    } else if auth_failures == 0 && successful > 0 {
        println!("✅ VERDICT: Auth is WORKING!");
        println!("   - All auth tokens accepted (0 auth failures)");
        println!("   - {} successful requests out of {}", successful, total);
        if with_data > 0 {
            println!("   - {} requests returned actual data", with_data);
        }
        println!("   - Auth token generation and verification: ✅ WORKING");
    } else if auth_failures == 0 {
        println!("⚠️  VERDICT: Auth tokens accepted but requests failed");
        println!("   - All auth tokens accepted (0 auth failures)");
        println!("   - But {} requests failed for other reasons", other_errors);
        println!("   - Auth is working, but endpoint may have issues");
    } else {
        println!("❌ VERDICT: Auth is NOT working");
        println!("   - {} auth token(s) were REJECTED (401 errors)", auth_failures);
        println!("   - This indicates a problem with auth token generation");
        println!("   - Failure rate: {:.1}%", (auth_failures as f64 / total as f64) * 100.0);
    }
    println!("{}", "=".repeat(80));
    
    Ok(())
}

