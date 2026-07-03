use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚫 CANCEL ALL ORDERS EXAMPLE");
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

    // Cancel all orders immediately
    // time_in_force: 0 = ImmediateCancelAll, 1 = ScheduledCancelAll, 2 = AbortScheduledCancelAll
    // time: timestamp for scheduled cancellation (0 for immediate)
    let time_in_force = 0u8; // Immediate
    let time = 0i64; // Not used for immediate cancellation

    println!("📝 Canceling all orders...");
    println!("  Time In Force: {} (0 = Immediate)", time_in_force);
    println!("  Time: {}", time);
    println!();

    let response = client.cancel_all_orders(time_in_force, time).await?;

    println!("✅ Cancel all orders submitted!");
    println!("📥 Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("\n✅ All orders canceled successfully!");
        if let Some(tx_hash) = response["tx_hash"].as_str() {
            println!("  Transaction Hash: {}", tx_hash);
        }
    } else {
        println!("\n⚠️  Cancel all orders returned code: {}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("  Message: {}", msg);
        }
    }

    Ok(())
}
