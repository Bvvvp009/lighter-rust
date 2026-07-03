/// Grouped Orders Live Example — OCO and OTOCO
/// Grouped Orders Live Example — OTO, OCO, and OTOCO
///
/// Protocol grouping types (from lighter-go/types/txtypes/constants.go):
///   1 = OTO   (One-Triggers-Other): entry fills → child order becomes active
///   2 = OCO   (One-Cancels-Other): 2 exit orders (SL+TP), first fill cancels the other
///   3 = OTOCO (One-Triggers-A-One-Cancels-Other): entry fills → OCO bracket activated
///
/// OTO rules:
///   - orders[0]: parent (Limit or Market, has base_amount, IoC or GTT+expiry)
///   - orders[1]: child SL/TP (base_amount=0, opposite direction, IoC, trigger+expiry set)
///
/// OCO rules (for closing an EXISTING position):
///   - 2 orders, SAME direction (e.g. both is_ask=true to close a long)
///   - Both must be reduce_only=1, same base_amount, SL+TP types, same order_expiry
///
/// OTOCO rules:
///   - orders[0]: parent entry (Limit or Market, base_amount set, GTT+expiry or IoC)
///   - orders[1]: SL child (base_amount=0, opposite direction, IoC, trigger+expiry)
///   - orders[2]: TP child (base_amount=0, opposite direction, IoC, trigger+expiry, same expiry as SL)
///
/// Flow:
///   STEP 0 — OCO: open a small long position, then submit reduce-only SL+TP exits
///   STEP 1 — OTOCO: entry BUY $2200 (resting, won't fill) + SL $2100 + TP $2600
///   STEP 2 — Fetch active orders → get server-assigned order_index for entry
///   STEP 3 — Cancel entry individually → OTOCO children auto-cancelled by server
///   STEP 4 — OTO: entry BUY limit (resting) + SL child (triggered on fill)
///   STEP 5 — cancel_all_orders cleanup
use api_client::{CreateGroupedOrdersRequest, CreateOrderRequest, LighterClient};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔗 GROUPED ORDERS — OTO / OCO / OTOCO LIVE EXAMPLE");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    client.check_api_key().await?;

    // Common expiry: 24 hours from now (required for GTT orders and SL/TP children)
    let expiry_24h = now_ms() + 24 * 3600 * 1000;

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 0 — OCO: One-Cancels-Other  (grouping_type = 2)
    //
    // Open a small long position first, then place two reduce-only exit orders
    // on the same side. This is the cleanest deterministic live OCO test.
    //   - Market BUY opens the position
    //   - OCO SELL exits: stop loss + take profit
    //   - Both legs must be reduce_only=true and use the same base_amount
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 0 — OCO (grouping_type=2): open 0.1 ETH long, then place SL + TP exits");
    println!("{}", "─".repeat(80));

    let oco_entry_client_order_index = now_ms() as u64 + 10;
    println!("📝 Opening a small long position with a market BUY...");
    let open_long = client
        .create_market_order(0, oco_entry_client_order_index, 1000, 260_000, false)
        .await?;
    println!("📥 Open-long response:");
    println!("{}", serde_json::to_string_pretty(&open_long)?);

    let open_long_code = open_long["code"].as_i64().unwrap_or_default();
    if open_long_code != 200 {
        return Err(format!(
            "opening long for OCO test failed with code {}: {}",
            open_long_code,
            open_long["message"].as_str().unwrap_or("unknown error")
        )
        .into());
    }

    let oco_sl = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: oco_entry_client_order_index + 1,
        base_amount: 1000,
        price: 205_000,
        is_ask: true,
        order_type: 3,               // StopLossLimit
        time_in_force: 0,            // IoC
        reduce_only: true,
        trigger_price: 210_000,
        order_expiry: expiry_24h,
    };

    let oco_tp = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: oco_entry_client_order_index + 2,
        base_amount: 1000,
        price: 275_000,
        is_ask: true,
        order_type: 5,               // TakeProfitLimit
        time_in_force: 0,            // IoC
        reduce_only: true,
        trigger_price: 270_000,
        order_expiry: expiry_24h,
    };

    println!("📝 Submitting OCO grouped order (grouping_type=2)...");
    println!("  SL: client_order_index={}, SELL reduce-only stop-loss", oco_entry_client_order_index + 1);
    println!("  TP: client_order_index={}, SELL reduce-only take-profit", oco_entry_client_order_index + 2);

    let oco_result = client.create_grouped_orders(CreateGroupedOrdersRequest {
        grouping_type: 2, // OCO
        orders: vec![oco_sl.clone(), oco_tp.clone()],
    }).await;

    match oco_result {
        Ok(resp) => {
            let code = resp["code"].as_i64().unwrap_or(0);
            println!("\n📥 OCO Response:");
            println!("{}", serde_json::to_string_pretty(&resp)?);
            if code == 200 {
                println!("✅ OCO grouped order placed!");
                if let Some(tx) = resp["tx_hash"].as_str() {
                    println!("📜 Tx Hash (OCO create): {}", tx);
                }
            } else {
                println!("⚠️  OCO returned code={} — {}", code, resp["message"].as_str().unwrap_or(""));
            }
        }
        Err(e) => println!("❌ OCO error: {}", e),
    }

    println!("🧹 Closing the position opened for the OCO test...");
    let account = client.get_my_account().await?;
    let position = account
        .positions
        .as_ref()
        .and_then(|positions| positions.iter().find(|position| position.market_index == 0));

    if let Some(position) = position {
        let pos_str = position.base_amount.as_deref().unwrap_or("0");
        let pos_f: f64 = pos_str.parse().unwrap_or(0.0);
        if pos_f.abs() >= 0.001 {
            let close_units = (pos_f.abs() * 10000.0).round() as i64;
            let close_is_ask = pos_f > 0.0;
            let close_order = client
                .create_market_order(0, oco_entry_client_order_index + 3, close_units, 200_000, close_is_ask)
                .await?;
            println!("📥 Close-position response:");
            println!("{}", serde_json::to_string_pretty(&close_order)?);
        }
    }

    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 1 — OTOCO: One-Triggers-A-One-Cancels-Other  (grouping_type = 3)
    //
    // Entry: LimitOrder BUY at $2200 (below market ~$2326, rests in book, won't fill)
    //   → This demonstrates OTOCO creation; if entry filled, SL+TP bracket activates
    // SL:  StopLossOrder SELL, trigger @ $2100, limit @ $2050, base_amount=0 (auto-size)
    // TP:  TakeProfitOrder SELL, trigger @ $2700, limit @ $2750, base_amount=0 (auto-size)
    //
    // Rules enforced:
    //   - Parent: LimitOrder, GTT, order_expiry set, trigger_price=0
    //   - Children: opposite is_ask from parent, base_amount=0, IoC, trigger+expiry set
    //   - Children: same order_expiry value
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 1 — OTOCO (grouping_type=3): Entry BUY $2200 + SL $2100 + TP $2700");
    println!("  (Entry at $2200 rests below market ~$2326; cancelling it kills SL+TP)");
    println!("{}", "─".repeat(80));

    let base_ts = now_ms();

    let otoco_entry = CreateOrderRequest {
        account_index,
        order_book_index: 0,           // ETH-USD perp
        client_order_index: base_ts as u64,
        base_amount: 10000,            // 1.0 ETH
        price: 220_000,                // $2200 — below market, rests without filling
        is_ask: false,                 // BUY
        order_type: 0,                 // LimitOrder
        time_in_force: 1,              // GoodTillTime — requires order_expiry to be set
        reduce_only: false,
        trigger_price: 0,              // no trigger (limit order)
        order_expiry: expiry_24h,      // required for GTT parent
    };

    let otoco_sl = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: (base_ts + 1) as u64,
        base_amount: 0,                // 0 = auto-size to match entry (required for OTOCO children)
        price: 205_000,                // $2050 limit price on SL execution
        is_ask: true,                  // SELL (opposite direction from BUY entry)
        order_type: 2,                 // StopLossOrder
        time_in_force: 0,              // IoC (required for SL/TP children)
        reduce_only: true,             // only reduce position
        trigger_price: 210_000,        // trigger when price ≤ $2100
        order_expiry: expiry_24h,      // required (must equal TP expiry)
    };

    let otoco_tp = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: (base_ts + 2) as u64,
        base_amount: 0,                // 0 = auto-size (required for OTOCO children)
        price: 275_000,                // $2750 limit price on TP execution
        is_ask: true,                  // SELL (opposite direction from BUY entry)
        order_type: 4,                 // TakeProfitOrder
        time_in_force: 0,              // IoC (required for SL/TP children)
        reduce_only: true,
        trigger_price: 270_000,        // trigger when price ≥ $2700
        order_expiry: expiry_24h,      // required (must equal SL expiry)
    };

    println!("📝 Submitting OTOCO grouped order (grouping_type=3)...");
    println!("  Entry: client_order_index={}, BUY LimitOrder $2200 GTT", base_ts);
    println!("  SL:    client_order_index={}, SELL StopLoss trigger=$2100, base=0", base_ts + 1);
    println!("  TP:    client_order_index={}, SELL TakeProfit trigger=$2700, base=0", base_ts + 2);

    let otoco_result = client.create_grouped_orders(CreateGroupedOrdersRequest {
        grouping_type: 3, // OTOCO
        orders: vec![otoco_entry.clone(), otoco_sl.clone(), otoco_tp.clone()],
    }).await;

    let mut otoco_entry_order_index: Option<i64> = None;

    match otoco_result {
        Ok(resp) => {
            let code = resp["code"].as_i64().unwrap_or(0);
            println!("\n📥 OTOCO Response:");
            println!("{}", serde_json::to_string_pretty(&resp)?);
            if code == 200 {
                println!("✅ OTOCO grouped order placed!");
                if let Some(tx) = resp["tx_hash"].as_str() {
                    println!("📜 Tx Hash (OTOCO create): {}", tx);
                }
            } else {
                println!("⚠️  OTOCO returned code={} — {}", code, resp["message"].as_str().unwrap_or(""));
            }
        }
        Err(e) => println!("❌ OTOCO error: {}", e),
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 2 — Fetch active orders → get server-assigned order_index for entry
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 2 — Fetch active orders (get server order_index for entry)");
    println!("{}", "─".repeat(80));

    let active = client
        .get_account_active_orders(account_index, Some(0), Some(50), None)
        .await?;

    println!("  Active orders on market 0: {}", active.items.len());
    for o in &active.items {
        println!(
            "    order_index={}  client_order_index={:?}  price={:?}  is_ask={:?}  type={:?}",
            o.order_index, o.client_order_index, o.price, o.is_ask, o.order_type
        );
        // Identify the OTOCO entry order by client_order_index
        if o.client_order_index == Some(base_ts as u64) {
            otoco_entry_order_index = Some(o.order_index);
        }
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 3 — Cancel OTOCO entry by server order_index
    // When the entry (parent) is cancelled, the server auto-cancels the pending
    // SL+TP child orders that were not yet activated.
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 3 — Cancel OTOCO entry (server auto-cancels pending SL+TP children)");
    println!("{}", "─".repeat(80));

    if let Some(idx) = otoco_entry_order_index {
        println!("📝 Cancelling OTOCO entry order_index={} (market=0)...", idx);
        match client.cancel_order(0, idx).await {
            Ok(resp) => {
                let code = resp["code"].as_i64().unwrap_or(0);
                println!("{}", serde_json::to_string_pretty(&resp)?);
                if code == 200 {
                    println!("✅ Entry cancelled — server auto-cancels SL+TP children");
                    println!("   Tx Hash: {}", resp["tx_hash"].as_str().unwrap_or("n/a"));
                } else {
                    println!("⚠️  code={} — {}", code, resp["message"].as_str().unwrap_or(""));
                }
            }
            Err(e) => println!("❌ Cancel error: {}", e),
        }
    } else {
        println!("  ℹ OTOCO entry not found in active orders — using cancel_all as fallback");
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 4 — OTO: One-Triggers-Other  (grouping_type = 1)
    //
    // Entry: LimitOrder BUY at $2200 (resting, won't fill immediately)
    // Child: StopLossOrder SELL, base_amount=0, IoC, opposite direction
    //   → If entry fills, SL child becomes active
    //   → Cancelling entry before fill removes the group
    //
    // OTO rules:
    //   - orders[0]: parent (Limit or Market, specific base_amount, GTT+expiry or IoC)
    //   - orders[1]: child (SL/TP type, base_amount=0, OPPOSITE is_ask, IoC, trigger+expiry)
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 4 — OTO (grouping_type=1): Entry BUY $2200 + SL child trigger $2100");
    println!("{}", "─".repeat(80));

    let base_ts2 = now_ms() + 100;

    let oto_entry = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: base_ts2 as u64,
        base_amount: 10000,          // 1.0 ETH — parent must have specific base_amount
        price: 220_000,              // $2200 — resting below market, won't fill
        is_ask: false,               // BUY
        order_type: 0,               // LimitOrder
        time_in_force: 1,            // GoodTillTime — requires order_expiry
        reduce_only: false,
        trigger_price: 0,
        order_expiry: expiry_24h,    // required for GTT parent
    };

    let oto_sl = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: (base_ts2 + 1) as u64,
        base_amount: 0,              // 0 = auto-size (required for OTO child)
        price: 205_000,              // $2050 limit price on SL execution
        is_ask: true,                // SELL — OPPOSITE direction from BUY entry
        order_type: 2,               // StopLossOrder
        time_in_force: 0,            // IoC (required for child SL/TP)
        reduce_only: true,
        trigger_price: 210_000,      // trigger when price ≤ $2100
        order_expiry: expiry_24h,    // required for SL/TP child
    };

    println!("📝 Submitting OTO grouped order (grouping_type=1)...");
    println!("  Entry: client_order_index={}, BUY LimitOrder $2200 GTT", base_ts2);
    println!("  Child SL: client_order_index={}, SELL StopLoss trigger=$2100, base=0", base_ts2 + 1);

    match client.create_grouped_orders(CreateGroupedOrdersRequest {
        grouping_type: 1, // OTO
        orders: vec![oto_entry.clone(), oto_sl.clone()],
    }).await {
        Ok(resp) => {
            let code = resp["code"].as_i64().unwrap_or(0);
            println!("\n📥 OTO Response:");
            println!("{}", serde_json::to_string_pretty(&resp)?);
            if code == 200 {
                println!("✅ OTO grouped order placed!");
                if let Some(tx) = resp["tx_hash"].as_str() {
                    println!("📜 Tx Hash (OTO create): {}", tx);
                }
            } else {
                println!("⚠️  OTO returned code={} — {}", code, resp["message"].as_str().unwrap_or(""));
            }
        }
        Err(e) => println!("❌ OTO error: {}", e),
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // STEP 5 — Final cleanup: cancel_all_orders
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "─".repeat(80));
    println!("STEP 5 — cleanup: cancel_all_orders");
    println!("{}", "─".repeat(80));
    // Refresh nonce first since prior txs may have shifted the server nonce
    let _ = client.refresh_nonce().await;
    match client.cancel_all_orders(0, 0).await {
        Ok(resp) => {
            let code = resp["code"].as_i64().unwrap_or(0);
            if code == 200 {
                println!("✅ cancel_all_orders — tx_hash: {}", resp["tx_hash"].as_str().unwrap_or("n/a"));
            } else {
                println!("ℹ cancel_all code={} — {}", code, resp["message"].as_str().unwrap_or(""));
            }
        }
        Err(e) => println!("❌ {}", e),
    }

    println!();
    println!("{}", "═".repeat(80));
    println!("✅ Grouped orders example complete.");
    println!();
    println!("  Protocol grouping types (lighter-go/types/txtypes/constants.go):");
    println!("    1 = OTO   — One-Triggers-Other (entry + one SL or TP)");
    println!("    2 = OCO   — One-Cancels-Other  (2 exit orders, same dir, reduce_only=1)");
    println!("    3 = OTOCO — One-Triggers-A-One-Cancels-Other (entry + SL + TP bracket)");
    println!();
    println!("  Key rules:");
    println!("    OTO child  : base_amount=0, opposite direction, IoC, trigger+expiry set");
    println!("    OTOCO child: base_amount=0, opposite direction, IoC, trigger+expiry set");
    println!("    OCO        : both reduce_only=1, same direction, same base_amount+expiry");
    println!();
    println!("  Cancel grouped orders:");
    println!("    cancel_order(market, server_order_index) — cancel parent → children cancelled");
    println!("    cancel_all_orders()                       — remove all open orders at once");
    println!("{}", "═".repeat(80));

    Ok(())
}

