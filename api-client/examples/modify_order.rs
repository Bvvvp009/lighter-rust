use api_client::LighterClient;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> MODIFY EXISTING ORDER");
    println!("{}", "=".repeat(80));
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
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 300;

    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 300;

    let order_index = 200u64; // The order index to modify (from create_modify_cancel_flow.rs example)

    println!("📝 Modifying order {}...", order_index);
    println!("  New size: 1100 units (0.11 ETH)");
    println!("  New price: $4100");
    println!();

    let modify_request = api_client::ModifyOrderRequest {
        market_index: 0,
        order_index: order_index as i64,
        base_amount: 1100,
        price: 4100_00,
        trigger_price: 0,
    };

    let response = client.modify_order(modify_request).await?;

    println!("✅ Modify request submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Order modified successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Modify returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}
