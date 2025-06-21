//! Causality Toolkit
//!
//! High-level development tools and utilities for building Causality applications.
//! This crate provides developer-friendly abstractions over the core Causality system.

pub mod cross_language;
pub mod debug;
pub mod dsl;
pub mod effects;
pub mod formal_verification;
pub mod interface_synthesis;
pub mod mocks;
pub mod primitives;
pub mod resources;
pub mod testing;
pub mod utils;
pub mod almanac_schema;
pub mod intent;
pub mod fixed_point;

// Re-exports
pub use causality_core::{Value, Effect, EffectExpr, EntityId};
use anyhow::Result;

/// Main toolkit interface for building Causality applications
pub struct CausalityToolkit {
    /// Configuration for the toolkit
    config: ToolkitConfig,
}

/// Configuration for the Causality toolkit
#[derive(Debug, Clone)]
#[derive(Default)]
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
    
    /// Get available primitives
    pub fn available_primitives(&self) -> Vec<&'static str> {
        vec![
            "fungible_token",
            "non_fungible_token", 
            "vault",
            "lending_market",
            "dex",
        ]
    }
    
    /// Create a fungible token primitive
    pub fn create_fungible_token(
        &self,
        token_info: primitives::TokenInfo,
        initial_supply: u128,
        initial_owner: String,
        config: Option<primitives::PrimitiveConfig>,
    ) -> Result<primitives::FungibleToken> {
        let config = config.unwrap_or_default();
        primitives::FungibleToken::new(token_info, initial_supply, initial_owner, config)
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

pub use intent::*;
pub use fixed_point::FixedPoint;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::DeFiPrimitive;
    use std::collections::BTreeMap;

    #[test]
    fn test_toolkit_creation() {
        let toolkit = CausalityToolkit::new();
        assert!(!toolkit.config.debug_config.verbose);
        assert!(toolkit.config.deployment_config.gas_optimization);
    }
    
    #[test]
    fn test_available_primitives() {
        let toolkit = CausalityToolkit::new();
        let primitives = toolkit.available_primitives();
        
        assert!(primitives.contains(&"fungible_token"));
        assert!(primitives.contains(&"non_fungible_token"));
        assert!(primitives.contains(&"vault"));
        assert!(primitives.contains(&"lending_market"));
        assert!(primitives.contains(&"dex"));
    }
    
    #[test]
    fn test_fungible_token_creation() {
        let toolkit = CausalityToolkit::new();
        
        let token_info = primitives::TokenInfo {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            max_supply: Some(1_000_000_000_000_000_000_000_000),
            description: Some("A test token for the toolkit".to_string()),
            logo_uri: None,
            metadata: BTreeMap::new(),
        };
        
        let token = toolkit.create_fungible_token(
            token_info,
            1_000_000_000_000_000_000_000,
            "owner".to_string(),
            None,
        ).unwrap();
        
        assert_eq!(token.state().token_info.name, "Test Token");
        assert_eq!(token.state().total_supply, 1_000_000_000_000_000_000_000);
    }
} 