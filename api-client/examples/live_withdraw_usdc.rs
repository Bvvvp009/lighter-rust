use api_client::{LighterClient, WithdrawRequest};
use std::env;

fn parse_bool_env(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================================");
    println!(">> LIVE WITHDRAW USDC");
    println!("================================================================================\n");

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    let withdraw_amount_raw: u64 = env::var("WITHDRAW_USDC_AMOUNT")
        .unwrap_or_else(|_| "1000000".to_string())
        .parse()?;
    let run_live = parse_bool_env("WITHDRAW_RUN_LIVE", true);

    println!("Base URL: {}", base_url);
    println!("Account index: {}", account_index);
    println!("API key index: {}", api_key_index);
    println!("Withdraw amount (raw USDC units): {}", withdraw_amount_raw);
    println!("Run live withdraw: {}", run_live);

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    println!("\nSTEP 1 - ACCOUNT SNAPSHOT");
    println!("--------------------------------------------------------------------------------");
    let account = client.get_account(account_index).await?;
    println!("l1_address={:?}", account.l1_address);
    println!("available_balance={:?}", account.available_balance);
    println!("collateral={:?}", account.collateral);

    println!("\nSTEP 2 - WITHDRAW HISTORY (BEFORE)");
    println!("--------------------------------------------------------------------------------");
    let before = client.get_withdraw_history(account_index, Some(5), None).await?;
    println!("withdraw history items (before): {}", before.items.len());
    if let Some(first) = before.items.first() {
        println!(
            "latest_before id={:?} amount={} status={:?} tx_hash={:?}",
            first.id, first.usdc_amount, first.status, first.tx_hash
        );
    }

    if !run_live {
        println!("\nSkipping live withdraw because WITHDRAW_RUN_LIVE=false");
        return Ok(());
    }

    println!("\nSTEP 3 - SUBMIT WITHDRAW");
    println!("--------------------------------------------------------------------------------");
    let response = client
        .withdraw(WithdrawRequest {
            usdc_amount: withdraw_amount_raw,
        })
        .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);

    let code = response["code"].as_i64().unwrap_or_default();
    if code != 200 {
        println!("\nWithdraw returned non-success code={}", code);
        if let Some(msg) = response["message"].as_str() {
            println!("message={}", msg);
        }
        return Ok(());
    }

    if let Some(tx_hash) = response["tx_hash"].as_str() {
        println!("withdraw_tx_hash={}", tx_hash);

        println!("\nSTEP 4 - TX LOOKUP");
        println!("--------------------------------------------------------------------------------");
        match client.get_transaction(tx_hash).await {
            Ok(tx) => {
                println!(
                    "tx_lookup type={} hash={} status={:?} created_at={:?}",
                    tx.tx_type, tx.tx_hash, tx.status, tx.created_at
                );
            }
            Err(e) => {
                println!("tx lookup not ready yet: {}", e);
            }
        }
    }

    println!("\nSTEP 5 - WITHDRAW HISTORY (AFTER)");
    println!("--------------------------------------------------------------------------------");
    let after = client.get_withdraw_history(account_index, Some(5), None).await?;
    println!("withdraw history items (after): {}", after.items.len());
    if let Some(first) = after.items.first() {
        println!(
            "latest_after id={:?} amount={} status={:?} tx_hash={:?}",
            first.id, first.usdc_amount, first.status, first.tx_hash
        );
    }

    println!("\nDone.");
    Ok(())
}
