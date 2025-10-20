use thiserror::Error;
use std::collections::HashMap;
use rust_decimal::Decimal;
use std::str::FromStr;
use serde_json::Value;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Error, Debug)]
pub enum MCPError {
    // JSON-RPC related errors
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),
    
    #[error("Invalid JSON-RPC request: {0}")]
    InvalidJsonRpcRequest(String),
    
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid parameter type: {0}")]
    InvalidParameterType(String),
    
    // Ethereum network related errors
    #[error("Ethereum RPC error: {0}")]
    EthereumRpc(String),
    
    #[error("Network connection failed: {0}")]
    NetworkError(String),
    
    #[error("RPC timeout: {0}")]
    RpcTimeout(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    // Address and contract related errors
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid token contract: {0}")]
    InvalidTokenContract(String),
    
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    
    #[error("Invalid contract ABI: {0}")]
    InvalidContractAbi(String),
    
    // Balance and transaction related errors
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Swap simulation failed: {0}")]
    SwapSimulationFailed(String),
    
    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),
    
    #[error("Slippage too high: {0}")]
    SlippageTooHigh(String),
    
    // Price and API related errors
    #[error("Price fetch failed: {0}")]
    PriceFetchFailed(String),
    
    #[error("API rate limit exceeded: {0}")]
    ApiRateLimitExceeded(String),
    
    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),
    
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    
    // Wallet and signature related errors
    #[error("Wallet error: {0}")]
    WalletError(String),
    
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    
    #[error("Wallet not initialized: {0}")]
    WalletNotInitialized(String),
    
    // Validation and configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    
    #[error("Invalid slippage: {0}")]
    InvalidSlippage(String),
    
    // System errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Ethers error: {0}")]
    Ethers(#[from] ethers::providers::ProviderError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Clone for MCPError {
    fn clone(&self) -> Self {
        match self {
            MCPError::JsonRpc(msg) => MCPError::JsonRpc(msg.clone()),
            MCPError::InvalidJsonRpcRequest(msg) => MCPError::InvalidJsonRpcRequest(msg.clone()),
            MCPError::MissingParameter(msg) => MCPError::MissingParameter(msg.clone()),
            MCPError::InvalidParameterType(msg) => MCPError::InvalidParameterType(msg.clone()),
            MCPError::EthereumRpc(msg) => MCPError::EthereumRpc(msg.clone()),
            MCPError::NetworkError(msg) => MCPError::NetworkError(msg.clone()),
            MCPError::RpcTimeout(msg) => MCPError::RpcTimeout(msg.clone()),
            MCPError::RateLimitExceeded(msg) => MCPError::RateLimitExceeded(msg.clone()),
            MCPError::InvalidAddress(msg) => MCPError::InvalidAddress(msg.clone()),
            MCPError::InvalidTokenContract(msg) => MCPError::InvalidTokenContract(msg.clone()),
            MCPError::ContractNotFound(msg) => MCPError::ContractNotFound(msg.clone()),
            MCPError::InvalidContractAbi(msg) => MCPError::InvalidContractAbi(msg.clone()),
            MCPError::InsufficientBalance(msg) => MCPError::InsufficientBalance(msg.clone()),
            MCPError::TransactionFailed(msg) => MCPError::TransactionFailed(msg.clone()),
            MCPError::GasEstimationFailed(msg) => MCPError::GasEstimationFailed(msg.clone()),
            MCPError::SlippageTooHigh(msg) => MCPError::SlippageTooHigh(msg.clone()),
            MCPError::PriceFetchFailed(msg) => MCPError::PriceFetchFailed(msg.clone()),
            MCPError::ApiRateLimitExceeded(msg) => MCPError::ApiRateLimitExceeded(msg.clone()),
            MCPError::InvalidPriceData(msg) => MCPError::InvalidPriceData(msg.clone()),
            MCPError::TokenNotFound(msg) => MCPError::TokenNotFound(msg.clone()),
            MCPError::WalletError(msg) => MCPError::WalletError(msg.clone()),
            MCPError::InvalidPrivateKey(msg) => MCPError::InvalidPrivateKey(msg.clone()),
            MCPError::SigningFailed(msg) => MCPError::SigningFailed(msg.clone()),
            MCPError::WalletNotInitialized(msg) => MCPError::WalletNotInitialized(msg.clone()),
            MCPError::ConfigurationError(msg) => MCPError::ConfigurationError(msg.clone()),
            MCPError::ValidationError(msg) => MCPError::ValidationError(msg.clone()),
            MCPError::InvalidAmount(msg) => MCPError::InvalidAmount(msg.clone()),
            MCPError::InvalidSlippage(msg) => MCPError::InvalidSlippage(msg.clone()),
            MCPError::Timeout(msg) => MCPError::Timeout(msg.clone()),
            // For error types that don't support Clone, convert to generic error
            MCPError::Serialization(_) => MCPError::Other(anyhow::anyhow!("Serialization error")),
            MCPError::Http(_) => MCPError::Other(anyhow::anyhow!("HTTP error")),
            MCPError::Ethers(_) => MCPError::Other(anyhow::anyhow!("Ethers error")),
            MCPError::Io(_) => MCPError::Other(anyhow::anyhow!("IO error")),
            MCPError::SwapSimulationFailed(msg) => MCPError::SwapSimulationFailed(msg.clone()),
            MCPError::Other(_) => MCPError::Other(anyhow::anyhow!("Other error")),
        }
    }
}

