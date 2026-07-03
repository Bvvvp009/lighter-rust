use api_client::{websocket::WsMessage, CreateOrderRequest, LighterClient, Order, OrderBook, WebSocketClient};
use rand::{thread_rng, RngCore};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const ORDER_COUNT: u32 = 10;
const DEFAULT_WATCH_SECONDS: u64 = 2;
const TARGET_CANCEL_CYCLES: [u32; 5] = [1, 3, 6, 8, 10];

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

fn print_order(order: &Order) {
    match serde_json::to_string_pretty(order) {
        Ok(text) => println!("{}", text),
        Err(_) => println!("order_index={} status={:?}", order.order_index, order.status),
    }
}

fn stable_order_id(order: &Order) -> String {
    order.order_index.to_string()
}

fn order_id_matches(order: &Order, target_order_id: &str) -> bool {
    stable_order_id(order) == target_order_id || order.order_index.to_string() == target_order_id
}

fn order_summary(order: &Order) -> String {
    format!(
        "order_id={} order_index={} client_order_index={:?} status={:?} market_index={} base_amount={} price={} is_ask={} reduce_only={} order_type={:?} time_in_force={:?}",
        stable_order_id(order),
        order.order_index,
        order.client_order_index,
        order.status,
        order.market_index,
        order.base_amount,
        order.price,
        order.is_ask,
        order.reduce_only,
        order.order_type,
        order.time_in_force,
    )
}

fn random_client_order_id(existing: &mut HashSet<u64>) -> (u64, String) {
    let mut rng = thread_rng();

    loop {
        let value = rng.next_u64() & ((1u64 << 40) - 1);
        if value == 0 || !existing.insert(value) {
            continue;
        }

        return (value, format!("{:010x}", value));
    }
}

