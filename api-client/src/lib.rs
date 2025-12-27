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

#[derive(Serialize, Deserialize)]
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
use rand::RngCore;
use tokio::sync::Mutex as AsyncMutex;

pub struct LighterClient {
    client: Client,
    base_url: String,
    key_manager: KeyManager,
    account_index: i64,
    api_key_index: u8,
    // Nonce cache for optimistic nonce management (like Python SDK)
    // Fetches once from API, then increments locally
    nonce_cache: Arc<AsyncMutex<NonceCache>>,
}

struct NonceCache {
    // Simple optimistic nonce management: fetch once, then increment locally
    last_fetched_nonce: i64,  // Last nonce fetched from API (stored as nonce - 1, like Python)
    nonce_offset: i64,        // How many nonces we've used since last fetch
}

impl NonceCache {
    fn new() -> Self {
        Self {
            last_fetched_nonce: -1,  // -1 means not initialized
            nonce_offset: 0,
        }
    }
    
    fn get_next_nonce(&mut self) -> Option<i64> {
        if self.last_fetched_nonce == -1 {
            None  // Not initialized, need to fetch from API
        } else {
            // Increment offset and return next nonce
            // Formula: (last_fetched_nonce - 1) + offset + 1 = last_fetched_nonce + offset
            self.nonce_offset += 1;
            Some(self.last_fetched_nonce + self.nonce_offset)
        }
    }
    
    fn set_fetched_nonce(&mut self, nonce: i64) {
        // Store as nonce - 1, so first increment gives us the correct nonce
        // This matches Python's OptimisticNonceManager behavior
        self.last_fetched_nonce = nonce - 1;
        self.nonce_offset = 0;
    }
    
    fn acknowledge_failure(&mut self) {
        // Decrement offset on failure to allow retry with same nonce
        // This matches Python's OptimisticNonceManager behavior
        if self.nonce_offset > 0 {
            self.nonce_offset -= 1;
        }
    }
    
}

impl LighterClient {
    pub fn new(
        base_url: String,
        private_key_hex: &str,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<Self> {
        let key_manager = KeyManager::from_hex(private_key_hex)?;
        let client = Client::new();
        
        Ok(Self {
            client,
            base_url,
            key_manager,
            account_index,
            api_key_index,
            nonce_cache: Arc::new(AsyncMutex::new(NonceCache::new())),
        })
    }
    
    pub async fn create_order(&self, order: CreateOrderRequest) -> Result<Value> {
        self.create_order_with_nonce(order, None).await
    }
    
    /// Create order with optional nonce parameter and retry logic
    /// If nonce is Some(n), uses that nonce (or -1 to fetch from API)
    /// If nonce is None, uses optimistic nonce management
    /// Creates an order with automatic retries for transient errors.
    /// On transient errors (nonce/signature), fetches fresh nonce from API and retries.
    /// This ensures server and client nonce state stay synchronized.
    pub async fn create_order_with_nonce(&self, order: CreateOrderRequest, nonce: Option<i64>) -> Result<Value> {
        const MAX_RETRIES: u32 = 2; // Retries for transient failures (nonce/sig)
        // Delay between retries: 500-1000ms for server to process previous attempt
        let retry_delay_ms: u64 = std::env::var("RETRY_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);
        
        // Fetch nonce once before retry loop
        let mut current_nonce = self.get_nonce_or_use(nonce).await?;
        let mut last_error: Option<ApiError> = None;
        
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                // Backoff before retry
                tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
                // On retry: fetch fresh nonce from API to sync with server state
                // This is critical for nonce/signature errors
                match self.fetch_nonce_from_api().await {
                    Ok(fresh_nonce) => {
                        current_nonce = fresh_nonce;
                        // Reset cache with fetched nonce
                        let mut cache = self.nonce_cache.lock().await;
                        cache.set_fetched_nonce(fresh_nonce);
                    }
                    Err(_) => {
                        // If fetch fails, proceed with current nonce
                        // (better than failing completely)
                    }
                }
            }
            
            match self.create_order_internal(&order, Some(current_nonce)).await {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    if code == 200 {
                        // Success
                        return Ok(response);
                    } else if attempt < MAX_RETRIES {
                        // Check for known transient errors
                        let msg = response["message"].as_str().unwrap_or("").to_lowercase();
                        let is_sig_err = code == 21120 || msg.contains("invalid signature");
                        let is_nonce_err = code == 21104 || msg.contains("nonce");
                        if is_sig_err || is_nonce_err {
                            // Retry with fresh nonce (fetched at top of next iteration)
                            last_error = Some(ApiError::Api(format!(
                                "Transient error (code {}){} - will retry",
                                code,
                                if is_sig_err { " [sig]" } else { " [nonce]" }
                            )));
                            continue;
                        } else {
                            // Permanent error - don't retry
                            {
                                let mut cache = self.nonce_cache.lock().await;
                                cache.acknowledge_failure();
                            }
                            return Ok(response);
                        }
                    } else {
                        // Max retries exhausted - return final response
                        {
                            let mut cache = self.nonce_cache.lock().await;
                            cache.acknowledge_failure();
                        }
                        return Ok(response);
                    }
                }
                Err(e) => {
                    // Network or serialization error
                    if attempt < MAX_RETRIES {
                        last_error = Some(e);
                        continue;
                    } else {
                        {
                            let mut cache = self.nonce_cache.lock().await;
                            cache.acknowledge_failure();
                        }
                        return Err(e);
                    }
                }
            }
        }
        
