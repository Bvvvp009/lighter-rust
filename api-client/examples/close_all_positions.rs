use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚫 CLOSE ALL POSITIONS EXAMPLE");
    println!("{}", "═".repeat(80));
    println!();

    // Load .env file manually
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut env_file = current_dir.join(".env");
    if !env_file.exists() {
        env_file = current_dir.parent()
            .map(|p| p.join(".env"))
            .unwrap_or_else(|| current_dir.join(".env"));
    }
    if !env_file.exists() {
        env_file = current_dir.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join(".env"))
            .unwrap_or_else(|| current_dir.join(".env"));
    }
    
    if env_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("--") {
                    continue;
                }
                if let Some(equal_pos) = line.find('=') {
                    let key = line[..equal_pos].trim();
                    let mut value = line[equal_pos + 1..].trim();
                    value = value.trim_matches('"').trim_matches('\'');
                    if value.starts_with("0x") || value.starts_with("0X") {
                        value = &value[2..];
                    }
                    if !key.is_empty() && !value.is_empty() {
                        std::env::set_var(key, value);
                    }
                }
            }
        }
    }

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let mut api_key = env::var("API_PRIVATE_KEY")?;
    
    // Clean private key
    api_key = api_key.trim().to_string();
    api_key = api_key.replace(" ", "").replace("\n", "").replace("\r", "").replace("\t", "");
    if api_key.starts_with("0x") || api_key.starts_with("0X") {
        api_key = api_key[2..].to_string();
    }
    let hex_only: String = api_key.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(80)
        .collect();

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &hex_only, account_index, api_key_index)?;

    // Method 1: Auto-detect and close all positions (RECOMMENDED)
    println!("📝 Method 1: Auto-detecting and closing all positions...");
    println!("  This queries your account first to find open positions");
    println!();

    // First, let's check what the account API returns
    println!("🔍 Debug: Fetching account info to see structure...");
    let account_info = client.get_account().await?;
    println!("📥 Account Info Structure:");
    println!("{}", serde_json::to_string_pretty(&account_info)?);
    println!();

    let response = client.close_all_positions_auto().await?;

    println!("✅ Auto close all positions completed!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        if let Some(positions_found) = response.get("positions_found") {
            println!("\n📊 Positions found: {}", positions_found);
        }
        if let Some(results) = response.get("results") {
            if let Some(results_array) = results.as_array() {
                println!("\n✅ Closed {} position(s):", results_array.len());
                for result in results_array {
                    if let Some(market) = result.get("market_index") {
                        if let Some(dir) = result.get("direction") {
                            if let Some(status) = result.get("status") {
                                println!("  Market {} - {} position: {}", market, dir, status);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n{}", "─".repeat(80));
    println!("\n📝 Method 2: Close positions in specific markets (manual)...");
    println!("  Use this if you know which markets have positions");
    
    // Method 2: Manual - specify markets
    let market_indices = vec![0u8, 1u8, 2u8, 3u8]; // Market indices (u8)

    println!("  Closing positions in markets: {:?}", market_indices);
    println!("  This creates market orders with reduce_only=true for both directions");
    println!();

    let response2 = client.close_all_positions(market_indices).await?;

    println!("✅ Manual close all positions completed!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response2)?);

    let code2 = response2["code"].as_i64().unwrap_or_default();
    if code2 == 200 {
        println!("\n✅ Manual close all positions completed!");
        if let Some(results) = response2.get("results") {
            println!("  Results: {}", results);
        }
    } else {
        println!("\n⚠️  Manual close all positions returned code: {}", code2);
        if let Some(msg) = response2["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}

