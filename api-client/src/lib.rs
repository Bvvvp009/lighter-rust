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

pub struct LighterClient {
    client: Client,
    base_url: String,
    key_manager: KeyManager,
    account_index: i64,
    api_key_index: u8,
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
        })
    }
    
    pub async fn create_order(&self, order: CreateOrderRequest) -> Result<Value> {
        // Get nonce from API
        let nonce = self.get_nonce().await?;
        
        // Create transaction info with expiry time
        // Transaction expires 10 minutes from now (599 seconds to account for timing differences)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let expired_at = now + 599_000; // 10 minutes - 1 second (in milliseconds)
        
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
            "OrderExpiry": 0, // NilOrderExpiry for market orders
            "ExpiredAt": expired_at,
            "Nonce": nonce,
            "Sig": ""
        });
        
        // Sign transaction (measure signing time if needed)
        let tx_json = serde_json::to_string(&tx_info)?;
        let signature = self.sign_transaction(&tx_json)?;
        
        // Add signature to transaction (base64-encoded)
        let mut final_tx_info = tx_info;
        final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));
        
        // Send request with form data
        let form_data = [
            ("tx_type", "14"), // CREATE_ORDER
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

    pub async fn create_market_order(
        &self,
        order_book_index: u8,
        client_order_index: u64,
        base_amount: i64,
        avg_execution_price: i64,
        is_ask: bool,
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
        self.create_order(order).await
    }

    pub async fn cancel_order(&self, order_book_index: u8, order_index: i64) -> Result<Value> {
        let nonce = self.get_nonce().await?;
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
        let nonce = self.get_nonce().await?;
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
        let nonce = self.get_nonce().await?;
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
        let nonce = self.get_nonce().await?;
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
            "Nonce": nonce,
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

        Ok(response_json)
    }
    
    pub async fn get_nonce(&self) -> Result<i64> {
        // Get next nonce from API endpoint
        let url = format!(
            "{}/api/v1/nextNonce?account_index={}&api_key_index={}",
            self.base_url, self.account_index, self.api_key_index
        );
        
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        
        // Extract nonce from JSON response
        let nonce = response_json["nonce"]
            .as_i64()
            .ok_or_else(|| ApiError::Api("Invalid nonce response format".to_string()))?;
        
        Ok(nonce)
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
                // Parse the transaction JSON to extract fields
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
            _ => {
                return Err(ApiError::Api(format!("Unsupported transaction type: {}", tx_type)));
            }
        };
        
                // Hash the Goldilocks field elements using Poseidon2 to produce a 40-byte hash
        // The result is a quintic extension field element (Fp5) which is then converted to bytes
        use poseidon_hash::hash_to_quintic_extension;
        let hash_result = hash_to_quintic_extension(&elements);
        let message_array = hash_result.to_bytes_le();

        // Sign the transaction hash using Schnorr signature
        self.key_manager.sign(&message_array)
            .map_err(|e| ApiError::Signer(e))
    }
}