impl MCPError {
    /// Get error code
    pub fn error_code(&self) -> i32 {
        match self {
            MCPError::JsonRpc(_) => -32600,
            MCPError::InvalidJsonRpcRequest(_) => -32600,
            MCPError::MissingParameter(_) => -32602,
            MCPError::InvalidParameterType(_) => -32602,
            MCPError::EthereumRpc(_) => -32603,
            MCPError::NetworkError(_) => -32603,
            MCPError::RpcTimeout(_) => -32603,
            MCPError::RateLimitExceeded(_) => -32603,
            MCPError::InvalidAddress(_) => -32602,
            MCPError::InvalidTokenContract(_) => -32602,
            MCPError::ContractNotFound(_) => -32602,
            MCPError::InvalidContractAbi(_) => -32602,
            MCPError::InsufficientBalance(_) => -32603,
            MCPError::TransactionFailed(_) => -32603,
            MCPError::GasEstimationFailed(_) => -32603,
            MCPError::SlippageTooHigh(_) => -32603,
            MCPError::PriceFetchFailed(_) => -32603,
            MCPError::ApiRateLimitExceeded(_) => -32603,
            MCPError::InvalidPriceData(_) => -32603,
            MCPError::TokenNotFound(_) => -32602,
            MCPError::WalletError(_) => -32603,
            MCPError::InvalidPrivateKey(_) => -32602,
            MCPError::SigningFailed(_) => -32603,
            MCPError::WalletNotInitialized(_) => -32603,
            MCPError::ConfigurationError(_) => -32603,
            MCPError::ValidationError(_) => -32602,
            MCPError::InvalidAmount(_) => -32602,
            MCPError::InvalidSlippage(_) => -32602,
            MCPError::Serialization(_) => -32603,
            MCPError::Http(_) => -32603,
            MCPError::Ethers(_) => -32603,
            MCPError::Io(_) => -32603,
            MCPError::Timeout(_) => -32603,
            MCPError::SwapSimulationFailed(_) => -32603,
            MCPError::Other(_) => -32603,
        }
    }
    
    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::JsonRpc(_) => ErrorSeverity::High,
            MCPError::InvalidJsonRpcRequest(_) => ErrorSeverity::High,
            MCPError::MissingParameter(_) => ErrorSeverity::Medium,
            MCPError::InvalidParameterType(_) => ErrorSeverity::Medium,
            MCPError::EthereumRpc(_) => ErrorSeverity::High,
            MCPError::NetworkError(_) => ErrorSeverity::High,
            MCPError::RpcTimeout(_) => ErrorSeverity::High,
            MCPError::RateLimitExceeded(_) => ErrorSeverity::Medium,
            MCPError::InvalidAddress(_) => ErrorSeverity::Medium,
            MCPError::InvalidTokenContract(_) => ErrorSeverity::Medium,
            MCPError::ContractNotFound(_) => ErrorSeverity::Medium,
            MCPError::InvalidContractAbi(_) => ErrorSeverity::High,
            MCPError::InsufficientBalance(_) => ErrorSeverity::Medium,
            MCPError::TransactionFailed(_) => ErrorSeverity::High,
            MCPError::GasEstimationFailed(_) => ErrorSeverity::High,
            MCPError::SlippageTooHigh(_) => ErrorSeverity::Medium,
            MCPError::PriceFetchFailed(_) => ErrorSeverity::Medium,
            MCPError::ApiRateLimitExceeded(_) => ErrorSeverity::Medium,
            MCPError::InvalidPriceData(_) => ErrorSeverity::Medium,
            MCPError::TokenNotFound(_) => ErrorSeverity::Medium,
            MCPError::WalletError(_) => ErrorSeverity::High,
            MCPError::InvalidPrivateKey(_) => ErrorSeverity::High,
            MCPError::SigningFailed(_) => ErrorSeverity::High,
            MCPError::WalletNotInitialized(_) => ErrorSeverity::High,
            MCPError::ConfigurationError(_) => ErrorSeverity::High,
            MCPError::ValidationError(_) => ErrorSeverity::Medium,
            MCPError::InvalidAmount(_) => ErrorSeverity::Medium,
            MCPError::InvalidSlippage(_) => ErrorSeverity::Medium,
            MCPError::Serialization(_) => ErrorSeverity::High,
            MCPError::Http(_) => ErrorSeverity::Medium,
            MCPError::Ethers(_) => ErrorSeverity::High,
            MCPError::Io(_) => ErrorSeverity::Medium,
            MCPError::Timeout(_) => ErrorSeverity::Medium,
            MCPError::SwapSimulationFailed(_) => ErrorSeverity::High,
            MCPError::Other(_) => ErrorSeverity::High,
        }
    }
    
    /// Get error context information
    pub fn context(&self) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        match self {
            MCPError::EthereumRpc(msg) => {
                context.insert("error_type".to_string(), "ethereum_rpc".to_string());
                context.insert("message".to_string(), msg.clone());
            },
            MCPError::NetworkError(msg) => {
                context.insert("error_type".to_string(), "network".to_string());
                context.insert("message".to_string(), msg.clone());
            },
            MCPError::InvalidAddress(addr) => {
                context.insert("error_type".to_string(), "validation".to_string());
                context.insert("invalid_address".to_string(), addr.clone());
            },
            MCPError::InsufficientBalance(msg) => {
                context.insert("error_type".to_string(), "balance".to_string());
                context.insert("message".to_string(), msg.clone());
            },
            _ => {
                context.insert("error_type".to_string(), "general".to_string());
                context.insert("message".to_string(), self.to_string());
            }
        }
        
        context
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

