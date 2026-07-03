//! Curated one-command validation matrix for the Lighter SDK.
//!
//! Prints a pass/fail table that covers signing operations, read-only REST
//! endpoints, and—when `LIGHTER_LIVE_CHECKS=1` is set—a safe live trading
//! round-trip (places a limit order 50 % below the market, then immediately
//! cancels it).
//!
//! ```text
//! # Read-only mode (safe, no side-effects)
//! cargo run -p lighter-sdk --example mainnet_validation_matrix
//!
//! # Enable live trading checks
//! LIGHTER_LIVE_CHECKS=1 cargo run -p lighter-sdk --example mainnet_validation_matrix
//! ```
//!
//! Required env vars (or a `.env` file):
//! - `BASE_URL`, `ACCOUNT_INDEX`, `API_KEY_INDEX`, `API_PRIVATE_KEY`
use lighter_sdk::prelude::*;
use lighter_sdk::{CreateOrderRequest, SignerClient};
use std::env;

/// A single row in the validation table.
struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

impl Check {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Check { name, passed: true, detail: detail.into() }
    }
    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Check { name, passed: false, detail: detail.into() }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let live_checks = env::var("LIGHTER_LIVE_CHECKS").unwrap_or_default() == "1";

    println!("================================================================");
    println!("  lighter-sdk  ·  mainnet_validation_matrix");
    println!("================================================================");
    println!("  Base URL:    {}", base_url);
    println!("  Account:     {}", account_index);
    println!(
        "  Live checks: {}",
        if live_checks { "ENABLED" } else { "disabled  (set LIGHTER_LIVE_CHECKS=1 to enable)" }
    );
    println!();

    let mut checks: Vec<Check> = Vec::new();
    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;

    // --- Signing (purely local, no network) --------------------------------
    match SignerClient::new(&api_key, account_index, api_key_index)
        .and_then(|s| s.create_auth_token(60).map(|t| t.len()))
    {
        Ok(len) => checks.push(Check::pass("sign · create_auth_token", format!("{len} chars"))),
        Err(e) => checks.push(Check::fail("sign · create_auth_token", e.to_string())),
    }

    // --- Read-only REST ----------------------------------------------------
    match client.get_status().await {
        Ok(s) => checks.push(Check::pass(
            "rest · get_status",
            format!("network_id={:?}", s.network_id),
        )),
        Err(e) => checks.push(Check::fail("rest · get_status", e.to_string())),
    }

    match client.get_nonce().await {
        Ok(n) => checks.push(Check::pass("rest · get_nonce", format!("nonce={n}"))),
        Err(e) => checks.push(Check::fail("rest · get_nonce", e.to_string())),
    }

    match client.get_my_account().await {
        Ok(a) => checks.push(Check::pass(
            "rest · get_my_account",
            format!("account_index={}", a.account_index),
        )),
        Err(e) => checks.push(Check::fail("rest · get_my_account", e.to_string())),
    }

    match client.get_order_books().await {
        Ok(bs) => checks.push(Check::pass(
            "rest · get_order_books",
            format!("{} markets", bs.len()),
        )),
        Err(e) => checks.push(Check::fail("rest · get_order_books", e.to_string())),
    }

    match client.get_order_book(0).await {
        Ok(ob) => checks.push(Check::pass(
            "rest · get_order_book(0)",
            format!("{} bids / {} asks", ob.bids.len(), ob.asks.len()),
        )),
        Err(e) => checks.push(Check::fail("rest · get_order_book(0)", e.to_string())),
    }

    match client.get_order_book_details(0).await {
        Ok(d) => checks.push(Check::pass(
            "rest · get_order_book_details(0)",
            format!("symbol={:?}", d.symbol),
        )),
        Err(e) => checks.push(Check::fail("rest · get_order_book_details(0)", e.to_string())),
    }

    match client.check_api_key().await {
        Ok(()) => checks.push(Check::pass("rest · check_api_key", "VALID")),
        Err(e) => checks.push(Check::fail("rest · check_api_key", e.to_string())),
    }

    // --- Live checks (optional) -------------------------------------------
    if live_checks {
        // Determine a safe limit price: 50 % below the current best bid so the
        // order will never match and can be cancelled immediately.
        let ob = client.get_order_book(0).await.unwrap_or_default();
        let details = client.get_order_book_details(0).await.unwrap_or_default();
        let price_decimals = details.price_decimals.unwrap_or(2) as u32;
        let price_scale = 10_i64.pow(price_decimals) as f64;
        let best_bid_raw: f64 = ob
            .bids
            .first()
            .and_then(|l| l.price.parse().ok())
            .unwrap_or(1000.0);
        let limit_price = ((best_bid_raw * 0.50) * price_scale).round() as i64;

        let order = CreateOrderRequest {
            account_index,
            order_book_index: 0,
            client_order_index: 99990,
            base_amount: 1000,
            price: limit_price,
            is_ask: false,
            order_type: 0,      // Limit
            time_in_force: 1,   // GoodTillTime
            reduce_only: false,
            trigger_price: 0,
            order_expiry: 0,
        };

        match client.create_order(order).await {
            Ok(resp) => {
                let code = resp["code"].as_i64().unwrap_or(0);
                if code == 200 {
                    checks.push(Check::pass(
                        "live · create_order (50 % below best bid)",
                        format!("code={code}  limit_price={limit_price}"),
                    ));
                    // Cancel it immediately using the server-assigned order index.
                    let order_index = resp["order_index"].as_i64().unwrap_or(99990);
                    match client.cancel_order(0, order_index).await {
                        Ok(_) => checks.push(Check::pass(
                            "live · cancel_order",
                            format!("order_index={order_index}"),
                        )),
                        Err(e) => checks.push(Check::fail("live · cancel_order", e.to_string())),
                    }
                } else {
                    let msg = resp["message"].as_str().unwrap_or("unknown");
                    checks.push(Check::fail(
                        "live · create_order (50 % below best bid)",
                        format!("code={code}  message={msg}"),
                    ));
                }
            }
            Err(e) => checks.push(Check::fail(
                "live · create_order (50 % below best bid)",
                e.to_string(),
            )),
        }
    }

    // --- Print table -------------------------------------------------------
    println!("  {:<52} {:<6}  {}", "CHECK", "RESULT", "DETAIL");
    println!("  {}", "─".repeat(92));
    let mut passed = 0usize;
    let mut failed = 0usize;
    for c in &checks {
        let result = if c.passed { "PASS" } else { "FAIL" };
        println!("  {:<52} {:<6}  {}", c.name, result, c.detail);
        if c.passed { passed += 1; } else { failed += 1; }
    }
    println!("  {}", "─".repeat(92));
    println!("  Passed: {}  Failed: {}", passed, failed);
    println!();

    if failed > 0 {
        eprintln!("❌  {} check(s) failed.", failed);
        std::process::exit(1);
    }
    println!("✅  All {} checks passed.", passed);
    Ok(())
}
