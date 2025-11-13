use api_client::LighterClient;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 SENDING 10 ORDERS");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .map_err(|_| "ACCOUNT_INDEX not found in environment variables")?
        .parse()
        .map_err(|_| "ACCOUNT_INDEX must be a valid integer")?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .map_err(|_| "API_KEY_INDEX not found in environment variables")?
        .parse()
        .map_err(|_| "API_KEY_INDEX must be a valid integer")?;
    let api_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY not found in environment variables")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut error_messages = Vec::new();

    let total_orders = 10;
    
    for i in 1..=total_orders {
        let client_order_index = (now_ms / 1000) as u64 * 1000 + i as u64;

        match client.create_market_order(
            0,
            client_order_index,
            1000,
            50000_0000,
            false,
        ).await {
            Ok(response) => {
                let code = response["code"].as_i64().unwrap_or_default();
                
                if code == 200 {
                    success_count += 1;
                    println!("  ✅ [{}] SUCCESS", i);
                } else {
                    fail_count += 1;
                    let msg = response["message"]
                        .as_str()
                        .unwrap_or("Unknown error")
                        .to_string();
                    println!("  ❌ [{}] FAILED - Code: {} - {}", i, code, msg);
                    error_messages.push(format!("Order {}: Code {} - {}", i, code, msg));
                }
            }
            Err(e) => {
                fail_count += 1;
                let error_msg = format!("Order {}: {}", i, e);
                error_messages.push(error_msg.clone());
                println!("  ❌ [{}] ERROR: {}", i, e);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("{}", "═".repeat(80));
    println!("📊 RESULTS");
    println!("{}", "═".repeat(80));
    println!("✅ Successful: {}/{}", success_count, total_orders);
    println!("❌ Failed: {}/{}", fail_count, total_orders);
    
    if !error_messages.is_empty() {
        println!();
        println!("Error Details:");
        for msg in error_messages.iter() {
            println!("  - {}", msg);
        }
    }

    if fail_count == 0 {
        Ok(())
    } else {
        Err("Some orders failed".into())
    }
}
