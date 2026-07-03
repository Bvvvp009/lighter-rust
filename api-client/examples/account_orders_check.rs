use api_client::{LighterClient, Order};
use std::env;
use std::error::Error;

type DynError = Box<dyn Error>;

fn env_var(name: &str) -> Result<String, DynError> {
    env::var(name).map_err(|_| format!("Missing required environment variable: {name}").into())
}

fn load_env_file() {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut candidates = vec![current_dir.join(".env")];

    if let Some(parent) = current_dir.parent() {
        candidates.push(parent.join(".env"));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(".env"));
        }
    }

    for env_file in candidates {
        if !env_file.exists() {
            continue;
        }

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

                    if !key.is_empty() && !value.is_empty() && std::env::var_os(key).is_none() {
                        std::env::set_var(key, value);
                    }
                }
            }
        }

        break;
    }
}

fn parse_limit() -> u32 {
    env::var("ORDERS_LIMIT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(200)
}

fn parse_market_filter() -> Option<u32> {
    env::var("ORDER_BOOK_INDEX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
}

fn order_label(order: &Order) -> String {
    format!(
        "order_id={} order_index={} client_order_index={:?} status={:?} market_index={} is_ask={} price={} base_amount={} reduce_only={} order_type={:?} time_in_force={:?}",
        order.order_index,
        order.order_index,
        order.client_order_index,
        order.status,
        order.market_index,
        order.is_ask,
        order.price,
        order.base_amount,
        order.reduce_only,
        order.order_type,
        order.time_in_force,
    )
}

async fn fetch_all_active_orders(
    client: &LighterClient,
    account_index: i64,
    market_index: Option<u32>,
    limit: u32,
) -> Result<Vec<Order>, DynError> {
    let mut all_orders = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let page = client
            .get_account_active_orders(account_index, market_index, Some(limit), cursor.as_deref())
            .await?;
        cursor = page.cursor.and_then(|value| value.next);
        all_orders.extend(page.items);

        if cursor.is_none() {
            break;
        }
    }

    Ok(all_orders)
}

async fn fetch_all_inactive_orders(
    client: &LighterClient,
    account_index: i64,
    market_index: Option<u32>,
    limit: u32,
) -> Result<Vec<Order>, DynError> {
    let mut all_orders = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let page = client
            .get_account_inactive_orders(account_index, market_index, Some(limit), cursor.as_deref())
            .await?;
        cursor = page.cursor.and_then(|value| value.next);
        all_orders.extend(page.items);

        if cursor.is_none() {
            break;
        }
    }

    Ok(all_orders)
}

fn print_orders(title: &str, orders: &[Order]) {
    println!("{} ({})", title, orders.len());
    for order in orders {
        println!("  {}", order_label(order));
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("{}", "=".repeat(80));
    println!("ACCOUNT ACTIVE + INACTIVE ORDERS CHECK");
    println!("{}", "=".repeat(80));
    println!();

    load_env_file();
    dotenv::dotenv().ok();

    let base_url = env_var("BASE_URL")?;
    let account_index: i64 = env_var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env_var("API_KEY_INDEX")?.parse()?;
    let api_key = env_var("API_PRIVATE_KEY")?;
    let market_index = parse_market_filter();
    let limit = parse_limit();

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!("  Market Filter: {:?}", market_index);
    println!("  Page Limit: {}", limit);
    println!();

    let client = LighterClient::new(base_url, &api_key, account_index, api_key_index)?;
    client.check_api_key().await?;

    let active_orders = fetch_all_active_orders(&client, account_index, market_index, limit).await?;
    let inactive_orders = fetch_all_inactive_orders(&client, account_index, market_index, limit).await?;

    print_orders("Active orders", &active_orders);
    println!();
    print_orders("Inactive orders", &inactive_orders);
    println!();
    println!("Summary:");
    println!("  active_total={}", active_orders.len());
    println!("  inactive_total={}", inactive_orders.len());
    println!("{}", "=".repeat(80));

    Ok(())
}
