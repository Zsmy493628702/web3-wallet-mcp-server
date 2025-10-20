use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub eth_balance: Decimal,
    pub token_balances: HashMap<String, TokenBalance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub contract_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub balance: Decimal,
    pub balance_formatted: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceInfo {
    pub token_address: String,
    pub symbol: String,
    pub price_usd: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapSimulation {
    pub from_token: String,
    pub to_token: String,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub gas_estimate: u64,
    pub gas_price: Decimal,
    pub total_cost: Decimal,
    pub route: Vec<String>,
    pub slippage_tolerance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: serde_json::Value,
    #[serde(skip)]
    pub request_context: Option<crate::logging::RequestContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPErrorResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPErrorResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: serde_json::Value,
    pub is_error: bool,
}
