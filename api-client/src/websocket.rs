use futures::{stream::StreamExt, SinkExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket client for Lighter Exchange
pub struct WebSocketClient {
    url: String,
    auth_token: Option<String>,
}

/// Message types from WebSocket
#[derive(Debug, Clone)]
pub enum WsMessage {
    OrderUpdate(Value),
    MarketData(Value),
    PositionUpdate(Value),
    Error(String),
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: String, auth_token: Option<String>) -> Self {
        Self { url, auth_token }
    }

    /// Connect to WebSocket and start listening for messages
    pub async fn connect(&self) -> Result<mpsc::UnboundedReceiver<WsMessage>, Box<dyn std::error::Error>> {
        let url = self.url.clone();
        let auth_token = self.auth_token.clone();
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| format!("Failed to connect to WebSocket: {}", e))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, rx) = mpsc::unbounded_channel();

        // Send authentication if token provided
        if let Some(token) = auth_token {
            let auth_msg = json!({
                "type": "auth",
                "token": token
            });
            write.send(Message::Text(auth_msg.to_string())).await?;
        }

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<Value>(&text) {
                            Ok(json) => {
                                let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let ws_msg = match msg_type {
                                    "order_update" => WsMessage::OrderUpdate(json),
                                    "market_data" => WsMessage::MarketData(json),
                                    "position_update" => WsMessage::PositionUpdate(json),
                                    _ => WsMessage::OrderUpdate(json),
                                };
                                let _ = tx.send(ws_msg);
                            }
                            Err(e) => {
                                let _ = tx.send(WsMessage::Error(format!("JSON parse error: {}", e)));
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        let text = String::from_utf8_lossy(&data);
                        let _ = tx.send(WsMessage::Error(format!("Binary message: {}", text)));
                    }
                    Ok(Message::Ping(_)) => {
                        // Handled automatically
                    }
                    Ok(Message::Pong(_)) => {
                        // Handled automatically
                    }
                    Ok(Message::Close(_)) => {
                        let _ = tx.send(WsMessage::Error("Connection closed".to_string()));
                        break;
                    }
                    Ok(_) => {
                        let _ = tx.send(WsMessage::Error("Unknown message type".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(WsMessage::Error(format!("WebSocket error: {}", e)));
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Subscribe to order updates
    pub async fn subscribe_orders(&self) -> Result<(), Box<dyn std::error::Error>> {
        let subscribe_msg = json!({
            "type": "subscribe",
            "channel": "orders"
        });
        
        println!("Subscribing to orders channel: {}", subscribe_msg);
        Ok(())
    }

    /// Subscribe to market data
    pub async fn subscribe_market_data(&self, market_index: u8) -> Result<(), Box<dyn std::error::Error>> {
        let subscribe_msg = json!({
            "type": "subscribe",
            "channel": "market_data",
            "market_index": market_index
        });
        
        println!("Subscribing to market data: {}", subscribe_msg);
        Ok(())
    }

    /// Subscribe to position updates
    pub async fn subscribe_positions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let subscribe_msg = json!({
            "type": "subscribe",
            "channel": "positions"
        });
        
        println!("Subscribing to positions: {}", subscribe_msg);
        Ok(())
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
        let unsub_msg = json!({
            "type": "unsubscribe",
            "channel": channel
        });
        
        println!("Unsubscribing from {}: {}", channel, unsub_msg);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_client_creation() {
        let client = WebSocketClient::new(
            "wss://mainnet.zklighter.elliot.ai/ws".to_string(),
            Some("test_token".to_string()),
        );
        assert_eq!(client.url, "wss://mainnet.zklighter.elliot.ai/ws");
        assert!(client.auth_token.is_some());
    }
}
