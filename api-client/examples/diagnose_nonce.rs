use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("API_KEY").unwrap_or_else(|_| {
        "4f9d6b5c3e4a2f1b8d7c6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a".to_string()
    });
    let base_url = env::var("API_URL").unwrap_or_else(|_| {
        "https://testnet.lighter.ai".to_string()
    });
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1081);
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    
    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();
    
    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;
    
    // Try creating a simple market order to see the actual error
    println!("Attempting to create a market order...");
    println!("(This should reveal the server's nonce state)");
    println!();
    
    match client.create_market_order(0, 9999, 1000, 350000, false).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            let msg = response["message"].as_str().unwrap_or("no message");
            println!("Response code: {}", code);
            println!("Response message: {}", msg);
            println!();
            
            if code == 200 {
                println!("✅ Order succeeded!");
            } else if code == 21120 {
                println!("❌ Invalid signature error");
                println!("   This suggests signature validation is working but output doesn't match");
            } else if code == 21104 {
                println!("❌ Invalid nonce error");
                println!("   This suggests the nonce on the server is different");
                println!("   Try with DEBUG_TX_JSON=1 to see actual nonce being used");
            } else if code == 23000 {
                println!("⚠️ Rate limited - quota exceeded");
                println!("   This is expected if many requests were recently sent");
            } else {
                println!("Response: {:#}", response);
            }
        }
        Err(e) => {
            println!("❌ Transport error: {}", e);
        }
    }
    
    Ok(())
}
