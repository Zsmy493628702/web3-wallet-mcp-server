use crate::error::MCPError;
use crate::error::ErrorHandler;
use crate::types::{MCPRequest, MCPResponse, MCPErrorResponse, ToolCall};
use crate::ethereum::EthereumClient;
use crate::tools::ToolHandler;
use crate::logging::{RequestContext, log_request_start, log_request_complete, log_error};
use serde_json::{Value, json};
use tracing::{info, error, debug, instrument};

pub struct MCPServer {
    tool_handler: ToolHandler,
}

impl MCPServer {
    pub async fn new(rpc_url: String, private_key: String) -> Result<Self, MCPError> {
        let ethereum_client = EthereumClient::new(rpc_url, private_key).await?;
        let tool_handler = ToolHandler::new(ethereum_client);
        
        Ok(Self { tool_handler })
    }

    pub async fn run(&self) -> Result<(), MCPError> {
        info!("MCP Server is running and ready to accept requests");
        
        // In a real implementation, this would listen on a socket or HTTP endpoint
        // For now, we'll just keep the server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    #[instrument(skip(self), fields(request_id = %request.request_context.as_ref().map(|ctx| ctx.request_id.as_str()).unwrap_or("unknown")))]
    pub async fn handle_request(&self, mut request: MCPRequest) -> Result<MCPResponse, MCPError> {
        // Create request context
        let ctx = request.request_context.take().unwrap_or_else(|| {
            RequestContext::new(request.method.clone())
        });
        
        log_request_start(&ctx);
        
        let result = match request.method.as_str() {
            "tools/list" => {
                debug!(request_id = %ctx.request_id, "Handling tools/list request");
                self.handle_tools_list(request.id, &ctx).await
            },
            "tools/call" => {
                debug!(request_id = %ctx.request_id, "Handling tools/call request");
                self.handle_tools_call(request.id, request.params, &ctx).await
            },
            _ => {
                let error_msg = format!("Method not found: {}", request.method);
                error!(request_id = %ctx.request_id, method = %request.method, "Unknown method");
                
                let error = MCPErrorResponse {
                    code: -32601,
                    message: error_msg,
                    data: None,
                };
                Ok(MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(error),
                })
            }
        };

        // Log request completion
        match &result {
            Ok(response) => {
                let success = response.error.is_none();
                log_request_complete(&ctx, success);
            },
            Err(e) => {
                log_error(&ctx, e, "Request processing failed");
                log_request_complete(&ctx, false);
                
                // Use error handler to generate better error response
                return Ok(ErrorHandler::handle_error(e.clone(), Some(&ctx.request_id)));
            }
        }

        result
    }

    #[instrument(skip(self), fields(request_id = %ctx.request_id))]
    async fn handle_tools_list(&self, id: Value, ctx: &RequestContext) -> Result<MCPResponse, MCPError> {
        let tools = json!([
            {
                "name": "get_balance",
                "description": "Get ETH and ERC20 token balances for a wallet address",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "string",
                            "description": "Wallet address to query"
                        },
                        "token_address": {
                            "type": "string",
                            "description": "Optional token contract address"
                        }
                    },
                    "required": ["address"]
                }
            },
            {
                "name": "get_token_price",
                "description": "Get current token price in USD and ETH",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "token_address": {
                            "type": "string",
                            "description": "Token contract address"
                        }
                    },
                    "required": ["token_address"]
                }
            },
            {
                "name": "swap_tokens",
                "description": "Simulate a token swap on Uniswap",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_token": {
                            "type": "string",
                            "description": "Source token contract address"
                        },
                        "to_token": {
                            "type": "string",
                            "description": "Destination token contract address"
                        },
                        "amount": {
                            "type": "string",
                            "description": "Amount to swap (as decimal string)"
                        },
                        "slippage_tolerance": {
                            "type": "string",
                            "description": "Slippage tolerance percentage (default: 0.5)"
                        }
                    },
                    "required": ["from_token", "to_token", "amount"]
                }
            }
        ]);

        Ok(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": tools
            })),
            error: None,
        })
    }

    #[instrument(skip(self), fields(request_id = %ctx.request_id))]
    async fn handle_tools_call(&self, id: Value, params: Value, ctx: &RequestContext) -> Result<MCPResponse, MCPError> {
        let tool_call = serde_json::from_value::<ToolCall>(params)
            .map_err(|e| MCPError::JsonRpc(format!("Invalid tool call parameters: {}", e)))?;

        match self.tool_handler.handle_tool_call(tool_call).await {
            Ok(result) => {
                if result.is_error {
                    let error = MCPErrorResponse {
                        code: -32603,
                        message: "Tool execution failed".to_string(),
                        data: Some(result.content),
                    };
                    Ok(MCPResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(error),
                    })
                } else {
                    Ok(MCPResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": result.content
                        })),
                        error: None,
                    })
                }
            }
            Err(e) => {
                error!("Tool execution error: {}", e);
                let error = MCPErrorResponse {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({
                        "error": e.to_string()
                    })),
                };
                Ok(MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(error),
                })
            }
        }
    }
}