lazy_static! {
    static ref ETH_ADDRESS_REGEX: Regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    static ref PRIVATE_KEY_REGEX: Regex = Regex::new(r"^(0x)?[a-fA-F0-9]{64}$").unwrap();
}

/// Input validator
pub struct InputValidator;

impl InputValidator {
    /// Validate Ethereum address
    pub fn validate_address(address: &str) -> Result<(), MCPError> {
        if address.is_empty() {
            return Err(MCPError::InvalidAddress("Address cannot be empty".to_string()));
        }
        
        if !ETH_ADDRESS_REGEX.is_match(address) {
            return Err(MCPError::InvalidAddress(
                format!("Invalid Ethereum address format: {}", address)
            ));
        }
        
        Ok(())
    }
    
    /// Validate private key
    pub fn validate_private_key(private_key: &str) -> Result<(), MCPError> {
        if private_key.is_empty() {
            return Err(MCPError::InvalidPrivateKey("Private key cannot be empty".to_string()));
        }
        
        if !PRIVATE_KEY_REGEX.is_match(private_key) {
            return Err(MCPError::InvalidPrivateKey(
                format!("Invalid private key format: {}", private_key)
            ));
        }
        
        Ok(())
    }
    
    /// Validate amount
    pub fn validate_amount(amount: &str) -> Result<Decimal, MCPError> {
        if amount.is_empty() {
            return Err(MCPError::InvalidAmount("Amount cannot be empty".to_string()));
        }
        
        let amount_decimal = Decimal::from_str(amount)
            .map_err(|e| MCPError::InvalidAmount(
                format!("Invalid amount format '{}': {}", amount, e)
            ))?;
        
        if amount_decimal <= Decimal::ZERO {
            return Err(MCPError::InvalidAmount(
                format!("Amount must be positive: {}", amount)
            ));
        }
        
        if amount_decimal > Decimal::from(1_000_000_000) {
            return Err(MCPError::InvalidAmount(
                format!("Amount too large: {}", amount)
            ));
        }
        
        Ok(amount_decimal)
    }
    
    /// Validate slippage
    pub fn validate_slippage(slippage: &str) -> Result<Decimal, MCPError> {
        if slippage.is_empty() {
            return Err(MCPError::InvalidSlippage("Slippage cannot be empty".to_string()));
        }
        
        let slippage_decimal = Decimal::from_str(slippage)
            .map_err(|e| MCPError::InvalidSlippage(
                format!("Invalid slippage format '{}': {}", slippage, e)
            ))?;
        
        if slippage_decimal < Decimal::ZERO {
            return Err(MCPError::InvalidSlippage(
                format!("Slippage cannot be negative: {}", slippage)
            ));
        }
        
        if slippage_decimal > Decimal::from(50) {
            return Err(MCPError::InvalidSlippage(
                format!("Slippage too high (max 50%): {}", slippage)
            ));
        }
        
        Ok(slippage_decimal)
    }
    
    /// Validate tool call parameters
    pub fn validate_tool_parameters(tool_name: &str, args: &Value) -> Result<(), MCPError> {
        match tool_name {
            "get_balance" => Self::validate_get_balance_params(args),
            "get_token_price" => Self::validate_get_token_price_params(args),
            "swap_tokens" => Self::validate_swap_tokens_params(args),
            _ => Err(MCPError::ValidationError(
                format!("Unknown tool: {}", tool_name)
            )),
        }
    }
    
