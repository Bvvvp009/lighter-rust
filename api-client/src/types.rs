use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type JsonObject = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cursor {
    #[serde(default, alias = "next_cursor")]
    pub next: Option<String>,
    #[serde(default)]
    pub has_next: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountAsset {
    #[serde(default, alias = "asset_id")]
    pub asset_index: u32,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default, alias = "balance")]
    pub total_balance: Option<String>,
    #[serde(default)]
    pub available_balance: Option<String>,
    #[serde(default)]
    pub locked_balance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountPosition {
    #[serde(default, alias = "market_id")]
    pub market_index: u32,
    #[serde(default, alias = "position")]
    pub base_amount: Option<String>,
    #[serde(default, alias = "position_value")]
    pub quote_amount: Option<String>,
    #[serde(default, alias = "avg_entry_price")]
    pub entry_price: Option<String>,
    #[serde(default)]
    pub liquidation_price: Option<String>,
    #[serde(default)]
    pub unrealized_pnl: Option<String>,
    #[serde(default)]
    pub initial_margin_fraction: Option<String>,
    #[serde(default)]
    pub margin_mode: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountLimits {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub account_index: i64,
    #[serde(default)]
    pub max_llp_percentage: Option<i32>,
    #[serde(default)]
    pub max_llp_amount: Option<String>,
    #[serde(default)]
    pub user_tier: Option<String>,
    #[serde(default)]
    pub can_create_public_pool: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    #[serde(alias = "account_id")]
    pub account_index: i64,
    #[serde(default)]
    pub index: Option<i64>,
    #[serde(default)]
    pub l1_address: Option<String>,
    #[serde(default)]
    pub available_balance: Option<String>,
    #[serde(default)]
    pub collateral: Option<String>,
    #[serde(default)]
    pub total_asset_value: Option<String>,
    #[serde(default)]
    pub cross_asset_value: Option<String>,
    #[serde(default)]
    pub account_trading_mode: Option<u8>,
    #[serde(default)]
    pub assets: Option<Vec<AccountAsset>>,
    #[serde(default)]
    pub positions: Option<Vec<AccountPosition>>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetailedAccounts {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub total: Option<i64>,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Status {
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub network_id: Option<i32>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZkLighterInfo {
    #[serde(default)]
    pub contract_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeyInfo {
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default)]
    pub api_key_index: Option<u8>,
    #[serde(default)]
    pub nonce: Option<i64>,
    #[serde(default)]
    pub public_key: Option<String>,
    #[serde(default)]
    pub transaction_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountApiKeys {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, alias = "api_keys")]
    pub api_keys: Vec<ApiKeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespGetMakerOnlyApiKeys {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub api_key_indexes: Vec<u16>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespSetMakerOnlyApiKeys {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PnLEntry {
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub trade_pnl: Option<f64>,
    #[serde(default)]
    pub trade_spot_pnl: Option<f64>,
    #[serde(default)]
    pub inflow: Option<f64>,
    #[serde(default)]
    pub outflow: Option<f64>,
    #[serde(default)]
    pub spot_outflow: Option<f64>,
    #[serde(default)]
    pub spot_inflow: Option<f64>,
    #[serde(default)]
    pub pool_pnl: Option<f64>,
    #[serde(default)]
    pub pool_inflow: Option<f64>,
    #[serde(default)]
    pub pool_outflow: Option<f64>,
    #[serde(default)]
    pub staking_pnl: Option<f64>,
    #[serde(default)]
    pub staking_inflow: Option<f64>,
    #[serde(default)]
    pub staking_outflow: Option<f64>,
    #[serde(default)]
    pub pool_total_shares: Option<f64>,
    #[serde(default)]
    pub staked_lit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountPnL {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub pnl: Vec<PnLEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountMetadata {
    #[serde(default)]
    pub account_index: i64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub can_invite: Option<bool>,
    #[serde(default)]
    pub referral_points_percentage: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountMetadatas {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub account_metadatas: Vec<AccountMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceLevel {
    pub price: String,
    #[serde(alias = "size")]
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBook {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default, alias = "market_id")]
    pub market_index: u32,
    #[serde(default)]
    pub market_type: Option<String>,
    #[serde(default)]
    pub asks: Vec<PriceLevel>,
    #[serde(default)]
    pub bids: Vec<PriceLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBooks {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub order_books: Vec<OrderBook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBookDepthWithBeginNonce {
    #[serde(default)]
    pub asks: Vec<PriceLevel>,
    #[serde(default)]
    pub bids: Vec<PriceLevel>,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub nonce: i64,
    #[serde(default)]
    pub begin_nonce: i64,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBookDetails {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default, alias = "market_id")]
    pub market_index: u32,
    #[serde(default)]
    pub market_type: Option<String>,
    #[serde(default)]
    pub min_base_amount: Option<String>,
    #[serde(default)]
    pub min_quote_amount: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub size_decimals: Option<u8>,
    #[serde(default)]
    pub price_decimals: Option<u8>,
    #[serde(default)]
    pub supported_size_decimals: Option<u8>,
    #[serde(default)]
    pub supported_price_decimals: Option<u8>,
    #[serde(default)]
    pub last_trade_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBookDetailsResponse {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub order_book_details: Vec<OrderBookDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Order {
    #[serde(default)]
    pub order_index: i64,
    #[serde(default)]
    pub client_order_index: Option<u64>,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default, alias = "owner_account_index")]
    pub account_index: i64,
    #[serde(default)]
    pub market_index: u32,
    #[serde(default, alias = "initial_base_amount")]
    pub base_amount: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub remaining_base_amount: Option<String>,
    #[serde(default)]
    pub filled_base_amount: Option<String>,
    #[serde(default)]
    pub filled_quote_amount: Option<String>,
    #[serde(default)]
    pub is_ask: bool,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default, alias = "type")]
    pub order_type: Option<String>,
    #[serde(default)]
    pub time_in_force: Option<String>,
    #[serde(default)]
    pub reduce_only: bool,
    #[serde(default)]
    pub trigger_price: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Orders {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trade {
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default, alias = "market_id")]
    pub market_index: u32,
    #[serde(default, alias = "ask_account_id")]
    pub taker_account_index: Option<i64>,
    #[serde(default, alias = "bid_account_id")]
    pub maker_account_index: Option<i64>,
    #[serde(default, alias = "size")]
    pub base_amount: String,
    #[serde(default)]
    pub price: String,
    #[serde(default, alias = "is_maker_ask")]
    pub taker_is_ask: Option<bool>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trades {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DepositHistoryItem {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<u16>,
    #[serde(default, alias = "amount")]
    pub usdc_amount: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub l1_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WithdrawHistoryItem {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<u16>,
    #[serde(default, alias = "amount")]
    pub usdc_amount: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
    #[serde(default, alias = "l1_tx_hash")]
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferHistoryItem {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<u16>,
    #[serde(default, alias = "amount")]
    pub usdc_amount: String,
    #[serde(default)]
    pub fee: Option<String>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default)]
    pub from_account_index: Option<i64>,
    #[serde(default)]
    pub to_account_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnrichedTx {
    #[serde(default, alias = "hash")]
    pub tx_hash: String,
    #[serde(default, alias = "type")]
    pub tx_type: u32,
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default)]
    pub status: Option<i64>,
    #[serde(default, alias = "queued_at")]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Candle {
    pub timestamp: i64,
    pub open: String,
    pub open_raw: Option<String>,
    pub high: String,
    pub high_raw: Option<String>,
    pub low: String,
    pub low_raw: Option<String>,
    pub close: String,
    pub close_raw: Option<String>,
    pub volume: String,
    pub volume_raw: Option<String>,
    pub last_trade_id: Option<i64>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FundingEntry {
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub rate: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FundingRate {
    #[serde(default, alias = "market_id")]
    pub market_index: u32,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExchangeStats {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub order_book_stats: Vec<serde_json::Value>,
    #[serde(default)]
    pub daily_usd_volume: Option<f64>,
    #[serde(default)]
    pub daily_trades_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsConnected {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsOrderBookUpdate {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub market_id: i64,
    #[serde(default)]
    pub order_book: OrderBookDepthWithBeginNonce,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsAccountMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub account_id: i64,
    #[serde(default)]
    pub available_balance: Option<String>,
    #[serde(default)]
    pub account_trading_mode: Option<u8>,
    #[serde(default)]
    pub account: Option<serde_json::Value>,
    #[serde(default)]
    pub positions: Option<serde_json::Value>,
    #[serde(default)]
    pub orders: Option<serde_json::Value>,
    #[serde(default)]
    pub open_orders: Option<serde_json::Value>,
    #[serde(default)]
    pub assets: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsAccountAssets {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub assets: BTreeMap<String, AccountAsset>,
    #[serde(default)]
    pub account_id: i64,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsAccountAllOrders {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub account: i64,
    #[serde(default)]
    pub nonce: i64,
    #[serde(default)]
    pub orders: BTreeMap<String, Vec<Order>>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartnerStats {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub total_fees_earned: Option<String>,
    #[serde(default)]
    pub total_taker_fees_earned: Option<String>,
    #[serde(default)]
    pub total_maker_fees_earned: Option<String>,
    #[serde(default)]
    pub total_volume: Option<String>,
    #[serde(default)]
    pub total_taker_volume: Option<String>,
    #[serde(default)]
    pub total_maker_volume: Option<String>,
    #[serde(default)]
    pub total_trades: Option<i64>,
    #[serde(default)]
    pub total_taker_trades: Option<i64>,
    #[serde(default)]
    pub total_maker_trades: Option<i64>,
    #[serde(default)]
    pub unique_clients: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Asset {
    #[serde(default, alias = "asset_id")]
    pub asset_index: u32,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub decimals: Option<u8>,
    #[serde(default)]
    pub l1_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetDetails {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub asset_details: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicPoolMetadata {
    #[serde(default)]
    pub account_index: i64,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub master_account_index: Option<i64>,
    #[serde(default)]
    pub account_type: Option<u8>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub l1_address: Option<String>,
    #[serde(default)]
    pub operator_fee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicPoolsMetadataResponse {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default, alias = "public_pools_metadata")]
    pub items: Vec<PublicPoolMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResultCode {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TxHashResponse {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, alias = "hash", alias = "tx_hash")]
    pub tx_hash: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct L1Metadata {
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default)]
    pub l1_address: Option<String>,
    #[serde(default, alias = "chain_id", alias = "l1_chain_id")]
    pub chain_id: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaseOption {
    #[serde(default)]
    pub duration_days: Option<u32>,
    #[serde(default)]
    pub apr: Option<serde_json::Value>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaseOptionsResponse {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default, alias = "lease_options", alias = "leaseOptions")]
    pub lease_options: Vec<LeaseOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaseInfo {
    #[serde(default, alias = "lease_id")]
    pub lease_id: Option<i64>,
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub duration_days: Option<u32>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiquidationInfo {
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default, alias = "market_id")]
    pub market_index: Option<u32>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub price: Option<serde_json::Value>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PositionFundingInfo {
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default, alias = "market_id")]
    pub market_index: Option<u32>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(default)]
    pub funding_rate: Option<serde_json::Value>,
    #[serde(default, alias = "timestamp")]
    pub created_at: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiTokenInfo {
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub account_index: Option<i64>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub scopes: Option<serde_json::Value>,
    #[serde(default)]
    pub expiry: Option<i64>,
    #[serde(default)]
    pub sub_account_access: Option<bool>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub revoked_at: Option<i64>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiTokensResponse {
    #[serde(default)]
    pub code: Option<i32>,
    #[serde(default, alias = "tokens", alias = "api_tokens")]
    pub tokens: Vec<ApiTokenInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOrderUpdate {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub order: Option<Order>,
    pub account_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMarketData {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub market_index: Option<u32>,
    pub best_ask: Option<String>,
    pub best_bid: Option<String>,
    pub last_price: Option<String>,
    pub volume_24h: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPositionUpdate {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub position: Option<AccountPosition>,
    pub account_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsTradeNotification {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub trade: Option<Trade>,
}
