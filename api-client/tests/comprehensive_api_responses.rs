/// Comprehensive API Response Capture Test
/// Runs all read and write operations, capturing real API responses and transaction hashes.
/// Run with: cargo test --package api-client --test comprehensive_api_responses -- --ignored --nocapture
#[cfg(test)]
mod tests {
    use api_client::{CreateOrderRequest, LighterClient};

    #[derive(Debug)]
    struct ApiResponse {
        endpoint: String,
        status: String,
        data: String,
    }

    fn client() -> (LighterClient, i64) {
        dotenv::dotenv().ok();
        let base_url = std::env::var("BASE_URL")
            .unwrap_or_else(|_| "https://testnet.zklighter.elliot.ai".to_string());
        let account_index: i64 = std::env::var("ACCOUNT_INDEX")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .expect("ACCOUNT_INDEX must be an integer");
        let api_key_index: u8 = std::env::var("API_KEY_INDEX")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .expect("API_KEY_INDEX must be u8");
        let private_key = std::env::var("API_PRIVATE_KEY").expect("API_PRIVATE_KEY must be set");

        let c = LighterClient::new(base_url, &private_key, account_index, api_key_index)
            .expect("Failed to build LighterClient");
        (c, account_index)
    }

    #[tokio::test]
    #[ignore]
    async fn test_comprehensive_api_responses() {
        let (c, account_index) = client();
        let mut results = Vec::new();

        println!("\n{}", "=".repeat(120));
        println!("📊 COMPREHENSIVE API ENDPOINT RESPONSE TEST");
        println!("{}", "=".repeat(120));
        println!();

        // ─── READ OPERATIONS ────────────────────────────────────────────

        println!("📖 READ OPERATIONS (Retrieving data from API)");
        println!("{}", "-".repeat(120));

        // 1. Get Nonce
        println!("1️⃣  GET /api/v1/nonce");
        match c.get_nonce().await {
            Ok(nonce) => {
                println!("   ✅ Response: nonce = {}", nonce);
                results.push(ApiResponse {
                    endpoint: "GET_NONCE".to_string(),
                    status: "200".to_string(),
                    data: format!("nonce: {}", nonce),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_NONCE".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 2. Get Account
        println!("2️⃣  GET /api/v1/account");
        match c.get_my_account().await {
            Ok(account) => {
                let acc_json = serde_json::to_string_pretty(&account).unwrap_or_default();
                println!("   ✅ Response:\n{}", indent(&acc_json));
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT".to_string(),
                    status: "200".to_string(),
                    data: "account_index, equity, etc.".to_string(),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 3. Get Account Limits
        println!("3️⃣  GET /api/v1/accountLimits");
        match c.get_account_limits(account_index).await {
            Ok(limits) => {
                let json = serde_json::to_string_pretty(&limits).unwrap_or_default();
                println!("   ✅ Response:\n{}", indent(&json));
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT_LIMITS".to_string(),
                    status: "200".to_string(),
                    data: "limits, leverage, etc.".to_string(),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT_LIMITS".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 4. Get Account Metadata
        println!("4️⃣  GET /api/v1/accountMetadata");
        match c.get_account_metadata(account_index).await {
            Ok(meta) => {
                let json = serde_json::to_string_pretty(&meta).unwrap_or_default();
                println!("   ✅ Response:\n{}", indent(&json));
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT_METADATA".to_string(),
                    status: "200".to_string(),
                    data: "metadata fields".to_string(),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_ACCOUNT_METADATA".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 5. Get Order Book
        println!("5️⃣  GET /api/v1/orderBooks/:market_index");
        match c.get_order_book(0).await {
            Ok(ob) => {
                let json = serde_json::to_string_pretty(&ob).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_ORDER_BOOK".to_string(),
                    status: "200".to_string(),
                    data: "bids, asks, market_index".to_string(),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_ORDER_BOOK".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 6. Get Recent Trades
        println!("6️⃣  GET /api/v1/trades/:market_index");
        match c.get_recent_trades(0, Some(3)).await {
            Ok(trades) => {
                let json = serde_json::to_string_pretty(&trades).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_RECENT_TRADES".to_string(),
                    status: "200".to_string(),
                    data: format!("{} trades", trades.len()),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_RECENT_TRADES".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 7. Get Candles
        println!("7️⃣  GET /api/v1/candles/:market_index");
        match c.get_candles(0, 60, None, None, Some(5), None).await {
            Ok(candles) => {
                let json = serde_json::to_string_pretty(&candles).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_CANDLES".to_string(),
                    status: "200".to_string(),
                    data: format!("{} candles", candles.len()),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_CANDLES".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 8. Get Funding Rates
        println!("8️⃣  GET /api/v1/funding-rates/:market_index");
        match c.get_funding_rates(0, Some(3), None).await {
            Ok(page) => {
                let json = serde_json::to_string_pretty(&page).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_FUNDING_RATES".to_string(),
                    status: "200".to_string(),
                    data: format!("{} items", page.items.len()),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_FUNDING_RATES".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 9. Get Active Orders
        println!("9️⃣  GET /api/v1/accountOrders (active)");
        match c
            .get_account_active_orders(account_index, None, Some(3), None)
            .await
        {
            Ok(page) => {
                let json = serde_json::to_string_pretty(&page).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_ACTIVE_ORDERS".to_string(),
                    status: "200".to_string(),
                    data: format!("{} orders", page.items.len()),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_ACTIVE_ORDERS".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 10. Get Deposit History
        println!("🔟 GET /api/v1/depositHistory");
        match c.get_deposit_history(account_index, Some(3), None).await {
            Ok(page) => {
                let json = serde_json::to_string_pretty(&page).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_DEPOSIT_HISTORY".to_string(),
                    status: "200".to_string(),
                    data: format!("{} deposits", page.items.len()),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_DEPOSIT_HISTORY".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // 11. Get Exchange Stats
        println!("1️⃣1️⃣  GET /api/v1/exchange-stats");
        match c.get_exchange_stats().await {
            Ok(stats) => {
                let json = serde_json::to_string_pretty(&stats).unwrap_or_default();
                println!(
                    "   ✅ Response (first 500 chars):\n{}",
                    indent(&json[..json.len().min(500)])
                );
                results.push(ApiResponse {
                    endpoint: "GET_EXCHANGE_STATS".to_string(),
                    status: "200".to_string(),
                    data: "stats, volumes, etc.".to_string(),
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "GET_EXCHANGE_STATS".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // ─── WRITE OPERATIONS ────────────────────────────────────────────

        println!();
        println!("{}", "=".repeat(120));
        println!("✍️  WRITE OPERATIONS (Creating transactions on-chain)");
        println!("{}", "-".repeat(120));

        // 12. Create Limit Order
        println!("1️⃣  POST /api/v1/createOrder (limit order)");
        let client_order_index = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let order = CreateOrderRequest {
            account_index,
            order_book_index: 0,
            client_order_index,
            base_amount: 100, // Small amount
            price: 203900,    // Market price ~2039 (within realistic range for ETH-USD)
            is_ask: false,
            order_type: 0,
            time_in_force: 1,
            reduce_only: false,
            trigger_price: 0,
        };

        match c.create_order(order).await {
            Ok(response) => {
                let response_json = serde_json::to_string_pretty(&response).unwrap_or_default();
                println!("   ✅ Response:\n{}", indent(&response_json));

                let tx_hash = if let Some(tx) = response.get("tx_hash").and_then(|v| v.as_str()) {
                    tx.to_string()
                } else {
                    "No tx_hash in response".to_string()
                };
                println!("   📝 Transaction Hash: {}", tx_hash);
                results.push(ApiResponse {
                    endpoint: "CREATE_ORDER".to_string(),
                    status: "200".to_string(),
                    data: tx_hash,
                });
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.push(ApiResponse {
                    endpoint: "CREATE_ORDER".to_string(),
                    status: "ERROR".to_string(),
                    data: e.to_string(),
                });
            }
        }
        println!();

        // ─── SUMMARY TABLE ──────────────────────────────────────────────

        println!();
        println!("{}", "=".repeat(120));
        println!("📊 SUMMARY TABLE: API ENDPOINTS & RESPONSES");
        println!("{}", "=".repeat(120));
        println!();

        print_table(&results);
    }

    fn indent(s: &str) -> String {
        s.lines()
            .map(|line| format!("      {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn print_table(results: &[ApiResponse]) {
        // Table headers
        println!(
            "{:<25} {:<12} {:<83}",
            "ENDPOINT", "STATUS", "RESPONSE / TX_HASH"
        );
        println!("{}", "-".repeat(120));

        // Table rows
        for result in results {
            let truncated = if result.data.len() > 82 {
                format!("{}...", &result.data[..79])
            } else {
                result.data.clone()
            };

            println!(
                "{:<25} {:<12} {:<83}",
                result.endpoint, result.status, truncated
            );
        }

        println!();
        println!("Legend:");
        println!("  - STATUS: HTTP code (200=OK), ERROR, or SUBMITTED");
        println!("  - RESPONSE: First response object or tx_hash for write operations");
        println!();
    }
}
