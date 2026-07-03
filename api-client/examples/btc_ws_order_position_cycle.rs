use api_client::{CreateOrderRequest, LighterClient, OrderBook, WebSocketClient, WsAccountMessage};
use api_client::websocket::WsMessage;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

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
    order_books
        .iter()
        .find(|book| {
            book.symbol
                .as_deref()
                .map(|symbol| symbol.to_uppercase().starts_with("BTC"))
                .unwrap_or(false)
        })
        .or_else(|| {
            order_books.iter().find(|book| {
                book.symbol
                    .as_deref()
                    .map(|symbol| symbol.to_uppercase().contains("BTC"))
                    .unwrap_or(false)
            })
        })
}

fn raw_account_json(message: &WsAccountMessage) -> Value {
    serde_json::to_value(message).unwrap_or(Value::Null)
}

fn json_find_position_size(value: &Value, market_index: u32, size_decimals: u32) -> Option<i64> {
    match value {
        Value::Object(map) => {
            let market_matches = map
                .get("market_index")
                .or_else(|| map.get("market_id"))
                .and_then(|current| current.as_u64())
                .map(|current| current == market_index as u64)
                .unwrap_or(false);

            if market_matches {
                if let Some(base_amount) = map
                    .get("base_amount")
                    .or_else(|| map.get("position"))
                    .and_then(|current| current.as_str())
                {
                    return Some(parse_fixed(Some(base_amount), size_decimals));
                }
            }

            map.values()
                .find_map(|child| json_find_position_size(child, market_index, size_decimals))
        }
        Value::Array(values) => values
            .iter()
            .find_map(|child| json_find_position_size(child, market_index, size_decimals)),
        _ => None,
    }
}

fn json_contains_client_order_index(value: &Value, target: u64) -> bool {
    match value {
        Value::Object(map) => {
            if map
                .get("client_order_index")
                .and_then(|v| v.as_u64())
                .map(|current| current == target)
                .unwrap_or(false)
            {
                return true;
            }

            if map
                .get("clientOrderIndex")
                .and_then(|v| v.as_u64())
                .map(|current| current == target)
                .unwrap_or(false)
            {
                return true;
            }

            map.values().any(|child| json_contains_client_order_index(child, target))
        }
        Value::Array(values) => values
            .iter()
            .any(|child| json_contains_client_order_index(child, target)),
        Value::String(value) => value
            .parse::<u64>()
            .map(|current| current == target)
            .unwrap_or(false),
        Value::Number(value) => value.as_u64().map(|current| current == target).unwrap_or(false),
        _ => false,
    }
}