fn parse_watch_seconds() -> u64 {
    env::var("WATCH_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_WATCH_SECONDS)
}

async fn watch_order_updates(
    rx: &mut mpsc::UnboundedReceiver<WsMessage>,
    duration: Duration,
    label: &str,
    market_index: u32,
    tracked_client_order_index: u64,
    tracked_order_id: Option<&str>,
) -> Result<(Option<String>, Option<String>), Box<dyn Error>> {
    let deadline = Instant::now() + duration;
    let mut resolved_order_id = tracked_order_id.map(|value| value.to_string());
    let mut tracked_status: Option<String> = None;

    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(250), rx.recv()).await {
            Ok(Some(message)) => match message {
                WsMessage::Connected(data) => {
                    println!("[{label}] connected session_id={:?}", data.session_id);
                }
                WsMessage::AccountAllOrders(data) => {
                    let total_orders = data.orders.values().map(|orders| orders.len()).sum::<usize>();
                    println!(
                        "[{label}] account_all_orders account={} groups={} total_orders={} nonce={}",
                        data.account,
                        data.orders.len(),
                        total_orders,
                        data.nonce,
                    );

                    let matched_order = data
                        .orders
                        .values()
                        .flat_map(|orders| orders.iter())
                        .find(|order| {
                            order.market_index == market_index
                                && (resolved_order_id
                                    .as_deref()
                                    .map(|target_order_id| order_id_matches(order, target_order_id))
                                    .unwrap_or(false)
                                    || order.client_order_index == Some(tracked_client_order_index))
                        });

                    if let Some(order) = matched_order {
                        resolved_order_id = Some(stable_order_id(order));
                        tracked_status = order.status.clone();
                        println!("[{label}] tracked order => {}", order_summary(order));
                    } else {
                        println!(
                            "[{label}] tracked order not yet visible in websocket payload",
                        );
                    }
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
                    println!("[{label}] account_all account_id={} raw_keys={}", data.account_id, data.extra.len());
                }
                WsMessage::AccountAssets(data) => {
                    println!(
                        "[{label}] account_all_assets account_id={} asset_count={}",
                        data.account_id,
                        data.assets.len(),
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

    Ok((resolved_order_id, tracked_status))
}

async fn find_order_state_by_client_order_index(
    client: &LighterClient,
    account_index: i64,
    market_index: u32,
    client_order_index: u64,
) -> Result<Option<Order>, Box<dyn Error>> {
    let active_orders = client
        .get_account_active_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    if let Some(order) = active_orders
        .items
        .into_iter()
        .find(|order| order.client_order_index == Some(client_order_index))
    {
        return Ok(Some(order));
    }

    let inactive_orders = client
        .get_account_inactive_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    if let Some(order) = inactive_orders
        .items
        .into_iter()
        .find(|order| order.client_order_index == Some(client_order_index))
    {
        return Ok(Some(order));
    }

    Ok(None)
}

async fn find_order_state_by_order_id(
    client: &LighterClient,
    account_index: i64,
    market_index: u32,
    target_order_id: &str,
) -> Result<Option<Order>, Box<dyn Error>> {
    let active_orders = client
        .get_account_active_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    if let Some(order) = active_orders
        .items
        .into_iter()
        .find(|order| order_id_matches(order, target_order_id))
    {
        return Ok(Some(order));
    }

    let inactive_orders = client
        .get_account_inactive_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    if let Some(order) = inactive_orders
        .items
        .into_iter()
        .find(|order| order_id_matches(order, target_order_id))
    {
        return Ok(Some(order));
    }

    Ok(None)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "═".repeat(80));
    println!("BTC RAPID 10 LIMIT CYCLE");
    println!("{}", "═".repeat(80));
    println!();

    load_env_file();
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let watch_seconds = parse_watch_seconds();
    let requested_size = env::var("RAPID_BTC_ORDER_SIZE")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0002_f64);
    let keep_remaining_open_orders = env::var("RAPID_KEEP_REMAINING_OPEN_ORDERS")
        .ok()
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Order Count: {}", ORDER_COUNT);
    println!("  Watch Seconds per Order: {}", watch_seconds);
    println!("  Keep Remaining Open Orders: {}", keep_remaining_open_orders);
    println!("  Requested Side: long / buy");
    println!("  Requested Size: {:.4}", requested_size);
    println!();

    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;
    client.check_api_key().await?;
    let account = client.get_my_account().await?;

    let order_books = client.get_order_books().await?;
    let market_override = env::var("BTC_MARKET_INDEX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());

    let selected_book = if let Some(market_index) = market_override {
        order_books
            .iter()
            .find(|book| book.market_index == market_index)
            .ok_or_else(|| format!("market {} not found in order book list", market_index))?
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
    println!();

    let startup_delay = if watch_seconds <= 2 {
        Duration::from_millis(250)
    } else {
        Duration::from_millis(500)
    };
    println!("Tracking cadence: websocket watch window={} seconds", watch_seconds);
    println!();

    let auth_token = client.create_auth_token(3600)?;
    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        + "/stream";
    let ws_client = WebSocketClient::new(ws_url, Some(auth_token));
    let mut ws_rx = ws_client.connect().await?;
    ws_client.subscribe_account_all_orders(account_index).await?;

    let mut generated_client_order_ids = HashSet::new();
    let mut canceled_order_ids: HashSet<String> = HashSet::new();

    for cycle in 0..ORDER_COUNT {
        let (client_order_index, client_order_hex) = random_client_order_id(&mut generated_client_order_ids);
        let should_cancel = TARGET_CANCEL_CYCLES.contains(&(cycle + 1));
        println!("{}", "-".repeat(80));
        println!("ORDER {}/{}", cycle + 1, ORDER_COUNT);
        println!("  Client Order ID: 0x{}", client_order_hex);
        println!("  Base Amount: {}", base_amount);
        println!("  Limit Price: {}", limit_price);
        println!("  Hold Time: {} seconds", watch_seconds);
        println!();

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
            order_expiry: 0,
        };

        let create_response = client.create_order(order).await?;
        println!("Create response:");
        println!("{}", serde_json::to_string_pretty(&create_response)?);

        let create_code = create_response["code"].as_i64().unwrap_or_default();
        if create_code != 200 {
            println!("Order {} was not accepted; moving on.", cycle + 1);
            println!();
            continue;
        }

        tokio::time::sleep(startup_delay).await;

        let (mut tracked_order_id, mut tracked_status) = watch_order_updates(
            &mut ws_rx,
            Duration::from_secs(watch_seconds),
            "pre-cancel",
            market_index,
            client_order_index,
            None,
        )
        .await?;

        if tracked_order_id.is_none() {
            let fallback_state = find_order_state_by_client_order_index(
                &client,
                account_index,
                market_index,
                client_order_index,
            )
            .await?;
            if let Some(ref order) = fallback_state {
                tracked_order_id = Some(stable_order_id(order));
                tracked_status = order.status.clone();
                println!("[pre-cancel] REST fallback => {}", order_summary(order));
            }
        }

        let resolved_order_id = match tracked_order_id {
            Some(order_id) => order_id,
            None => {
                println!("Could not resolve order_id for client order 0x{}; skipping this cycle.", client_order_hex);
                println!();
                continue;
            }
        };

        if let Some(status) = tracked_status.clone() {
            println!("Tracked websocket status: {}", status);
        }

        if should_cancel {
            let order_index_to_cancel = resolved_order_id
                .parse::<i64>()
                .map_err(|err| format!("failed to parse order_id {} into order_index: {}", resolved_order_id, err))?;

            println!("Canceling selected order by order_id {}...", resolved_order_id);
            let cancel_response = client.cancel_order(market_index as u8, order_index_to_cancel).await?;
            println!("Cancel response:");
            println!("{}", serde_json::to_string_pretty(&cancel_response)?);

            let (post_cancel_order_id, post_cancel_status) = watch_order_updates(
                &mut ws_rx,
                Duration::from_secs(watch_seconds),
                "post-cancel",
                market_index,
                client_order_index,
                Some(resolved_order_id.as_str()),
            )
            .await?;

            if post_cancel_order_id.is_none() {
                let after_cancel = find_order_state_by_order_id(
                    &client,
                    account_index,
                    market_index,
                    &resolved_order_id,
                )
                .await?;
                println!("Post-cancel status:");
                match after_cancel {
                    Some(ref order) => {
                        println!("  status={:?} order_id={}", order.status, stable_order_id(order));
                        print_order(order);
                    }
                    None => {
                        println!("  status=not-found");
                    }
                }
            } else if let Some(status) = post_cancel_status {
                println!("Post-cancel websocket status: {}", status);
            }

            canceled_order_ids.insert(resolved_order_id);
        } else {
            println!("Leaving order {} open for websocket-only status tracking.", cycle + 1);
        }

        println!();
    }

    if keep_remaining_open_orders {
        println!("Leaving remaining open orders on the book for inspection.");
    } else {
        println!("Cleaning up any remaining open orders by order_id...");
        let active_orders = client
            .get_account_active_orders(account_index, Some(market_index), Some(100), None)
            .await?;
        for order in active_orders.items {
            let order_id = stable_order_id(&order);
            if canceled_order_ids.contains(&order_id) {
                continue;
            }

            let order_index = match order_id.parse::<i64>() {
                Ok(value) => value,
                Err(_) => order.order_index,
            };
            let cancel_response = client.cancel_order(market_index as u8, order_index).await?;
            println!("Cleanup canceled order_id={}: {}", order_id, cancel_response);
            canceled_order_ids.insert(order_id);
        }
    }

    println!("{}", "═".repeat(80));
    println!("Completed rapid BTC cycle. Final live order count for market {}:", market_index);
    let final_active_orders = client
        .get_account_active_orders(account_index, Some(market_index), Some(100), None)
        .await?;
    println!("  active_orders={}", final_active_orders.items.len());
    for order in &final_active_orders.items {
        println!("  stayed order => {}", order_summary(order));
    }
    println!("  available_balance={:?}", account.available_balance);
    println!("{}", "═".repeat(80));

    Ok(())
}