# Web3 Wallet MCP Server

A Model Context Protocol (MCP) server in Rust that enables AI agents to interact with Ethereum blockchain. It provides three core tools: query wallet balances, fetch token prices, and simulate token swaps using Uniswap protocols.

## 🚀 Features

- **Balance Queries**: Get ETH and ERC20 token balances for any wallet address
- **Token Price Fetching**: Real-time token prices via Alchemy Price API
- **Swap Simulation**: Simulate token swaps using Uniswap V3 Quoter with V2 fallback
- **MCP Protocol**: Clean JSON-RPC 2.0 interface for AI agent integration
- **Structured Logging**: Request-level tracing with unique request IDs
- **Robust Error Handling**: Comprehensive error codes and recovery mechanisms

## 📋 Prerequisites

- Rust (stable) and Cargo
- Set `PRIVATE_KEY` environment variable (required for startup)
- Uses hardcoded Alchemy mainnet RPC and Price API key

## 🛠️ Installation & Setup

1. **Clone the repository**:
```bash
git clone https://github.com/yourusername/web3-wallet-mcp-server.git
cd web3-wallet-mcp-server
```

2. **Set environment variables**:
```bash
export PRIVATE_KEY=0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export RUST_LOG=info
export LOG_FORMAT=json  # or pretty
```

3. **Run the server**:
```bash
cargo run --bin simple_server
```

The server will start on `http://localhost:3000` with these endpoints:
- **MCP API**: `http://localhost:3000/mcp`
- **Health Check**: `http://localhost:3000/health`

## 🔧 API Tools

### `get_balance`
Query ETH and ERC20 token balances for a wallet address.

**Parameters**:
- `address` (required): Ethereum wallet address
- `token_address` (optional): Specific token address to query

**Returns**: ETH balance and token balances with metadata (symbol, name, decimals, raw and formatted amounts)

### `get_token_price`
Fetch real-time token prices from Alchemy Price API.

**Parameters**:
- `token_address` (required): Token contract address

**Returns**: `{ token_address, symbol, price_usd }`

### `swap_tokens`
Simulate token swaps using Uniswap protocols.

**Parameters**:
- `from_token`: Source token address
- `to_token`: Destination token address  
- `amount`: Amount to swap (decimal string)
- `slippage_tolerance`: Maximum slippage percentage (string)

**Returns**: `{ amount_in, amount_out, gas_estimate, gas_price, total_cost, route, slippage_tolerance }`

**Engine**: Uses Uniswap V3 Quoter v1 via `eth_call` with V2 reserve-based fallback for maximum compatibility.

## 📖 Usage Examples

### Query Wallet Balance
```bash
curl -s -X POST http://localhost:3000/mcp \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc":"2.0","id":1,
    "method":"tools/call",
    "params":{
      "name":"get_balance",
      "arguments":{
        "address":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "token_address":"0xdAC17F958D2ee523a2206206994597C13D831ec7"
      }
    }
  }'
```

### Get Token Price
```bash
curl -s -X POST http://localhost:3000/mcp \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc":"2.0","id":2,
    "method":"tools/call",
    "params":{
      "name":"get_token_price",
      "arguments":{ "token_address":"0xdAC17F958D2ee523a2206206994597C13D831ec7" }
    }
  }'
```

### Simulate Token Swap
```bash
curl -s -X POST http://localhost:3000/mcp \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc":"2.0","id":3,
    "method":"tools/call",
    "params":{
      "name":"swap_tokens",
      "arguments":{
        "from_token":"0x6B175474E89094C44Da98b954EedeAC495271d0F",
        "to_token":"0x1f9840a85d5af5bf1d1762f925bdaddc4201f984",
        "amount":"100",
        "slippage_tolerance":"1.0"
      }
    }
  }'
```

## 🏗️ Architecture

The project follows a modular architecture with clear separation of concerns:

- **`src/bin/simple_server.rs`**: Axum HTTP server exposing MCP endpoints
- **`src/mcp_server.rs`**: MCP protocol dispatcher and request lifecycle management
- **`src/tools.rs`**: Implementation of the three core tools
- **`src/ethereum.rs`**: Ethereum blockchain interaction logic
- **`src/error.rs`**: Comprehensive error handling and validation
- **`src/logging.rs`**: Structured logging with request tracing
- **`src/types.rs`**: Type definitions for requests and responses

## 🧪 Testing

Run the test suite:
```bash
cargo test -- --test-threads=1
```

## ⚠️ Limitations

- RPC endpoint is hardcoded to Alchemy mainnet
- Gas estimation may fail for dry-run transactions (fallback values used)
- Token metadata queries with known-token fallback
- No actual transaction signing or execution

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [ethers-rs](https://github.com/gakonst/ethers-rs) for Ethereum interactions
- Powered by [Alchemy](https://www.alchemy.com/) for RPC and price data
- Implements [Model Context Protocol](https://modelcontextprotocol.io/) specification
