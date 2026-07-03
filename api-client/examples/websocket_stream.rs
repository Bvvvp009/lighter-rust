use api_client::{LighterClient, WebSocketClient};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> WEBSOCKET ORDER BOOK STREAMING");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let auth_required = env::var("WEBSOCKET_AUTH")
        .ok()
        .map(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Auth Required: {}", auth_required);
    println!();

    let auth_token = if auth_required {
        let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
        let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
        let api_key = env::var("API_PRIVATE_KEY")?;
        println!("  Account Index: {}", account_index);
        println!("  API Key Index: {}", api_key_index);
        println!();

        let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;

        println!("Generating WebSocket authentication token...");
        let token = client.create_auth_token(3600)?;
        println!("Token generated successfully!");
        println!();
        Some(token)
    } else {
        println!("Skipping auth token generation; using the public order book stream only.");
        println!();
        None
    };

    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        + "/stream";
    println!("Connecting to WebSocket: {}", ws_url);
    println!();

    let ws_client = WebSocketClient::new(ws_url, auth_token);

    match ws_client.connect().await {
        Ok(mut rx) => {
            println!("Connected to WebSocket!");
            println!();

            println!("Subscribing to order book 0...");
            ws_client.subscribe_order_book(0).await?;

            if auth_required {
                let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
                println!("Subscribing to account_all {}...", account_index);
                ws_client.subscribe_account_all(account_index).await?;

                println!("Subscribing to account_all_orders {}...", account_index);
                ws_client.subscribe_account_all_orders(account_index).await?;

                println!("Subscribing to account_all_assets {}...", account_index);
                ws_client.subscribe_account_all_assets(account_index).await?;
            }
            println!();

            println!("Listening for messages (30 seconds timeout)...");

            let start = std::time::Instant::now();
            let timeout = Duration::from_secs(30);

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(msg) => {
                                println!("Received message:");
                                match msg {
                                            api_client::websocket::WsMessage::Connected(data) => {
                                                println!("  Type: Connected");
                                                println!("  Session ID: {:?}", data.session_id);
                                            }
                                            api_client::websocket::WsMessage::OrderBook(data) => {
                                                println!("  Type: Order Book");
                                                println!("  Market ID: {}", data.market_id);
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                            api_client::websocket::WsMessage::Account(data) => {
                                                println!("  Type: Account");
                                                println!("  Account ID: {}", data.account_id);
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                            api_client::websocket::WsMessage::AccountAssets(data) => {
                                                println!("  Type: Account Assets");
                                                println!("  Account ID: {}", data.account_id);
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                            api_client::websocket::WsMessage::AccountAllOrders(data) => {
                                                println!("  Type: Account All Orders");
                                                println!("  Account ID: {}", data.account);
                                                let total_orders = data.orders.values().map(|orders| orders.len()).sum::<usize>();
                                                println!("  Order Groups: {}", data.orders.len());
                                                println!("  Total Orders: {}", total_orders);
                                            }
                                            api_client::websocket::WsMessage::Ping => {
                                                println!("  Type: Ping");
                                            }
                                    api_client::websocket::WsMessage::Error(err) => {
                                        println!("  Type: Error");
                                        println!("  Message: {}", err);
                                    }
                                        api_client::websocket::WsMessage::Unknown(data) => {
                                            println!("  Type: Unknown");
                                            println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                        }
                                    }
                                println!();
                            }
                            None => {
                                println!("Channel closed");
                                break;
                            }
                        }
                    }
                    _ = sleep(Duration::from_millis(100)), if start.elapsed() > timeout => {
                        println!("Timeout reached (30 seconds)");
                        break;
                    }
                }
            }

            println!();
            println!("Unsubscribing from channels...");
            ws_client.unsubscribe("order_book/0").await?;
            if auth_required {
                let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
                ws_client.unsubscribe(&format!("account_all/{}", account_index)).await?;
                ws_client.unsubscribe(&format!("account_all_orders/{}", account_index)).await?;
                ws_client.unsubscribe(&format!("account_all_assets/{}", account_index)).await?;
            }

            println!();
            println!("Example completed successfully!");
        }
        Err(e) => {
            println!("Failed to connect to WebSocket: {}", e);
            println!();
            println!("Note: WebSocket functionality requires compatible server implementation");
            println!("This example demonstrates the client interface pattern");
        }
    }

    Ok(())
}
