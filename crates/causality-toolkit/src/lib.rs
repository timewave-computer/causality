/// Causality Toolkit
///
/// High-level development tools and utilities for building Causality applications.
/// This crate provides developer-friendly abstractions over the core Causality system.
// Core modules - working
pub mod cross_language;
pub mod debug;
// pub mod dsl; // Temporarily disabled due to intent_builder API changes
// pub mod effects; // Temporarily disabled due to type compatibility issues
pub mod formal_verification;
// pub mod interface_synthesis; // Temporarily disabled due to doc comment issues
// pub mod mocks; // Temporarily disabled due to type compatibility issues
// pub mod primitives; // Temporarily disabled due to type compatibility issues
pub mod resources;
// pub mod testing; // Temporarily disabled due to type compatibility issues
pub mod utils;
// pub mod almanac_schema; // Temporarily disabled due to import conflicts
pub mod fixed_point;

// Re-exports - updated for new causality-core API
pub use causality_core::{Value, EntityId};

/// Main toolkit interface for building Causality applications
pub struct CausalityToolkit {
    /// Configuration for the toolkit
    config: ToolkitConfig,
}

/// Configuration for the Causality toolkit
#[derive(Debug, Clone, Default)]
pub struct ToolkitConfig {
    /// Default deployment settings
    pub deployment_config: DeploymentConfig,
    
    /// Logging and debugging options
    pub debug_config: DebugConfig,
}

/// Deployment configuration
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    /// Target chains for deployment
    pub target_chains: Vec<String>,
    
    /// Gas optimization settings
    pub gas_optimization: bool,
    
    /// ZK proof generation settings
    pub zk_proofs_enabled: bool,
    
    /// Cross-chain bridging settings
    pub cross_chain_enabled: bool,
}

/// Debug and logging configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable verbose logging
    pub verbose: bool,
    
    /// Enable execution tracing
    pub trace_execution: bool,
    
    /// Enable performance profiling
    pub profile_performance: bool,
    
    /// Output directory for logs and artifacts
    pub output_dir: String,
}

impl Default for CausalityToolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalityToolkit {
    /// Create a new toolkit instance with default configuration
    pub fn new() -> Self {
        Self {
            config: ToolkitConfig::default(),
        }
    }
    
    /// Create a new toolkit instance with custom configuration
    pub fn with_config(config: ToolkitConfig) -> Self {
        Self { config }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &ToolkitConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: ToolkitConfig) {
        self.config = config;
    }
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            target_chains: vec!["ethereum".to_string(), "polygon".to_string()],
            gas_optimization: true,
            zk_proofs_enabled: true,
            cross_chain_enabled: true,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            trace_execution: false,
            profile_performance: false,
            output_dir: "./causality_output".to_string(),
        }
    }
}

/// Convenience function to create a new toolkit
pub fn toolkit() -> CausalityToolkit {
    CausalityToolkit::new()
}

/// Convenience function to create a toolkit with verbose logging
pub fn toolkit_verbose() -> CausalityToolkit {
    let mut config = ToolkitConfig::default();
    config.debug_config.verbose = true;
    config.debug_config.trace_execution = true;
    CausalityToolkit::with_config(config)
}

pub use fixed_point::FixedPoint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolkit_creation() {
        let toolkit = CausalityToolkit::new();
        assert!(!toolkit.config.debug_config.verbose);
        assert!(toolkit.config.deployment_config.gas_optimization);
    }
}
