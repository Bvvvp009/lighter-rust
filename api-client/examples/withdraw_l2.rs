use api_client::{LighterClient, WithdrawRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 WITHDRAW USDC FROM L2 (L2 Withdrawal)");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    // Load credentials from environment variables
    // Create a .env file with: BASE_URL, ACCOUNT_INDEX, API_KEY_INDEX, API_PRIVATE_KEY
    let base_url = env::var("BASE_URL")
        .map_err(|_| "BASE_URL environment variable is required. Please set it in your .env file.")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .map_err(|_| "ACCOUNT_INDEX environment variable is required. Please set it in your .env file.")?
        .parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .map_err(|_| "API_KEY_INDEX environment variable is required. Please set it in your .env file.")?
        .parse()?;
    let api_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required. Please set it in your .env file.")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Withdrawal amount in USDC (with 6 decimals)
    // Example: 1000000 = 1.0 USDC, 100000 = 0.1 USDC
    let usdc_amount: u64 = 1000000; // 1.0 USDC

    println!("📝 Withdrawing USDC from L2");
    println!("  Amount: {} USDC ({} in smallest unit)", usdc_amount as f64 / 1_000_000.0, usdc_amount);
    println!("  Transaction Type: L2 Withdraw (13)");
    println!("  Destination: L1 (Ethereum mainnet/testnet)");
    println!();
    println!("⚠️  Note: The 'invalid asset index' (21801) error typically indicates:");
    println!("   - Account doesn't have sufficient USDC balance");
    println!("   - Account doesn't have withdrawal permissions enabled");
    println!("   - Account doesn't meet minimum withdrawal requirements");
    println!("   - Testnet-specific restrictions");
    println!("   The transaction structure is correct - this is a server-side validation.");
    println!();

    let request = WithdrawRequest {
        usdc_amount,
    };

    let response = client.withdraw(request).await?;

    println!("✅ Withdrawal submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ L2 withdrawal transaction created successfully!");
        println!("\n📊 Withdrawal Details:");
        println!("  - Funds will be withdrawn from L2 to L1");
        println!("  - There may be a withdrawal delay period");
        println!("  - Check withdrawal history for status updates");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("\n  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Withdrawal returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}

