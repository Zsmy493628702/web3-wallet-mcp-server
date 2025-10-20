use crate::error::MCPError;
use crate::types::{ToolCall, ToolResult};
use crate::ethereum::EthereumClient;
use crate::error::InputValidator;
use serde_json::{Value, json};
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{instrument, info, error, warn};
use std::time::Instant;

pub struct ToolHandler {
    ethereum_client: EthereumClient,
}

impl ToolHandler {
    pub fn new(ethereum_client: EthereumClient) -> Self {
        Self { ethereum_client }
    }

    #[instrument(skip(self), fields(tool_name = %tool_call.name))]
    pub async fn handle_tool_call(&self, tool_call: ToolCall) -> Result<ToolResult, MCPError> {
        let start_time = Instant::now();
        
        // Log tool call start
        info!(
            tool_name = %tool_call.name,
            "Tool call started"
        );
        
        // Validate input parameters
        if let Err(validation_error) = InputValidator::validate_tool_parameters(&tool_call.name, &tool_call.arguments) {
            error!(
                tool_name = %tool_call.name,
                error = %validation_error,
                "Input validation failed"
            );
            return Err(validation_error);
        }
        
        let result = match tool_call.name.as_str() {
            "get_balance" => self.handle_get_balance(tool_call.arguments).await,
            "get_token_price" => self.handle_get_token_price(tool_call.arguments).await,
            "swap_tokens" => self.handle_swap_tokens(tool_call.arguments).await,
            _ => {
                error!(tool_name = %tool_call.name, "Unknown tool requested");
                Err(MCPError::ValidationError(format!("Unknown tool: {}", tool_call.name)))
            },
        };

        let duration = start_time.elapsed();
        let success = result.is_ok();
        
        // Log tool call result
        if success {
            info!(
                tool_name = %tool_call.name,
                duration_ms = duration.as_millis(),
                "Tool call completed successfully"
            );
        } else {
            let error = result.as_ref().unwrap_err();
            let severity = error.severity();
            
            match severity {
                crate::error::ErrorSeverity::Critical | crate::error::ErrorSeverity::High => {
                    error!(
                        tool_name = %tool_call.name,
                        duration_ms = duration.as_millis(),
                        error = %error,
                        severity = ?severity,
                        "Tool call failed with high severity error"
                    );
                },
                crate::error::ErrorSeverity::Medium => {
                    warn!(
                        tool_name = %tool_call.name,
                        duration_ms = duration.as_millis(),
                        error = %error,
                        severity = ?severity,
                        "Tool call failed with medium severity error"
                    );
                },
                crate::error::ErrorSeverity::Low => {
                    info!(
                        tool_name = %tool_call.name,
                        duration_ms = duration.as_millis(),
                        error = %error,
                        severity = ?severity,
                        "Tool call failed with low severity error"
                    );
                }
            }
        }

        result
    }

    #[instrument(skip(self), fields(address = %args.get("address").and_then(|v| v.as_str()).unwrap_or("unknown")))]
    async fn handle_get_balance(&self, args: Value) -> Result<ToolResult, MCPError> {
        let address = args.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::JsonRpc("Missing 'address' parameter".to_string()))?;

        let token_address = args.get("token_address")
            .and_then(|v| v.as_str());

        info!(
            address = %address,
            token_address = token_address.unwrap_or("all"),
            "Fetching balance information"
        );

        let balance_info = self.ethereum_client.get_balance(address, token_address).await?;

        info!(
            address = %address,
            eth_balance = %balance_info.eth_balance,
            token_count = balance_info.token_balances.len(),
            "Balance information retrieved successfully"
        );

        Ok(ToolResult {
            content: json!(balance_info),
            is_error: false,
        })
    }

    #[instrument(skip(self), fields(token_address = %args.get("token_address").and_then(|v| v.as_str()).unwrap_or("unknown")))]
    async fn handle_get_token_price(&self, args: Value) -> Result<ToolResult, MCPError> {
        let token_address = args.get("token_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::JsonRpc("Missing 'token_address' parameter".to_string()))?;

        info!(
            token_address = %token_address,
            "Fetching token price information"
        );

        let price_info = self.ethereum_client.get_token_price(token_address).await?;

        info!(
            token_address = %token_address,
            price_usd = %price_info.price_usd,
            "Token price information retrieved successfully"
        );

        Ok(ToolResult {
            content: json!(price_info),
            is_error: false,
        })
    }

    #[instrument(skip(self), fields(from_token = %args.get("from_token").and_then(|v| v.as_str()).unwrap_or("unknown"), to_token = %args.get("to_token").and_then(|v| v.as_str()).unwrap_or("unknown")))]
    async fn handle_swap_tokens(&self, args: Value) -> Result<ToolResult, MCPError> {
        let from_token = args.get("from_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::JsonRpc("Missing 'from_token' parameter".to_string()))?;

        let to_token = args.get("to_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::JsonRpc("Missing 'to_token' parameter".to_string()))?;

        let amount_str = args.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::JsonRpc("Missing 'amount' parameter".to_string()))?;

        let amount = Decimal::from_str(amount_str)
            .map_err(|e| MCPError::JsonRpc(format!("Invalid amount: {}", e)))?;

        let slippage_str = args.get("slippage_tolerance")
            .and_then(|v| v.as_str())
            .unwrap_or("0.5");

        let slippage = Decimal::from_str(slippage_str)
            .map_err(|e| MCPError::JsonRpc(format!("Invalid slippage: {}", e)))?;

        info!(
            from_token = %from_token,
            to_token = %to_token,
            amount = %amount,
            slippage = %slippage,
            "Simulating token swap"
        );

        let simulation = self.ethereum_client.simulate_swap(from_token, to_token, amount, slippage).await?;

        info!(
            from_token = %from_token,
            to_token = %to_token,
            amount_in = %simulation.amount_in,
            amount_out = %simulation.amount_out,
            gas_estimate = simulation.gas_estimate,
            "Token swap simulation completed successfully"
        );

        Ok(ToolResult {
            content: json!(simulation),
            is_error: false,
        })
    }
}
