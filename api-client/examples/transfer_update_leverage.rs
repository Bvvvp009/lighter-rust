use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 TRANSFER & UPDATE LEVERAGE EXAMPLE");
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

    let client = LighterClient::new(base_url.clone(), &hex_only, account_index, api_key_index)?;

    // Transfer USDC to another account
    // Note: Transfer uses L2 signing (API key), no L1 private key needed
    println!("📝 Transfer Example");
    println!("  Transfer uses L2 signing with your API key");
    println!("  You need:");
    println!("  1. Transfer fee information from API");
    println!("  2. 32-byte memo");
    println!("  3. To account index");
    println!();

    // Update Leverage
    println!("📝 Updating Leverage...");
    let market_index = 0u8;
    let leverage = 5u16; // 3x leverage
    let margin_mode = 0u8; // 0 = CROSS_MARGIN, 1 = ISOLATED_MARGIN

    match client.update_leverage(market_index, leverage, margin_mode).await {
        Ok(response) => {
            let code = response["code"].as_i64().unwrap_or_default();
            if code == 200 {
                println!("✅ Leverage updated successfully!");
                println!("  Market Index: {}", market_index);
                println!("  Leverage: {}x", leverage);
                println!("  Margin Mode: {}", if margin_mode == 0 { "CROSS" } else { "ISOLATED" });
            } else {
                println!("⚠️  Leverage update returned code: {}", code);
                if let Some(msg) = response["message"].as_str() {
                    println!("  Message: {}", msg);
                }
            }
        }
        Err(e) => println!("❌ Error updating leverage: {}", e),
    }

    Ok(())
}

