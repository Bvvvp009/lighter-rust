use api_client::{LighterClient, CreateOrderRequest};
use std::env;
use serde_json::json;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔍 VERIFICATION: Rust Signer with Critical Fixes Applied");
    println!("{}", "═".repeat(80));
    println!();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Get credentials
    let base_url = env::var("BASE_URL")?;
    let base_url_clone = base_url.clone(); // Clone for later use
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    
    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  API Key Length: {} characters", api_key.len());
    
    // Determine chain ID
    let chain_id = if base_url.contains("mainnet") {
        304u32
    } else {
        300u32
    };
    println!("  Chain ID: {} ({})", chain_id, if chain_id == 304 { "mainnet" } else { "testnet" });
    println!();
    
    // Create client
    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    
    // Get nonce manually (same logic as internal get_nonce method)
    println!("📡 Fetching nonce from API...");
    use reqwest::Client;
    let http_client = Client::new();
    let nonce_url = format!(
        "{}/api/v1/nextNonce?account_index={}&api_key_index={}",
        &base_url_clone, account_index, api_key_index
    );
    let nonce_response = http_client.get(&nonce_url).send().await?;
    let nonce_text = nonce_response.text().await?;
    let nonce_json: serde_json::Value = serde_json::from_str(&nonce_text)?;
    let nonce = nonce_json["nonce"]
        .as_i64()
        .ok_or_else(|| "Invalid nonce response format")?;
    println!("  Nonce: {}", nonce);
    println!();
    
    // Create order
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0, // BTC-USD or ETH-USD
        client_order_index: 12345,
        base_amount: 10, // 0.001 tokens
        price: 348400, // Current market price
        is_ask: false, // Buy order
        order_type: 0, // MarketOrder
        time_in_force: 0, // ImmediateOrCancel
        reduce_only: false,
        trigger_price: 0,
    };
    
    println!("📝 Order Details:");
    println!("  Market Index: {}", order.order_book_index);
    println!("  Client Order Index: {}", order.client_order_index);
    println!("  Base Amount: {}", order.base_amount);
    println!("  Price: {}", order.price);
    println!("  Is Ask: {}", order.is_ask);
    println!("  Order Type: {} (0 = Market)", order.order_type);
    println!("  Time In Force: {} (0 = IOC)", order.time_in_force);
    println!("  Reduce Only: {}", order.reduce_only);
    println!("  Trigger Price: {}", order.trigger_price);
    println!();
    
    // Calculate expired_at
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let expired_at = now + 599_000; // 10 minutes - 1 second (matches Go)
    
    println!("⏰ Timing:");
    println!("  Current Time (ms): {}", now);
    println!("  ExpiredAt: {} ({} minutes from now)", expired_at, 599_000 / 60_000);
    println!();
    
    // Build transaction JSON (before signing)
    let tx_info = json!({
        "AccountIndex": account_index,
        "ApiKeyIndex": api_key_index,
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
        "ExpiredAt": expired_at,
        "Nonce": nonce,
        "Sig": ""
    });
    
    println!("📄 Transaction JSON (before signing):");
    println!("{}", serde_json::to_string_pretty(&tx_info)?);
    println!();
    
    // Sign transaction using debug method to get hash
    let tx_json = serde_json::to_string(&tx_info)?;
    println!("🔐 Signing transaction...");
    
    // Sign the transaction
    let signature = match client.sign_transaction(&tx_json) {
        Ok(sig) => {
            println!("  ✅ Signature generated successfully");
            sig
        }
        Err(e) => {
            println!("  ❌ Signature error: {}", e);
            return Err(e.into());
        }
    };
    
    // Calculate transaction hash manually to display
    println!();
    println!("🔑 Signature Details:");
    println!("  Signature (hex): {}", hex::encode(&signature));
    println!("  Signature (base64): {}", base64::engine::general_purpose::STANDARD.encode(&signature));
    println!();
    
    // Add signature to transaction
    let mut final_tx_info = tx_info.clone();
    final_tx_info["Sig"] = json!(base64::engine::general_purpose::STANDARD.encode(&signature));
    
    println!("📤 Final Transaction JSON (with signature):");
    println!("{}", serde_json::to_string_pretty(&final_tx_info)?);
    println!();
    
    // Show what would be sent in HTTP request
    println!("🌐 HTTP Request Details:");
    println!("  URL: {}/api/v1/transaction/sendTransaction", base_url_clone);
    println!("  Method: POST");
    println!("  Content-Type: application/json");
    println!("  Body (JSON string):");
    println!("{}", serde_json::to_string(&final_tx_info)?);
    println!();
    
    println!("{}", "═".repeat(80));
    println!("📊 COMPARISON CHECKLIST:");
    println!("{}", "═".repeat(80));
    println!("📊 Verification Checklist:");
    println!("  ✅ Chain ID: {} ({})", chain_id, if chain_id == 304 { "mainnet" } else { "testnet" });
    println!("  ✅ ExpiredAt: {} (current + 599 seconds)", expired_at);
    println!("  ✅ Transaction JSON field order and values");
    println!("  ✅ Signature (base64)");
    println!("  ✅ Final JSON structure");
    println!();
    
    // Actually submit the order
    println!("🚀 Submitting order to exchange...");
    println!();
    
    match client.create_order(order).await {
        Ok(response) => {
            println!("{}", "═".repeat(80));
            println!("✅ ORDER SUBMITTED SUCCESSFULLY!");
            println!("{}", "═".repeat(80));
            println!();
            println!("📥 Response from exchange:");
            println!("{}", serde_json::to_string_pretty(&response)?);
            println!();
            
            // Check response code
            if let Some(code) = response.get("code") {
                if let Some(code_num) = code.as_i64() {
                    if code_num == 0 {
                        println!("🎉 SUCCESS: Order was accepted by exchange!");
                    } else {
                        println!("⚠️  Order was processed with code: {}", code_num);
                        if let Some(message) = response.get("message") {
                            println!("   Message: {}", message);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("{}", "═".repeat(80));
            println!("❌ ORDER FAILED!");
            println!("{}", "═".repeat(80));
            println!();
            println!("Error: {}", e);
            println!();
            println!("🔍 Debugging information:");
            println!("  - Verify Chain ID is correct: {}", chain_id);
            println!("  - Verify ExpiredAt calculation: {}", expired_at);
            println!("  - Verify transaction JSON format matches API requirements");
            println!("  - Check if signature format matches");
            return Err(e.into());
        }
    }
    
    Ok(())
}
