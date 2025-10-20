use crate::error::MCPError;
use crate::types::{BalanceInfo, TokenBalance, PriceInfo, SwapSimulation};
use crate::error::InputValidator;
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress},
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use tracing::{info, debug, instrument, warn};
use reqwest;
use serde_json;

pub struct EthereumClient {
    provider: Provider<Http>,
}

impl EthereumClient {
    pub async fn new(rpc_url: String, _private_key: String) -> Result<Self, MCPError> {
        // Validate configuration
        InputValidator::validate_config(&rpc_url, &_private_key)?;
        
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| MCPError::EthereumRpc(e.to_string()))?;

        info!("Ethereum client initialized successfully");
        Ok(Self { provider })
    }

    pub async fn get_token_price(&self, token_address: &str) -> Result<PriceInfo, MCPError> {
        info!(
            token_address = %token_address,
            "Fetching token price from Alchemy API"
        );
        let (_, symbol, _) = self.get_known_token_info(token_address);
        let price_usd = self.get_price_from_alchemy(token_address).await?;
        info!(token_address = %token_address, symbol = %symbol, price_usd = %price_usd, "Token price fetched");
        Ok(PriceInfo { token_address: token_address.to_string(), symbol, price_usd })
    }

    async fn get_price_from_alchemy(&self, token_address: &str) -> Result<Decimal, MCPError> {
        let client = reqwest::Client::new();
        let url = "https://api.g.alchemy.com/prices/v1/JZUYcRpkXq25weYd16Fuu/tokens/by-address";
        let request_body = serde_json::json!({
            "addresses": [ { "network": "eth-mainnet", "address": token_address } ]
        });
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| MCPError::NetworkError(format!("Failed to call Alchemy API: {}", e)))?;
        if !response.status().is_success() {
            return Err(MCPError::PriceFetchFailed(format!("Alchemy API returned status: {}", response.status())));
        }
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| MCPError::PriceFetchFailed(format!("Failed to parse Alchemy API response: {}", e)))?;
        if let Some(data_array) = json.get("data").and_then(|d| d.as_array()) {
            if let Some(token_data) = data_array.first() {
                if let Some(prices) = token_data.get("prices").and_then(|p| p.as_array()) {
                    if let Some(price_info) = prices.first() {
                        if let Some(price_usd_str) = price_info.get("value").and_then(|v| v.as_str()) {
                            let price_usd_decimal = price_usd_str
                                .parse::<Decimal>()
                                .map_err(|_| MCPError::PriceFetchFailed("Failed to parse price value".to_string()))?;
                            return Ok(price_usd_decimal);
                        }
                    }
                }
            }
        }
        Err(MCPError::PriceFetchFailed("No price data found in Alchemy API response".to_string()))
    }
    

    #[instrument(skip(self), fields(address = %address, token_address = %token_address.unwrap_or("all")))]
    pub async fn get_balance(&self, address: &str, token_address: Option<&str>) -> Result<BalanceInfo, MCPError> {
        let addr = address.parse::<Address>()
            .map_err(|_| MCPError::InvalidAddress(address.to_string()))?;

        // Get ETH balance
        debug!(address = %address, "Fetching ETH balance");
        let eth_balance_wei = self.provider.get_balance(addr, None).await?;
        let eth_balance = Decimal::from(eth_balance_wei.as_u128()) / dec!(1_000_000_000_000_000_000);
        
        info!(
            address = %address,
            eth_balance_wei = %eth_balance_wei,
            eth_balance = %eth_balance,
            "ETH balance retrieved"
        );

        let mut token_balances = HashMap::new();

        if let Some(token_addr) = token_address {
            // Get specific token balance
            info!(address = %address, token_address = %token_addr, "Fetching specific token balance");
            let token_balance = self.get_token_balance(addr, token_addr).await?;
            token_balances.insert(token_addr.to_string(), token_balance);
        } else {
            // Get common token balances (USDC, USDT, WETH, etc.)
            info!(address = %address, "Fetching common token balances");
            let common_tokens = vec![
                ("0xA0b86a33E6441b8C4C8C0C4C8C0C4C8C0C4C8C0C", "USDC", "USD Coin", 6),
                ("0xdAC17F958D2ee523a2206206994597C13D831ec7", "USDT", "Tether USD", 6),
                ("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", "WETH", "Wrapped Ether", 18),
            ];

            for (contract_addr, _symbol, _name, _decimals) in common_tokens {
                if let Ok(balance) = self.get_token_balance(addr, contract_addr).await {
                    token_balances.insert(contract_addr.to_string(), balance);
                }
            }
        }

        info!(
            address = %address,
            eth_balance = %eth_balance,
            token_count = token_balances.len(),
            "Balance information retrieved successfully"
        );

        Ok(BalanceInfo {
            address: address.to_string(),
            eth_balance,
            token_balances,
        })
    }

    async fn get_token_balance(&self, wallet_addr: Address, token_addr: &str) -> Result<TokenBalance, MCPError> {
        let token_address = token_addr.parse::<Address>()
            .map_err(|_| MCPError::InvalidTokenContract(token_addr.to_string()))?;

        // Try to get token info dynamically, fallback to known tokens or defaults
        let (name, symbol, decimals) = match self.get_token_info(token_address).await {
            Ok(info) => info,
            Err(e) => {
                // If dynamic lookup fails, try known tokens, then use defaults
                warn!(
                    token_address = %token_addr,
                    error = %e,
                    "Failed to get token info dynamically, trying known tokens"
                );
                self.get_known_token_info(token_addr)
            }
        };

        // ERC20 balanceOf function selector: 0x70a08231
        let balance_of_selector = [0x70, 0xa0, 0x82, 0x31];
        let mut data = Vec::from(balance_of_selector);
        
        // Properly encode the wallet address (32 bytes, padded)
        let mut wallet_bytes = [0u8; 32];
        wallet_bytes[12..].copy_from_slice(&wallet_addr.as_bytes());
        data.extend_from_slice(&wallet_bytes);

        let call_data = ethers::types::Bytes::from(data);
        let tx = ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(token_address)),
            data: Some(call_data),
            ..Default::default()
        };
        
        let result = self.provider.call(&tx.into(), None).await?;

        let balance_uint = U256::from(result.as_ref());
        let balance = Decimal::from(balance_uint.as_u128());

        let balance_formatted = balance / Decimal::from(10_u64.pow(decimals as u32));

        Ok(TokenBalance {
            contract_address: token_addr.to_string(),
            symbol,
            name,
            decimals,
            balance,
            balance_formatted: balance_formatted.to_string(),
        })
    }



    /// Get token info from known tokens or return defaults
    fn get_known_token_info(&self, token_addr: &str) -> (String, String, u8) {
        match token_addr.to_lowercase().as_str() {
            "0xa0b86a33e6441b8c4c8c0c4c8c0c4c8c0c4c8c0c" => ("USD Coin".to_string(), "USDC".to_string(), 6),
            "0xdac17f958d2ee523a2206206994597c13d831ec7" => ("Tether USD".to_string(), "USDT".to_string(), 6),
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => ("Wrapped Ether".to_string(), "WETH".to_string(), 18),
            "0x6b175474e89094c44da98b954eedeac495271d0f" => ("Dai Stablecoin".to_string(), "DAI".to_string(), 18),
            "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599" => ("Wrapped BTC".to_string(), "WBTC".to_string(), 8),
            "0x514910771af9ca656af840dff83e8264ecf986ca" => ("ChainLink Token".to_string(), "LINK".to_string(), 18),
            "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984" => ("Uniswap".to_string(), "UNI".to_string(), 18),
            "0x7d1afa7b718fb893db30a3abc0cfc608aacfebb0" => ("Polygon".to_string(), "MATIC".to_string(), 18),
            "0x4fabb145d64652a948d72533023f6e7a623c7c53" => ("Binance USD".to_string(), "BUSD".to_string(), 18),
            "0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce" => ("Shiba Inu".to_string(), "SHIB".to_string(), 18),
            _ => ("Token".to_string(), "TOKEN".to_string(), 18), // Default for unknown tokens
        }
    }

    async fn get_token_info(&self, token_address: Address) -> Result<(String, String, u8), MCPError> {
        // Get token name
        let name_selector = [0x06, 0xfd, 0xde, 0x03]; // name()
        let name_data = ethers::types::Bytes::from(name_selector);
        let name_tx = ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(token_address)),
            data: Some(name_data),
            ..Default::default()
        };
        
        let name_result = self.provider.call(&name_tx.into(), None).await?;
        let name = self.parse_string_from_bytes(&name_result)?;

        // Get token symbol
        let symbol_selector = [0x95, 0xd8, 0x9b, 0x41]; // symbol()
        let symbol_data = ethers::types::Bytes::from(symbol_selector);
        let symbol_tx = ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(token_address)),
            data: Some(symbol_data),
            ..Default::default()
        };
        
        let symbol_result = self.provider.call(&symbol_tx.into(), None).await?;
        let symbol = self.parse_string_from_bytes(&symbol_result)?;

        // Get token decimals
        let decimals_selector = [0x31, 0x3c, 0xe5, 0x67]; // decimals()
        let decimals_data = ethers::types::Bytes::from(decimals_selector);
        let decimals_tx = ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(token_address)),
            data: Some(decimals_data),
            ..Default::default()
        };
        
        let decimals_result = self.provider.call(&decimals_tx.into(), None).await?;
        let decimals_uint = U256::from(decimals_result.as_ref());
        let decimals = decimals_uint.as_u32() as u8;

        Ok((name, symbol, decimals))
    }

    /// Parse string from contract call result bytes
    fn parse_string_from_bytes(&self, bytes: &[u8]) -> Result<String, MCPError> {
        if bytes.len() < 32 {
            return Err(MCPError::ValidationError("Invalid response length".to_string()));
        }

        // Skip the first 32 bytes (offset) and get the length
        let offset = U256::from(&bytes[0..32]).as_usize();
        if offset + 32 > bytes.len() {
            return Err(MCPError::ValidationError("Invalid string offset".to_string()));
        }

        let length = U256::from(&bytes[offset..offset + 32]).as_usize();
        if offset + 32 + length > bytes.len() {
            return Err(MCPError::ValidationError("Invalid string length".to_string()));
        }

        let string_bytes = &bytes[offset + 32..offset + 32 + length];
        let result = String::from_utf8_lossy(string_bytes).trim_end_matches('\0').to_string();
        
        Ok(result)
    }

    // Price endpoints removed to simplify code; swap simulation uses on-chain reserves only.
    
    pub async fn simulate_swap(&self, from_token: &str, to_token: &str, amount: Decimal, slippage: Decimal) -> Result<SwapSimulation, MCPError> {
        info!(
            from_token = %from_token,
            to_token = %to_token,
            amount = %amount,
            slippage = %slippage,
            "Starting Uniswap V3 swap simulation (Quoter v1)"
        );

        // Validate token addresses
        let from_addr = from_token.parse::<Address>()
            .map_err(|_| MCPError::InvalidTokenContract(from_token.to_string()))?;
        
        let to_addr = to_token.parse::<Address>()
            .map_err(|_| MCPError::InvalidTokenContract(to_token.to_string()))?;

        // Get current gas price from the network
        let gas_price = self.provider.get_gas_price().await
            .map_err(|e| MCPError::NetworkError(format!("Failed to get gas price: {}", e)))?;
        let gas_price_decimal = Decimal::from(gas_price.as_u128()) / dec!(1_000_000_000_000_000_000);

        // Get token decimals
        let (_, _, from_decimals) = self.get_known_token_info(from_token);
        let (_, _, to_decimals) = self.get_known_token_info(to_token);

        // Convert amount to wei based on token decimals
        let amount_wei = (amount * Decimal::from(10u128.pow(from_decimals as u32))).to_u128()
            .ok_or_else(|| MCPError::InvalidAmount("Amount too large".to_string()))?;

        // Get Uniswap V2 Router address
        let router_address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()
            .map_err(|_| MCPError::ConfigurationError("Invalid router address".to_string()))?;
        
        info!(
            "🔍 DEBUG: Router address: 0x{:x}",
            router_address
        );

        // Prefer Uniswap V3 Quoter (quoteExactInputSingle) with fee tier fallbacks, then fallback to V2 reserves
        let v3_fees: [u32; 3] = [3000, 500, 10000];
        let mut amount_out_wei_opt: Option<u128> = None;
        for fee in v3_fees.iter() {
            match self.v3_quote_exact_input_single(from_addr, to_addr, *fee, amount_wei).await {
                Ok(v) if v > 0 => {
                    info!(fee = *fee, amount_out_wei = v, "✅ V3 quoter success");
                    amount_out_wei_opt = Some(v);
                    break;
                },
                Ok(_) => {
                    debug!(fee = *fee, "V3 quoter returned zero");
                },
                Err(e) => {
                    debug!(fee = *fee, error = %e, "V3 quoter failed");
                }
            }
        }

        let amount_out_wei = amount_out_wei_opt
            .ok_or_else(|| MCPError::SwapSimulationFailed("Uniswap V3 quoter failed on all fee tiers".to_string()))?;

        let amount_out_decimal = Decimal::from(amount_out_wei) / Decimal::from(10u128.pow(to_decimals as u32));
        
        info!(
            "📊 swap quote: amount_out_wei={}, to_decimals={}, amount_out_decimal={}",
            amount_out_wei, to_decimals, amount_out_decimal
        );
        
        let amount_out_decimal = amount_out_decimal;

        // Apply slippage tolerance
        let slippage_factor = (dec!(100) - slippage) / dec!(100);
        let final_amount_out = amount_out_decimal * slippage_factor;


        // Estimate gas usage using eth_estimateGas
        let gas_estimate = self.estimate_swap_gas(from_addr, to_addr, amount_wei, router_address).await?;
        let total_cost = Decimal::from(gas_estimate) * gas_price_decimal;

        let simulation = SwapSimulation {
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            amount_in: amount,
            amount_out: final_amount_out,
            gas_estimate,
            gas_price: gas_price_decimal,
            total_cost,
            route: vec![from_token.to_string(), to_token.to_string()],
            slippage_tolerance: slippage,
        };

        info!(
            amount_in = %simulation.amount_in,
            amount_out = %simulation.amount_out,
            gas_estimate = simulation.gas_estimate,
            "Uniswap V3 swap simulation completed successfully"
        );

        Ok(simulation)
    }

    /// Uniswap V3 Quoter v1: quoteExactInputSingle(address,address,uint24,uint256,uint160) → uint256 amountOut
    async fn v3_quote_exact_input_single(
        &self,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_in_wei: u128,
    ) -> Result<u128, MCPError> {
        use ethers::abi::{encode, Token};

        // Quoter v1 mainnet
        let quoter: Address = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".parse().unwrap();

        // Function selector for quoteExactInputSingle(address,address,uint24,uint256,uint160)
        let selector = ethers::utils::keccak256(
            "quoteExactInputSingle(address,address,uint24,uint256,uint160)".as_bytes()
        )[0..4].to_vec();

        // Encode params
        let params = vec![
            Token::Address(token_in),
            Token::Address(token_out),
            Token::Uint(U256::from(fee as u64)),
            Token::Uint(U256::from(amount_in_wei)),
            Token::Uint(U256::zero()), // sqrtPriceLimitX96 = 0 (no limit)
        ];
        let mut data = selector;
        data.extend_from_slice(&encode(&params));

        let bytes = self.call_alchemy_eth_call(quoter, data).await?;
        if bytes.len() < 32 { return Err(MCPError::SwapSimulationFailed("Invalid V3 quoter response".to_string())); }

        // Parse single uint256 return (take last 16 bytes for u128)
        let b = &bytes[bytes.len()-16..bytes.len()];
        let mut arr = [0u8;16];
        arr.copy_from_slice(b);
        let amount_out = u128::from_be_bytes(arr);
        Ok(amount_out)
    }


    async fn call_alchemy_eth_call(&self, to: Address, data: Vec<u8>) -> Result<Vec<u8>, MCPError> {
        use serde_json::json;
        
        let client = reqwest::Client::new();
        let url = "https://eth-mainnet.g.alchemy.com/v2/JZUYcRpkXq25weYd16Fuu";
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [
                {
                    "to": format!("0x{:x}", to),
                    "data": format!("0x{}", hex::encode(&data))
                },
                "latest"
            ],
            "id": 1
        });
        
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| MCPError::NetworkError(format!("Failed to call Alchemy API: {}", e)))?;
        
        let response_text = response.text().await
            .map_err(|e| MCPError::NetworkError(format!("Failed to read response: {}", e)))?;
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| MCPError::NetworkError(format!("Failed to parse response: {}", e)))?;
        
        if let Some(result) = response_json.get("result") {
            if let Some(result_str) = result.as_str() {
                if result_str.starts_with("0x") {
                    let hex_data = &result_str[2..];
                    let bytes = hex::decode(hex_data)
                        .map_err(|e| MCPError::NetworkError(format!("Failed to decode hex: {}", e)))?;
                    return Ok(bytes);
                }
            }
        }
        
        if let Some(error) = response_json.get("error") {
            return Err(MCPError::SwapSimulationFailed(format!("Alchemy API error: {}", error)));
        }
        
        Err(MCPError::SwapSimulationFailed("No result in Alchemy response".to_string()))
    }

    async fn estimate_swap_gas(&self, from_token: Address, to_token: Address, amount_in: u128, router_address: Address) -> Result<u64, MCPError> {
        use ethers::abi::{encode, Token};
        use serde_json::json;
        
        // Build swapExactTokensForTokens transaction data
        let function_selector = "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)";
        let selector = ethers::utils::keccak256(function_selector.as_bytes())[0..4].to_vec();
        
        // Parameters for swapExactTokensForTokens:
        // - amountIn: amount_in
        // - amountOutMin: 0 (we're just estimating gas)
        // - path: [from_token, to_token]
        // - to: wallet address (use a dummy address for estimation)
        // - deadline: current timestamp + 1 hour
        let wallet_address = "0x0000000000000000000000000000000000000001".parse::<Address>().unwrap();
        let deadline = chrono::Utc::now().timestamp() as u64 + 3600; // 1 hour from now
        
        let params = vec![
            Token::Uint(amount_in.into()),
            Token::Uint(0u64.into()), // amountOutMin = 0 for estimation
            Token::Array(vec![Token::Address(from_token), Token::Address(to_token)]),
            Token::Address(wallet_address),
            Token::Uint(deadline.into()),
        ];
        
        let encoded_params = encode(&params);
        let mut data = selector;
        data.extend_from_slice(&encoded_params);
        
        // Call eth_estimateGas
        let client = reqwest::Client::new();
        let url = "https://eth-mainnet.g.alchemy.com/v2/JZUYcRpkXq25weYd16Fuu";
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [
                {
                    "to": format!("0x{:x}", router_address),
                    "data": format!("0x{}", hex::encode(&data)),
                    "from": format!("0x{:x}", wallet_address)
                }
            ],
            "id": 1
        });
        
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| MCPError::NetworkError(format!("Failed to estimate gas: {}", e)))?;
        
        let response_text = response.text().await
            .map_err(|e| MCPError::NetworkError(format!("Failed to read gas estimation response: {}", e)))?;
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| MCPError::NetworkError(format!("Failed to parse gas estimation response: {}", e)))?;
        
        if let Some(result) = response_json.get("result") {
            if let Some(result_str) = result.as_str() {
                if result_str.starts_with("0x") {
                    let hex_data = &result_str[2..];
                    let gas_u64 = u64::from_str_radix(&hex_data, 16)
                        .map_err(|e| MCPError::NetworkError(format!("Failed to parse gas estimate: {}", e)))?;
                    
                    info!(
                        gas_estimate = gas_u64,
                        "Gas estimation completed via eth_estimateGas"
                    );
                    
                    return Ok(gas_u64);
                }
            }
        }
        
        if let Some(error) = response_json.get("error") {
            warn!(
                error = ?error,
                "Gas estimation failed, using fallback estimate"
            );
            // Fallback to typical gas estimate if eth_estimateGas fails
            return Ok(200000u64);
        }
        
        Err(MCPError::SwapSimulationFailed("No result in gas estimation response".to_string()))
    }


}
