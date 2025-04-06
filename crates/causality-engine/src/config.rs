// Configuration for the Causality Engine
//
// This module provides configuration options for the Causality Engine.

/// Configuration for the Causality Engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Timeout in milliseconds for operation invocations
    pub invocation_timeout_ms: u64,
    
    /// Whether to enable logging
    pub enable_logging: bool,
    
    /// Whether to enable debug mode
    pub debug_mode: bool,
    
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            invocation_timeout_ms: 5000, // 5 seconds
            enable_logging: true,
            debug_mode: false,
            max_concurrent_operations: 10,
        }
    }
}

impl EngineConfig {
    /// Create a new engine configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the invocation timeout
    pub fn with_invocation_timeout(mut self, timeout_ms: u64) -> Self {
        self.invocation_timeout_ms = timeout_ms;
        self
    }
    
    /// Enable or disable logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }
    
    /// Enable or disable debug mode
    pub fn with_debug_mode(mut self, enable: bool) -> Self {
        self.debug_mode = enable;
        self
    }
    
    /// Set the maximum number of concurrent operations
    pub fn with_max_concurrent_operations(mut self, max: usize) -> Self {
        self.max_concurrent_operations = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.invocation_timeout_ms, 5000);
        assert!(config.enable_logging);
        assert!(!config.debug_mode);
        assert_eq!(config.max_concurrent_operations, 10);
    }
    
    #[test]
    fn test_config_builder() {
        let config = EngineConfig::new()
            .with_invocation_timeout(10000)
            .with_logging(false)
            .with_debug_mode(true)
            .with_max_concurrent_operations(20);
        
        assert_eq!(config.invocation_timeout_ms, 10000);
        assert!(!config.enable_logging);
        assert!(config.debug_mode);
        assert_eq!(config.max_concurrent_operations, 20);
    }
} 