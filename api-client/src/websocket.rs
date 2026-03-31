use futures::{stream::StreamExt, SinkExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::sync::Arc;

use crate::types::{WsMarketData, WsOrderUpdate, WsPositionUpdate, WsTradeNotification};

/// WebSocket client for Lighter Exchange
pub struct WebSocketClient {
    url: String,
    auth_token: Option<String>,
    command_tx: Arc<tokio::sync::Mutex<Option<mpsc::UnboundedSender<Message>>>>,
}

/// Typed message variants received over the WebSocket.
///
/// Each variant carries a strongly-typed struct instead of a raw `Value`,
/// giving callers compile-time field access.
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// An order was created, modified, filled, or cancelled.
    OrderUpdate(WsOrderUpdate),
    /// A market data tick (best ask/bid, last price, 24 h volume).
    MarketData(WsMarketData),
    /// A position changed (size, entry price, PnL, etc.).
    PositionUpdate(WsPositionUpdate),
    /// A trade was executed (taker fill).
    Trade(WsTradeNotification),
    /// A message type not yet modelled; carries the raw JSON for forward-
    /// compatibility so consumers can inspect it themselves.
    Unknown(Value),
    /// A transport or parse error.
    Error(String),
}

impl WebSocketClient {
    /// Create a new WebSocket client.
    pub fn new(url: String, auth_token: Option<String>) -> Self {
        Self {
            url,
            auth_token,
            command_tx: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Connect to the WebSocket and start receiving typed messages.
    ///
    /// Returns an unbounded channel receiver.  Messages are dispatched until the
    /// connection closes or an unrecoverable error occurs.
    pub async fn connect(&self) -> Result<mpsc::UnboundedReceiver<WsMessage>, Box<dyn std::error::Error>> {
        let url = self.url.clone();
        let auth_token = self.auth_token.clone();

        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| format!("Failed to connect to WebSocket: {}", e))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, rx) = mpsc::unbounded_channel();
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();

        {
            let mut guard = self.command_tx.lock().await;
            *guard = Some(command_tx);
        }

        if let Some(token) = auth_token {
            let auth_msg = json!({ "type": "auth", "token": token });
            write.send(Message::Text(auth_msg.to_string())).await?;
        }

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    maybe_command = command_rx.recv() => {
                        match maybe_command {
                            Some(command) => {
                                if let Err(e) = write.send(command).await {
                                    let _ = tx.send(WsMessage::Error(format!("WebSocket send error: {}", e)));
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    maybe_msg = read.next() => {
                        match maybe_msg {
                            Some(Ok(Message::Text(text))) => {
                                match serde_json::from_str::<Value>(&text) {
                                    Ok(json_val) => {
                                        let ws_msg = Self::parse_message(json_val);
                                        let _ = tx.send(ws_msg);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(WsMessage::Error(format!("JSON parse error: {}", e)));
                                    }
                                }
                            }
                            Some(Ok(Message::Binary(data))) => {
                                let text = String::from_utf8_lossy(&data);
                                let _ = tx.send(WsMessage::Error(format!("Unexpected binary message: {}", text)));
                            }
                            Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                                // handled automatically by tungstenite
                            }
                            Some(Ok(Message::Close(_))) => {
                                let _ = tx.send(WsMessage::Error("Connection closed".to_string()));
                                break;
                            }
                            Some(Ok(_)) => {
                                let _ = tx.send(WsMessage::Error("Unknown frame type".to_string()));
                            }
                            Some(Err(e)) => {
                                let _ = tx.send(WsMessage::Error(format!("WebSocket error: {}", e)));
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Deserialise a JSON value into the correct `WsMessage` variant.
    fn parse_message(v: Value) -> WsMessage {
        let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();

        match msg_type.as_str() {
            "order_update" => {
                match serde_json::from_value::<WsOrderUpdate>(v) {
                    Ok(u) => WsMessage::OrderUpdate(u),
                    Err(e) => WsMessage::Error(format!("order_update parse error: {}", e)),
                }
            }
            "market_data" => {
                match serde_json::from_value::<WsMarketData>(v) {
                    Ok(m) => WsMessage::MarketData(m),
                    Err(e) => WsMessage::Error(format!("market_data parse error: {}", e)),
                }
            }
            "position_update" => {
                match serde_json::from_value::<WsPositionUpdate>(v) {
                    Ok(p) => WsMessage::PositionUpdate(p),
                    Err(e) => WsMessage::Error(format!("position_update parse error: {}", e)),
                }
            }
            "trade" => {
                match serde_json::from_value::<WsTradeNotification>(v) {
                    Ok(t) => WsMessage::Trade(t),
                    Err(e) => WsMessage::Error(format!("trade parse error: {}", e)),
                }
            }
            _ => WsMessage::Unknown(v),
        }
    }

    /// Send a subscription message for an account's order updates.
    ///
    /// The caller must hold on to the `write` half; this is a convenience helper
    /// showing the canonical subscription payload.
    pub fn order_subscribe_payload(account_index: i64) -> String {
        json!({
            "type": "subscribe",
            "channel": "orders",
            "account_index": account_index,
        })
        .to_string()
    }

    /// Subscription payload for market data on a specific market.
    pub fn market_data_subscribe_payload(market_index: u32) -> String {
        json!({
            "type": "subscribe",
            "channel": "market_data",
            "market_index": market_index,
        })
        .to_string()
    }

    /// Subscription payload for position updates.
    pub fn position_subscribe_payload(account_index: i64) -> String {
        json!({
            "type": "subscribe",
            "channel": "positions",
            "account_index": account_index,
        })
        .to_string()
    }

    /// Subscription payload for trade notifications on a market.
    pub fn trade_subscribe_payload(market_index: u32) -> String {
        json!({
            "type": "subscribe",
            "channel": "trades",
            "market_index": market_index,
        })
        .to_string()
    }

    /// Send a subscription request for the account's order updates.
    pub async fn subscribe_orders(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::order_subscribe_payload(0))).await?;
        Ok(())
    }

    /// Send a subscription request for market data on `market_index`.
    pub async fn subscribe_market_data(&self, market_index: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::market_data_subscribe_payload(market_index as u32))).await?;
        Ok(())
    }

    /// Send a subscription request for position updates.
    pub async fn subscribe_positions(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::position_subscribe_payload(0))).await?;
        Ok(())
    }

    /// Send an unsubscribe request for a channel.
    pub async fn unsubscribe(&self, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
        let payload = json!({
            "type": "unsubscribe",
            "channel": channel,
        })
        .to_string();
        self.send_command(Message::Text(payload)).await?;
        Ok(())
    }

    async fn send_command(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let sender = {
            let guard = self.command_tx.lock().await;
            guard.clone()
        };

        let sender = sender.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "WebSocket client is not connected")
        })?;

        sender
            .send(message)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "WebSocket command channel closed"))?;

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