async fn watch_ws_updates(
    rx: &mut mpsc::UnboundedReceiver<WsMessage>,
    duration: Duration,
    label: &str,
    market_index: u32,
    size_decimals: u32,
    tracked_client_order_index: Option<u64>,
) -> Result<(Option<i64>, Option<i64>), Box<dyn Error>> {
    let deadline = Instant::now() + duration;
    let tracked_order_index: Option<i64> = None;
    let mut tracked_position_size: Option<i64> = None;

    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(250), rx.recv()).await {
            Ok(Some(message)) => match message {
                WsMessage::Connected(data) => {
                    println!("[{label}] connected session_id={:?}", data.session_id);
                }
                WsMessage::OrderBook(data) => {
                    if data.market_id as u32 == market_index {
                        let order_book = serde_json::to_value(&data.order_book).unwrap_or_default();
                        let best_bid = order_book
                            .get("bids")
                            .and_then(|levels| levels.as_array())
                            .and_then(|levels| levels.first())
                            .and_then(|level| level.get("price"))
                            .and_then(|price| price.as_str())
                            .map(|price| price.to_string());
                        let best_ask = order_book
                            .get("asks")
                            .and_then(|levels| levels.as_array())
                            .and_then(|levels| levels.first())
                            .and_then(|level| level.get("price"))
                            .and_then(|price| price.as_str())
                            .map(|price| price.to_string());
                        let nonce = order_book
                            .get("nonce")
                            .and_then(|value| value.as_i64())
                            .unwrap_or_default();
                        let begin_nonce = order_book
                            .get("begin_nonce")
                            .and_then(|value| value.as_i64())
                            .unwrap_or_default();
                        println!(
                            "[{label}] order_book market={} best_bid={:?} best_ask={:?} nonce={} begin_nonce={}",
                            data.market_id,
                            best_bid,
                            best_ask,
                            nonce,
                            begin_nonce,
                        );
                    }
                }
                WsMessage::Account(data) => {
                    let raw = raw_account_json(&data);

                    println!(
                        "[{label}] account_id={} raw_keys={}",
                        data.account_id,
                        raw.as_object().map(|map| map.len()).unwrap_or_default(),
                    );

                    if let Some(order_id) = tracked_client_order_index {
                        if json_contains_client_order_index(&raw, order_id) {
                            println!("[{label}] tracked order id {} is present in the websocket payload", order_id);
                        } else {
                            println!("[{label}] tracked order id {} not yet visible in websocket payload", order_id);
                        }
                    }

                    if let Some(position_size) = json_find_position_size(&raw, market_index, size_decimals) {
                        tracked_position_size = Some(position_size);
                        println!("[{label}] BTC position found in raw payload => base_amount={}", position_size);
                    }

                    println!("[{label}] raw account payload:");
                    println!("{}", serde_json::to_string_pretty(&raw)?);
                }
                WsMessage::AccountAssets(data) => {
                    println!(
                        "[{label}] account_assets account_id={} asset_count={}",
                        data.account_id,
                        data.assets.len(),
                    );
                    if let Some(usdc) = data.assets.get("USDC") {
                        println!(
                            "[{label}] USDC => available={:?} total={:?} locked={:?}",
                            usdc.available_balance,
                            usdc.total_balance,
                            usdc.locked_balance,
                        );
                    }
                }
                WsMessage::AccountAllOrders(data) => {
                    let total_orders = data.orders.values().map(|orders| orders.len()).sum::<usize>();
                    println!(
                        "[{label}] account_all_orders account_id={} groups={} total_orders={}",
                        data.account,
                        data.orders.len(),
                        total_orders,
                    );
                }
                WsMessage::Ping => {
                    println!("[{label}] ping");
                }
                WsMessage::Unknown(raw) => {
                    println!("[{label}] unknown websocket message:");
                    println!("{}", serde_json::to_string_pretty(&raw)?);
                }
                WsMessage::Error(err) => {
                    println!("[{label}] websocket error: {}", err);
                }
                WsMessage::OrderUpdate(data) => {
                    println!("[{label}] legacy order_update: {}", serde_json::to_string(&data)?);
                }
                WsMessage::MarketData(data) => {
                    println!("[{label}] legacy market_data: {}", serde_json::to_string(&data)?);
                }
                WsMessage::PositionUpdate(data) => {
                    println!("[{label}] legacy position_update: {}", serde_json::to_string(&data)?);
                }
                WsMessage::Trade(data) => {
                    println!("[{label}] legacy trade: {}", serde_json::to_string(&data)?);
                }
            },
            Ok(None) => break,
            Err(_) => {}
        }
    }

    Ok((tracked_order_index, tracked_position_size))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "═".repeat(80));
    println!("BTC WEBSOCKET ORDER + POSITION CYCLE");
    println!("{}", "═".repeat(80));
    println!();

    load_env_file();
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let order_watch_seconds = env::var("ORDER_WATCH_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2);
    let position_watch_seconds = env::var("POSITION_WATCH_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5);
    let requested_size = env::var("BTC_ORDER_SIZE")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0002_f64);

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Order Watch Seconds: {}", order_watch_seconds);
    println!("  Position Watch Seconds: {}", position_watch_seconds);
    println!("  Requested Size: {:.4}", requested_size);
    println!();

    let rest_client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;
    rest_client.check_api_key().await?;
    let account = rest_client.get_my_account().await?;

    let order_books = rest_client.get_order_books().await?;
    let btc_book = find_btc_market(&order_books)
        .ok_or_else(|| "could not find a BTC market in the order book list".to_string())?;
    let market_index = btc_book.market_index;
    let details = rest_client.get_order_book_details(market_index).await?;
    let book = rest_client.get_order_book(market_index).await?;

    let size_decimals = details.size_decimals.unwrap_or(5) as u32;
    let price_decimals = details.price_decimals.unwrap_or(1) as u32;
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
    let best_ask = book
        .asks
        .first()
        .map(|level| parse_fixed(Some(&level.price), price_decimals))
        .unwrap_or_else(|| {
            ((details.last_trade_price.unwrap_or(1.0) * 1.001) * price_scale).round() as i64
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
    let resting_limit_price = best_bid.saturating_sub(1).min(margin_price_cap.max(1));
    let open_market_price = ((best_ask as f64) * 1.02).ceil() as i64;
    let close_market_price = ((best_bid * 98) / 100).max(1);

    println!("Selected market:");
    println!("  Symbol: {:?}", btc_book.symbol);
    println!("  Market Index: {}", market_index);
    println!("  Size Decimals: {}", size_decimals);
    println!("  Price Decimals: {}", price_decimals);
    println!("  Available Balance: {:?}", account.available_balance);
    println!("  Best Bid: {}", best_bid);
    println!("  Best Ask: {}", best_ask);
    println!("  Resting Limit Price: {}", resting_limit_price);
    println!("  Open Market Price: {}", open_market_price);
    println!("  Base Amount (smallest units): {}", base_amount);
    println!("  Expected Notional: {:.8}", (base_amount as f64 / size_scale as f64) * (resting_limit_price as f64 / price_scale));
    println!();

    let auth_token = rest_client.create_auth_token(3600)?;
    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        + "/stream";
    let ws_client = WebSocketClient::new(ws_url, Some(auth_token));
    let mut rx = ws_client.connect().await?;
    ws_client.subscribe_order_book(market_index).await?;
    ws_client.subscribe_account_all(account_index).await?;
    ws_client.subscribe_account_all_assets(account_index).await?;

    let order_client_order_id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    let limit_order = CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index: order_client_order_id,
        base_amount,
        price: resting_limit_price,
        is_ask: false,
        order_type: 0,
        time_in_force: 1,
        reduce_only: false,
        trigger_price: 0,
        order_expiry: 0,
    };

    println!("Submitting resting BTC limit order...");
    let limit_response = rest_client.create_order(limit_order).await?;
    println!("Limit order response:");
    println!("{}", serde_json::to_string_pretty(&limit_response)?);

    let (tracked_order_index, _) = watch_ws_updates(
        &mut rx,
        Duration::from_secs(order_watch_seconds),
        "limit-order",
        market_index,
        size_decimals,
        Some(order_client_order_id),
    )
    .await?;

    let order_index_to_cancel = if let Some(order_index) = tracked_order_index {
        order_index
    } else {
        let active_orders = rest_client
            .get_account_active_orders(account_index, Some(market_index), Some(100), None)
            .await?
            .items;
        active_orders
            .iter()
            .find(|order| order.client_order_index == Some(order_client_order_id))
            .map(|order| order.order_index)
            .ok_or_else(|| {
                "rest fallback could not find the limit order by client_order_index".to_string()
            })?
    };

    println!("Canceling resting limit order...");
    let limit_cancel_response = rest_client
        .cancel_order(market_index as u8, order_index_to_cancel)
        .await?;
    println!("Limit cancel response:");
    println!("{}", serde_json::to_string_pretty(&limit_cancel_response)?);

    let _ = watch_ws_updates(
        &mut rx,
        Duration::from_secs(2),
        "limit-cancel",
        market_index,
        size_decimals,
        Some(order_client_order_id),
    )
    .await?;

    let open_client_order_id = order_client_order_id + 1;
    println!("Opening BTC long position with a market order...");
    let open_market_response = rest_client
        .create_market_order(
            market_index as u8,
            open_client_order_id,
            base_amount,
            open_market_price,
            false,
        )
        .await?;
    println!("Open market response:");
    println!("{}", serde_json::to_string_pretty(&open_market_response)?);

    let (_, position_after_open_ws) = watch_ws_updates(
        &mut rx,
        Duration::from_secs(position_watch_seconds),
        "position-open",
        market_index,
        size_decimals,
        Some(open_client_order_id),
    )
    .await?;

    let account_after_open = rest_client.get_my_account().await?;
    let position_after_open = position_after_open_ws.or_else(|| {
        account_after_open
            .positions
            .unwrap_or_default()
            .into_iter()
            .find(|position| position.market_index == market_index)
            .map(|position| parse_fixed(position.base_amount.as_deref(), size_decimals))
    });

    let position_after_open = position_after_open
        .ok_or_else(|| "could not determine the BTC position after opening".to_string())?;

    if position_after_open == 0 {
        return Err("BTC position did not open; aborting cleanup to avoid guessing".into());
    }

    let close_amount = position_after_open.abs().max(min_base_amount_scaled);
    let close_client_order_id = open_client_order_id + 1;
    let close_side_is_ask = position_after_open > 0;

    println!(
        "Closing BTC position delta with reduce-only market order (position={}, close_amount={})...",
        position_after_open, close_amount
    );
    let close_market_response = rest_client
        .create_market_order(
            market_index as u8,
            close_client_order_id,
            close_amount,
            close_market_price,
            close_side_is_ask,
        )
        .await?;
    println!("Close market response:");
    println!("{}", serde_json::to_string_pretty(&close_market_response)?);

    let (_, position_after_close_ws) = watch_ws_updates(
        &mut rx,
        Duration::from_secs(position_watch_seconds),
        "position-close",
        market_index,
        size_decimals,
        Some(close_client_order_id),
    )
    .await?;

    let account_after_close = rest_client.get_my_account().await?;
    let remaining_position = position_after_close_ws.or_else(|| {
        account_after_close
            .positions
            .unwrap_or_default()
            .into_iter()
            .find(|position| position.market_index == market_index)
            .map(|position| parse_fixed(position.base_amount.as_deref(), size_decimals))
    });

    println!("Final BTC position size: {:?}", remaining_position.unwrap_or(0));
    println!("Final open orders:");
    let final_open_orders = rest_client
        .get_account_active_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    println!("  count={}", final_open_orders.items.len());

    println!("{}", "═".repeat(80));
    println!("Websocket order and position cycle complete.");
    println!("{}", "═".repeat(80));

    Ok(())
}