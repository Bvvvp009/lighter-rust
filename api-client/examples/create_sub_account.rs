use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 CREATE SUB-ACCOUNT");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Parent Account: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    println!("📝 Creating sub-account...");
    println!("  Parent: Account {}", account_index);
    println!();

    let response = client.create_sub_account().await?;

    println!("✅ Sub-account creation submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Sub-account created successfully!");
        
        if let Some(data) = response.get("data") {
            if let Some(new_account_idx) = data.get("account_index") {
                println!("   New Sub-Account Index: {}", new_account_idx);
            }
        }
        
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("📜 Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("⚠️  Sub-account creation returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("   Message: {}", msg);
        }
    }

    Ok(())
}
