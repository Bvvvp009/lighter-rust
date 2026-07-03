use api_client::{CreateOrderRequest, LighterClient, OrderBook};
use std::env;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

fn load_env_file() {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut candidates = vec![current_dir.join(".env")];

    if let Some(parent) = current_dir.parent() {
        candidates.push(parent.join(".env"));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(".env"));
        }
    }

    for env_file in candidates {
        if !env_file.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&env_file) {
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

        break;
    }
}

fn find_btc_market(order_books: &[OrderBook]) -> Option<&OrderBook> {
    order_books.iter().find(|book| {
        book.symbol
            .as_deref()
            .map(|symbol| symbol.to_uppercase().starts_with("BTC"))
            .unwrap_or(false)
    }).or_else(|| {
        order_books.iter().find(|book| {
            book.symbol
                .as_deref()
                .map(|symbol| symbol.to_uppercase().contains("BTC"))
                .unwrap_or(false)
        })
    })
}

fn order_summary(order: &serde_json::Value) -> String {
    serde_json::to_string_pretty(order).unwrap_or_else(|_| order.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "═".repeat(80));
    println!("BTC LIMIT ORDER WATCH + CLOSE");
    println!("{}", "═".repeat(80));
    println!();

    load_env_file();
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let requested_size = 0.0002_f64;

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Requested side: long / buy");
    println!("  Requested size: {:.4}", requested_size);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    client.check_api_key().await?;
    let account = client.get_my_account().await?;

    let order_books = client.get_order_books().await?;
    let market_override = env::var("BTC_MARKET_INDEX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());

    let selected_book = if let Some(market_index) = market_override {
        let book = order_books
            .iter()
            .find(|book| book.market_index == market_index)
            .ok_or_else(|| format!("market {} not found in order book list", market_index))?;
        book
    } else {
        find_btc_market(&order_books).ok_or_else(|| {
            let preview = order_books
                .iter()
                .take(15)
                .map(|book| format!("{}:{:?}", book.market_index, book.symbol))
                .collect::<Vec<_>>()
                .join(", ");
            format!("could not find a BTC market in the order book list; preview: {}", preview)
        })?
    };

    let market_index = selected_book.market_index;
    let details = client.get_order_book_details(market_index).await?;
    let book = client.get_order_book(market_index).await?;
    let size_decimals = details.size_decimals.unwrap_or(4) as u32;
    let price_decimals = details.price_decimals.unwrap_or(2) as u32;
    let size_scale = 10_i64.saturating_pow(size_decimals);
    let price_scale = 10_i64.saturating_pow(price_decimals) as f64;
    let requested_base_amount = (requested_size * size_scale as f64).ceil() as i64;
    let min_base_amount = details
        .min_base_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let min_base_amount_scaled = ((min_base_amount * size_scale as f64).ceil() as i64).max(1);
    let base_amount = requested_base_amount.max(min_base_amount_scaled);

    let best_bid = book
        .bids
        .first()
        .map(|level| parse_fixed(Some(&level.price), price_decimals))
        .unwrap_or_else(|| {
            ((details.last_trade_price.unwrap_or(1.0) * 0.999) * price_scale).round() as i64
        });
    let min_quote_amount = details
        .min_quote_amount
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let available_balance_real = account
        .available_balance
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let target_notional = (available_balance_real * 0.85).max(min_quote_amount * 1.05);
    let margin_price_cap = ((target_notional / (base_amount as f64 / size_scale as f64)) * price_scale)
        .floor() as i64;
    let limit_price = best_bid
        .saturating_sub(1)
        .min(margin_price_cap.max(1));

    println!("Selected market:");
    println!("  Symbol: {:?}", selected_book.symbol);
    println!("  Market Index: {}", market_index);
    println!("  Details Symbol: {:?}", details.symbol);
    println!("  Size Decimals: {}", size_decimals);
    println!("  Price Decimals: {}", price_decimals);
    println!("  Min Base Amount: {:?}", details.min_base_amount);
    println!("  Min Quote Amount: {:?}", details.min_quote_amount);
    println!("  Available Balance: {:?}", account.available_balance);
    println!("  Best Bid: {}", best_bid);
    println!("  Margin-Capped Limit Price: {}", limit_price);
    println!("  Base Amount (smallest units): {}", base_amount);
    println!("  Target Notional: {:.8}", target_notional);
    println!("  Expected Notional: {:.8}", (base_amount as f64 / size_scale as f64) * (limit_price as f64 / price_scale));
    if min_quote_amount > 0.0 {
        println!("  Minimum Quote Requirement: {:.8}", min_quote_amount);
    }
    println!();

    let client_order_index = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    let order = CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index,
        base_amount,
        price: limit_price,
        is_ask: false,
        order_type: 0,
        time_in_force: 1,
        reduce_only: false,
        trigger_price: 0,
    };

    println!("Submitting resting BTC long limit order...");
    let create_response = client.create_order(order).await?;
    println!("Order submission response:");
    println!("{}", serde_json::to_string_pretty(&create_response)?);

    let create_code = create_response["code"].as_i64().unwrap_or_default();
    if create_code != 200 {
        println!("Order was not accepted, stopping early.");
        return Ok(());
    }

    println!();
    println!("Watching live status for 30 seconds...");
    println!();

    let watch_started = Instant::now();
    let mut tracked_order_index: Option<i64> = None;
    loop {
        let elapsed = watch_started.elapsed().as_secs();
        let active_orders = client
            .get_account_active_orders(account_index, Some(market_index), Some(100), None)
            .await?;
        let inactive_orders = client
            .get_account_inactive_orders(account_index, Some(market_index), Some(100), None)
            .await?;

        println!("--- t = {}s ---", elapsed);
        if let Some(order) = active_orders
            .items
            .iter()
            .find(|order| order.client_order_index == Some(client_order_index))
        {
            tracked_order_index = Some(order.order_index);
            println!("Status: active / open");
            println!("Order details:");
            println!("{}", order_summary(&serde_json::to_value(order)?));
        } else if let Some(order) = inactive_orders
            .items
            .iter()
            .find(|order| order.client_order_index == Some(client_order_index))
        {
            tracked_order_index = Some(order.order_index);
            println!("Status: inactive / {:?}", order.status);
            println!("Order details:");
            println!("{}", order_summary(&serde_json::to_value(order)?));
        } else {
            println!("Status: not found in active or inactive order lists");
        }

        if elapsed >= 30 {
            break;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    println!();
    println!("Refreshing nonce before cancel...");
    client.refresh_nonce().await?;

    println!("Closing the open order...");
    let order_index_to_cancel = match tracked_order_index {
        Some(order_index) => order_index,
        None => {
            println!("No tracked order index was captured during the watch period.");
            return Ok(());
        }
    };

    let cancel_response = client
        .cancel_order(market_index as u8, order_index_to_cancel)
        .await;

    let cancel_response = match cancel_response {
        Ok(response) => response,
        Err(err) => {
            println!("Cancel request failed: {}", err);
            return Ok(());
        }
    };

    println!("Cancel response:");
    println!("{}", serde_json::to_string_pretty(&cancel_response)?);

    let cancel_code = cancel_response["code"].as_i64().unwrap_or_default();
    if cancel_code != 200 {
        println!("Cancel order returned code {}", cancel_code);
    }

    let inactive_after_cancel = client
        .get_account_inactive_orders(account_index, Some(market_index), Some(100), None)
        .await?;

    println!();
    println!("Post-cancel status check:");
    if let Some(order) = inactive_after_cancel
        .items
        .iter()
        .find(|order| order.client_order_index == Some(client_order_index))
    {
        println!("Status: inactive / {:?}", order.status);
        println!("Order details:");
        println!("{}", order_summary(&serde_json::to_value(order)?));
    } else {
        println!("Order not found in inactive orders after cancel.");
    }

    Ok(())
}