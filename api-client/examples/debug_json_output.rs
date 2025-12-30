// Debug utility to compare JSON output between json!() and typed struct
use serde::{Serialize};
use serde_json::{json, Value};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateOrderTxInfo {
    // CRITICAL: Fields MUST be in alphabetical order (by PascalCase key name)
    // to match json!() macro output and produce correct signatures
    account_index: i64,      // AccountIndex
    api_key_index: u8,       // ApiKeyIndex
    base_amount: i64,        // BaseAmount (alphabetically before ClientOrderIndex)
    client_order_index: u64, // ClientOrderIndex
    expired_at: i64,         // ExpiredAt
    is_ask: u8,              // IsAsk (0 or 1)
    market_index: u8,        // MarketIndex
    nonce: i64,              // Nonce
    order_expiry: i64,       // OrderExpiry
    price: i64,              // Price
    reduce_only: u8,         // ReduceOnly (0 or 1)
    sig: String,             // Sig
    time_in_force: u8,       // TimeInForce
    trigger_price: i64,      // TriggerPrice
    r#type: u8,              // Type (reserved keyword, use raw identifier)
}

fn main() {
    // Sample values
    let account_index = 9036;
    let api_key_index = 0;
    let market_index = 0;
    let client_order_index = 1234567890;
    let base_amount = 1000i64;
    let price = 350000i64;
    let is_ask = false;
    let order_type = 1u8; // MarketOrder
    let time_in_force = 0u8; // ImmediateOrCancel
    let reduce_only = false;
    let trigger_price = 0i64;
    let order_expiry = 0i64;
    let expired_at = 1704067200000i64;
    let nonce = 100000i64;

    // Original json!() approach (working baseline - 0% sig errors)
    let baseline_json = json!({
        "AccountIndex": account_index,
        "ApiKeyIndex": api_key_index,
        "MarketIndex": market_index,
        "ClientOrderIndex": client_order_index,
        "BaseAmount": base_amount,
        "Price": price,
        "IsAsk": if is_ask { 1 } else { 0 },
        "Type": order_type,
        "TimeInForce": time_in_force,
        "ReduceOnly": if reduce_only { 1 } else { 0 },
        "TriggerPrice": trigger_price,
        "OrderExpiry": order_expiry,
        "ExpiredAt": expired_at,
        "Nonce": nonce,
        "Sig": ""
    });
    
    // New typed struct approach (20-25% sig errors)
    let typed_struct = CreateOrderTxInfo {
        account_index,
        api_key_index,
        market_index,
        client_order_index,
        base_amount,
        price,
        is_ask: if is_ask { 1 } else { 0 },
        r#type: order_type,
        time_in_force,
        reduce_only: if reduce_only { 1 } else { 0 },
        trigger_price,
        order_expiry,
        expired_at,
        nonce,
        sig: String::new(),
    };
    
    let baseline_str = serde_json::to_string(&baseline_json).unwrap();
    let typed_str = serde_json::to_string(&typed_struct).unwrap();
    
    println!("=== BASELINE (json! macro - 0% sig errors) ===");
    println!("{}", baseline_str);
    println!("\n{:#}", baseline_json);
    
    println!("\n=== TYPED STRUCT (20-25% sig errors) ===");
    println!("{}", typed_str);
    println!("\n{:#}", serde_json::from_str::<Value>(&typed_str).unwrap());
    
    println!("\n=== COMPARISON ===");
    if baseline_str == typed_str {
        println!("✅ JSON output is IDENTICAL");
    } else {
        println!("❌ JSON output DIFFERS");
        println!("\nBaseline length: {}", baseline_str.len());
        println!("Typed length: {}", typed_str.len());
        
        // Character-by-character diff
        println!("\n=== BYTE-BY-BYTE DIFFERENCE ===");
        let max_len = baseline_str.len().max(typed_str.len());
        for i in 0..max_len {
            let b_char = baseline_str.chars().nth(i);
            let t_char = typed_str.chars().nth(i);
            if b_char != t_char {
                println!("Position {}: baseline={:?} typed={:?}", i, b_char, t_char);
            }
        }
    }
}
