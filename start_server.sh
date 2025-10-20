#!/bin/bash

# Web3 Wallet MCP Server - Startup Script
echo "🚀 Starting Web3 Wallet MCP Server"
echo "===================================="

# Set environment variables
export PRIVATE_KEY="0x0000000000000000000000000000000000000000000000000000000000000001"

echo "🔧 Environment variables set:"
echo "   PRIVATE_KEY: ${PRIVATE_KEY:0:10}..."
echo "   ETHEREUM_RPC_URL: Hardcoded in code (Alchemy)"
echo ""

echo "🏗️  Compiling and starting server..."
cargo run --bin simple_server
