use anyhow::Result;
use std::env;
use tracing::info;
use web3_wallet::mcp_server::MCPServer;
use web3_wallet::logging::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    init_logging()?;

    info!("Starting Web3 Wallet MCP Server");

    // Get configuration from environment
    let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/JZUYcRpkXq25weYd16Fuu".to_string();
    
    let private_key = env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable is required");

    info!(
        rpc_url = %rpc_url,
        "Connecting to Ethereum RPC"
    );

    // Create and start the MCP server
    let server = MCPServer::new(rpc_url, private_key).await?;
    
    info!("MCP Server initialized successfully");
    
    // Start the server
    server.run().await?;

    Ok(())
}
