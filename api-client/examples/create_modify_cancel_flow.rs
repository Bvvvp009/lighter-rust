use api_client::{CreateOrderRequest, LighterClient, ModifyOrderRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn parse_i64_like(value: Option<&str>) -> i64 {
    let raw = value.unwrap_or("0").trim();
    raw.parse::<i64>()
        .or_else(|_| raw.parse::<f64>().map(|v| v.round() as i64))
        .unwrap_or(0)
}

fn parse_fixed(value: Option<&str>, decimals: u32) -> i64 {
    let raw = value.unwrap_or("0").trim();
    let factor = 10_i64.saturating_pow(decimals);
    raw.parse::<f64>()
        .map(|v| (v * factor as f64).round() as i64)
        .unwrap_or_else(|_| parse_i64_like(Some(raw)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> CREATE -> MODIFY -> CANCEL ORDER FLOW");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let market_index: u32 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Market Index: {}", market_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    client.check_api_key().await?;

    let account = client.get_my_account().await?;
    let details = client.get_order_book_details(market_index).await?;
    let book = client.get_order_book(market_index).await?;

    let size_decimals = details.size_decimals.unwrap_or(4) as u32;
    let price_decimals = details.price_decimals.unwrap_or(2) as u32;
    let size_scale = 10_i64.saturating_pow(size_decimals);
    let price_scale = 10_i64.saturating_pow(price_decimals) as f64;

    let usdc_balance = account
        .available_balance
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let leverage_multiplier: f64 = env::var("LIVE_PRECHECK_LEVERAGE_MULTIPLIER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3.0);
    let force_live_attempt = env::var("FORCE_LIVE_ORDER_ATTEMPT")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let max_notional_at_leverage = usdc_balance * leverage_multiplier;
    let min_quote_amount = details
        .min_quote_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let min_base_amount_real = details
        .min_base_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    if max_notional_at_leverage < min_quote_amount {
        if force_live_attempt {
            println!(
                "⚠️  Proceeding with forced live attempt despite conservative collateral precheck (max notional at {}x = {} < minimum quote = {})",
                leverage_multiplier, max_notional_at_leverage, min_quote_amount
            );
        } else {
            println!(
                "⚠️  Skipping live create/modify/cancel: insufficient available collateral (max notional at {}x = {} < minimum quote = {})",
                leverage_multiplier, max_notional_at_leverage, min_quote_amount
            );
            return Ok(());
        }
    }

    let current_ask = book
        .asks
        .first()
        .map(|p| parse_fixed(Some(&p.price), price_decimals))
        .unwrap_or_else(|| ((details.last_trade_price.unwrap_or(1.0) * 1.001) * price_scale).round() as i64);
    let current_ask_real = current_ask as f64 / price_scale;
    let min_base_amount = ((min_base_amount_real * size_scale as f64).ceil() as i64).max(1);
    let target_notional = (min_quote_amount * 1.05).max(min_quote_amount + 0.25);
    let required_base_real = (target_notional / current_ask_real).max(min_base_amount_real);
    let base_amount = ((required_base_real * size_scale as f64).ceil() as i64).max(min_base_amount);

    let limit_price = ((current_ask * 105) / 100).max(current_ask + 1);
    let modify_price = ((current_ask * 110) / 100).max(limit_price + 1);
    let client_order_index = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

    println!("STEP 1 - CREATE LIMIT ORDER");
    println!("{}", "-".repeat(80));
    let order = CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index,
        base_amount,
        price: limit_price,
        is_ask: true,
        order_type: 0,
        time_in_force: 1,
        reduce_only: false,
        trigger_price: 0,
        order_expiry: 0,
    };

    let create_response = client.create_order(order).await?;
    println!("{}", serde_json::to_string_pretty(&create_response)?);
    println!();

    let code = create_response["code"].as_i64().unwrap_or_default();
    if code != 200 {
        println!("❌ Failed to create order. Aborting...");
        return Ok(());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let active_orders = client
        .get_account_active_orders(account_index, Some(market_index), Some(50), None)
        .await?
        .items;
    let created_order = match active_orders
        .iter()
        .find(|o| o.client_order_index == Some(client_order_index))
    {
        Some(order) => order,
        None => {
            println!("⚠️  Created order not found in active orders; stopping before modify/cancel.");
            return Ok(());
        }
    };

    println!("STEP 2 - MODIFY ORDER");
    println!("{}", "-".repeat(80));
    let modify_request = ModifyOrderRequest {
        market_index: market_index as u8,
        order_index: created_order.order_index,
        base_amount: base_amount + min_base_amount,
        price: modify_price.try_into().unwrap_or(u32::MAX),
        trigger_price: 0,
    };

    let modify_response = client.modify_order(modify_request).await?;
    println!("{}", serde_json::to_string_pretty(&modify_response)?);
    println!();

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("STEP 3 - CANCEL ORDER");
    println!("{}", "-".repeat(80));
    let cancel_response = client
        .cancel_order(market_index as u8, created_order.order_index)
        .await?;
    println!("{}", serde_json::to_string_pretty(&cancel_response)?);
    println!();

    let code = cancel_response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Complete flow executed successfully!");
        println!("   Created → Modified → Cancelled");
    } else {
        println!("⚠️  Final cancel returned code: {}", code);
    }

    Ok(())
}
