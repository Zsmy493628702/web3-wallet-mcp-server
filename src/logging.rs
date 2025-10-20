use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;
use std::collections::HashMap;

/// Request context containing request ID and other metadata
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub method: String,
    pub start_time: std::time::Instant,
    pub metadata: HashMap<String, String>,
}

impl RequestContext {
    pub fn new(method: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            method,
            start_time: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Initialize structured logging
pub fn init_logging() -> anyhow::Result<()> {
    // Set up environment filter with default level of info
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Create JSON format logger
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    // Create human-readable format logger (for development)
    let human_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    // Choose log format based on environment variable
    let use_json = std::env::var("LOG_FORMAT").unwrap_or_default() == "json";

    if use_json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(human_layer)
            .init();
    }

    info!("Logging initialized successfully");
    Ok(())
}

/// Log request start
pub fn log_request_start(ctx: &RequestContext) {
    info!(
        request_id = %ctx.request_id,
        method = %ctx.method,
        "Request started"
    );
}

/// Log request completion
pub fn log_request_complete(ctx: &RequestContext, success: bool) {
    let duration = ctx.duration();
    
    if success {
        info!(
            request_id = %ctx.request_id,
            method = %ctx.method,
            duration_ms = duration.as_millis(),
            success = success,
            "Request completed successfully"
        );
    } else {
        tracing::warn!(
            request_id = %ctx.request_id,
            method = %ctx.method,
            duration_ms = duration.as_millis(),
            success = success,
            "Request completed with errors"
        );
    }
}

/// Log tool call
pub fn log_tool_call(ctx: &RequestContext, tool_name: &str, args: &serde_json::Value) {
    debug!(
        request_id = %ctx.request_id,
        tool_name = %tool_name,
        args = %serde_json::to_string(args).unwrap_or_default(),
        "Tool call initiated"
    );
}

/// Log tool result
pub fn log_tool_result(ctx: &RequestContext, tool_name: &str, success: bool, duration: std::time::Duration) {
    if success {
        info!(
            request_id = %ctx.request_id,
            tool_name = %tool_name,
            duration_ms = duration.as_millis(),
            success = success,
            "Tool call completed successfully"
        );
    } else {
        tracing::error!(
            request_id = %ctx.request_id,
            tool_name = %tool_name,
            duration_ms = duration.as_millis(),
            success = success,
            "Tool call failed"
        );
    }
}

/// Log Ethereum operation
pub fn log_ethereum_operation(ctx: &RequestContext, operation: &str, details: &str) {
    debug!(
        request_id = %ctx.request_id,
        operation = %operation,
        details = %details,
        "Ethereum operation"
    );
}

/// Log error
pub fn log_error(ctx: &RequestContext, error: &dyn std::error::Error, context: &str) {
    error!(
        request_id = %ctx.request_id,
        error = %error,
        context = %context,
        "Error occurred"
    );
}

/// Log warning
pub fn log_warning(ctx: &RequestContext, message: &str, details: Option<&str>) {
    warn!(
        request_id = %ctx.request_id,
        message = %message,
        details = details.unwrap_or(""),
        "Warning"
    );
}

/// Log performance metrics
pub fn log_performance(ctx: &RequestContext, metric_name: &str, value: f64, unit: &str) {
    info!(
        request_id = %ctx.request_id,
        metric_name = %metric_name,
        value = value,
        unit = %unit,
        "Performance metric"
    );
}
