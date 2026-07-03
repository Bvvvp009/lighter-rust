#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use api_client;
pub use api_client::*;
pub use goldilocks_crypto;
pub use poseidon_hash;
pub use signer;
pub use signer::{KeyManager, SignerError};

/// Convenience prelude — import with `use lighter_sdk::prelude::*;`.
///
/// Covers the client entry-points you need for the most common workflows:
/// - `LighterClient`  — async REST + signing
/// - `SignerClient`   — local signing only, no HTTP pool
/// - `CombinedClient` — REST + WebSocket in one handle
/// - `WebSocketClient`— WebSocket-only streaming
/// - `KeyManager`     — key derivation and management
/// - `ApiError`       — unified error type
/// - `WsMessage`      — typed WebSocket message enum
/// - Core request/response types: `CreateOrderRequest`, `Account`, `OrderBook`,
///   `OrderBookDetails`, `Status`, `Order`, `PriceLevel`, `Page`
pub mod prelude {
    pub use crate::{
        // Clients
        CombinedClient,
        KeyManager,
        LighterClient,
        SignerClient,
        WebSocketClient,
        // Error
        ApiError,
        // WebSocket message type
        websocket::WsMessage,
        // Commonly used request/response types
        CreateOrderRequest,
        Account,
        Order,
        OrderBook,
        OrderBookDetails,
        PriceLevel,
        Status,
        Page,
    };
}

/// Crate version string (from `Cargo.toml`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexports_core_types() {
        let _ = std::any::type_name::<LighterClient>();
        let _ = std::any::type_name::<KeyManager>();
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn prelude_compiles() {
        use prelude::*;
        let _ = std::any::type_name::<LighterClient>();
        let _ = std::any::type_name::<SignerClient>();
        let _ = std::any::type_name::<CombinedClient>();
        let _ = std::any::type_name::<WebSocketClient>();
        let _ = std::any::type_name::<ApiError>();
        let _ = std::any::type_name::<WsMessage>();
        let _ = std::any::type_name::<CreateOrderRequest>();
    }
}
