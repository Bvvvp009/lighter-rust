use api_client::LighterClient;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    let account_index: i64 = env::var("ACCOUNT_INDEX")
        .unwrap_or_else(|_| "361816".to_string())
        .parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")
        .unwrap_or_else(|_| "6".to_string())
        .parse()?;
    let api_key = env::var("API_PRIVATE_KEY")
        .unwrap_or_else(|_| "c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51".to_string());

    println!("Testing single market order submission...");
    println!("  URL: {}", base_url);
    println!("  Account: {}", account_index);
    println!("  API Key Index: {}", api_key_index);

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    // Single market order
    let client_order_index = 99999u64;
    println!("\nSubmitting order (client_order_index={})...", client_order_index);
    
    let resp = client
        .create_market_order(
            0,                 // order_book_index (BTC/ETH)
            client_order_index,
            1000,              // base_amount
            350000,            // avg_execution_price
            false,             // is_ask (buy)
        )
        .await?;

    let code = resp["code"].as_i64().unwrap_or_default();
    println!("\nResponse code: {}", code);
    println!("Response: {}", serde_json::to_string_pretty(&resp)?);

    if code == 200 {
        println!("\n✅ Order succeeded!");
    } else if code == 21120 {
        println!("\n❌ Invalid signature (code 21120)");
    } else {
        println!("\n⚠️  Other error");
    }

    Ok(())
}