        // All retries exhausted
        {
            let mut cache = self.nonce_cache.lock().await;
            cache.acknowledge_failure();
        }
        Err(last_error.unwrap_or_else(|| ApiError::Api("Failed after all retries".to_string())))
    }
    
    /// Internal method to create order (without retry logic)
    /// This is called by create_order_with_nonce for each retry attempt
    /// Uses the provided nonce directly (no fetching)
    async fn create_order_internal(&self, order: &CreateOrderRequest, nonce: Option<i64>) -> Result<Value> {
        let nonce = nonce.expect("Nonce should be provided to create_order_internal");
        
        // Create transaction info with expiry time
        // Match Go SDK: DefaultExpireTime = time.Minute*10 - time.Second
        // This gives a 1 second margin to eliminate millisecond differences
        // Calculate timestamp right before creating tx_info to minimize clock skew
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        // Use 10 minutes - 1 second (599,000 ms) to match Go SDK exactly
        let expired_at = now + 599_000; // 10 minutes - 1 second (matches Go SDK)
        
        // OrderExpiry: For limit orders with GoodTillTime, set to 28 days
        // For other orders, use 0 (nil)
        let order_expiry = if order.time_in_force == 1 && order.order_type == 0 {
            // GoodTillTime limit order: 28 days expiry
            now + (28 * 24 * 60 * 60 * 1000)
        } else {
            0 // NilOrderExpiry
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
        
        let final_tx_json = serde_json::to_string(&final_tx_info)?;
        
        let form_data = [
            ("tx_type", "14"), // CREATE_ORDER
            ("tx_info", &final_tx_json),
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

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": order_book_index,
            "Index": order_index,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 15)?; // TX_TYPE_CANCEL_ORDER

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "15"), // CANCEL_ORDER
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

    pub async fn cancel_all_orders(&self, time_in_force: u8, time: i64) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
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

        let form_data = [
            ("tx_type", "16"), // CANCEL_ALL_ORDERS
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

    pub async fn change_api_key(&self, new_public_key: &[u8; 40]) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
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

        let form_data = [
            ("tx_type", "8"), // CHANGE_PUB_KEY
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
                        let mut cache = self.nonce_cache.lock().await;
                        cache.set_fetched_nonce(fresh_nonce);
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

            let tx_info = json!({
                "AccountIndex": self.account_index,
                "ApiKeyIndex": self.api_key_index,
                "MarketIndex": market_index,
                "InitialMarginFraction": initial_margin_fraction,
                "MarginMode": margin_mode,
                "ExpiredAt": expired_at,
                "Nonce": current_nonce,
                "Sig": ""
            });

            let tx_json = serde_json::to_string(&tx_info)?;
            let signature = self.sign_transaction_with_type(&tx_json, 20)?; // TX_TYPE_UPDATE_LEVERAGE

            let mut final_tx_info = tx_info;
            final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

            let form_data = [
                ("tx_type", "20"), // UPDATE_LEVERAGE
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
                {
                    let mut cache = self.nonce_cache.lock().await;
                    cache.acknowledge_failure();
                }
                return Ok(response_json);
            }
        }
        
        // If we get here, all retries failed
        {
            let mut cache = self.nonce_cache.lock().await;
            cache.acknowledge_failure();
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

        let tx_info = json!({
            "FromAccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "USDCAmount": request.usdc_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 13)?; // TX_TYPE_WITHDRAW

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "13"), // WITHDRAW
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

    /// Modify an existing order
    pub async fn modify_order(&self, request: ModifyOrderRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": request.market_index,
            "Index": request.order_index,
            "BaseAmount": request.base_amount,
            "Price": request.price,
            "TriggerPrice": request.trigger_price,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 17)?; // TX_TYPE_MODIFY_ORDER

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "17"), // MODIFY_ORDER
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

    /// Create a sub account
    pub async fn create_sub_account(&self) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
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

        let form_data = [
            ("tx_type", "9"), // CREATE_SUB_ACCOUNT
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

    /// Create a public pool
    pub async fn create_public_pool(&self, request: CreatePublicPoolRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "OperatorFee": request.operator_fee,
            "InitialTotalShares": request.initial_total_shares,
            "MinOperatorShareRate": request.min_operator_share_rate,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 10)?; // TX_TYPE_CREATE_PUBLIC_POOL

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "10"), // CREATE_PUBLIC_POOL
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

    /// Update a public pool
    pub async fn update_public_pool(&self, request: UpdatePublicPoolRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": request.public_pool_index,
            "Status": request.status,
            "OperatorFee": request.operator_fee,
            "MinOperatorShareRate": request.min_operator_share_rate,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 11)?; // TX_TYPE_UPDATE_PUBLIC_POOL

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "11"), // UPDATE_PUBLIC_POOL
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

    /// Mint shares in a public pool
    pub async fn mint_shares(&self, request: MintSharesRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": request.public_pool_index,
            "ShareAmount": request.share_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 18)?; // TX_TYPE_MINT_SHARES

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "18"), // MINT_SHARES
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

    /// Burn shares from a public pool
    pub async fn burn_shares(&self, request: BurnSharesRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "PublicPoolIndex": request.public_pool_index,
            "ShareAmount": request.share_amount,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 19)?; // TX_TYPE_BURN_SHARES

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "19"), // BURN_SHARES
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

    /// Update margin for isolated margin positions
    pub async fn update_margin(&self, request: UpdateMarginRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "MarketIndex": request.market_index,
            "USDCAmount": request.usdc_amount,
            "Direction": request.direction,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 29)?; // TX_TYPE_UPDATE_MARGIN

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "29"), // UPDATE_MARGIN
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

    /// Create grouped orders (OCO, OTO, etc.)
    pub async fn create_grouped_orders(&self, request: CreateGroupedOrdersRequest) -> Result<Value> {
        let nonce = self.get_next_nonce_from_cache().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000;

        let orders_json: Vec<serde_json::Value> = request.orders.iter().map(|order| {
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
                "OrderExpiry": 0,
            })
        }).collect();

        let tx_info = json!({
            "AccountIndex": self.account_index,
            "ApiKeyIndex": self.api_key_index,
            "GroupingType": request.grouping_type,
            "Orders": orders_json,
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });

        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction_with_type(&tx_json, 28)?; // TX_TYPE_CREATE_GROUPED_ORDERS

        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));

        let form_data = [
            ("tx_type", "28"), // CREATE_GROUPED_ORDERS
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
    
    /// Fetch a single nonce from API
    async fn fetch_nonce_from_api(&self) -> Result<i64> {
        let url = format!(
            "{}/api/v1/nextNonce?account_index={}&api_key_index={}",
            self.base_url, self.account_index, self.api_key_index
        );
        
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        
        let nonce = response_json["nonce"]
            .as_i64()
            .ok_or_else(|| ApiError::Api("Invalid nonce response format".to_string()))?;
        
        Ok(nonce)
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
    /// Fetches from API once, then increments locally
    /// Only fetches again if cache is not initialized
    async fn get_next_nonce_from_cache(&self) -> Result<i64> {
        let mut cache = self.nonce_cache.lock().await;
        
        // If cache is initialized, use optimistic nonce management
        if let Some(nonce) = cache.get_next_nonce() {
            return Ok(nonce);
        }
        
        // Cache not initialized, fetch from API
        drop(cache); // Release lock before async call
        let nonce = self.fetch_nonce_from_api().await?;
        
        // Update cache with fetched nonce and immediately use it
        let mut cache = self.nonce_cache.lock().await;
        cache.set_fetched_nonce(nonce);
        
        // Get the first nonce from cache (this increments offset)
        // This ensures the next call will return the next sequential nonce
        let first_nonce = cache.get_next_nonce().expect("Cache just initialized, should have nonce");
        
        Ok(first_nonce)
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
    pub async fn refresh_nonce(&self) -> Result<i64> {
        let nonce = self.fetch_nonce_from_api().await?;
        let mut cache = self.nonce_cache.lock().await;
        cache.set_fetched_nonce(nonce);
        Ok(nonce)
    }
    
    /// Get next nonce from API (public method)
    /// This fetches a fresh nonce from the API each time
    /// For optimistic nonce management, use get_next_nonce_from_cache instead
    pub async fn get_nonce(&self) -> Result<i64> {
        self.fetch_nonce_from_api().await
    }
    
            /// Signs a transaction JSON string and returns the signature.
    /// 
    /// This method is a convenience wrapper for CREATE_ORDER transactions (type 14).
    /// For other transaction types, use `sign_transaction_with_type`.
    /// 
    /// # Arguments
    /// * `tx_json` - JSON string representation of the transaction
    /// 
    /// # Returns
    /// An 80-byte signature array
    pub fn sign_transaction(&self, tx_json: &str) -> Result<[u8; 80]> {
        self.sign_transaction_internal(tx_json, 14) // CREATE_ORDER
    }

    /// Signs a transaction with a specific transaction type.
    /// 
    /// # Arguments
    /// * `tx_json` - JSON string representation of the transaction
    /// * `tx_type` - Transaction type code (e.g., 14 for CREATE_ORDER, 15 for CANCEL_ORDER, 20 for UPDATE_LEVERAGE)
    /// 
    /// # Returns
    /// An 80-byte signature array
    pub fn sign_transaction_with_type(&self, tx_json: &str, tx_type: u32) -> Result<[u8; 80]> {
        self.sign_transaction_internal(tx_json, tx_type)
    }

    /// Internal method to sign a transaction.
    /// 
    /// This method extracts fields from the transaction JSON, converts them to Goldilocks
    /// field elements in the correct order, hashes them using Poseidon2, and signs the hash.
    /// 
    /// The transaction hash includes:
    /// - Chain ID (304 for mainnet, 300 for testnet)
    /// - Transaction type
    /// - Common fields: nonce, expired_at, account_index, api_key_index
    /// - Transaction-specific fields (varies by type)
    /// 
    /// # Arguments
    /// * `tx_json` - JSON string representation of the transaction
    /// * `tx_type` - Transaction type code
    /// 
    /// # Returns
    /// An 80-byte signature array (s || e format)
    fn sign_transaction_internal(&self, tx_json: &str, tx_type: u32) -> Result<[u8; 80]> {
        let tx_value: Value = serde_json::from_str(tx_json)?;

        // Determine chain ID based on base URL
        // Mainnet: 304, Testnet: 300
        let lighter_chain_id = if self.base_url.contains("mainnet") {
            304u32
        } else {
            300u32
        };
        let nonce = tx_value["Nonce"].as_i64().unwrap_or(0);
        let expired_at = tx_value["ExpiredAt"].as_i64().unwrap_or(0);
        let account_index = tx_value["AccountIndex"].as_i64().unwrap_or(0);
        let api_key_index = tx_value["ApiKeyIndex"].as_u64().unwrap_or(0) as u32;

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
                // Matches Go SDK: HashNoPad for each order, then HashNToOne to aggregate
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
        
        // Optional debug: dump signing inputs (limited to first few orders to avoid spam)
        let sign_debug = std::env::var("SIGN_DEBUG").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

        if sign_debug && nonce < 600 {
            // Only debug first ~8 nonces to avoid terminal spam
            eprintln!("\n=== TRANSACTION SIGNING DEBUG ===");
            eprintln!("TX Type: {}", tx_type);
            eprintln!("Nonce: {} ExpiredAt: {}", nonce, expired_at);
            eprintln!("Elements to hash ({}): {:?}", elements.len(), elements.iter().map(|e| e.0).collect::<Vec<_>>());
        }

        // Hash the Goldilocks field elements using Poseidon2 to produce a 40-byte hash
        use poseidon_hash::hash_to_quintic_extension;
        
        let hash_result = hash_to_quintic_extension(&elements);
        let message_array = hash_result.to_bytes_le();
        
        if sign_debug && nonce < 600 {
            eprintln!("Hash (Fp5): {:?}", hash_result.0.iter().map(|e| e.0).collect::<Vec<_>>());
            eprintln!("Message bytes: {}", hex::encode(&message_array));
            eprintln!("Public Key: {}", hex::encode(&self.key_manager.public_key_bytes()));
            eprintln!("=================================\n");
        }
        
        let mut hash_bytes = [0u8; 40];
        hash_bytes.copy_from_slice(&message_array[..40]);

        // Sign the transaction hash using Schnorr signature
        let signature = self.key_manager.sign(&hash_bytes)
            .map_err(|e| ApiError::Signer(e))?;
        
        Ok(signature)
    }

    // ============================================================================
    // Sign-only methods (return JSON, don't send to API) - for FFI compatibility
    // These match Go SDK's Sign* functions
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
        let signature = self.sign_transaction_with_type(&tx_json, 15)?; // TX_TYPE_CANCEL_ORDER

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

        // Add MessageToSign field (like Go SDK does)
        // For transfer, the L1 signature body is the memo as a string
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

        // Add MessageToSign field (like Go SDK does)
        // For change pub key, the L1 signature body is a formatted string
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
                "OrderExpiry": 0,
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
