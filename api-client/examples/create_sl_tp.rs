use api_client::{LighterClient, CreateOrderRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE STOP LOSS & TAKE PROFIT ORDERS EXAMPLE");
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

    let client_order_index_base = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;

    // Create Take Profit Order (Type 4: TAKE_PROFIT)
    println!("📝 Creating Take Profit Order...");
    let tp_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: client_order_index_base + 1,
        base_amount: 1000,
        price: 500000,
        is_ask: false,
        order_type: 4, // TAKE_PROFIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 500000,
    };

    match client.create_order(tp_order).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("✅ Take Profit Order created successfully!");
            } else {
                println!("⚠️  Take Profit Order returned code: {}", code);
                if let Some(msg) = response["message"].as_str() {
                    println!("  Message: {}", msg);
                }
            }
        }
        Err(e) => println!("❌ Error creating Take Profit Order: {}", e),
    }

    // Create Stop Loss Order (Type 2: STOP_LOSS)
    println!("\n📝 Creating Stop Loss Order...");
    let sl_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: client_order_index_base + 2,
        base_amount: 1000,
        price: 500000,
        is_ask: false,
        order_type: 2, // STOP_LOSS
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 500000,
    };

    match client.create_order(sl_order).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("✅ Stop Loss Order created successfully!");
            } else {
                println!("⚠️  Stop Loss Order returned code: {}", code);
                if let Some(msg) = response["message"].as_str() {
                    println!("  Message: {}", msg);
                }
            }
        }
        Err(e) => println!("❌ Error creating Stop Loss Order: {}", e),
    }

    // Create Take Profit Limit Order (Type 5: TAKE_PROFIT_LIMIT)
    println!("\n📝 Creating Take Profit Limit Order...");
    let tp_limit_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: client_order_index_base + 3,
        base_amount: 1000,
        price: 500000,
        is_ask: false,
        order_type: 5, // TAKE_PROFIT_LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 500000,
    };

    match client.create_order(tp_limit_order).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("✅ Take Profit Limit Order created successfully!");
            } else {
                println!("⚠️  Take Profit Limit Order returned code: {}", code);
                if let Some(msg) = response["message"].as_str() {
                    println!("  Message: {}", msg);
                }
            }
        }
        Err(e) => println!("❌ Error creating Take Profit Limit Order: {}", e),
    }

    // Create Stop Loss Limit Order (Type 3: STOP_LOSS_LIMIT)
    println!("\n📝 Creating Stop Loss Limit Order...");
    let sl_limit_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: client_order_index_base + 4,
        base_amount: 1000,
        price: 500000,
        is_ask: false,
        order_type: 3, // STOP_LOSS_LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 500000,
    };

    match client.create_order(sl_limit_order).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("✅ Stop Loss Limit Order created successfully!");
            } else {
                println!("⚠️  Stop Loss Limit Order returned code: {}", code);
                if let Some(msg) = response["message"].as_str() {
                    println!("  Message: {}", msg);
                }
            }
        }
        Err(e) => println!("❌ Error creating Stop Loss Limit Order: {}", e),
    }

    Ok(())
}


