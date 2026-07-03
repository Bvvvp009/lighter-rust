use api_client::{CreateOrderRequest, LighterClient};
use std::env;
use std::error::Error;

fn parse_i64_like(value: Option<&str>) -> i64 {
    let raw = value.unwrap_or("0").trim();
    raw.parse::<i64>()
        .or_else(|_| raw.parse::<f64>().map(|parsed| parsed.round() as i64))
        .unwrap_or(0)
}

fn parse_fixed(value: Option<&str>, decimals: u32) -> i64 {
    let raw = value.unwrap_or("0").trim();
    let factor = 10_i64.saturating_pow(decimals);
    raw.parse::<f64>()
        .map(|parsed| (parsed * factor as f64).round() as i64)
        .unwrap_or_else(|_| parse_i64_like(Some(raw)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 SEQUENTIAL ORDER SUBMISSION WITH NONCE MANAGEMENT");
    println!("{}", "═".repeat(80));
    println!();

    // Load .env file manually
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut env_file = current_dir.join(".env");
    if !env_file.exists() {
        env_file = current_dir
            .parent()
            .map(|p| p.join(".env"))
            .unwrap_or_else(|| current_dir.join(".env"));
    }
    if !env_file.exists() {
        env_file = current_dir
            .parent()
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
                    if !key.is_empty() && !value.is_empty() && std::env::var_os(key).is_none() {
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
    let market_index: u8 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3);

    // Clean private key
    api_key = api_key.trim().to_string();
    api_key = api_key
        .replace(" ", "")
        .replace("\n", "")
        .replace("\r", "")
        .replace("\t", "");
    if api_key.starts_with("0x") || api_key.starts_with("0X") {
        api_key = api_key[2..].to_string();
    }
    let api_key = api_key
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_string();
    let hex_only: String = api_key
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(80)
        .collect();

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Market Index: {}", market_index);
    println!();

    let client = LighterClient::new(base_url.clone(), &hex_only, account_index, api_key_index)?;

    let details = client.get_order_book_details(market_index as u32).await?;
    let book = client.get_order_book(market_index as u32).await?;
    let size_decimals = details.size_decimals.unwrap_or(4) as u32;
    let price_decimals = details.price_decimals.unwrap_or(2) as u32;
    let size_scale = 10_i64.saturating_pow(size_decimals);
    let price_scale = 10_i64.saturating_pow(price_decimals) as f64;

    let current_bid = book
        .bids
        .first()
        .map(|level| parse_fixed(Some(&level.price), price_decimals))
        .unwrap_or_else(|| {
            ((details.last_trade_price.unwrap_or(1.0) * 0.999) * price_scale).round() as i64
        });
    let current_ask = book
        .asks
        .first()
        .map(|level| parse_fixed(Some(&level.price), price_decimals))
        .unwrap_or_else(|| {
            ((details.last_trade_price.unwrap_or(1.0) * 1.001) * price_scale).round() as i64
        });
    let current_ask_real = current_ask as f64 / price_scale;
    let min_base_amount_real = details
        .min_base_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let min_quote_amount = details
        .min_quote_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let min_base_amount = ((min_base_amount_real * size_scale as f64).ceil() as i64).max(1);
    let target_notional = (min_quote_amount * 1.05).max(min_quote_amount + 0.25);
    let required_base_real = (target_notional / current_ask_real).max(min_base_amount_real);
    let order_base_amount = ((required_base_real * size_scale as f64).ceil() as i64).max(min_base_amount);
    let ask_price = ((current_ask * 101) / 100).max(current_ask + 1);
    let bid_price = ((current_bid * 99) / 100).max(1);

    // Get initial nonce
    let initial_nonce = client.get_nonce_or_use(None).await?;
    let mut current_nonce = initial_nonce;

    // Note: Batch transactions require signing orders manually
    // For now, we'll submit orders sequentially as a demonstration
    // Full batch support would require a sign_create_order method that returns signed JSON

    println!("📝 Creating first order (ASK)...");
    let ask_order = CreateOrderRequest {
        account_index,
        order_book_index: market_index,
        client_order_index: 1001,
        base_amount: order_base_amount,
        price: ask_price,
        is_ask: true,
        order_type: 0,    // LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 0,
        order_expiry: 0,
    };

    let ask_response = client
        .create_order_with_nonce(ask_order, Some(current_nonce))
        .await?;
    current_nonce += 1;
    println!("✅ First order submitted");
    println!(
        "  Response: {}",
        serde_json::to_string_pretty(&ask_response)?
    );

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let cleanup_client = LighterClient::new(base_url.clone(), &hex_only, account_index, api_key_index)?;
    if let Some(order) = cleanup_client
        .get_account_active_orders(account_index, Some(market_index as u32), Some(50), None)
        .await?
        .items
        .into_iter()
        .find(|order| order.client_order_index == Some(1001))
    {
        let cancel_response = cleanup_client.cancel_order(market_index, order.order_index).await?;
        println!("✅ First order canceled: {}", cancel_response);
    }
    current_nonce += 1;

    println!("\n📝 Creating second order (BID)...");
    let bid_order = CreateOrderRequest {
        account_index,
        order_book_index: market_index,
        client_order_index: 1002,
        base_amount: order_base_amount,
        price: bid_price,
        is_ask: false,
        order_type: 0,    // LIMIT
        time_in_force: 1, // GOOD_TILL_TIME
        reduce_only: false,
        trigger_price: 0,
        order_expiry: 0,
    };

    let bid_response = client
        .create_order_with_nonce(bid_order, Some(current_nonce))
        .await?;
    println!("✅ Second order submitted");
    println!(
        "  Response: {}",
        serde_json::to_string_pretty(&bid_response)?
    );

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let cleanup_client = LighterClient::new(base_url.clone(), &hex_only, account_index, api_key_index)?;
    if let Some(order) = cleanup_client
        .get_account_active_orders(account_index, Some(market_index as u32), Some(50), None)
        .await?
        .items
        .into_iter()
        .find(|order| order.client_order_index == Some(1002))
    {
        let cancel_response = cleanup_client.cancel_order(market_index, order.order_index).await?;
        println!("✅ Second order canceled: {}", cancel_response);
    }

    println!("\n📊 Summary:");
    println!("  Both orders submitted sequentially with manual nonce management");
    println!("  Note: True batch transactions require signing orders without submitting,");
    println!("  then sending the signed transactions together via sendTxBatch endpoint");

    Ok(())
}
