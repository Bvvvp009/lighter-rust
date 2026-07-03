use api_client::LighterClient;
use std::env;
use std::error::Error;
use std::collections::HashSet;
use tokio::time::{sleep, Duration};

type DynError = Box<dyn Error>;

fn env_var(name: &str) -> Result<String, DynError> {
    env::var(name).map_err(|_| format!("Missing required environment variable: {name}").into())
}

fn parse_target_order_indexes() -> HashSet<i64> {
    env::var("TARGET_ORDER_IDS")
        .ok()
        .map(|value| {
            value
                .split(|ch: char| ch == ',' || ch.is_whitespace())
                .filter_map(|entry| entry.trim().parse::<i64>().ok())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    dotenv::dotenv().ok();

    let base_url = env_var("BASE_URL")?;
    let account_index: i64 = env_var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env_var("API_KEY_INDEX")?.parse()?;
    let api_key = env_var("API_PRIVATE_KEY")?;
    let market_index: u32 = env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let target_order_indexes = parse_target_order_indexes();
    let targeted_mode = !target_order_indexes.is_empty();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;

    let active = client
        .get_account_active_orders(account_index, Some(market_index), Some(200), None)
        .await?
        .items;

    println!("Active orders on market {} before cleanup: {}", market_index, active.len());
    if targeted_mode {
        println!("Targeted order indexes: {:?}", target_order_indexes);
    }
    for order in &active {
        println!(
            "  order_index={} client_order_index={:?} is_ask={:?} price={:?} base_amount={:?}",
            order.order_index,
            order.client_order_index,
            order.is_ask,
            order.price,
            order.base_amount
        );
    }

    for order in active {
        if targeted_mode && !target_order_indexes.contains(&order.order_index) {
            continue;
        }

        let response = client.cancel_order(market_index as u8, order.order_index).await?;
        println!("cancel {} => {}", order.order_index, response);
        sleep(Duration::from_millis(400)).await;
    }

    let remaining = client
        .get_account_active_orders(account_index, Some(market_index), Some(200), None)
        .await?
        .items;
    println!("Active orders on market {} after cleanup: {}", market_index, remaining.len());

    if targeted_mode {
        for order in remaining.iter().filter(|order| target_order_indexes.contains(&order.order_index)) {
            println!(
                "  remaining targeted order => order_index={} client_order_index={:?} status={:?} price={:?} base_amount={:?}",
                order.order_index,
                order.client_order_index,
                order.status,
                order.price,
                order.base_amount,
            );
        }
    }

    Ok(())
}
