use futures::{stream::StreamExt, SinkExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::types::{
    WsAccountAllOrders, WsAccountAssets, WsAccountMessage, WsConnected, WsOrderBookUpdate,
};

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
    /// The websocket connected and assigned a session id.
    Connected(WsConnected),
    /// An order book snapshot or delta for a market.
    OrderBook(WsOrderBookUpdate),
    /// A general account payload for `account_all/...` channels.
    Account(WsAccountMessage),
    /// All account orders grouped by market for `account_all_orders/...`.
    AccountAllOrders(WsAccountAllOrders),
    /// Asset balances for the `account_all_assets/...` channel.
    AccountAssets(WsAccountAssets),
    /// A JSON ping message; callers should respond with pong if they are
    /// bypassing the internal client loop.
    Ping,
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
    pub async fn connect(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<WsMessage>, Box<dyn std::error::Error>> {
        let url = self.url.clone();
        let auth_token = self.auth_token.clone();

        let (ws_stream, _) = match connect_async(&url).await {
            Ok(ok) => ok,
            Err(primary_err) => {
                if url.ends_with("/ws") {
                    let fallback_url = format!("{}/stream", url.trim_end_matches("/ws"));
                    connect_async(&fallback_url).await.map_err(|fallback_err| {
                        format!(
                            "Failed to connect to WebSocket: {}; fallback {} also failed: {}",
                            primary_err, fallback_url, fallback_err
                        )
                    })?
                } else {
                    return Err(format!("Failed to connect to WebSocket: {}", primary_err).into());
                }
            }
        };

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
                                        if json_val.get("type").and_then(|value| value.as_str()) == Some("ping") {
                                            if let Err(e) = write.send(Message::Text(json!({"type": "pong"}).to_string())).await {
                                                let _ = tx.send(WsMessage::Error(format!("WebSocket pong send error: {}", e)));
                                                break;
                                            }
                                            let _ = tx.send(WsMessage::Ping);
                                            continue;
                                        }

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
        let msg_type = v
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        match msg_type.as_str() {
            "connected" => match serde_json::from_value::<WsConnected>(v) {
                Ok(mut message) => {
                    if message.session_id.is_none() {
                        message.session_id = message
                            .extra
                            .get("session_id")
                            .and_then(|value| value.as_str())
                            .map(|value| value.to_string());
                    }
                    WsMessage::Connected(message)
                }
                Err(e) => WsMessage::Error(format!("connected parse error: {}", e)),
            },
            "subscribed/order_book" | "update/order_book" => {
                match serde_json::from_value::<WsOrderBookUpdate>(v) {
                    Ok(mut message) => {
                        if message.market_id == 0 {
                            message.market_id = Self::channel_id(&message.channel).unwrap_or_default();
                        }
                        WsMessage::OrderBook(message)
                    }
                    Err(e) => WsMessage::Error(format!("order_book parse error: {}", e)),
                }
            },
            "subscribed/account_all" | "update/account_all" => {
                match serde_json::from_value::<WsAccountMessage>(v) {
                    Ok(mut message) => {
                        if message.account_id == 0 {
                            message.account_id = message
                                .account
                                .as_ref()
                                .and_then(|value| value.as_i64())
                                .or_else(|| Self::channel_id(&message.channel))
                                .unwrap_or_default();
                        }
                        WsMessage::Account(message)
                    }
                    Err(e) => WsMessage::Error(format!("account_all parse error: {}", e)),
                }
            },
            "subscribed/account_all_orders" | "update/account_all_orders" => {
                match serde_json::from_value::<WsAccountAllOrders>(v) {
                    Ok(mut message) => {
                        if message.account == 0 {
                            message.account = Self::channel_id(&message.channel).unwrap_or_default();
                        }
                        WsMessage::AccountAllOrders(message)
                    }
                    Err(e) => WsMessage::Error(format!("account_all_orders parse error: {}", e)),
                }
            },
            "subscribed/account_all_assets" | "update/account_all_assets" => {
                match serde_json::from_value::<WsAccountAssets>(v) {
                    Ok(mut message) => {
                        if message.account_id == 0 {
                            message.account_id = Self::channel_id(&message.channel).unwrap_or_default();
                        }
                        WsMessage::AccountAssets(message)
                    }
                    Err(e) => WsMessage::Error(format!("account_all_assets parse error: {}", e)),
                }
            },
            _ => WsMessage::Unknown(v),
        }
    }

    fn channel_id(channel: &str) -> Option<i64> {
        channel
            .split_once(':')
            .or_else(|| channel.split_once('/'))
            .and_then(|(_, id)| id.parse::<i64>().ok())
    }

    /// Send a subscription message for an order book snapshot and updates.
    ///
    /// The caller must hold on to the `write` half; this is a convenience helper
    /// showing the canonical subscription payload.
    pub fn order_book_subscribe_payload(market_index: u32) -> String {
        json!({
            "type": "subscribe",
            "channel": format!("order_book/{}", market_index),
        })
        .to_string()
    }

    /// Subscription payload for general account updates.
    pub fn account_all_subscribe_payload(account_index: i64, auth_token: Option<&str>) -> String {
        let mut payload = json!({
            "type": "subscribe",
            "channel": format!("account_all/{}", account_index),
        });
        if let Some(auth_token) = auth_token {
            payload["auth"] = json!(auth_token);
        }
        payload.to_string()
    }

    /// Subscription payload for account asset updates.
    pub fn account_all_assets_subscribe_payload(
        account_index: i64,
        auth_token: Option<&str>,
    ) -> String {
        let mut payload = json!({
            "type": "subscribe",
            "channel": format!("account_all_assets/{}", account_index),
        });
        if let Some(auth_token) = auth_token {
            payload["auth"] = json!(auth_token);
        }
        payload.to_string()
    }

    /// Subscription payload for all account orders grouped by market.
    pub fn account_all_orders_subscribe_payload(
        account_index: i64,
        auth_token: Option<&str>,
    ) -> String {
        let mut payload = json!({
            "type": "subscribe",
            "channel": format!("account_all_orders/{}", account_index),
        });
        if let Some(auth_token) = auth_token {
            payload["auth"] = json!(auth_token);
        }
        payload.to_string()
    }

    /// Send a subscription request for an order book.
    pub async fn subscribe_order_book(
        &self,
        market_index: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::order_book_subscribe_payload(market_index)))
            .await?;
        Ok(())
    }

    /// Send a subscription request for general account updates.
    pub async fn subscribe_account_all(
        &self,
        account_index: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::account_all_subscribe_payload(
            account_index,
            self.auth_token.as_deref(),
        )))
            .await?;
        Ok(())
    }

    /// Send a subscription request for account asset updates.
    pub async fn subscribe_account_all_assets(
        &self,
        account_index: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::account_all_assets_subscribe_payload(
            account_index,
            self.auth_token.as_deref(),
        )))
            .await?;
        Ok(())
    }

    /// Send a subscription request for all account orders.
    pub async fn subscribe_account_all_orders(
        &self,
        account_index: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Message::Text(Self::account_all_orders_subscribe_payload(
            account_index,
            self.auth_token.as_deref(),
        )))
            .await?;
        Ok(())
    }

    /// Backwards-compatible wrapper for earlier examples.
    pub async fn subscribe_orders(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.subscribe_order_book(0).await
    }

    /// Backwards-compatible wrapper for earlier examples.
    pub async fn subscribe_market_data(
        &self,
        market_index: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.subscribe_order_book(market_index as u32).await
    }

    /// Backwards-compatible wrapper for earlier examples.
    pub async fn subscribe_positions(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.subscribe_account_all(0).await
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
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "WebSocket client is not connected",
            )
        })?;

        sender.send(message).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "WebSocket command channel closed",
            )
        })?;

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

    #[test]
    fn test_order_book_subscription_payload() {
        let payload = WebSocketClient::order_book_subscribe_payload(12);
        assert!(payload.contains("order_book/12"));
    }

    #[test]
    fn test_account_all_orders_subscription_payload() {
        let payload = WebSocketClient::account_all_orders_subscribe_payload(12, Some("auth"));
        assert!(payload.contains("account_all_orders/12"));
        assert!(payload.contains("auth"));
    }
}