    /// Validate balance query parameters
    fn validate_get_balance_params(args: &Value) -> Result<(), MCPError> {
        let address = args.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::MissingParameter("address".to_string()))?;
        
        Self::validate_address(address)?;
        
        // Validate optional token_address
        if let Some(token_address) = args.get("token_address") {
            if let Some(addr) = token_address.as_str() {
                Self::validate_address(addr)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate price query parameters
    fn validate_get_token_price_params(args: &Value) -> Result<(), MCPError> {
        let token_address = args.get("token_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::MissingParameter("token_address".to_string()))?;
        
        Self::validate_address(token_address)?;
        Ok(())
    }
    
    /// Validate swap parameters
    fn validate_swap_tokens_params(args: &Value) -> Result<(), MCPError> {
        let from_token = args.get("from_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::MissingParameter("from_token".to_string()))?;
        
        let to_token = args.get("to_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::MissingParameter("to_token".to_string()))?;
        
        let amount = args.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::MissingParameter("amount".to_string()))?;
        
        Self::validate_address(from_token)?;
        Self::validate_address(to_token)?;
        Self::validate_amount(amount)?;
        
        // Validate optional slippage
        if let Some(slippage) = args.get("slippage_tolerance") {
            if let Some(slippage_str) = slippage.as_str() {
                Self::validate_slippage(slippage_str)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate RPC URL
    pub fn validate_rpc_url(url: &str) -> Result<(), MCPError> {
        if url.is_empty() {
            return Err(MCPError::ConfigurationError("RPC URL cannot be empty".to_string()));
        }
        
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(MCPError::ConfigurationError(
                format!("Invalid RPC URL format: {}", url)
            ));
        }
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate_config(rpc_url: &str, private_key: &str) -> Result<(), MCPError> {
        Self::validate_rpc_url(rpc_url)?;
        Self::validate_private_key(private_key)?;
        Ok(())
    }
}

/// Error recovery strategy
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Check if error is recoverable
    pub fn is_recoverable(error: &MCPError) -> bool {
        match error {
            MCPError::NetworkError(_) => true,
            MCPError::RpcTimeout(_) => true,
            MCPError::RateLimitExceeded(_) => true,
            MCPError::ApiRateLimitExceeded(_) => true,
            MCPError::Http(_) => true,
            MCPError::Timeout(_) => true,
            _ => false,
        }
    }
    
    /// Get retry delay time (seconds)
    pub fn retry_delay(error: &MCPError, attempt: u32) -> u64 {
        match error {
            MCPError::RateLimitExceeded(_) => 60,
            MCPError::ApiRateLimitExceeded(_) => 60,
            MCPError::NetworkError(_) => 2_u64.pow(attempt.min(5)),
            MCPError::RpcTimeout(_) => 2_u64.pow(attempt.min(3)),
            MCPError::Http(_) => 2_u64.pow(attempt.min(3)),
            _ => 1,
        }
    }
    
    /// Get maximum retry count
    pub fn max_retries(error: &MCPError) -> u32 {
        match error {
            MCPError::RateLimitExceeded(_) => 3,
            MCPError::ApiRateLimitExceeded(_) => 3,
            MCPError::NetworkError(_) => 5,
            MCPError::RpcTimeout(_) => 3,
            MCPError::Http(_) => 3,
            _ => 1,
        }
    }
}

/// Error handler
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle error and generate appropriate response
    pub fn handle_error(error: MCPError, request_id: Option<&str>) -> crate::types::MCPResponse {
        let error_code = error.error_code();
        let severity = error.severity();
        let context = error.context();
        
        // Log based on error severity
        match severity {
            ErrorSeverity::Critical | ErrorSeverity::High => {
                tracing::error!(
                    error_code = %error_code,
                    severity = ?severity,
                    context = ?context,
                    "Critical/High severity error occurred"
                );
            },
            ErrorSeverity::Medium => {
                tracing::warn!(
                    error_code = %error_code,
                    severity = ?severity,
                    context = ?context,
                    "Medium severity error occurred"
                );
            },
            ErrorSeverity::Low => {
                tracing::info!(
                    error_code = %error_code,
                    severity = ?severity,
                    context = ?context,
                    "Low severity error occurred"
                );
            },
        }
        
        // Create error response
        let error_response = crate::types::MCPErrorResponse {
            code: error_code,
            message: error.to_string(),
            data: Some(serde_json::json!({
                "severity": format!("{:?}", severity),
                "context": context,
                "request_id": request_id
            })),
        };
        
        crate::types::MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id.map(|id| serde_json::Value::String(id.to_string())).unwrap_or(serde_json::Value::Null),
            result: None,
            error: Some(error_response),
        }
    }
}
