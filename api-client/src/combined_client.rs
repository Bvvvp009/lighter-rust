use crate::{LighterClient, Result, WebSocketClient};

/// Combined transport client that bundles REST and WebSocket clients.
///
/// - REST is handled by `LighterClient`
/// - WebSocket is handled by `WebSocketClient`
///
/// Signing remains available independently via `SignerClient`.
pub struct CombinedClient {
    pub rest: LighterClient,
    pub ws: WebSocketClient,
}

impl CombinedClient {
    /// Build a combined client and derive WS URL from `base_url`.
    ///
    /// Example:
    /// - `https://mainnet.zklighter.elliot.ai` -> `wss://mainnet.zklighter.elliot.ai/stream`
    pub fn new(
        base_url: String,
        private_key_hex: &str,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<Self> {
        let rest = LighterClient::new(
            base_url.clone(),
            private_key_hex,
            account_index,
            api_key_index,
        )?;
        let ws_url = Self::derive_ws_url(&base_url);
        let auth_token = Some(rest.create_auth_token(600)?);
        let ws = WebSocketClient::new(ws_url, auth_token);
        Ok(Self { rest, ws })
    }

    /// Build a combined client with explicit WebSocket URL.
    pub fn with_ws_url(
        base_url: String,
        ws_url: String,
        private_key_hex: &str,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<Self> {
        let rest = LighterClient::new(base_url, private_key_hex, account_index, api_key_index)?;
        let auth_token = Some(rest.create_auth_token(600)?);
        let ws = WebSocketClient::new(ws_url, auth_token);
        Ok(Self { rest, ws })
    }

    /// Borrow REST client.
    pub fn rest(&self) -> &LighterClient {
        &self.rest
    }

    /// Borrow WebSocket client.
    pub fn ws(&self) -> &WebSocketClient {
        &self.ws
    }

    /// Consume and split into `(rest, ws)`.
    pub fn into_parts(self) -> (LighterClient, WebSocketClient) {
        (self.rest, self.ws)
    }

    fn derive_ws_url(base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        if let Some(host) = base.strip_prefix("https://") {
            return format!("wss://{}/stream", host);
        }
        if let Some(host) = base.strip_prefix("http://") {
            return format!("ws://{}/stream", host);
        }
        format!("wss://{}/stream", base)
    }
}

#[cfg(test)]
mod tests {
    use super::CombinedClient;

    #[test]
    fn derives_ws_url_from_https() {
        let got = CombinedClient::derive_ws_url("https://mainnet.zklighter.elliot.ai");
        assert_eq!(got, "wss://mainnet.zklighter.elliot.ai/stream");
    }

    #[test]
    fn derives_ws_url_from_http() {
        let got = CombinedClient::derive_ws_url("http://localhost:8080");
        assert_eq!(got, "ws://localhost:8080/stream");
    }
}
