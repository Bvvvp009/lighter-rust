//! Full trading round-trip on Lighter mainnet using the `lighter_sdk` facade.
//!
//! **Uses real funds.  Understand the order parameters before running.**
//!
//! Flow:
//! 1. Fetch market 0 order-book details and current best bid.
//! 2. Compute a limit price 50 % below the best bid (will never fill).
//! 3. Place a BUY LIMIT order via `create_order`.
//! 4. Wait 500 ms, then list active orders and find the just-placed one.
//! 5. Cancel it via `cancel_order` using the server-assigned `order_index`.
//!
//! ```text
//! cargo run -p lighter-sdk --example mainnet_live_order_cycle
//! ```
//!
//! Required env vars (or a `.env` file):
//! - `BASE_URL`, `ACCOUNT_INDEX`, `API_KEY_INDEX`, `API_PRIVATE_KEY`
use lighter_sdk::prelude::*;
use lighter_sdk::CreateOrderRequest;
use std::{env, time::Duration};
use tokio::time::sleep;

/// Sentinel client_order_index; unique enough for this standalone example.
const CLIENT_ORDER_INDEX: u64 = 77_777;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("================================================================");
    println!("  lighter-sdk  ·  mainnet_live_order_cycle");
    println!("================================================================");
    println!("  Base URL:    {}", base_url);
    println!("  Account:     {}", account_index);
    println!("================================================================\n");

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // ------------------------------------------------------------------
    // 1. Fetch market metadata + best bid to compute a safe limit price.
    // ------------------------------------------------------------------
    let details = client.get_order_book_details(0).await?;
    let price_decimals = details.price_decimals.unwrap_or(2) as u32;
    let price_scale = 10_i64.pow(price_decimals) as f64;
    println!(
        "Market 0  symbol={:?}  price_decimals={}  size_decimals={:?}",
        details.symbol, price_decimals, details.size_decimals
    );

    let ob = client.get_order_book(0).await?;
    let best_bid_raw: f64 = ob
        .bids
        .first()
        .and_then(|l| l.price.parse().ok())
        .unwrap_or(1000.0);
    let limit_price = ((best_bid_raw * 0.50) * price_scale).round() as i64;
    println!(
        "Best bid raw={best_bid_raw}  → limit_price={limit_price}  (50 % below; will never fill)\n"
    );

    // ------------------------------------------------------------------
    // 2. Place the order.
    // ------------------------------------------------------------------
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: CLIENT_ORDER_INDEX,
        base_amount: 1_000,
        price: limit_price,
        is_ask: false,         // BUY side
        order_type: 0,         // Limit
        time_in_force: 1,      // GoodTillTime
        reduce_only: false,
        trigger_price: 0,
        order_expiry: 0,
    };

    println!("→ create_order  client_order_index={CLIENT_ORDER_INDEX}");
    let resp = client.create_order(order).await?;
    let code = resp["code"].as_i64().unwrap_or(0);
    println!("  response: code={code}  raw={}", resp);
    if code != 200 {
        let msg = resp["message"].as_str().unwrap_or("(no message)");
        return Err(format!("create_order failed: code={code} message={msg}").into());
    }

    // ------------------------------------------------------------------
    // 3. Wait briefly for the order to appear in the active-orders list.
    // ------------------------------------------------------------------
    println!("\n→ waiting 500 ms …");
    sleep(Duration::from_millis(500)).await;

    // ------------------------------------------------------------------
    // 4. List active orders and locate ours by client_order_index.
    // ------------------------------------------------------------------
    println!("→ get_account_active_orders");
    let page = client
        .get_account_active_orders(account_index, Some(0), Some(50), None)
        .await?;
    println!("  {} active order(s) on market 0", page.items.len());

    let placed: Option<&Order> = page
        .items
        .iter()
        .find(|o| o.client_order_index == Some(CLIENT_ORDER_INDEX));

    let server_order_index = match placed {
        Some(o) => {
            println!(
                "  found it: order_index={}  price={}  base_amount={}",
                o.order_index, o.price, o.base_amount
            );
            o.order_index
        }
        None => {
            eprintln!(
                "  ⚠  Order with client_order_index={CLIENT_ORDER_INDEX} not found in active list.\n  \
                 It may have been placed on a different market or already expired.\n  \
                 Attempting cancel using response order_index if available…"
            );
            resp["order_index"].as_i64().unwrap_or(0)
        }
    };

    // ------------------------------------------------------------------
    // 5. Cancel.
    // ------------------------------------------------------------------
    println!("\n→ cancel_order  order_book_index=0  order_index={server_order_index}");
    let cancel_resp = client.cancel_order(0, server_order_index).await?;
    println!("  response: {cancel_resp}");

    println!("\n✅  Order cycle complete.");
    Ok(())
}
