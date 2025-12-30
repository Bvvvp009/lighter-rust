use api_client::{LighterClient, WebSocketClient};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> WEBSOCKET ORDER AND MARKET DATA STREAMING");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    let client = LighterClient::new(base_url.clone(), &api_key, account_index, api_key_index)?;
    
    println!("Generating WebSocket authentication token...");
    let auth_token = client.create_auth_token(3600)?;
    println!("Token generated successfully!");
    println!();

    let ws_url = base_url.replace("http://", "ws://").replace("https://", "wss://") + "/ws";
    println!("Connecting to WebSocket: {}", ws_url);
    println!();

    let ws_client = WebSocketClient::new(ws_url, Some(auth_token));

    match ws_client.connect().await {
        Ok(mut rx) => {
            println!("Connected to WebSocket!");
            println!();

            println!("Subscribing to order updates...");
            ws_client.subscribe_orders().await?;

            println!("Subscribing to market data (market 0)...");
            ws_client.subscribe_market_data(0).await?;

            println!("Subscribing to position updates...");
            ws_client.subscribe_positions().await?;
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
                                    api_client::websocket::WsMessage::OrderUpdate(data) => {
                                        println!("  Type: Order Update");
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                    api_client::websocket::WsMessage::MarketData(data) => {
                                        println!("  Type: Market Data");
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                    api_client::websocket::WsMessage::PositionUpdate(data) => {
                                        println!("  Type: Position Update");
                                        println!("  Data: {}", serde_json::to_string_pretty(&data)?);
                                    }
                                    api_client::websocket::WsMessage::Error(err) => {
                                        println!("  Type: Error");
                                        println!("  Message: {}", err);
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
            ws_client.unsubscribe("orders").await?;
            ws_client.unsubscribe("market_data").await?;
            ws_client.unsubscribe("positions").await?;

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
