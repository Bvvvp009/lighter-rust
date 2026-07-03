use signer::KeyManager;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Result;

/// Standalone signing client for auth-token and message signing flows.
///
/// This client is intentionally separate from HTTP/WebSocket transports,
/// so callers can use signing independently.
pub struct SignerClient {
    key_manager: KeyManager,
    account_index: i64,
    api_key_index: u8,
}

impl SignerClient {
    /// Create a signer client from API private key and account metadata.
    pub fn new(private_key_hex: &str, account_index: i64, api_key_index: u8) -> Result<Self> {
        let key_manager = KeyManager::from_hex(private_key_hex)?;
        Ok(Self {
            key_manager,
            account_index,
            api_key_index,
        })
    }

    /// Sign a pre-hashed 40-byte message.
    pub fn sign(&self, message: &[u8; 40]) -> Result<[u8; 80]> {
        Ok(self.key_manager.sign(message)?)
    }

    /// Build a Lighter auth token with `expiry_seconds` from now.
    pub fn create_auth_token(&self, expiry_seconds: i64) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let deadline = now + expiry_seconds;
        Ok(self
            .key_manager
            .create_auth_token(deadline, self.account_index, self.api_key_index)?)
    }

    /// Return the configured account index.
    pub fn account_index(&self) -> i64 {
        self.account_index
    }

    /// Return the configured API key index.
    pub fn api_key_index(&self) -> u8 {
        self.api_key_index
    }

    /// Get public key bytes for key registration flows.
    pub fn public_key_bytes(&self) -> [u8; 40] {
        self.key_manager.public_key_bytes()
    }
}
