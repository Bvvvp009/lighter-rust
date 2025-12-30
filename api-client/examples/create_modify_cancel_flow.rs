use api_client::{LighterClient, CreateOrderRequest};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> CREATE -> MODIFY -> CANCEL ORDER FLOW");
    println!("{}", "=".repeat(80));
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
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 300; // 5 minutes from now

    // STEP 1: CREATE ORDER
    println!("STEP 1 - CREATE LIMIT ORDER");
    println!("{}", "-".repeat(80));
    let order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // ETH-USD
        client_order_index: 200,  // unique identifier
        base_amount: 1000,        // 0.1 ETH
        price: 4050_00,           // $4050
        is_ask: true,             // sell order
        order_type: 0,            // limit order
        time_in_force: 1,         // good till time
        reduce_only: false,
        trigger_price: 0,
    };

    let create_response = client.create_order(order).await?;
    println!("✅ Order created!");
    println!("{}", serde_json::to_string_pretty(&create_response)?);
    println!();

    let code = create_response["code"].as_i64().unwrap_or_default();
    if code != 200 {
        println!("❌ Failed to create order. Aborting...");
        return Ok(());
    }

    // Wait a moment before next operation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // STEP 2: MODIFY ORDER
    println!("STEP 2 - MODIFY ORDER");
    println!("{}", "-".repeat(80));
    println!("Increasing size to 0.11 ETH and price to $4100");
    println!();

    let modify_request = api_client::ModifyOrderRequest {
        market_index: 0,
        order_index: 200,
        base_amount: 1100,
        price: 4100_00,
        trigger_price: 0,
    };

    let modify_response = client.modify_order(modify_request).await?;

    println!("✅ Order modified!");
    println!("{}", serde_json::to_string_pretty(&modify_response)?);
    println!();

    let code = modify_response["code"].as_i64().unwrap_or_default();
    if code != 200 {
        println!("❌ Failed to modify order. Proceeding to cancel...");
    }

    // Wait a moment before cancellation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // STEP 3: CANCEL ORDER
    println!("STEP 3 - CANCEL ORDER");
    println!("{}", "-".repeat(80));
    println!();

    let cancel_response = client.cancel_order(
        0,      // order_book_index (ETH-USD)
        200,    // order_index
    ).await?;

    println!("✅ Order cancelled!");
    println!("{}", serde_json::to_string_pretty(&cancel_response)?);
    println!();

    let code = cancel_response["code"].as_i64().unwrap_or_default();
    if code == 200 {
        println!("✅ Complete flow executed successfully!");
        println!("   Created → Modified → Cancelled");
    } else {
        println!("⚠️  Final cancel returned code: {}", code);
    }

    Ok(())
}
