use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚫 CANCEL ORDER EXAMPLE");
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

    // Get order index from command line or use default
    let order_index: i64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(12345);

    println!("📝 Canceling order...");
    println!("  Market Index: 0");
    println!("  Order Index: {}", order_index);
    println!();

    let response = client.cancel_order(0, order_index).await?;

    println!("✅ Cancel order submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ Order canceled successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Cancel order returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}
