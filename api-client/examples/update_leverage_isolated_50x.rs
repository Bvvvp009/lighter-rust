use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 UPDATE LEVERAGE - ISOLATED MARGIN 50X");
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

    // Set ETH-USD market (index 0) to 50x leverage with isolated margin mode
    println!("📝 Setting leverage configuration...");
    println!("  Market: ETH-USD (index 0)");
    println!("  Leverage: 50x");
    println!("  Margin Mode: Isolated");
    println!();

    let response = client.update_leverage(
        0,      // market_index (0 = ETH-USD)
        50,     // leverage (50x)
        1,      // margin_mode (1 = isolated margin)
    ).await?;

    println!("✅ Leverage update submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Leverage updated successfully!");
        println!("   ETH-USD is now on 50x isolated margin");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Leverage update returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}
