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

    let before = client.get_account_limits(account_index).await?;
    let current_tier = before
        .user_tier
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    println!("before_tier={}", current_tier);

    let target_tier = if current_tier.eq_ignore_ascii_case("premium") {
        "standard"
    } else {
        "premium"
    };
    println!("target_tier={}", target_tier);

    let change = client.change_account_tier(account_index, target_tier, None).await?;
    println!("change_response={:?}", change);

    let after = client.get_account_limits(account_index).await?;
    let after_tier = after
        .user_tier
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    println!("after_tier={}", after_tier);

    if after_tier.eq_ignore_ascii_case(target_tier) {
        let restore = client.change_account_tier(account_index, &current_tier, None).await?;
        println!("restore_response={:?}", restore);
        let restored = client.get_account_limits(account_index).await?;
        println!(
            "restored_tier={}",
            restored.user_tier.unwrap_or_else(|| "unknown".to_string())
        );
    }

    Ok(())
}
