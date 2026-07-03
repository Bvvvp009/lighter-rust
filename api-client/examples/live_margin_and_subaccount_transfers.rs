use api_client::LighterClient;
use std::env;
use tokio::time::{sleep, Duration};

fn parse_bool_env(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> LIVE MARGIN + SUBACCOUNT TRANSFERS");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let parent_account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;
    let market_index: u8 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let usdc_amount: i64 = env::var("LIVE_TRANSFER_TEST_AMOUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000); // 0.1 USDC if 6 decimals
    let fee: i64 = env::var("LIVE_TRANSFER_TEST_FEE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let skip_sub_transfer = parse_bool_env("SKIP_SUBACCOUNT_TRANSFER");

    println!("Base URL: {}", base_url);
    println!("Parent account: {}", parent_account_index);
    println!("API key index: {}", api_key_index);
    println!("Market index: {}", market_index);
    println!("Test amount (raw USDC units): {}", usdc_amount);
    println!();

    let parent = LighterClient::new(base_url.clone(), &api_key, parent_account_index, api_key_index)?;
    parent.check_api_key().await?;

    println!("STEP 1 - PERP -> SPOT");
    println!("{}", "-".repeat(80));
    let perp_to_spot = parent
        .transfer_with_routes(
            parent_account_index,
            3, // USDC
            0, // perp
            1, // spot
            usdc_amount,
            fee,
            [0x31; 32],
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&perp_to_spot)?);
    println!();

    sleep(Duration::from_secs(2)).await;

    println!("STEP 2 - SPOT -> PERP");
    println!("{}", "-".repeat(80));
    let spot_to_perp = parent
        .transfer_with_routes(
            parent_account_index,
            3, // USDC
            1, // spot
            0, // perp
            usdc_amount,
            fee,
            [0x32; 32],
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&spot_to_perp)?);
    println!();

    if skip_sub_transfer {
        println!("Skipping sub-account transfer steps because SKIP_SUBACCOUNT_TRANSFER=true");
        return Ok(());
    }

    let me = parent.get_my_account().await?;
    let l1_address = match me.l1_address {
        Some(addr) if !addr.is_empty() => addr,
        _ => {
            println!("No L1 address available on account record; cannot auto-discover sub-accounts.");
            return Ok(());
        }
    };

    let accounts = parent.get_accounts_by_l1_address(&l1_address).await?;
    let sub_accounts: Vec<i64> = accounts
        .into_iter()
        .map(|a| a.account_index)
        .filter(|idx| *idx != parent_account_index)
        .collect();

    if sub_accounts.is_empty() {
        println!("No sub-account discovered under this L1 address yet; cannot run move-to/move-out flow.");
        return Ok(());
    }

    let sub_account_index = sub_accounts[0];
    println!("Discovered sub-account: {}", sub_account_index);
    println!();

    println!("STEP 3 - MOVE TO SUB-ACCOUNT");
    println!("{}", "-".repeat(80));
    let to_sub = parent
        .transfer_with_routes(
            sub_account_index,
            3, // USDC
            0, // parent perp
            1, // sub-account spot
            usdc_amount,
            fee,
            [0x11; 32],
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&to_sub)?);
    println!();

    sleep(Duration::from_secs(2)).await;

    println!("STEP 4 - MOVE OUT OF SUB-ACCOUNT");
    println!("{}", "-".repeat(80));
    let sub_client = LighterClient::new(base_url, &api_key, sub_account_index, api_key_index)?;
    let _ = sub_client.refresh_nonce().await?;
    let from_sub = sub_client
        .transfer_with_routes(
            parent_account_index,
            3, // USDC
            1, // sub-account spot
            0, // parent perp
            usdc_amount,
            fee,
            [0x22; 32],
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&from_sub)?);
    println!();

    Ok(())
}
