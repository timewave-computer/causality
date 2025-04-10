//! Target abstractions for code generation
//!
//! This module provides abstractions for target platforms when generating code
//! from Temporal Effect Graphs (TEG).

use std::collections::HashMap;
use anyhow::Result;
use crate::TemporalEffectGraph;
use super::{CodeGenConfig, GeneratedCode};

/// Capabilities of a target platform
#[derive(Debug, Clone)]
pub struct TargetCapabilities {
    /// Whether the target supports concurrency
    pub supports_concurrency: bool,
    
    /// Whether the target supports algebraic effects directly
    pub supports_algebraic_effects: bool,
    
    /// Whether the target supports resource capabilities
    pub supports_capabilities: bool,
    
    /// Resource types supported by this target
    pub supported_resource_types: Vec<String>,
    
    /// Effect types supported by this target
    pub supported_effect_types: Vec<String>,
    
    /// Additional capabilities specific to this target
    pub additional_capabilities: HashMap<String, bool>,
}

/// Interface for target platforms
pub trait Target {
    /// Get the name of this target
    fn name(&self) -> &str;
    
    /// Get the capabilities of this target
    fn capabilities(&self) -> &TargetCapabilities;
    
    /// Check if a TEG is compatible with this target
    fn is_compatible(&self, teg: &TemporalEffectGraph) -> bool;
    
    /// Preprocess a TEG for this target
    fn preprocess(&self, teg: &mut TemporalEffectGraph, config: &CodeGenConfig) -> Result<()>;
    
    /// Generate target-specific code
    fn generate_code(&self, teg: &TemporalEffectGraph, config: &CodeGenConfig) -> Result<GeneratedCode>;
}

/// Abstract base implementation for targets
pub struct BaseTarget {
    /// Name of the target
    name: String,
    
    /// Description of the target
    description: String,
    
    /// Capabilities of this target
    capabilities: TargetCapabilities,
}

impl BaseTarget {
    /// Create a new base target
    pub fn new(name: &str, description: &str, capabilities: TargetCapabilities) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            capabilities,
        }
    }
    
    /// Get the description of this target
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Standard target capabilities
pub fn standard_capabilities() -> TargetCapabilities {
    TargetCapabilities {
        supports_concurrency: false,
        supports_algebraic_effects: false,
        supports_capabilities: false,
        supported_resource_types: vec!["basic".to_string()],
        supported_effect_types: vec!["basic".to_string()],
        additional_capabilities: HashMap::new(),
    }
}

/// Advanced target capabilities with full support
pub fn advanced_capabilities() -> TargetCapabilities {
    TargetCapabilities {
        supports_concurrency: true,
        supports_algebraic_effects: true,
        supports_capabilities: true,
        supported_resource_types: vec![
            "basic".to_string(), 
            "complex".to_string(), 
            "streaming".to_string()
        ],
        supported_effect_types: vec![
            "basic".to_string(), 
            "io".to_string(), 
            "state".to_string(), 
            "async".to_string()
        ],
        additional_capabilities: {
            let mut caps = HashMap::new();
            caps.insert("advanced_optimization".to_string(), true);
            caps.insert("type_checking".to_string(), true);
            caps
        },
    }
} 