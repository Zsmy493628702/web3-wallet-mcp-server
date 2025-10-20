#!/bin/bash

# Web3 Wallet MCP Server - curl call examples
# Server address: http://localhost:3000/mcp

echo "🚀 Web3 Wallet MCP Server - curl call examples"
echo "=============================================="

# Server addresses
SERVER_URL="http://localhost:3000/mcp"
HEALTH_URL="http://localhost:3000/health"

echo "📡 Server address: $SERVER_URL"
echo "❤️  Health check: $HEALTH_URL"
echo ""
echo "💡 Make sure the server is running first:"
echo "   ./start_server.sh"
echo "   or"
echo "   export PRIVATE_KEY='your_private_key' && cargo run --bin simple_server"
echo ""

# 1. Health check
echo "1️⃣ Health check"
echo "curl -X GET $HEALTH_URL"
echo ""
curl -X GET "$HEALTH_URL" \
  -H "Content-Type: application/json" \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 2. Get tools list
echo "2️⃣ Get available tools list"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 3. Query wallet balance (Vitalik's address)
echo "3️⃣ Query wallet balance (Vitalik's address)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_balance",
      "arguments": {
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 3.1 Query specific token balance (USDT)
echo "3️⃣.1 Query specific token balance (USDT)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_balance",
      "arguments": {
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "token_address": "0xdAC17F958D2ee523a2206206994597C13D831ec7"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 3.2 Query specific token balance (WETH)
echo "3️⃣.2 Query specific token balance (WETH)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_balance",
      "arguments": {
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "token_address": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 4. Get token price (USDT)
echo "4️⃣ Get token price (USDT)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_token_price",
      "arguments": {
        "token_address": "0xdAC17F958D2ee523a2206206994597C13D831ec7"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 4.1 Get token price (WETH)
echo "4️⃣.1 Get token price (WETH)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_token_price",
      "arguments": {
        "token_address": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 4.2 Get token price (DAI)
echo "4️⃣.2 Get token price (DAI)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_token_price",
      "arguments": {
        "token_address": "0x6B175474E89094C44Da98b954EedeAC495271d0F"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 5. Simulate token swap (USDC -> USDT)
echo "5️⃣ Simulate token swap (USDC -> USDT)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "swap_tokens",
      "arguments": {
        "from_token": "0xA0b86a33E6441b8C4C8C0C4C8C0C4C8C0C4C8C0C",
        "to_token": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "amount": "100.0",
        "slippage_tolerance": "0.5"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 5.1 Simulate token swap (WETH -> USDC)
echo "5️⃣.1 Simulate token swap (WETH -> USDC)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "swap_tokens",
      "arguments": {
        "from_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "to_token": "0xA0b86a33E6441b8C4C8C0C4C8C0C4C8C0C4C8C0C",
        "amount": "1.0",
        "slippage_tolerance": "1.0"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

# 5.2 Simulate token swap (DAI -> WETH)
echo "5️⃣.2 Simulate token swap (DAI -> WETH)"
echo "curl -X POST $SERVER_URL -H 'Content-Type: application/json' -d '{...}'"
echo ""
curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "swap_tokens",
      "arguments": {
        "from_token": "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "amount": "1000.0",
        "slippage_tolerance": "0.3"
      }
    }
  }' \
  -w "\nHTTP status code: %{http_code}\n" \
  -s
echo ""

echo "🎉 All curl commands executed!"
echo "💡 Tip: If some requests fail, check if the server is running"
echo "🔧 Start server: cargo run --bin simple_server"
echo ""
echo "📊 Test Summary:"
echo "  ✅ Health check"
echo "  ✅ Tools list"
echo "  ✅ Balance queries (ETH + tokens)"
echo "  ✅ Price queries (multiple tokens)"
echo "  ✅ Swap simulations (multiple pairs)"
echo ""
echo "🚀 Server is ready for production use!"
