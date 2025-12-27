use api_client::{LighterClient, CreateOrderRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 SEQUENTIAL ORDER SUBMISSION WITH NONCE MANAGEMENT");
    println!("{}", "═".repeat(80));
    println!();

    // Load .env file manually
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut env_file = current_dir.join(".env");
    if !env_file.exists() {
        env_file = current_dir.parent()
            .map(|p| p.join(".env"))
            .unwrap_or_else(|| current_dir.join(".env"));
    }
    if !env_file.exists() {
        env_file = current_dir.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join(".env"))
            .unwrap_or_else(|| current_dir.join(".env"));
    }
    
    if env_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("--") {
                    continue;
                }
                if let Some(equal_pos) = line.find('=') {
                    let key = line[..equal_pos].trim();
                    let mut value = line[equal_pos + 1..].trim();
                    value = value.trim_matches('"').trim_matches('\'');
                    if value.starts_with("0x") || value.starts_with("0X") {
                        value = &value[2..];
                    }
                    if !key.is_empty() && !value.is_empty() {
                        std::env::set_var(key, value);
                    }
                }
            }
        }
    }

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let mut api_key = env::var("API_PRIVATE_KEY")?;
    
    // Clean private key
    api_key = api_key.trim().to_string();
    api_key = api_key.replace(" ", "").replace("\n", "").replace("\r", "").replace("\t", "");
    if api_key.starts_with("0x") || api_key.starts_with("0X") {
        api_key = api_key[2..].to_string();
    }
    let hex_only: String = api_key.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(80)
        .collect();

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url.clone(), &hex_only, account_index, api_key_index)?;

    // Get initial nonce
    let initial_nonce = client.get_nonce_or_use(None).await?;
    let mut current_nonce = initial_nonce;

    // Note: Batch transactions require signing orders manually
    // For now, we'll submit orders sequentially as a demonstration
    // Full batch support would require a sign_create_order method that returns signed JSON
    
    println!("📝 Creating first order (ASK)...");
    let ask_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 1001,
        base_amount: 100000,
        price: 280000,
        is_ask: true,
        order_type: 0, // LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 0,
    };

    let ask_response = client.create_order_with_nonce(ask_order, Some(current_nonce)).await?;
    current_nonce += 1;
    println!("✅ First order submitted");
    println!("  Response: {}", serde_json::to_string_pretty(&ask_response)?);

    println!("\n📝 Creating second order (BID)...");
    let bid_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 1002,
        base_amount: 200000,
        price: 200000,
        is_ask: false,
        order_type: 0, // LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 0,
    };

    let bid_response = client.create_order_with_nonce(bid_order, Some(current_nonce)).await?;
    println!("✅ Second order submitted");
    println!("  Response: {}", serde_json::to_string_pretty(&bid_response)?);

    println!("\n📊 Summary:");
    println!("  Both orders submitted sequentially with manual nonce management");
    println!("  Note: True batch transactions require signing orders without submitting,");
    println!("  then sending the signed transactions together via sendTxBatch endpoint");

    Ok(())
}

