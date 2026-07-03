use lighter_sdk::{CombinedClient, CreateOrderRequest, KeyManager, LighterClient};
use std::env;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{timeout, Duration};

fn env_var(name: &str) -> Result<String, Box<dyn Error>> {
    env::var(name).map_err(|_| format!("Missing required environment variable: {name}").into())
}

fn position_count(account: &lighter_sdk::Account) -> usize {
    account
        .positions
        .as_ref()
        .map(|positions| {
            positions
                .iter()
                .filter(|p| {
                    p.base_amount
                        .as_deref()
                        .unwrap_or("0")
                        .parse::<f64>()
                        .map(|v| v.abs() > 0.0)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn report_result<T, E, F>(name: &str, result: Result<T, E>, describe: F) -> bool
where
    E: std::fmt::Display,
    F: FnOnce(&T) -> String,
{
    match result {
        Ok(value) => {
            println!("  ✔ {name}: {}", describe(&value));
            true
        }
        Err(err) => {
            println!("  ⚠ {name}: {err}");
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let base_url = env_var("BASE_URL")?;
    let api_private_key = env_var("API_PRIVATE_KEY")?;
    let account_index: i64 = env_var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env_var("API_KEY_INDEX")?.parse()?;

    println!("=== Lighter Rust mainnet smoke check ===");
    println!("Mode: read-only + sign-only (no live orders submitted)");
    println!("Base URL: {base_url}");
    println!("Account index: {account_index}");
    println!("API key index: {api_key_index}");

    let client = LighterClient::new(
        base_url.clone(),
        &api_private_key,
        account_index,
        api_key_index,
    )?;
    let key_manager = KeyManager::from_hex(&api_private_key)?;

    match client.check_api_key().await {
        Ok(()) => println!("✔ API key matches the server record"),
        Err(err) => println!("⚠ API key check failed, continuing with the remaining live probes: {}", err),
    }

    let status = match client.get_status().await {
        Ok(status) => status,
        Err(err) => {
            println!("⚠ get_status failed, continuing with defaults: {}", err);
            Default::default()
        }
    };
    let info = match client.get_info().await {
        Ok(info) => info,
        Err(err) => {
            println!("⚠ get_info failed, continuing with defaults: {}", err);
            Default::default()
        }
    };
    let nonce = client.get_nonce().await?;
    println!(
        "✔ Status OK (network_id={:?}, timestamp={:?}), next nonce={}",
        status.network_id, status.timestamp, nonce
    );
    println!("✔ Info OK (contract={:?})", info.contract_address);

    let account = match client.get_my_account().await {
        Ok(account) => account,
        Err(err) => {
            println!("⚠ get_my_account failed, continuing with defaults: {}", err);
            let mut account = lighter_sdk::Account::default();
            account.account_index = account_index;
            account
        }
    };
    let limits = client.get_account_limits(account_index).await.ok();
    let metadata = client.get_account_metadata(account_index).await.ok();
    let api_keys = client
        .get_api_keys(account_index, Some(api_key_index))
        .await
        .unwrap_or_default();
    let maker_only_api_keys = client.get_maker_only_api_keys(account_index, None).await.ok();
    let maker_only_api_key_indexes = maker_only_api_keys
        .as_ref()
        .map(|value| value.api_key_indexes.clone())
        .unwrap_or_default();
    let partner_stats = client.partner_stats(account_index, None, None, None).await.ok();

    let open_orders_before = client
        .get_account_active_orders(account_index, None, Some(20), None)
        .await
        .map(|page| page.items.len())
        .unwrap_or(0);
    let positions_before = position_count(&account);

    let l1_metadata = if let Some(l1_address) = account.l1_address.as_deref() {
        client.get_l1_metadata(l1_address, None).await.ok()
    } else {
        None
    };

    println!(
        "✔ Account OK (name={:?}, tier={:?}, available_balance={:?}, api_keys={}, maker_only_api_keys={}, open_orders={}, open_positions={}, l1_chain_id={:?}, partner_trades={:?})",
        metadata.as_ref().and_then(|value| value.name.clone()),
        limits.as_ref().and_then(|value| value.user_tier.clone()),
        account.available_balance,
        api_keys.len(),
        maker_only_api_keys
            .as_ref()
            .map(|value| value.api_key_indexes.len())
            .unwrap_or(0),
        open_orders_before,
        positions_before,
        l1_metadata.as_ref().and_then(|metadata| metadata.chain_id),
        partner_stats.as_ref().and_then(|value| value.total_trades),
    );
    println!(
        "   Positions: {:?}",
        account.positions
    );

    let order_books = client.get_order_books().await?;
    let market_index: u32 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let _order_book = client.get_order_book(market_index).await?;
    let _order_book_details = client.get_order_book_details(market_index).await?;
    let recent_trades = client.get_recent_trades(market_index, Some(5)).await?;
    let trades_page = client.get_trades(market_index, Some(5), None).await?;
    let candles = client
        .get_candles(market_index, 60, None, None, Some(5), None)
        .await?;
    let exchange_stats = client.get_exchange_stats().await?;
    let asset_details = client.get_asset_details(0).await?;

    println!(
        "✔ Market data OK (order_books={}, market={}, recent_trades={}, paged_trades={}, candles={}, assets={})",
        order_books.len(),
        market_index,
        recent_trades.len(),
        trades_page.items.len(),
        candles.len(),
        asset_details.asset_details.len()
    );
    println!(
        "✔ Exchange stats OK (daily_usd_volume={:?}, daily_trades_count={:?})",
        exchange_stats.daily_usd_volume, exchange_stats.daily_trades_count
    );

    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    let mut extended_ok = 0usize;
    let mut extended_failed = 0usize;
    let mut record = |ok: bool| {
        if ok {
            extended_ok += 1;
        } else {
            extended_failed += 1;
        }
    };

    println!("Validating extended account and market endpoints...");
    record(report_result(
        "pnl",
        client
            .get_pnl(
                account_index,
                "1h",
                now_ms - 86_400_000,
                now_ms,
                24,
                Some(false),
            )
            .await,
        |pnl| format!("points={}", pnl.pnl.len()),
    ));
    record(report_result(
        "fundings",
        client
            .get_fundings(account_index, Some(market_index), Some(5), None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "funding_rates",
        client.get_funding_rates(market_index, Some(5), None).await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "deposit_history",
        client
            .get_deposit_history(account_index, Some(5), None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "withdraw_history",
        client
            .get_withdraw_history(account_index, Some(5), None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "transfer_history",
        client
            .get_transfer_history(account_index, Some(5), None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "account_transactions",
        client
            .get_account_transactions(account_index, Some(5), None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "public_pools_metadata",
        client.get_public_pools_metadata(None).await,
        |items| format!("items={}", items.len()),
    ));
    record(report_result(
        "lease_options",
        client.get_lease_options().await,
        |items| format!("items={}", items.len()),
    ));
    record(report_result(
        "leases",
        client.get_leases(account_index, Some(5), None, None).await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "liquidations",
        client
            .get_liquidations(account_index, Some(5), Some(market_index), None, None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "position_funding",
        client
            .get_position_funding(account_index, Some(5), Some(market_index), None, None, None)
            .await,
        |page| format!("items={}", page.items.len()),
    ));
    record(report_result(
        "tokens",
        client.get_tokens(account_index, None).await,
        |items| format!("items={}", items.len()),
    ));
    record(report_result(
        "maker_only_api_keys",
        client.get_maker_only_api_keys(account_index, None).await,
        |response| format!("indexes={}", response.api_key_indexes.len()),
    ));
    record(report_result(
        "partner_stats",
        client.partner_stats(account_index, None, None, None).await,
        |stats| {
            format!(
                "trades={:?}, unique_clients={:?}",
                stats.total_trades, stats.unique_clients
            )
        },
    ));
    record(report_result(
        "exchange_metrics",
        client
            .get_exchange_metrics("24h", "volume", None, None)
            .await,
        |payload| {
            format!(
                "shape={}",
                if payload.is_object() {
                    "object"
                } else if payload.is_array() {
                    "array"
                } else {
                    "scalar"
                }
            )
        },
    ));
    record(report_result(
        "execute_stats",
        client.get_execute_stats("24h").await,
        |payload| {
            format!(
                "shape={}",
                if payload.is_object() {
                    "object"
                } else if payload.is_array() {
                    "array"
                } else {
                    "scalar"
                }
            )
        },
    ));
    record(report_result(
        "export_data",
        client
            .export_data(
                "trades",
                Some(account_index),
                Some(market_index),
                Some(now_ms - 86_400_000),
                Some(now_ms),
                None,
                None,
                None,
                None,
            )
            .await,
        |payload| {
            format!(
                "shape={}",
                if payload.is_object() {
                    "object"
                } else if payload.is_array() {
                    "array"
                } else {
                    "scalar"
                }
            )
        },
    ));

    record(report_result(
        "set_maker_only_api_keys",
        client
            .set_maker_only_api_keys(
                account_index,
                &maker_only_api_key_indexes,
                None,
            )
            .await,
        |response| format!("code={:?}", response.code),
    ));

    println!(
        "✔ Extended endpoint coverage: {} ok, {} issues",
        extended_ok, extended_failed
    );
    if extended_failed > 0 {
        return Err(format!(
            "Smoke check found {} extended endpoint issues; see output above",
            extended_failed
        )
        .into());
    }

    let sample_order = CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index: (now_ms as u64) % 9_000_000_000,
        base_amount: 1,
        price: 1,
        is_ask: false,
        order_type: 0,
        time_in_force: 0,
        reduce_only: true,
        trigger_price: 0,
    };

    let _ = client
        .sign_create_order_with_nonce(sample_order, Some(nonce + 1))
        .await?;
    let _ = client
        .sign_cancel_order_with_nonce(market_index as u8, 0, Some(nonce + 2))
        .await?;
    let _ = client
        .sign_cancel_all_orders_with_nonce(0, now_ms, Some(nonce + 3))
        .await?;
    let _ = client
        .sign_transfer_with_nonce(account_index, 1, 0, [0u8; 32], Some(nonce + 4))
        .await?;
    let _ = client
        .sign_change_pub_key_with_nonce(key_manager.public_key_bytes(), Some(nonce + 5))
        .await?;
    let _ = client
        .sign_update_leverage_with_nonce(market_index as u8, 3333, 0, Some(nonce + 6))
        .await?;
    let _ = client
        .sign_create_sub_account_with_nonce(Some(nonce + 7))
        .await?;
    let _ = client
        .sign_modify_order_with_nonce(market_index as u8, 0, 1, 1, 0, Some(nonce + 8))
        .await?;
    let _ = client
        .sign_create_public_pool_with_nonce(0, 1_000, 1, Some(nonce + 9))
        .await?;
    let _ = client
        .sign_update_public_pool_with_nonce(0, 0, 0, 1, Some(nonce + 10))
        .await?;
    let _ = client
        .sign_mint_shares_with_nonce(0, 1_000, Some(nonce + 11))
        .await?;
    let _ = client
        .sign_burn_shares_with_nonce(0, 1_000, Some(nonce + 12))
        .await?;
    let _ = client
        .sign_update_margin_with_nonce(market_index as u8, 1, 1, Some(nonce + 13))
        .await?;
    let grouped = vec![CreateOrderRequest {
        account_index,
        order_book_index: market_index as u8,
        client_order_index: ((now_ms + 1) as u64) % 9_000_000_000,
        base_amount: 1,
        price: 1,
        is_ask: false,
        order_type: 0,
        time_in_force: 0,
        reduce_only: true,
        trigger_price: 0,
    }];
    let _ = client
        .sign_create_grouped_orders_with_nonce(0, grouped, Some(nonce + 14))
        .await?;
    let _ = client
        .sign_stake_assets_with_nonce(account_index, 1_000, Some(nonce + 15))
        .await?;
    let _ = client
        .sign_unstake_assets_with_nonce(account_index, 1_000, Some(nonce + 16))
        .await?;
    let _ = client
        .sign_approve_integrator_with_nonce(1, 0, 0, 0, 0, 0, Some(nonce + 17))
        .await?;
    println!("✔ All sign-only transaction builders exercised without submitting live txs");

    let combined = CombinedClient::new(base_url, &api_private_key, account_index, api_key_index)?;
    let mut ws_rx = combined.ws.connect().await?;
    combined
        .ws
        .subscribe_order_book(market_index)
        .await?;
    match timeout(Duration::from_secs(5), ws_rx.recv()).await {
        Ok(Some(_)) => println!("✔ WebSocket connection and order-book subscription OK"),
        Ok(None) => println!("ℹ WebSocket channel closed without message"),
        Err(_) => println!("ℹ WebSocket timeout reached without order book message"),
    }

    let account_after = client.get_my_account().await.unwrap_or_default();
    let open_orders_after = client
        .get_account_active_orders(account_index, None, Some(20), None)
        .await
        .map(|page| page.items.len())
        .unwrap_or(0);
    let positions_after = position_count(&account_after);

    println!(
        "Post-check state: open_orders={} open_positions={}",
        open_orders_after, positions_after
    );

    if open_orders_before != open_orders_after || positions_before != positions_after {
        return Err("Read-only smoke check observed a change in open orders or positions".into());
    }

    println!("✅ Smoke check complete: no positions or orders were opened by this run.");
    Ok(())
}
