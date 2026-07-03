/// Integration tests for the `api-client` crate.
///
/// These tests run against the live testnet by default.  They require the
/// following environment variables (best set via `.env`):
///
/// | Variable          | Example                                        |
/// |-------------------|------------------------------------------------|
/// | `BASE_URL`        | `https://testnet.zklighter.elliot.ai`          |
/// | `ACCOUNT_INDEX`   | `42`                                           |
/// | `API_KEY_INDEX`   | `0`                                            |
/// | `API_PRIVATE_KEY` | `0x…64-hex-chars…`                             |
///
/// All tests are gated behind `#[ignore]` – run them explicitly with:
///
/// ```bash
/// cargo test --package api-client -- --ignored --nocapture
/// ```
#[cfg(test)]
mod tests {
    use api_client::{LighterClient, Page};

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

    // ─── Nonce ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_nonce() {
        let (c, _) = client();
        let nonce = c.get_nonce().await.expect("get_nonce failed");
        assert!(nonce >= 0, "nonce must be non-negative");
    }

    // ─── Account ────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_my_account() {
        let (c, _) = client();
        let acc = c.get_my_account().await.expect("get_my_account failed");
        assert!(acc.account_index >= 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_account_limits() {
        let (c, idx) = client();
        let limits = c
            .get_account_limits(idx)
            .await
            .expect("get_account_limits failed");
        assert_eq!(limits.account_index, idx);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_account_metadata() {
        let (c, idx) = client();
        let meta = c
            .get_account_metadata(idx)
            .await
            .expect("get_account_metadata failed");
        assert_eq!(meta.account_index, idx);
    }

    // ─── Orders ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_active_orders() {
        let (c, idx) = client();
        let page = c
            .get_account_active_orders(idx, None, Some(10), None)
            .await
            .expect("get_account_active_orders failed");
        // Just assert the call succeeds; there may be 0 open orders.
        let _ = page.items.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_inactive_orders() {
        let (c, idx) = client();
        let page = c
            .get_account_inactive_orders(idx, None, Some(5), None)
            .await
            .expect("get_account_inactive_orders failed");
        let _ = page.items.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_active_orders_pagination() {
        let (c, idx) = client();
        // Fetch page 1 with limit=2
        let page1: Page<_> = c
            .get_account_active_orders(idx, None, Some(2), None)
            .await
            .expect("page 1 failed");

        if let Some(cursor) = page1.cursor.as_ref().and_then(|cur| cur.next.as_deref()) {
            // Fetch page 2 using the cursor
            let page2 = c
                .get_account_active_orders(idx, None, Some(2), Some(cursor))
                .await
                .expect("page 2 failed");
            let _ = page2.items.len();
        }
    }

    // ─── Market data ────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_order_book() {
        let (c, _) = client();
        // Market 0 should always exist on testnet.
        let ob = c.get_order_book(0).await.expect("get_order_book failed");
        assert_eq!(ob.market_index, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_order_book_details() {
        let (c, _) = client();
        let details = c
            .get_order_book_details(0)
            .await
            .expect("get_order_book_details failed");
        assert_eq!(details.market_index, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_recent_trades() {
        let (c, _) = client();
        let trades = c
            .get_recent_trades(0, Some(5))
            .await
            .expect("get_recent_trades failed");
        let _ = trades.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_candles() {
        let (c, _) = client();
        // 1-minute candles, last 60 bars
        let candles = c
            .get_candles(0, 60, None, None, Some(60), None)
            .await
            .expect("get_candles failed");
        for candle in &candles {
            assert!(!candle.open.is_empty(), "open price must not be empty");
        }
    }

    // ─── Funding ────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_funding_rates() {
        let (c, _) = client();
        let page = c
            .get_funding_rates(0, Some(5), None)
            .await
            .expect("get_funding_rates failed");
        let _ = page.items.len();
    }

    // ─── History ────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_deposit_history() {
        let (c, idx) = client();
        let page = c
            .get_deposit_history(idx, Some(5), None)
            .await
            .expect("get_deposit_history failed");
        let _ = page.items.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_withdraw_history() {
        let (c, idx) = client();
        let page = c
            .get_withdraw_history(idx, Some(5), None)
            .await
            .expect("get_withdraw_history failed");
        let _ = page.items.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_transfer_history() {
        let (c, idx) = client();
        let page = c
            .get_transfer_history(idx, Some(5), None)
            .await
            .expect("get_transfer_history failed");
        let _ = page.items.len();
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_account_transactions() {
        let (c, idx) = client();
        let page = c
            .get_account_transactions(idx, Some(5), None)
            .await
            .expect("get_account_transactions failed");
        let _ = page.items.len();
    }

    // ─── Exchange / assets ──────────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn test_get_exchange_stats() {
        let (c, _) = client();
        let _stats = c
            .get_exchange_stats()
            .await
            .expect("get_exchange_stats failed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_asset_details() {
        let (c, _) = client();
        let _asset = c
            .get_asset_details(0)
            .await
            .expect("get_asset_details failed");
    }

    // ─── Auth token ─────────────────────────────────────────────────────────────

    #[test]
    fn test_create_auth_token() {
        // Does not require network access.
        let base_url = "https://testnet.zklighter.elliot.ai".to_string();
        // Use a dummy key with the expected 40-byte / 80-hex-char length.
        let dummy_key =
            "00000000000000000000000000000000000000000000000000000000000000000000000000000001";
        let c = LighterClient::new(base_url, dummy_key, 1, 0)
            .expect("Failed to build client with dummy key");
        let token = c.create_auth_token(3600).expect("create_auth_token failed");
        assert!(!token.is_empty());
    }

    // ─── Crypto helpers ─────────────────────────────────────────────────────────

    #[test]
    fn test_generate_random_nonce() {
        let n1 = LighterClient::generate_random_nonce();
        let n2 = LighterClient::generate_random_nonce();
        // Should be different with overwhelming probability.
        assert_ne!(n1, n2, "Two random nonces should differ");
    }
}
