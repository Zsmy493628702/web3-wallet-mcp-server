use anyhow::Result;
use std::env;
use tracing::info;
use web3_wallet::mcp_server::MCPServer;
use web3_wallet::logging::init_logging;
use web3_wallet::types::MCPRequest;
use serde_json::json;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    init_logging()?;

    info!("🚀 Starting Web3 Wallet MCP HTTP Server");

    // Get configuration from environment
    let rpc_url = "https://eth-mainnet.g.alchemy.com/v2/JZUYcRpkXq25weYd16Fuu".to_string();
    
    let private_key = env::var("PRIVATE_KEY")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000001".to_string());

    // Create MCP server
    let mcp_server = Arc::new(MCPServer::new(rpc_url, private_key).await?);
    
    info!("✅ MCP Server initialized successfully");

    // Create HTTP router
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", get(handle_health))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(mcp_server);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("🌐 HTTP server listening on http://0.0.0.0:3000");
    info!("📡 MCP endpoint: http://localhost:3000/mcp");
    info!("❤️  Health check: http://localhost:3000/health");
    info!("🔧 Ready to accept requests!");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_mcp_request(
    State(mcp_server): State<Arc<MCPServer>>,
    Json(request): Json<MCPRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match mcp_server.handle_request(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(e) => {
            tracing::error!("MCP request failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "Web3 Wallet MCP Server",
        "version": "1.0.0",
        "endpoints": {
            "mcp": "/mcp",
            "health": "/health"
        }
    })))
}
