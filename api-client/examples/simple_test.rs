use api_client::{LighterClient, CreateOrderRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust Signer Test ===");
    
    // Load environment variables from .env file
    match dotenv::dotenv() {
        Ok(path) => println!("✅ Loaded .env file from: {:?}", path),
        Err(_) => println!("⚠️  No .env file found, using system environment variables or defaults"),
    }
    
    // Test configuration (use environment variables if available)
    let api_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY not found in environment variables")?;
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
    
    println!("Configuration loaded:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  API Key Length: {} characters", api_key.len());
    
    println!("Creating client...");
    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    
    println!("Creating test order...");
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0, // BTC-USD
        client_order_index: 12345,
        base_amount: 1000, // 0.0001 BTC
        price: 50000_0000, // $50,000
        is_ask: false, // Buy
        order_type: 0, // Market (MarketOrder = 0)
        time_in_force: 0, // ImmediateOrCancel
        reduce_only: false,
        trigger_price: 0,
    };
    
    println!("Submitting order...");
    match client.create_order(order).await {
        Ok(response) => {
            println!("✅ Order submitted successfully!");
            println!("Response: {}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            println!("❌ Order failed: {}", e);
        }
    }
    
    Ok(())
}
