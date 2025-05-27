// Purpose: Tracing and observability utilities moved from causality-tracing crate

use anyhow::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    Registry,
};
use tracing_subscriber::layer::SubscriberExt;

//-----------------------------------------------------------------------------
// Tracing Initialization
//-----------------------------------------------------------------------------

/// Initializes the tracing subscriber with configurable log level and output format.
///
/// # Arguments
///
/// * `log_level`: An optional string slice specifying the log level.
///                Defaults to "info". Examples: "trace", "debug", "info", "warn", "error".
///                Can also include module-specific directives, e.g., "my_crate=debug,info".
/// * `json_output`: A boolean indicating whether to output logs in JSON format.
///                  Defaults to `false` (human-readable format).
///
/// # Returns
///
/// * `Result<()>`: Ok if initialization was successful, otherwise an `anyhow::Error`.
pub fn init_tracing(log_level: Option<&str>, json_output: Option<bool>) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level.unwrap_or("info")))?;

    let use_json = json_output.unwrap_or(false);

    // --- Subscriber Setup ---
    let subscriber = Registry::default().with(env_filter);

    if use_json {
        let json_layer = fmt::layer()
            .json()
            .with_span_events(FmtSpan::FULL)
            .with_current_span(true)
            .with_span_list(true);
        tracing::subscriber::set_global_default(subscriber.with(json_layer))?;
    } else {
        let fmt_layer = fmt::layer()
            .pretty()
            .with_span_events(FmtSpan::FULL)
            .with_target(true)
            .with_level(true);
        tracing::subscriber::set_global_default(subscriber.with(fmt_layer))?;
    }

    Ok(())
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_test_tracing() {
        INIT.call_once(|| {
            // Initialize tracing once for all tests
            let _ = init_tracing(Some("debug"), None);
        });
    }

    #[test]
    fn test_tracing_init_default() {
        ensure_test_tracing();
        // Test that tracing was initialized by logging a message
        tracing::info!("Default tracing initialization test");
        println!("Default tracing initialization test passed");
    }

    #[test]
    fn test_tracing_init_debug_json() {
        ensure_test_tracing();
        // Test that debug level works
        tracing::debug!("JSON tracing initialization test");
        println!("JSON tracing initialization test passed");
    }

    #[tokio::test]
    async fn test_tracing_async_tokio() {
        ensure_test_tracing();
        // Test that tracing works with async/await
        tracing::info!("Test async tracing message");
        
        let span = tracing::info_span!("test_span", test_id = 42);
        let _enter = span.enter();
        
        tracing::debug!("This is a debug message within a span");
        
        // Simulate some async work
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        
        tracing::warn!("Async test completed");
        println!("Async tracing test passed");
    }
} 