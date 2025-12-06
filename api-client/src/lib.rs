use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use base64::Engine;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Signer error: {0}")]
    Signer(#[from] signer::SignerError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("API error: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateOrderRequest {
    pub account_index: i64,
    pub order_book_index: u8,
    pub client_order_index: u64,
    pub base_amount: i64,
    pub price: i64,
    pub is_ask: bool,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: bool,
    pub trigger_price: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    pub to_account_index: i64,
    pub usdc_amount: i64,
    pub fee: i64,
    pub memo: [u8; 32],
}

#[derive(Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub usdc_amount: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ModifyOrderRequest {
    pub market_index: u8,
    pub order_index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub trigger_price: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGroupedOrdersRequest {
    pub grouping_type: u8,
    pub orders: Vec<CreateOrderRequest>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePublicPoolRequest {
    pub operator_fee: i64,
    pub initial_total_shares: i64,
    pub min_operator_share_rate: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePublicPoolRequest {
    pub public_pool_index: i64,
    pub status: u8,
    pub operator_fee: i64,
    pub min_operator_share_rate: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MintSharesRequest {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BurnSharesRequest {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateMarginRequest {
    pub market_index: u8,
    pub usdc_amount: i64,
    pub direction: u8, // 0 = RemoveFromIsolatedMargin, 1 = AddToIsolatedMargin
}

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use rand::RngCore;
use tokio::sync::Mutex as AsyncMutex;

pub struct LighterClient {
    client: Client,
    base_url: String,
    key_manager: KeyManager,
    account_index: i64,
    api_key_index: u8,
    // OPTIMIZED: Use atomic for fast nonce increment (lock-free for common case)
    // last_fetched_nonce: base nonce from API (nonce - 1)
    // nonce_offset: atomic counter for incrementing
    // is_fetching: atomic flag to prevent multiple concurrent fetches
    last_fetched_nonce: Arc<AtomicI64>,
    nonce_offset: Arc<AtomicI64>,
    is_fetching: Arc<AtomicI64>, // -1 = not fetching, otherwise = fetched nonce (temporary storage)
    // Mutex only for initialization (rare case)
    nonce_init_lock: Arc<AsyncMutex<()>>,
    // Hybrid approach: Track last API fetch time to periodically refresh
    last_api_fetch_time: Arc<AtomicI64>, // Unix timestamp in milliseconds
    // OPTIMIZED: Cache frequently used URLs to avoid format! overhead
    send_tx_url: String,
    next_nonce_url: String,
    // Retry configuration
    enable_retry: bool,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl LighterClient {
    pub fn new(
        base_url: String,
        private_key_hex: &str,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<Self> {
        Self::new_with_retry(base_url, private_key_hex, account_index, api_key_index, false, 5, 3000)
    }
    
    /// Create a new LighterClient with optional retry configuration
    /// 
    /// # Arguments
    /// * `base_url` - Base URL of the API
    /// * `private_key_hex` - Private key in hex format
    /// * `account_index` - Account index
    /// * `api_key_index` - API key index
    /// * `enable_retry` - Enable automatic retry on 21120 errors (default: false for HFT performance)
    /// * `max_retries` - Maximum number of retries (default: 5)
    /// * `retry_delay_ms` - Delay between retries in milliseconds (default: 3000)
    /// 
    /// # Examples
    /// ```rust,no_run
    /// // Fast path (no retry) - for HFT scenarios
    /// let client = LighterClient::new_with_retry(
    ///     "https://testnet.zklighter.elliot.ai".to_string(),
    ///     "private_key_hex",
    ///     271,
    ///     4,
    ///     false, // No retry - maximum speed
    ///     5,
    ///     3000,
    /// )?;
    /// 
    /// // Reliable path (with retry) - for production scenarios
    /// let client = LighterClient::new_with_retry(
    ///     "https://testnet.zklighter.elliot.ai".to_string(),
    ///     "private_key_hex",
    ///     271,
    ///     4,
    ///     true, // Enable retry - maximum reliability
    ///     5,
    ///     3000,
    /// )?;
    /// ```
    pub fn new_with_retry(
        base_url: String,
        private_key_hex: &str,
        account_index: i64,
        api_key_index: u8,
        enable_retry: bool,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<Self> {
        let key_manager = KeyManager::from_hex(private_key_hex)?;
        // ULTRA-OPTIMIZED: HTTP client configuration for maximum HFT performance
        // HTTP client configuration optimized for high-frequency trading:
        // - Minimal timeouts for fastest failure detection
        // - Maximum connection pool for extreme parallelism
        // - Aggressive keep-alive settings
        // - HTTP/1.1 with optimizations (HTTP/2 has overhead for small requests)
        let client = Client::builder()
            .user_agent("Lighter-Rust-Client/1.0")
            .timeout(std::time::Duration::from_secs(3)) // Ultra-aggressive timeout
            .connect_timeout(std::time::Duration::from_secs(1)) // Very fast connection timeout
            .tcp_keepalive(std::time::Duration::from_secs(30)) // Shorter keep-alive
            .pool_max_idle_per_host(200) // Massive pool for extreme concurrency
            .pool_idle_timeout(std::time::Duration::from_secs(10)) // Shorter idle timeout
            .http1_title_case_headers() // Performance improvement
            .http1_allow_obsolete_multiline_headers_in_responses(true)
            .build()?;
        
        // OPTIMIZED: Pre-compute URLs to avoid format! overhead in hot path
        let send_tx_url = format!("{}/api/v1/sendTx", &base_url);
        let next_nonce_url = format!("{}/api/v1/nextNonce?account_index={}&api_key_index={}", &base_url, account_index, api_key_index);
        
        Ok(Self {
            client,
            base_url,
            key_manager,
            account_index,
            api_key_index,
            // Initialize with i64::MIN (uninitialized sentinel) and offset 0
            // Use i64::MIN instead of -1 to handle nonce=0 case (where base would be -1)
            last_fetched_nonce: Arc::new(AtomicI64::new(i64::MIN)),
            nonce_offset: Arc::new(AtomicI64::new(0)),
            is_fetching: Arc::new(AtomicI64::new(i64::MIN)), // i64::MIN = not fetching
            nonce_init_lock: Arc::new(AsyncMutex::new(())),
            last_api_fetch_time: Arc::new(AtomicI64::new(0)), // 0 = never fetched
            send_tx_url,
            next_nonce_url,
            enable_retry,
            max_retries,
            retry_delay_ms,
        })
    }
    
    /// Create order using internal nonce cache (optimistic nonce management)
    /// 
    /// # Nonce Management
    /// - If cache is initialized: Uses cached nonce and increments atomically (no API call)
    /// - If cache is empty: Automatically fetches from `/api/v1/nextNonce` API endpoint
    /// - After first fetch: Stores as (nonce - 1) and increments locally for subsequent calls
    /// 
    /// This provides optimal performance: first call fetches from API, subsequent calls use cache.
    pub async fn create_order(&self, order: CreateOrderRequest) -> Result<Value> {
        if self.enable_retry {
            self.create_order_with_nonce(order, None).await
        } else {
            self.create_order_internal(&order, None).await
        }
    }
    
    /// Create order with explicit nonce - NO retry logic, NO delays
    /// Use this for maximum performance when managing nonces externally
    #[inline]
    pub async fn create_order_direct(&self, order: CreateOrderRequest, nonce: i64) -> Result<Value> {
        self.create_order_internal(&order, Some(nonce)).await
    }
    
    /// Create order with optional nonce parameter and retry logic
    /// If nonce is Some(n), uses that nonce (or -1 to fetch from API)
    /// If nonce is None, uses optimistic nonce management
    /// Automatically retries on invalid signature errors (21120)
    pub async fn create_order_with_nonce(&self, order: CreateOrderRequest, nonce: Option<i64>) -> Result<Value> {
        let max_retries = self.max_retries;
        let retry_delay_ms = self.retry_delay_ms;
        
        let mut current_nonce = self.get_nonce_or_use(nonce).await?;
        let mut last_error: Option<ApiError> = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
                match self.fetch_nonce_from_api().await {
                    Ok(fresh_nonce) => {
                        current_nonce = fresh_nonce;
                        // Reset cache with fresh nonce
                        self.last_fetched_nonce.store(fresh_nonce - 1, Ordering::Release);
                        self.nonce_offset.store(0, Ordering::Release);
                    }
                    Err(_) => {}
                }
            }
            
            match self.create_order_internal(&order, Some(current_nonce)).await {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    if code == 200 {
                        return Ok(response);
                    } else if code == 21120 && attempt < max_retries {
                        last_error = Some(ApiError::Api(format!("Invalid signature (code 21120) after {} attempts", attempt + 1)));
                        continue;
                    } else {
                        // On failure, decrement offset (nonce wasn't used)
                        let offset = self.nonce_offset.load(Ordering::Acquire);
                        if offset > 0 {
                            self.nonce_offset.store(offset - 1, Ordering::Release);
                        }
                        return Ok(response);
                    }
                }
                Err(e) => {
                    if attempt < max_retries {
                        last_error = Some(e);
                        continue;
                    } else {
                        // On failure, decrement offset (nonce wasn't used)
                        let offset = self.nonce_offset.load(Ordering::Acquire);
                        if offset > 0 {
                            self.nonce_offset.store(offset - 1, Ordering::Release);
                        }
                        return Err(e);
                    }
                }
            }
        }
        
        // On failure, decrement offset (nonce wasn't used)
        let offset = self.nonce_offset.load(Ordering::Acquire);
        if offset > 0 {
            self.nonce_offset.store(offset - 1, Ordering::Release);
        }
        Err(last_error.unwrap_or_else(|| ApiError::Api("Failed after all retries".to_string())))
    }
    
    async fn create_order_internal(&self, order: &CreateOrderRequest, nonce: Option<i64>) -> Result<Value> {
        // CRITICAL: Track if we used cached nonce (need to handle failures)
        let used_cached_nonce = nonce.is_none();
        
        // CRITICAL: Get nonce ONCE at the very beginning and lock it in
        // This prevents ANY possibility of nonce changing between JSON construction and signing
        // The nonce is fetched/calculated atomically and then used consistently throughout
        let nonce = if let Some(n) = nonce {
            // User provided explicit nonce - use it directly (bypasses internal cache)
            n
        } else {
            // No nonce provided - use internal cache system
            // This will:
            // 1. If cache is initialized: Return cached nonce and increment atomically (FAST - no API call)
            // 2. If cache is empty: Automatically fetch from /api/v1/nextNonce API endpoint (SLOW - API call)
            //    Then initialize cache and return the fetched nonce
            self.get_next_nonce_from_cache().await?
        };
        
        // Log nonce for tracing (only in debug builds to avoid performance impact)
        #[cfg(debug_assertions)]
        eprintln!("create_order_internal: Using nonce {} (cached: {})", nonce, used_cached_nonce);
        
        // OPTIMIZED: Capture timestamp once to ensure consistency
        // Use checked arithmetic to avoid potential overflow
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ApiError::Api("System time error".to_string()))?
            .as_millis() as i64;
        let expired_at = now.saturating_add(599_000); // Use saturating_add for safety
        
        let order_expiry = if order.time_in_force == 1 && order.order_type == 0 {
            now + (28 * 24 * 60 * 60 * 1000)
        } else {
            0
        };
        
        let price_u32 = order.price as u32;
        let trigger_price_u32 = order.trigger_price as u32;
        if order.price != price_u32 as i64 {
            return Err(ApiError::Api(format!("Price {} exceeds u32::MAX", order.price)));
        }
        if order.trigger_price != trigger_price_u32 as i64 {
            return Err(ApiError::Api(format!("TriggerPrice {} exceeds u32::MAX", order.trigger_price)));
        }
        
        // CRITICAL: Build transaction JSON with the EXACT nonce we got above
        // No async operations between nonce fetch and JSON construction
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"ClientOrderIndex":{},"BaseAmount":{},"Price":{},"IsAsk":{},"Type":{},"TimeInForce":{},"ReduceOnly":{},"TriggerPrice":{},"OrderExpiry":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            order.order_book_index,
            order.client_order_index,
            order.base_amount,
            price_u32,
            if order.is_ask { 1u8 } else { 0u8 },
            order.order_type,
            order.time_in_force,
            if order.reduce_only { 1u8 } else { 0u8 },
            trigger_price_u32,
            order_expiry,
            expired_at,
            nonce  // Use the SAME nonce we fetched above
        );

        // CRITICAL: Validate nonce in JSON matches what we expect
        // Parse JSON to verify nonce is correctly embedded
        #[cfg(debug_assertions)]
        {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tx_json) {
                let json_nonce = parsed["Nonce"].as_i64().unwrap_or(-1);
                if json_nonce != nonce {
                    eprintln!("Nonce mismatch! Expected {}, but JSON has {}", nonce, json_nonce);
                }
            }
        }

        // CRITICAL: Sign immediately using the EXACT JSON string above
        // The signature MUST be computed over the JSON that contains the same nonce
        // No async operations, no nonce changes possible
        
        // Log the exact JSON being signed (only in debug builds)
        #[cfg(debug_assertions)]
        {
            eprintln!("create_order_internal: JSON being signed: {}", tx_json);
            // Parse and verify all critical fields
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tx_json) {
                eprintln!("create_order_internal: Parsed JSON fields:");
                eprintln!("  AccountIndex: {:?}", parsed["AccountIndex"].as_i64());
                eprintln!("  ApiKeyIndex: {:?}", parsed["ApiKeyIndex"].as_u64().or_else(|| parsed["ApiKeyIndex"].as_i64().map(|v| v as u64)));
                eprintln!("  MarketIndex: {:?}", parsed["MarketIndex"].as_u64().or_else(|| parsed["MarketIndex"].as_i64().map(|v| v as u64)));
                eprintln!("  Nonce: {:?}", parsed["Nonce"].as_i64());
                eprintln!("  ExpiredAt: {:?}", parsed["ExpiredAt"].as_i64());
                eprintln!("  Price: {:?}", parsed["Price"].as_u64().or_else(|| parsed["Price"].as_i64().map(|v| v as u64)));
                eprintln!("  IsAsk: {:?}", parsed["IsAsk"].as_u64().or_else(|| parsed["IsAsk"].as_i64().map(|v| v as u64)));
            }
        }
        
        let signature = self.sign_transaction(&tx_json)?;
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);
        
        #[cfg(debug_assertions)]
        {
            eprintln!("create_order_internal: Signature (first 20 bytes): {:?}", &signature[..20.min(signature.len())]);
        }

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"ClientOrderIndex":{},"BaseAmount":{},"Price":{},"IsAsk":{},"Type":{},"TimeInForce":{},"ReduceOnly":{},"TriggerPrice":{},"OrderExpiry":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            order.order_book_index,
            order.client_order_index,
            order.base_amount,
            price_u32,
            if order.is_ask { 1u8 } else { 0u8 },
            order.order_type,
            order.time_in_force,
            if order.reduce_only { 1u8 } else { 0u8 },
            trigger_price_u32,
            order_expiry,
            expired_at,
            nonce,
            sig_base64
        );

        // OPTIMIZED: Use cached URL (no format! overhead)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "14")
            .text("tx_info", final_tx_json); // Remove clone - we don't need it after this
        
        let response = self
            .client
            .post(&self.send_tx_url)
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;
        
        let status = response.status();
        let response_text = response.text().await?;
        
        // OPTIMIZED: Fast path for empty responses (avoid trim allocation)
        if response_text.is_empty() {
            return Err(ApiError::Api(format!(
                "Empty response from API (status: {})",
                status
            )));
        }
        
        // OPTIMIZED: Parse JSON with minimal error allocation
        let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            // Only allocate error string if parsing fails
            let error_preview = if response_text.len() > 200 {
                format!("{}...", &response_text[..200])
            } else {
                response_text.clone()
            };
            ApiError::Api(format!(
                "JSON parse error (status: {}): {}. Response: {}",
                status, e, error_preview
            ))
        })?;
        
        // CRITICAL: Handle failures - if transaction failed and we used cached nonce,
        // we need to handle the nonce appropriately
        let code = response_json["code"].as_i64().unwrap_or_default();
        if code != 200 && used_cached_nonce {
            // Transaction failed - check error type
            if code == 21104 {
                // Invalid nonce - our cache is out of sync with server
                // Refresh cache immediately to prevent further errors
                if let Ok(fresh_nonce) = self.fetch_nonce_from_api().await {
                    self.last_fetched_nonce.store(fresh_nonce - 1, Ordering::Release);
                    self.nonce_offset.store(0, Ordering::Release);
                    self.is_fetching.store(i64::MIN, Ordering::Release); // Reset fetching flag
                }
            }
            // For 21120 (invalid signature) and other errors, don't modify cache
            // The nonce might have been consumed by server even if transaction failed
            // Only refresh on explicit nonce errors (21104)
        }
        
        Ok(response_json)
    }

    pub async fn create_market_order(
        &self,
        order_book_index: u8,
        client_order_index: u64,
        base_amount: i64,
        avg_execution_price: i64,
        is_ask: bool,
    ) -> Result<Value> {
        self.create_market_order_with_nonce(
            order_book_index,
            client_order_index,
            base_amount,
            avg_execution_price,
            is_ask,
            None,
        ).await
    }
    
    /// Create market order with optional nonce parameter
    pub async fn create_market_order_with_nonce(
        &self,
        order_book_index: u8,
        client_order_index: u64,
        base_amount: i64,
        avg_execution_price: i64,
        is_ask: bool,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let order = CreateOrderRequest {
            account_index: self.account_index,
            order_book_index,
            client_order_index,
            base_amount,
            price: avg_execution_price,
            is_ask,
            order_type: 1, // MarketOrder
            time_in_force: 0, // ImmediateOrCancel
            reduce_only: false,
            trigger_price: 0,
        };
        self.create_order_with_nonce(order, nonce).await
    }

    pub async fn cancel_order(&self, order_book_index: u8, order_index: i64) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"Index":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            order_book_index,
            order_index,
            expired_at,
            nonce
        );

        let signature = self.sign_transaction_with_type(&tx_json, 15)?;
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"Index":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            order_book_index,
            order_index,
            expired_at,
            nonce,
            sig_base64
        );

        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "15")
            .text("tx_info", final_tx_json.clone());

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    pub async fn cancel_all_orders(&self, time_in_force: u8, time: i64) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, TimeInForce, Time, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"TimeInForce":{},"Time":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            time_in_force,
            time,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 16)?; // TX_TYPE_CANCEL_ALL_ORDERS

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"TimeInForce":{},"Time":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            time_in_force,
            time,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "16") // CANCEL_ALL_ORDERS
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Close a position in a specific market
    /// 
    /// Creates a market order with reduce_only=true to close the position.
    /// Use this to close a position when you know the market and direction.
    /// 
    /// # Arguments
    /// * `market_index` - Market index (0-based)
    /// * `is_ask` - true to close long position (sell), false to close short position (buy)
    /// 
    /// # Returns
    /// JSON response from the API
    pub async fn close_position(&self, market_index: u8, is_ask: bool) -> Result<Value> {
        let order = CreateOrderRequest {
            account_index: self.account_index,
            order_book_index: market_index,
            client_order_index: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as u64,
            base_amount: i64::MAX / 2, // Large amount to ensure position is closed
            price: 0, // Market order
            is_ask,
            order_type: 1, // Market order
            time_in_force: 0, // ImmediateOrCancel
            reduce_only: true, // Only reduce position
            trigger_price: 0,
        };
        
        self.create_order(order).await
    }
    
    /// Get account information including positions
    /// 
    /// # Returns
    /// JSON response with account details including positions
    pub async fn get_account(&self) -> Result<Value> {
        let auth_token = self.create_auth_token(600)?; // 10 minute expiry
        let account_index_str = self.account_index.to_string();
        
        let response = self
            .client
            .get(&format!("{}/api/v1/account", self.base_url))
            .query(&[("by", "index"), ("value", &account_index_str)])
            .header("Authorization", &auth_token)
            .header("Auth", &auth_token)
            .send()
            .await?;
        
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        
        Ok(response_json)
    }
    
    /// Get API keys for an account
    /// 
    /// # Arguments
    /// * `account_index` - Account index to query
    /// * `api_key_index` - Optional API key index (use 255 to get all API keys)
    /// 
    /// # Returns
    /// JSON response with API keys information
    pub async fn get_api_keys(&self, account_index: i64, api_key_index: Option<u8>) -> Result<Value> {
        let auth_token = self.create_auth_token(600)?; // 10 minute expiry
        let account_index_str = account_index.to_string();
        
        let mut query_params = vec![("account_index", account_index_str)];
        if let Some(idx) = api_key_index {
            query_params.push(("api_key_index", idx.to_string()));
        }
        
        let response = self
            .client
            .get(&format!("{}/api/v1/apikeys", self.base_url))
            .query(&query_params)
            .header("Authorization", &auth_token)
            .header("Auth", &auth_token)
            .send()
            .await?;
        
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        
        Ok(response_json)
    }
    
    /// Close all positions by querying account first
    /// 
    /// This method queries the account to find open positions, then closes them.
    /// More efficient than close_all_positions() as it only closes positions that exist.
    /// 
    /// # Returns
    /// JSON response with results for each closed position
    pub async fn close_all_positions_auto(&self) -> Result<Value> {
        // First, get account info to see what positions exist
        let account_info = self.get_account().await?;
        
        // Account API returns: { "accounts": [...], "code": 200, "total": 1 }
        // Extract the first account from the accounts array
        let account_data = if let Some(accounts_array) = account_info.get("accounts").and_then(|a| a.as_array()) {
            // Get first account from accounts array
            accounts_array.first()
        } else if account_info.is_array() {
            // Fallback: if root is array, get first element
            account_info.as_array().and_then(|a| a.first())
        } else {
            // Single account object
            Some(&account_info)
        };
        
        // Extract positions from the account
        let empty_vec: Vec<Value> = Vec::new();
        let positions = account_data
            .and_then(|acc| acc.get("positions"))
            .or_else(|| account_data.and_then(|acc| acc.get("Positions")))
            .and_then(|p| p.as_array())
            .unwrap_or(&empty_vec);
        
        let mut results = Vec::new();
        
        for position in positions {
            // API uses "market_id" (snake_case), not "market_index"
            let market_index = position.get("market_id")
                .or_else(|| position.get("marketIndex"))
                .or_else(|| position.get("market_index"))
                .or_else(|| position.get("marketId"))
                .and_then(|m| m.as_u64().or_else(|| m.as_i64().map(|v| v as u64)))
                .map(|v| v as u8);
            
            if let Some(market_index) = market_index {
                // Get position sign: 1 = Long, -1 = Short
                let sign = position.get("sign")
                    .or_else(|| position.get("Sign"))
                    .and_then(|s| s.as_i64())
                    .unwrap_or(0);
                
                // Get position amount - try multiple formats (string or number)
                let position_amount = position.get("position")
                    .or_else(|| position.get("Position"))
                    .and_then(|p| {
                        if let Some(s) = p.as_str() {
                            s.parse::<f64>().ok()
                        } else {
                            p.as_f64().or_else(|| p.as_i64().map(|v| v as f64))
                        }
                    })
                    .unwrap_or(0.0);
                
                // Only close if position exists (non-zero)
                if position_amount.abs() > 0.0001 {
                    // sign = 1 means long position, close by selling (is_ask = true)
                    // sign = -1 means short position, close by buying (is_ask = false)
                    let is_ask = sign > 0;
                    
                    match self.close_position(market_index, is_ask).await {
                        Ok(response) => {
                            let code = response["code"].as_i64().unwrap_or_default();
                            results.push(json!({
                                "market_index": market_index,
                                "direction": if sign > 0 { "long" } else { "short" },
                                "position_amount": position_amount,
                                "status": if code == 200 { "success" } else { "failed" },
                                "code": code,
                                "response": response
                            }));
                        }
                        Err(e) => {
                            results.push(json!({
                                "market_index": market_index,
                                "direction": if sign > 0 { "long" } else { "short" },
                                "position_amount": position_amount,
                                "status": "error",
                                "error": e.to_string()
                            }));
                        }
                    }
                }
            }
        }
        
        Ok(json!({
            "code": 200,
            "message": "Close all positions completed",
            "positions_found": positions.len(),
            "positions_closed": results.len(),
            "results": results
        }))
    }
    
    /// Close all positions in specified markets
    /// 
    /// Attempts to close positions by creating market orders with reduce_only=true
    /// for both directions (buy and sell) in each market. Only the order matching
    /// the position direction will execute.
    /// 
    /// Note: This method doesn't check if positions exist first. For better efficiency,
    /// use close_all_positions_auto() which queries positions first.
    /// 
    /// # Arguments
    /// * `market_indices` - Vector of market indices where positions should be closed
    /// 
    /// # Returns
    /// JSON response with results for each market
    pub async fn close_all_positions(&self, market_indices: Vec<u8>) -> Result<Value> {
        let mut results = Vec::new();
        
        for market_index in market_indices {
            // Try closing long position (sell)
            let close_long = self.close_position(market_index, true).await;
            if let Ok(response) = &close_long {
                let code = response["code"].as_i64().unwrap_or_default();
                if code == 200 {
                    results.push(json!({
                        "market_index": market_index,
                        "direction": "long",
                        "status": "success",
                        "response": response
                    }));
                }
            }
            
            // Try closing short position (buy)
            let close_short = self.close_position(market_index, false).await;
            if let Ok(response) = &close_short {
                let code = response["code"].as_i64().unwrap_or_default();
                if code == 200 {
                    results.push(json!({
                        "market_index": market_index,
                        "direction": "short",
                        "status": "success",
                        "response": response
                    }));
                }
            }
        }
        
        Ok(json!({
            "code": 200,
            "message": "Close all positions completed",
            "results": results
        }))
    }

    pub async fn change_api_key(&self, new_public_key: &[u8; 40]) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, PubKey, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let pub_key_hex = hex::encode(new_public_key);
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PubKey":"{}","ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            pub_key_hex,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 8)?; // TX_TYPE_CHANGE_PUB_KEY

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PubKey":"{}","ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            pub_key_hex,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "8") // CHANGE_PUB_KEY
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    pub fn create_auth_token(&self, expiry_seconds: i64) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let deadline = now + expiry_seconds;
        self.key_manager
            .create_auth_token(deadline, self.account_index, self.api_key_index)
            .map_err(|e| ApiError::Signer(e))
    }

    /// Update leverage for a market
    /// 
    /// # Arguments
    /// * `market_index` - Market index (0-based)
    /// * `leverage` - Leverage value (e.g., 3 for 3x leverage)
    /// * `margin_mode` - Margin mode: 0 for CROSS_MARGIN, 1 for ISOLATED_MARGIN
    /// 
    /// # Returns
    /// JSON response from the API
    pub async fn update_leverage(
        &self,
        market_index: u8,
        leverage: u16,
        margin_mode: u8,
    ) -> Result<Value> {
        const MAX_RETRIES: u32 = 5;
        const RETRY_DELAY_MS: u64 = 3000; // 3 seconds between retries
        
        // Fetch nonce once before retry loop
        let mut current_nonce = self.get_nonce_or_use(None).await?;
        
        let mut last_error: Option<ApiError> = None;
        
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                // Wait 3 seconds between retries for 21120 errors (nonce timing issue)
                tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                
                // Refresh nonce from API on retry
                match self.fetch_nonce_from_api().await {
                    Ok(fresh_nonce) => {
                        current_nonce = fresh_nonce;
                        // Reset cache with fresh nonce
                        self.last_fetched_nonce.store(fresh_nonce - 1, Ordering::Release);
                        self.nonce_offset.store(0, Ordering::Release);
                    }
                    Err(_) => {
                        // If fetch fails, continue with current nonce
                    }
                }
            }
            
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
            let expired_at = now + 599_000;

            // Calculate InitialMarginFraction: IMF = 10,000 / leverage
            // Example: leverage 3x = 10,000 / 3 = 3333
            let initial_margin_fraction = (10_000u32 / leverage as u32) as u16;

            // Build transaction JSON with exact field order
            // Field order: AccountIndex, ApiKeyIndex, MarketIndex, InitialMarginFraction, MarginMode, ExpiredAt, Nonce, Sig
            // Manually construct JSON string to ensure exact field order
            let tx_json = format!(
                r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"InitialMarginFraction":{},"MarginMode":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
                self.account_index,
                self.api_key_index,
                market_index,
                initial_margin_fraction,
                margin_mode,
                expired_at,
                current_nonce
            );

            // Sign immediately after building JSON
            let signature = self.sign_transaction_with_type(&tx_json, 20)?; // TX_TYPE_UPDATE_LEVERAGE

            // Add signature to JSON string (maintain exact field order)
            let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

            let final_tx_json = format!(
                r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"InitialMarginFraction":{},"MarginMode":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
                self.account_index,
                self.api_key_index,
                market_index,
                initial_margin_fraction,
                margin_mode,
                expired_at,
                current_nonce,
                sig_base64
            );

            // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
            use reqwest::multipart;
            let form = multipart::Form::new()
                .text("tx_type", "20") // UPDATE_LEVERAGE
                .text("tx_info", final_tx_json.clone());
            // NOTE: Not including price_protection to match Python's default (None = not sent)

            let response = self
                .client
                .post(&format!("{}/api/v1/sendTx", self.base_url))
                .header("Accept", "application/json")
                .multipart(form)
                .send()
                .await?;

            let response_text = response.text().await?;
            let response_json: Value = serde_json::from_str(&response_text)?;
            
            let code = response_json["code"].as_i64().unwrap_or_default();
            if code == 200 {
                // Success - nonce was used, cache is already correct
                return Ok(response_json);
            } else if code == 21120 && attempt < MAX_RETRIES {
                // Invalid signature - retry with refreshed nonce after delay
                last_error = Some(ApiError::Api(format!("Invalid signature (code 21120) after {} attempts", attempt + 1)));
                continue;
            } else {
                // Other error or max retries reached
                // On failure, decrement offset (nonce wasn't used)
                let offset = self.nonce_offset.load(Ordering::Acquire);
                if offset > 0 {
                    self.nonce_offset.store(offset - 1, Ordering::Release);
                }
                return Ok(response_json);
            }
        }
        
        // If we get here, all retries failed
        // On failure, decrement offset (nonce wasn't used)
        let offset = self.nonce_offset.load(Ordering::Acquire);
        if offset > 0 {
            self.nonce_offset.store(offset - 1, Ordering::Release);
        }
        Err(last_error.unwrap_or_else(|| ApiError::Api("Failed after all retries".to_string())))
    }

    /// Transfer USDC to another account
    pub async fn transfer(&self, request: TransferRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "FromAccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "ToAccountIndex": request.to_account_index,
            "USDCAmount": request.usdc_amount,
            "Fee": request.fee,
            "Memo": hex::encode(request.memo),
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 12)?; // TX_TYPE_TRANSFER

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "12"), // TRANSFER
            ("tx_info", &serde_json::to_string(&final_tx_info)?),
            ("price_protection", "true"),
        ];

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .form(&form_data)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Withdraw USDC from L2 to L1
    pub async fn withdraw(&self, request: WithdrawRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: FromAccountIndex, ApiKeyIndex, USDCAmount, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"FromAccountIndex":{},"ApiKeyIndex":{},"USDCAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.usdc_amount,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 13)?; // TX_TYPE_WITHDRAW

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"FromAccountIndex":{},"ApiKeyIndex":{},"USDCAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.usdc_amount,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "13") // WITHDRAW
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Modify an existing order
    pub async fn modify_order(&self, request: ModifyOrderRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, MarketIndex, Index, BaseAmount, Price, TriggerPrice, ExpiredAt, Nonce, Sig
        // Price and TriggerPrice are u32 values
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"Index":{},"BaseAmount":{},"Price":{},"TriggerPrice":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.market_index,
            request.order_index,
            request.base_amount,
            request.price,
            request.trigger_price,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 17)?; // TX_TYPE_MODIFY_ORDER

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"Index":{},"BaseAmount":{},"Price":{},"TriggerPrice":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.market_index,
            request.order_index,
            request.base_amount,
            request.price,
            request.trigger_price,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "17") // MODIFY_ORDER
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Create a sub account
    pub async fn create_sub_account(&self) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 9)?; // TX_TYPE_CREATE_SUB_ACCOUNT

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "9") // CREATE_SUB_ACCOUNT
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Create a public pool
    pub async fn create_public_pool(&self, request: CreatePublicPoolRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, OperatorFee, InitialTotalShares, MinOperatorShareRate, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"OperatorFee":{},"InitialTotalShares":{},"MinOperatorShareRate":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.operator_fee,
            request.initial_total_shares,
            request.min_operator_share_rate,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 10)?; // TX_TYPE_CREATE_PUBLIC_POOL

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"OperatorFee":{},"InitialTotalShares":{},"MinOperatorShareRate":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.operator_fee,
            request.initial_total_shares,
            request.min_operator_share_rate,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "10") // CREATE_PUBLIC_POOL
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Update a public pool
    pub async fn update_public_pool(&self, request: UpdatePublicPoolRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, PublicPoolIndex, Status, OperatorFee, MinOperatorShareRate, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"Status":{},"OperatorFee":{},"MinOperatorShareRate":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.status,
            request.operator_fee,
            request.min_operator_share_rate,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 11)?; // TX_TYPE_UPDATE_PUBLIC_POOL

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"Status":{},"OperatorFee":{},"MinOperatorShareRate":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.status,
            request.operator_fee,
            request.min_operator_share_rate,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "11") // UPDATE_PUBLIC_POOL
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Mint shares in a public pool
    pub async fn mint_shares(&self, request: MintSharesRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, PublicPoolIndex, ShareAmount, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"ShareAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.share_amount,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 18)?; // TX_TYPE_MINT_SHARES

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"ShareAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.share_amount,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "18") // MINT_SHARES
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Burn shares from a public pool
    pub async fn burn_shares(&self, request: BurnSharesRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, PublicPoolIndex, ShareAmount, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"ShareAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.share_amount,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 19)?; // TX_TYPE_BURN_SHARES

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"PublicPoolIndex":{},"ShareAmount":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.public_pool_index,
            request.share_amount,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "19") // BURN_SHARES
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Update margin for isolated margin positions
    pub async fn update_margin(&self, request: UpdateMarginRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, MarketIndex, USDCAmount, Direction, ExpiredAt, Nonce, Sig
        // Manually construct JSON string to ensure exact field order
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"USDCAmount":{},"Direction":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.market_index,
            request.usdc_amount,
            request.direction,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 29)?; // TX_TYPE_UPDATE_MARGIN

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"MarketIndex":{},"USDCAmount":{},"Direction":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.market_index,
            request.usdc_amount,
            request.direction,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "29") // UPDATE_MARGIN
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }

    /// Create grouped orders (OCO, OTO, OTOCO)
    pub async fn create_grouped_orders(&self, request: CreateGroupedOrdersRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        // Build orders array JSON with exact field order
        // OrderInfo field order: MarketIndex, ClientOrderIndex, BaseAmount, Price, IsAsk, Type, TimeInForce, ReduceOnly, TriggerPrice, OrderExpiry
        let orders_json: Vec<String> = request.orders.iter().map(|order| {
            // Calculate OrderExpiry: if time_in_force is 1 (GoodTillTime), set to 28 days
            // Otherwise, set to 0 (for ImmediateOrCancel or other types)
            let order_expiry = if order.time_in_force == 1 {
                now + (28 * 24 * 60 * 60 * 1000) // 28 days in milliseconds
            } else {
                0
            };
            
            format!(
                r#"{{"MarketIndex":{},"ClientOrderIndex":{},"BaseAmount":{},"Price":{},"IsAsk":{},"Type":{},"TimeInForce":{},"ReduceOnly":{},"TriggerPrice":{},"OrderExpiry":{}}}"#,
                order.order_book_index,
                order.client_order_index,
                order.base_amount,
                order.price,
                if order.is_ask { 1u8 } else { 0u8 },
                order.order_type,
                order.time_in_force,
                if order.reduce_only { 1u8 } else { 0u8 },
                order.trigger_price,
                order_expiry
            )
        }).collect();

        // Build transaction JSON with exact field order
        // Field order: AccountIndex, ApiKeyIndex, GroupingType, Orders, ExpiredAt, Nonce, Sig
        // Note: Orders array is constructed manually to ensure field order within each order
        let orders_array = format!("[{}]", orders_json.join(","));
        let tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"GroupingType":{},"Orders":{},"ExpiredAt":{},"Nonce":{},"Sig":""}}"#,
            self.account_index,
            self.api_key_index,
            request.grouping_type,
            orders_array,
            expired_at,
            nonce
        );

        // Sign immediately after building JSON
        let signature = self.sign_transaction_with_type(&tx_json, 28)?; // TX_TYPE_CREATE_GROUPED_ORDERS

        // Add signature to JSON string (maintain exact field order)
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        let final_tx_json = format!(
            r#"{{"AccountIndex":{},"ApiKeyIndex":{},"GroupingType":{},"Orders":{},"ExpiredAt":{},"Nonce":{},"Sig":"{}"}}"#,
            self.account_index,
            self.api_key_index,
            request.grouping_type,
            orders_array,
            expired_at,
            nonce,
            sig_base64
        );

        // CRITICAL: Use multipart/form-data (not application/x-www-form-urlencoded)
        use reqwest::multipart;
        let form = multipart::Form::new()
            .text("tx_type", "28") // CREATE_GROUPED_ORDERS
            .text("tx_info", final_tx_json.clone());
        // NOTE: Not including price_protection to match Python's default (None = not sent)

        let response = self
            .client
            .post(&format!("{}/api/v1/sendTx", self.base_url))
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;

        Ok(response_json)
    }
    
    /// Fetch a single nonce from API
    /// OPTIMIZED: Minimal allocations and fast error handling
    async fn fetch_nonce_from_api(&self) -> Result<i64> {
        // OPTIMIZED: Use cached URL (no format! overhead)
        let response = self.client.get(&self.next_nonce_url).send().await?;
        let status = response.status();
        let response_text = response.text().await?;
        
        // OPTIMIZED: Fast path for empty responses
        if response_text.is_empty() {
            return Err(ApiError::Api(format!(
                "Empty response from nonce API (status: {})",
                status
            )));
        }
        
        // OPTIMIZED: Parse JSON with minimal error allocation
        let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            let error_preview = if response_text.len() > 100 {
                format!("{}...", &response_text[..100])
            } else {
                response_text.clone()
            };
            ApiError::Api(format!(
                "Failed to parse nonce JSON (status: {}): {}. Response: {}",
                status, e, error_preview
            ))
        })?;
        
        // OPTIMIZED: Direct access without intermediate error string allocation
        let nonce = response_json["nonce"]
            .as_i64()
            .ok_or_else(|| {
                let error_preview = if response_text.len() > 100 {
                    format!("{}...", &response_text[..100])
                } else {
                    response_text.clone()
                };
                ApiError::Api(format!(
                    "Invalid nonce response format. Response: {}",
                    error_preview
                ))
            })?;
        
        // CRITICAL: Validate nonce is positive
        // Log the full response for debugging if nonce is invalid
        // NOTE: Nonce 0 might be valid for brand new accounts that haven't sent any transactions yet
        // In that case, the first transaction should use nonce 1
        if nonce < 0 {
            #[cfg(debug_assertions)]
            eprintln!("fetch_nonce_from_api: Received negative nonce {} from API. Full response: {}", nonce, response_text);
            return Err(ApiError::Api(format!(
                "Invalid nonce from API: {} (must be >= 0). Full response: {}",
                nonce, response_text
            )));
        }
        
        // If nonce is 0, it means no transactions have been sent yet
        // The first transaction should use nonce 1
        // So we return 1 instead of 0 to make it work
        if nonce == 0 {
            #[cfg(debug_assertions)]
            eprintln!("fetch_nonce_from_api: Received nonce 0 from API (account may be new). Using nonce 1 for first transaction.");
            Ok(1)
        } else {
        Ok(nonce)
        }
    }
    
    /// Generate a 12-byte random nonce converted to i64
    /// Uses cryptographically secure random number generation
    pub fn generate_random_nonce() -> i64 {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 12];
        rng.fill_bytes(&mut bytes);
        
        // Convert 12 bytes to i64 (taking first 8 bytes, little-endian)
        // This gives us a large random number
        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&bytes[..8]);
        i64::from_le_bytes(nonce_bytes)
    }
    
    /// Get next nonce using optimistic nonce management
    /// OPTIMIZED: Uses atomic operations for lock-free fast path (99% of cases)
    /// Fetches from API once, then increments locally using atomic operations
    /// 
    /// CRITICAL: Uses atomic "fetching" flag to prevent multiple threads from
    /// fetching the same nonce simultaneously, while keeping fetch outside lock
    /// for maximum performance
    /// Get next nonce from internal cache (optimistic nonce management)
    /// 
    /// # Behavior
    /// - **FAST PATH** (cache initialized): Returns cached nonce + offset, increments atomically (no API call)
    /// - **SLOW PATH** (cache empty): Fetches from `/api/v1/nextNonce` API endpoint, initializes cache
    /// 
    /// # Cache Strategy
    /// - First call: Fetches nonce from API, stores as `(nonce - 1)`, returns `nonce`
    /// - Subsequent calls: Returns `(base + offset)` where base = `(nonce - 1)` and offset increments atomically
    /// - Thread-safe: Uses atomic operations for lock-free performance
    async fn get_next_nonce_from_cache(&self) -> Result<i64> {
        // FAST PATH: Try atomic increment (lock-free, most common case)
        // Use Relaxed ordering for maximum performance - we only need atomicity, not ordering
        let base = self.last_fetched_nonce.load(Ordering::Relaxed);
        if base != i64::MIN {
            // Cache is initialized - increment atomically (lock-free!)
            // No API call needed - uses cached value
            let offset = self.nonce_offset.fetch_add(1, Ordering::Relaxed) + 1;
            let result_nonce = base + offset;
            
            // CRITICAL: Validate result is positive (nonce must be > 0)
            if result_nonce <= 0 {
                // Cache is corrupted - reset and fetch fresh
                #[cfg(debug_assertions)]
                eprintln!("get_next_nonce_from_cache: Invalid nonce computed: {} (base={}, offset={}). Resetting cache.", 
                    result_nonce, base, offset);
                // Reset cache to force fresh fetch
                self.last_fetched_nonce.store(i64::MIN, Ordering::Release);
                self.nonce_offset.store(0, Ordering::Release);
                // Fall through to SLOW PATH to fetch fresh nonce
            } else {
            // Log nonce generation
            #[cfg(debug_assertions)]
            eprintln!("get_next_nonce_from_cache: base={}, offset={}, result={}", base, offset, result_nonce);
            
            return Ok(result_nonce);
            }
        }
        
        // SLOW PATH: Cache is empty - automatically fetch from API
        // This calls /api/v1/nextNonce endpoint to get fresh nonce from server
        // CRITICAL: Use atomic flag to prevent multiple threads from fetching simultaneously
        // Try to claim the "fetching" flag (use Relaxed for performance)
        let fetch_result = self.is_fetching.compare_exchange(
            i64::MIN, // Expected: not fetching
            0,  // New: fetching (temporary, will store nonce)
            Ordering::Relaxed,
            Ordering::Relaxed
        );
        
        if fetch_result.is_ok() {
            // We won the race - we're the fetcher
            let nonce = self.fetch_nonce_from_api().await?;
            
            // Update last fetch time for hybrid approach
            let now = SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            self.last_api_fetch_time.store(now, Ordering::Relaxed);
            
            // Store fetched nonce temporarily in is_fetching (use Relaxed)
            self.is_fetching.store(nonce, Ordering::Relaxed);
            
            // Now acquire lock to initialize
            let _init_guard = self.nonce_init_lock.lock().await;
            
            // Double-check: another thread might have initialized while we were fetching
            let base_after_lock = self.last_fetched_nonce.load(Ordering::Relaxed);
            if base_after_lock != i64::MIN {
                // Cache was initialized by another thread - use atomic increment
                // Reset fetching flag
                self.is_fetching.store(i64::MIN, Ordering::Relaxed);
                let offset = self.nonce_offset.fetch_add(1, Ordering::Relaxed) + 1;
                return Ok(base_after_lock + offset);
            }
            
            // We're the first to initialize - set up the cache
            // Initialize: set base = nonce - 1, offset = 0
            // Use Release ordering only for the final store to ensure visibility
            // fetch_nonce_from_api already handles nonce 0 by returning 1
            // So nonce will always be >= 1 here, no need to validate again
            
            self.last_fetched_nonce.store(nonce - 1, Ordering::Release);
            self.nonce_offset.store(0, Ordering::Relaxed);
            // Reset fetching flag
            self.is_fetching.store(i64::MIN, Ordering::Relaxed);
            
            // Log cache initialization
            #[cfg(debug_assertions)]
            eprintln!("get_next_nonce_from_cache: Initialized cache with fetched_nonce={}, base={}", nonce, nonce - 1);
            
            // Return first nonce using atomic increment for consistency
            let offset = self.nonce_offset.fetch_add(1, Ordering::Relaxed) + 1;
            let result_nonce = (nonce - 1) + offset;
            
            // CRITICAL: Validate result is positive
            if result_nonce <= 0 {
                return Err(ApiError::Api(format!(
                    "Computed invalid nonce: {} (base={}, offset={}, fetched_nonce={})",
                    result_nonce, nonce - 1, offset, nonce
                )));
            }
            
            #[cfg(debug_assertions)]
            eprintln!("get_next_nonce_from_cache: Returning first nonce={} (offset={})", result_nonce, offset);
            
            Ok(result_nonce)
        } else {
            // Another thread is fetching - wait for it to complete
            // Poll until cache is initialized (last_fetched_nonce != i64::MIN)
            // Use exponential backoff to reduce CPU spinning
            let mut backoff = 1u64;
            loop {
                let base = self.last_fetched_nonce.load(Ordering::Relaxed);
                if base != i64::MIN {
                    // Cache initialized - use atomic increment
                    let offset = self.nonce_offset.fetch_add(1, Ordering::Relaxed) + 1;
                    return Ok(base + offset);
                }
                // Exponential backoff: yield, then sleep briefly
                tokio::task::yield_now().await;
                if backoff < 100 {
                    tokio::time::sleep(tokio::time::Duration::from_micros(backoff)).await;
                    backoff *= 2;
                }
            }
        }
    }
    
    /// Get next nonce using optimistic nonce management
    /// If provided_nonce is Some(n), uses that nonce (or -1 to fetch from cache)
    /// If provided_nonce is None, gets nonce from cache (fetches once, then increments)
    pub async fn get_nonce_or_use(&self, provided_nonce: Option<i64>) -> Result<i64> {
        if let Some(nonce) = provided_nonce {
            if nonce == -1 {
                self.get_next_nonce_from_cache().await
            } else {
                Ok(nonce)
            }
        } else {
            self.get_next_nonce_from_cache().await
        }
    }
    
    /// Refresh nonce from API (useful for manual refresh)
    /// IMPORTANT: This resets the cache and fetches a fresh nonce from the API.
    /// The returned nonce is the first nonce that will be used by the next call to get_next_nonce_from_cache().
    pub async fn refresh_nonce(&self) -> Result<i64> {
        let _init_guard = self.nonce_init_lock.lock().await;
        let nonce = self.fetch_nonce_from_api().await?;
        
        // CRITICAL: Validate nonce is positive before using it
        if nonce <= 0 {
            return Err(ApiError::Api(format!(
                "Invalid nonce from API: {} (must be > 0). Cannot initialize cache with nonce 0.",
                nonce
            )));
        }
        
        // Initialize cache: base = nonce - 1, offset = 0
        // Next call to get_next_nonce_from_cache will increment offset to 1
        // and return (nonce - 1) + 1 = nonce
        self.last_fetched_nonce.store(nonce - 1, Ordering::Release);
        self.nonce_offset.store(0, Ordering::Release);
        self.is_fetching.store(i64::MIN, Ordering::Release); // Reset fetching flag
        Ok(nonce)
    }
    
    /// Get current nonce from API
    /// Returns the next nonce that should be used for transactions
    /// CRITICAL: Validates that nonce is > 0 before returning
    pub async fn get_nonce(&self) -> Result<i64> {
        let nonce = self.fetch_nonce_from_api().await?;
        
        // CRITICAL: Validate nonce is positive
        if nonce <= 0 {
            return Err(ApiError::Api(format!(
                "Invalid nonce from API: {} (must be > 0). Server returned invalid nonce.",
                nonce
            )));
        }
        
        Ok(nonce)
    }
    
    pub fn sign_transaction(&self, tx_json: &str) -> Result<[u8; 80]> {
        self.sign_transaction_internal(tx_json, 14)
    }
    pub fn sign_transaction_with_type(&self, tx_json: &str, tx_type: u32) -> Result<[u8; 80]> {
        self.sign_transaction_internal(tx_json, tx_type)
    }

    /// Internal method to sign a transaction.
    /// 
    /// This method extracts fields from the transaction JSON, converts them to Goldilocks
    /// field elements in the correct order, hashes them using Poseidon2, and signs the hash.
    /// 
    /// The transaction hash includes:
    /// - Chain ID: 304 for mainnet (zklighter.com), 300 for testnet (testnet.zklighter.elliot.ai or elliot.ai)
    /// - Transaction type
    /// - Common fields: nonce, expired_at, account_index, api_key_index
    /// - Transaction-specific fields (varies by type)
    /// 
    /// Signature format: 80 bytes (s || e), where s and e are each 40-byte Schnorr signature components.
    /// The signature is base64-encoded before being sent to the API.
    /// 
    /// # Arguments
    /// * `tx_json` - JSON string representation of the transaction
    /// * `tx_type` - Transaction type code
    /// 
    /// # Returns
    /// An 80-byte signature array (s || e format)
    fn sign_transaction_internal(&self, tx_json: &str, tx_type: u32) -> Result<[u8; 80]> {
        let tx_value: Value = serde_json::from_str(tx_json)?;

        // Determine chain ID: 304 for mainnet, 300 for testnet
        // Chain ID is critical for signature validation - must match server's expectation
        // Testnet URLs: contain "testnet" or "elliot.ai" → Chain ID 300
        // Mainnet URLs: contain "zklighter.com" (but NOT "elliot.ai") → Chain ID 304
        // Examples:
        //   - https://testnet.zklighter.elliot.ai → 300 (testnet, has elliot.ai)
        //   - https://mainnet.zklighter.elliot.ai → 300 (testnet, has elliot.ai)
        //   - https://zklighter.com → 304 (mainnet, no elliot.ai)
        // CRITICAL: Check for elliot.ai FIRST, as it's the definitive testnet indicator
        let lighter_chain_id = if self.base_url.contains("elliot.ai") || 
                                   self.base_url.contains("testnet") {
            300u32
        } else if self.base_url.contains("zklighter.com") {
            304u32
        } else {
            // Default to testnet for safety (most common during development)
            // If you're using a custom URL, explicitly check the chain ID
            300u32
        };
        let nonce = tx_value["Nonce"].as_i64().unwrap_or(0);
        let expired_at = tx_value["ExpiredAt"].as_i64().unwrap_or(0);
        let account_index = tx_value["AccountIndex"].as_i64().unwrap_or(0);
        
        // Extract API key index from JSON - handle both u64 and i64 JSON number types
        let api_key_index = tx_value["ApiKeyIndex"]
            .as_u64()
            .or_else(|| tx_value["ApiKeyIndex"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        
        // Log extracted values for signature computation (debug builds only)
        #[cfg(debug_assertions)]
        eprintln!("sign_transaction_internal: chain_id={}, tx_type={}, nonce={}, expired_at={}, account_index={}, api_key_index={}", 
                 lighter_chain_id, tx_type, nonce, expired_at, account_index, api_key_index);
        
        // Verify API key index matches client's API key index
        // The signature must be computed with the private key corresponding to this API key index
        if api_key_index != self.api_key_index as u32 {
            return Err(ApiError::Api(format!(
                "API key index mismatch: transaction has {}, but client is configured for {}",
                api_key_index, self.api_key_index
            )));
        }
        
        // Validate nonce is not zero (should never be zero for real transactions)
        if nonce == 0 {
            #[cfg(debug_assertions)]
            eprintln!("sign_transaction_internal: Nonce is 0, this might indicate a problem");
        }

        use poseidon_hash::Goldilocks;

        // Helper function to convert signed i64 to Goldilocks field element
        // Handles sign extension properly for negative values
        let to_goldi_i64 = |val: i64| Goldilocks::from_i64(val);

        let elements = match tx_type {
            14 => {
                // CREATE_ORDER: 16 elements
        let market_index = tx_value["MarketIndex"].as_u64().unwrap_or(0) as u32;
        let client_order_index = tx_value["ClientOrderIndex"].as_i64().unwrap_or(0);
        let base_amount = tx_value["BaseAmount"].as_i64().unwrap_or(0);
        let price = tx_value["Price"]
            .as_u64()
            .or_else(|| tx_value["Price"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let is_ask = tx_value["IsAsk"]
            .as_u64()
            .or_else(|| tx_value["IsAsk"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let order_type = tx_value["Type"]
            .as_u64()
            .or_else(|| tx_value["Type"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let time_in_force = tx_value["TimeInForce"]
            .as_u64()
            .or_else(|| tx_value["TimeInForce"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let reduce_only = tx_value["ReduceOnly"]
            .as_u64()
            .or_else(|| tx_value["ReduceOnly"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let trigger_price = tx_value["TriggerPrice"]
            .as_u64()
            .or_else(|| tx_value["TriggerPrice"].as_i64().map(|v| v as u64))
            .unwrap_or(0) as u32;
        let order_expiry = tx_value["OrderExpiry"].as_i64().unwrap_or(0);
        
        vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(market_index as u64),
                    to_goldi_i64(client_order_index),
                    to_goldi_i64(base_amount),
                    Goldilocks::from_canonical_u64(price as u64),
                    Goldilocks::from_canonical_u64(is_ask as u64),
                    Goldilocks::from_canonical_u64(order_type as u64),
                    Goldilocks::from_canonical_u64(time_in_force as u64),
                    Goldilocks::from_canonical_u64(reduce_only as u64),
                    Goldilocks::from_canonical_u64(trigger_price as u64),
                    to_goldi_i64(order_expiry),
                ]
            }
            15 => {
                // CANCEL_ORDER: 8 elements
                let market_index = tx_value["MarketIndex"].as_u64().unwrap_or(0) as u32;
                let order_index = tx_value["Index"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(market_index as u64),
                    to_goldi_i64(order_index),
                ]
            }
            16 => {
                // CANCEL_ALL_ORDERS: 8 elements
                let time_in_force = tx_value["TimeInForce"]
                    .as_u64()
                    .or_else(|| tx_value["TimeInForce"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let time = tx_value["Time"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(time_in_force as u64),
                    to_goldi_i64(time),
                ]
            }
            8 => {
                // CHANGE_PUB_KEY: needs pubkey parsing (ArrayFromCanonicalLittleEndianBytes)
                let pubkey_hex = tx_value["PubKey"].as_str().unwrap_or("");
                let pubkey_bytes = hex::decode(pubkey_hex)
                    .map_err(|e| ApiError::Api(format!("Invalid PubKey hex: {}", e)))?;
                if pubkey_bytes.len() != 40 {
                    return Err(ApiError::Api("PubKey must be 40 bytes".to_string()));
                }
                // Convert 40-byte public key to 5 Goldilocks elements (8 bytes per element)
                let mut pubkey_elems = Vec::new();
                for i in 0..5 {
                    let chunk = &pubkey_bytes[i*8..(i+1)*8];
                    let val = u64::from_le_bytes(chunk.try_into().unwrap());
                    pubkey_elems.push(Goldilocks::from_canonical_u64(val));
                }

                let mut elems = vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                ];
                elems.extend(pubkey_elems);
                elems
            }
            20 => {
                // UPDATE_LEVERAGE: 9 elements
                // Order: lighterChainId, txType, nonce, expiredAt, accountIndex, apiKeyIndex, marketIndex, initialMarginFraction, marginMode
                let market_index = tx_value["MarketIndex"]
                    .as_u64()
                    .or_else(|| tx_value["MarketIndex"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let initial_margin_fraction = tx_value["InitialMarginFraction"]
                    .as_u64()
                    .or_else(|| tx_value["InitialMarginFraction"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let margin_mode = tx_value["MarginMode"]
                    .as_u64()
                    .or_else(|| tx_value["MarginMode"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(market_index as u64),
                    Goldilocks::from_canonical_u64(initial_margin_fraction as u64),
                    Goldilocks::from_canonical_u64(margin_mode as u64),
                ]
            }
            9 => {
                // CREATE_SUB_ACCOUNT: 6 elements
                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                ]
            }
            10 => {
                // CREATE_PUBLIC_POOL: 9 elements
                let operator_fee = tx_value["OperatorFee"].as_i64().unwrap_or(0);
                let initial_total_shares = tx_value["InitialTotalShares"].as_i64().unwrap_or(0);
                let min_operator_share_rate = tx_value["MinOperatorShareRate"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    to_goldi_i64(operator_fee),
                    to_goldi_i64(initial_total_shares),
                    to_goldi_i64(min_operator_share_rate),
                ]
            }
            11 => {
                // UPDATE_PUBLIC_POOL: 9 elements
                let public_pool_index = tx_value["PublicPoolIndex"].as_i64().unwrap_or(0);
                let status = tx_value["Status"]
                    .as_u64()
                    .or_else(|| tx_value["Status"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let operator_fee = tx_value["OperatorFee"].as_i64().unwrap_or(0);
                let min_operator_share_rate = tx_value["MinOperatorShareRate"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    to_goldi_i64(public_pool_index),
                    Goldilocks::from_canonical_u64(status as u64),
                    to_goldi_i64(operator_fee),
                    to_goldi_i64(min_operator_share_rate),
                ]
            }
            12 => {
                // TRANSFER: 11 elements
                // Note: Transfer uses FromAccountIndex, not AccountIndex
                let from_account_index = tx_value["FromAccountIndex"].as_i64().unwrap_or(account_index);
                let to_account_index = tx_value["ToAccountIndex"].as_i64().unwrap_or(0);
                let usdc_amount = tx_value["USDCAmount"].as_i64().unwrap_or(0);
                let fee = tx_value["Fee"].as_i64().unwrap_or(0);

                // USDCAmount and Fee are split into two u64 elements each (low 32 bits, high 32 bits)
                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(from_account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    to_goldi_i64(to_account_index),
                    Goldilocks::from_canonical_u64((usdc_amount as u64 & 0xFFFFFFFF) as u64),
                    Goldilocks::from_canonical_u64((usdc_amount as u64 >> 32) as u64),
                    Goldilocks::from_canonical_u64((fee as u64 & 0xFFFFFFFF) as u64),
                    Goldilocks::from_canonical_u64((fee as u64 >> 32) as u64),
                ]
            }
            13 => {
                // WITHDRAW: 8 elements
                // Note: Withdraw uses FromAccountIndex, not AccountIndex
                let from_account_index = tx_value["FromAccountIndex"].as_i64().unwrap_or(account_index);
                let usdc_amount = tx_value["USDCAmount"].as_u64().unwrap_or(0);

                // USDCAmount is split into two u64 elements (low 32 bits, high 32 bits)
                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(from_account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(usdc_amount & 0xFFFFFFFF),
                    Goldilocks::from_canonical_u64(usdc_amount >> 32),
                ]
            }
            17 => {
                // MODIFY_ORDER: 11 elements
                let market_index = tx_value["MarketIndex"].as_u64().unwrap_or(0) as u32;
                let order_index = tx_value["Index"].as_i64().unwrap_or(0);
                let base_amount = tx_value["BaseAmount"].as_i64().unwrap_or(0);
                let price = tx_value["Price"]
                    .as_u64()
                    .or_else(|| tx_value["Price"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let trigger_price = tx_value["TriggerPrice"]
                    .as_u64()
                    .or_else(|| tx_value["TriggerPrice"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(market_index as u64),
                    to_goldi_i64(order_index),
                    to_goldi_i64(base_amount),
                    Goldilocks::from_canonical_u64(price as u64),
                    Goldilocks::from_canonical_u64(trigger_price as u64),
                ]
            }
            18 => {
                // MINT_SHARES: 8 elements
                let public_pool_index = tx_value["PublicPoolIndex"].as_i64().unwrap_or(0);
                let share_amount = tx_value["ShareAmount"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    to_goldi_i64(public_pool_index),
                    to_goldi_i64(share_amount),
                ]
            }
            19 => {
                // BURN_SHARES: 8 elements
                let public_pool_index = tx_value["PublicPoolIndex"].as_i64().unwrap_or(0);
                let share_amount = tx_value["ShareAmount"].as_i64().unwrap_or(0);

                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    to_goldi_i64(public_pool_index),
                    to_goldi_i64(share_amount),
                ]
            }
            28 => {
                // CREATE_GROUPED_ORDERS: variable elements
                // Hash each order individually using HashNoPad, then aggregate using HashNToOne
                use poseidon_hash::{hash_no_pad, hash_n_to_one, empty_hash_out};
                
                let grouping_type = tx_value["GroupingType"]
                    .as_u64()
                    .or_else(|| tx_value["GroupingType"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                
                let orders_array = tx_value["Orders"].as_array().cloned().unwrap_or_default();
                
                let mut elems = vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(grouping_type as u64),
                ];

                // Hash each order individually using HashNoPad, then aggregate
                let mut aggregated_order_hash = empty_hash_out();
                for (index, order) in orders_array.iter().enumerate() {
                    let market_index = order["MarketIndex"].as_u64().unwrap_or(0) as u32;
                    let client_order_index = order["ClientOrderIndex"].as_i64().unwrap_or(0);
                    let base_amount = order["BaseAmount"].as_i64().unwrap_or(0);
                    let price = order["Price"]
                        .as_u64()
                        .or_else(|| order["Price"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let is_ask = order["IsAsk"]
                        .as_u64()
                        .or_else(|| order["IsAsk"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let order_type = order["Type"]
                        .as_u64()
                        .or_else(|| order["Type"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let time_in_force = order["TimeInForce"]
                        .as_u64()
                        .or_else(|| order["TimeInForce"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let reduce_only = order["ReduceOnly"]
                        .as_u64()
                        .or_else(|| order["ReduceOnly"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let trigger_price = order["TriggerPrice"]
                        .as_u64()
                        .or_else(|| order["TriggerPrice"].as_i64().map(|v| v as u64))
                        .unwrap_or(0) as u32;
                    let order_expiry = order["OrderExpiry"].as_i64().unwrap_or(0);

                    // Hash this order's fields (10 elements → 4 elements)
                    let order_fields = vec![
                        Goldilocks::from_canonical_u64(market_index as u64),
                        to_goldi_i64(client_order_index),
                        to_goldi_i64(base_amount),
                        Goldilocks::from_canonical_u64(price as u64),
                        Goldilocks::from_canonical_u64(is_ask as u64),
                        Goldilocks::from_canonical_u64(order_type as u64),
                        Goldilocks::from_canonical_u64(time_in_force as u64),
                        Goldilocks::from_canonical_u64(reduce_only as u64),
                        Goldilocks::from_canonical_u64(trigger_price as u64),
                        to_goldi_i64(order_expiry),
                    ];
                    
                    let order_hash = hash_no_pad(&order_fields);
                    
                    if index == 0 {
                        aggregated_order_hash = order_hash;
                    } else {
                        aggregated_order_hash = hash_n_to_one(&[aggregated_order_hash, order_hash]);
                    }
                }

                // Append aggregated hash (4 elements) to main elements
                elems.extend_from_slice(&aggregated_order_hash);

                elems
            }
            29 => {
                // UPDATE_MARGIN: 10 elements
                let market_index = tx_value["MarketIndex"]
                    .as_u64()
                    .or_else(|| tx_value["MarketIndex"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;
                let usdc_amount = tx_value["USDCAmount"].as_i64().unwrap_or(0);
                let direction = tx_value["Direction"]
                    .as_u64()
                    .or_else(|| tx_value["Direction"].as_i64().map(|v| v as u64))
                    .unwrap_or(0) as u32;

                // USDCAmount is split into two u64 elements (low 32 bits, high 32 bits)
                vec![
                    Goldilocks::from_canonical_u64(lighter_chain_id as u64),
                    Goldilocks::from_canonical_u64(tx_type as u64),
                    to_goldi_i64(nonce),
                    to_goldi_i64(expired_at),
                    to_goldi_i64(account_index),
                    Goldilocks::from_canonical_u64(api_key_index as u64),
                    Goldilocks::from_canonical_u64(market_index as u64),
                    Goldilocks::from_canonical_u64((usdc_amount as u64 & 0xFFFFFFFF) as u64),
                    Goldilocks::from_canonical_u64((usdc_amount as u64 >> 32) as u64),
                    Goldilocks::from_canonical_u64(direction as u64),
                ]
            }
            _ => {
                return Err(ApiError::Api(format!("Unsupported transaction type: {}", tx_type)));
            }
        };
        
        // Hash the Goldilocks field elements using Poseidon2 to produce a 40-byte hash
        use poseidon_hash::hash_to_quintic_extension;
        let hash_result = hash_to_quintic_extension(&elements);
        let message_array = hash_result.to_bytes_le();
        
        let mut hash_bytes = [0u8; 40];
        hash_bytes.copy_from_slice(&message_array[..40]);

        // Sign the transaction hash using Schnorr signature
        // Signature format: 80 bytes (s || e), where s and e are each 40 bytes
        let signature = self.key_manager.sign(&hash_bytes)
            .map_err(|e| ApiError::Signer(e))?;
        
        // Validate signature length (must be exactly 80 bytes)
        if signature.len() != 80 {
            return Err(ApiError::Api(format!(
                "Invalid signature length: expected 80 bytes, got {} bytes",
                signature.len()
            )));
        }
        
        Ok(signature)
    }

    // ============================================================================
    // Sign-only methods (return JSON, don't send to API) - for FFI compatibility
    // These methods sign transactions and return the signed JSON without submitting to the API
    // ============================================================================

    /// Sign a create order transaction and return JSON (doesn't send to API)
    pub async fn sign_create_order_with_nonce(
        &self,
        order: CreateOrderRequest,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000; // 10 minutes - 1 second (in milliseconds)
        
        let order_expiry = if order.trigger_price == 0 && order.order_type == 0 {
            // Default expiry for limit orders: 28 days
            now + (28 * 24 * 60 * 60 * 1000)
        } else {
            0
        };

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": order.order_book_index,
            "ClientOrderIndex": order.client_order_index,
            "BaseAmount": order.base_amount,
            "Price": order.price,
            "IsAsk": if order.is_ask { 1 } else { 0 },
            "Type": order.order_type,
            "TimeInForce": order.time_in_force,
            "ReduceOnly": if order.reduce_only { 1 } else { 0 },
            "TriggerPrice": order.trigger_price,
            "OrderExpiry": order_expiry,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });
        
        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction(&tx_json)?;
        
        let mut final_tx_info = tx_info;
        let sig_base64 = base64::engine::general_purpose::STANDARD.encode(&signature);
        final_tx_info["Sig"] = json!(sig_base64);
        
        Ok(final_tx_info)
    }

    /// Sign a cancel order transaction and return JSON (doesn't send to API)
    pub async fn sign_cancel_order_with_nonce(
        &self,
        market_index: u8,
        order_index: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": market_index,
            "Index": order_index,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 15)?;

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a cancel all orders transaction and return JSON (doesn't send to API)
    pub async fn sign_cancel_all_orders_with_nonce(
        &self,
        time_in_force: u8,
        time: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "TimeInForce": time_in_force,
            "Time": time,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 16)?; // TX_TYPE_CANCEL_ALL_ORDERS

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a withdraw transaction and return JSON (doesn't send to API)
    pub async fn sign_withdraw_with_nonce(
        &self,
        usdc_amount: u64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "FromAccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "USDCAmount": usdc_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 13)?; // TX_TYPE_WITHDRAW

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a transfer transaction and return JSON with MessageToSign (doesn't send to API)
    pub async fn sign_transfer_with_nonce(
        &self,
        to_account_index: i64,
        usdc_amount: i64,
        fee: i64,
        memo: [u8; 32],
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "FromAccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "ToAccountIndex": to_account_index,
            "USDCAmount": usdc_amount,
            "Fee": fee,
            "Memo": hex::encode(memo),
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 12)?; // TX_TYPE_TRANSFER

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        // Add MessageToSign field for L1 signature
        // For transfer transactions, the L1 signature body is the memo as a string
        let message_to_sign = String::from_utf8_lossy(&memo).to_string();
        final_tx_info["MessageToSign"] = json!(message_to_sign);

        Ok(final_tx_info)
    }

    /// Sign a change pub key transaction and return JSON with MessageToSign (doesn't send to API)
    pub async fn sign_change_pub_key_with_nonce(
        &self,
        new_public_key: [u8; 40],
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PubKey": hex::encode(new_public_key),
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 8)?; // TX_TYPE_CHANGE_PUB_KEY

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        // Add MessageToSign field for L1 signature
        // For change pub key transactions, the L1 signature body is a formatted string
        let message_to_sign = format!(
            "ChangePubKey\nAccountIndex: {}\nApiKeyIndex: {}\nPubKey: {}",
            self.account_index,
            self.api_key_index,
            hex::encode(new_public_key)
        );
        final_tx_info["MessageToSign"] = json!(message_to_sign);

        Ok(final_tx_info)
    }

    /// Sign an update leverage transaction and return JSON (doesn't send to API)
    pub async fn sign_update_leverage_with_nonce(
        &self,
        market_index: u8,
        initial_margin_fraction: u16,
        margin_mode: u8,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": market_index,
            "InitialMarginFraction": initial_margin_fraction,
            "MarginMode": margin_mode,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 20)?; // TX_TYPE_UPDATE_LEVERAGE

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a create sub account transaction and return JSON (doesn't send to API)
    pub async fn sign_create_sub_account_with_nonce(
        &self,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 9)?; // TX_TYPE_CREATE_SUB_ACCOUNT

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a modify order transaction and return JSON (doesn't send to API)
    pub async fn sign_modify_order_with_nonce(
        &self,
        market_index: u8,
        order_index: i64,
        base_amount: i64,
        price: u32,
        trigger_price: u32,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": market_index,
            "Index": order_index,
            "BaseAmount": base_amount,
            "Price": price,
            "TriggerPrice": trigger_price,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 17)?; // TX_TYPE_MODIFY_ORDER

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a create public pool transaction and return JSON (doesn't send to API)
    pub async fn sign_create_public_pool_with_nonce(
        &self,
        operator_fee: i64,
        initial_total_shares: i64,
        min_operator_share_rate: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "OperatorFee": operator_fee,
            "InitialTotalShares": initial_total_shares,
            "MinOperatorShareRate": min_operator_share_rate,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 10)?; // TX_TYPE_CREATE_PUBLIC_POOL

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign an update public pool transaction and return JSON (doesn't send to API)
    pub async fn sign_update_public_pool_with_nonce(
        &self,
        public_pool_index: i64,
        status: u8,
        operator_fee: i64,
        min_operator_share_rate: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": public_pool_index,
            "Status": status,
            "OperatorFee": operator_fee,
            "MinOperatorShareRate": min_operator_share_rate,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 11)?; // TX_TYPE_UPDATE_PUBLIC_POOL

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a mint shares transaction and return JSON (doesn't send to API)
    pub async fn sign_mint_shares_with_nonce(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": public_pool_index,
            "ShareAmount": share_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 18)?; // TX_TYPE_MINT_SHARES

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a burn shares transaction and return JSON (doesn't send to API)
    pub async fn sign_burn_shares_with_nonce(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": public_pool_index,
            "ShareAmount": share_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 19)?; // TX_TYPE_BURN_SHARES

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign an update margin transaction and return JSON (doesn't send to API)
    pub async fn sign_update_margin_with_nonce(
        &self,
        market_index: u8,
        usdc_amount: i64,
        direction: u8,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": market_index,
            "USDCAmount": usdc_amount,
            "Direction": direction,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 29)?; // TX_TYPE_UPDATE_MARGIN

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    /// Sign a create grouped orders transaction and return JSON (doesn't send to API)
    pub async fn sign_create_grouped_orders_with_nonce(
        &self,
        grouping_type: u8,
        orders: Vec<CreateOrderRequest>,
        nonce: Option<i64>,
    ) -> Result<Value> {
        let nonce = self.get_nonce_or_use(nonce).await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;
        let orders_json: Vec<serde_json::Value> = orders.iter().map(|order| {
            // Calculate OrderExpiry: if time_in_force is 1 (GoodTillTime), set to 28 days
            // Otherwise, set to 0 (for ImmediateOrCancel or other types)
            let order_expiry = if order.time_in_force == 1 {
                now + (28 * 24 * 60 * 60 * 1000) // 28 days in milliseconds
            } else {
                0
            };
            
            json!({
                "MarketIndex": order.order_book_index,
                "ClientOrderIndex": order.client_order_index,
                "BaseAmount": order.base_amount,
                "Price": order.price,
                "IsAsk": if order.is_ask { 1 } else { 0 },
                "Type": order.order_type,
                "TimeInForce": order.time_in_force,
                "ReduceOnly": if order.reduce_only { 1 } else { 0 },
                "TriggerPrice": order.trigger_price,
                "OrderExpiry": order_expiry,
            })
        }).collect();

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "GroupingType": grouping_type,
            "Orders": orders_json,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 28)?; // TX_TYPE_CREATE_GROUPED_ORDERS

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(final_tx_info)
    }

    // ============================================================================
    // Helper methods for accessing client state (for FFI)
    // ============================================================================

    /// Get account index
    pub fn account_index(&self) -> i64 {
        self.account_index
    }

    /// Get API key index
    pub fn api_key_index(&self) -> u8 {
        self.api_key_index
    }

    /// Get key manager (for auth token generation)
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Check API key on server (for CheckClient functionality)
    pub async fn check_api_key(&self) -> Result<()> {
        let url = format!(
            "{}/api/v1/apiKey?account_index={}&api_key_index={}",
            self.base_url, self.account_index, self.api_key_index
        );
        
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        
        let server_pubkey = response_json["public_key"]
            .as_str()
            .ok_or_else(|| ApiError::Api("Invalid API key response format".to_string()))?;
        
        let local_pubkey_bytes = self.key_manager.public_key_bytes();
        let local_pubkey_hex = hex::encode(local_pubkey_bytes);
        
        // Remove 0x prefix if present
        let server_pubkey_clean = server_pubkey.strip_prefix("0x").unwrap_or(server_pubkey);
        
        if server_pubkey_clean != local_pubkey_hex {
            return Err(ApiError::Api(format!(
                "private key does not match the one on Lighter. ownPubKey: {} response: {}",
                local_pubkey_hex, server_pubkey
            )));
        }
        
        Ok(())
    }
}
