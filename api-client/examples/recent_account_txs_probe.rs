use api_client::LighterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    let page = client.get_account_transactions(account_index, Some(20), None).await?;

    for tx in page.items {
        println!("type={} hash={} info={}", tx.tx_type, tx.tx_hash, tx.info.unwrap_or_default());
    }

    Ok(())
}
