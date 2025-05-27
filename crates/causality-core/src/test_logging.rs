//! Test logging utilities for system-level testing

use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

static INIT: Once = Once::new();
static DEBUG_INIT: Once = Once::new();

/// Initialize test logging with info level (called once per test run)
pub fn init_test_logging() {
    INIT.call_once(|| {
        let _ = init_tracing_subscriber("info", false);
    });
}

/// Initialize test logging with debug level (called once per test run)
pub fn init_debug_logging() {
    DEBUG_INIT.call_once(|| {
        let _ = init_tracing_subscriber("debug", false);
    });
}

/// Initialize tracing subscriber with specified level and format
fn init_tracing_subscriber(level: &str, json_output: bool) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))?;

    let subscriber = Registry::default().with(env_filter);

    if json_output {
        let json_layer = fmt::layer()
            .json()
            .with_target(true)
            .with_level(true);
        tracing::subscriber::set_global_default(subscriber.with(json_layer))?;
    } else {
        let fmt_layer = fmt::layer()
            .pretty()
            .with_target(true)
            .with_level(true)
            .with_test_writer(); // Use test writer for better test output
        tracing::subscriber::set_global_default(subscriber.with(fmt_layer))?;
    }

    Ok(())
}

/// Test logger that captures log entries for verification
pub struct TestLogger {
    entries: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl TestLogger {
    /// Create a new test logger
    pub fn new() -> Self {
        Self {
            entries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Add a log entry
    pub fn log(&self, message: String) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(message);
        }
    }

    /// Get all log entries
    pub fn entries(&self) -> Vec<String> {
        self.entries.lock().unwrap_or_else(|_| panic!("Failed to lock entries")).clone()
    }

    /// Clear all log entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    /// Check if any entries contain the specified text
    pub fn contains(&self, text: &str) -> bool {
        self.entries().iter().any(|entry| entry.contains(text))
    }
}

impl Default for TestLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_basic_functionality() {
        let logger = TestLogger::new();
        
        logger.log("Test message 1".to_string());
        logger.log("Test message 2".to_string());
        
        let entries = logger.entries();
        assert_eq!(entries.len(), 2);
        assert!(logger.contains("Test message 1"));
        assert!(logger.contains("Test message 2"));
        assert!(!logger.contains("Non-existent message"));
        
        logger.clear();
        assert_eq!(logger.entries().len(), 0);
    }

    #[test]
    fn test_init_logging() {
        // These should not panic when called multiple times
        init_test_logging();
        init_test_logging();
        init_debug_logging();
        init_debug_logging();
    }
} 