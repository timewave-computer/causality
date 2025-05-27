//! Code Generation Framework for TEG
//!
//! This module provides a framework for generating code from Temporal Effect Graphs
//! for various target platforms.

pub mod target;

use anyhow::Result;
use causality_ir::graph::TemporalEffectGraph;

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    /// Target-specific options
    pub target_options: std::collections::HashMap<String, String>,
    
    /// Optimization level (0-3)
    pub optimization_level: u8,
    
    /// Whether to include debug information
    pub include_debug_info: bool,
    
    /// Output directory for generated code
    pub output_dir: Option<String>,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        let mut target_options = std::collections::HashMap::new();
        
        // Set some reasonable defaults
        target_options.insert("target_version".to_string(), "latest".to_string());
        
        Self {
            target_options,
            optimization_level: 2,
            include_debug_info: false,
            output_dir: None,
        }
    }
}

/// Target-specific code generator
pub trait CodeGenerator {
    /// Get the name of this target
    fn name(&self) -> &str;
    
    /// Get a description of this target
    fn description(&self) -> &str;
    
    /// Generate code for the target from a TEG
    fn generate(&self, teg: &TemporalEffectGraph, config: &CodeGenConfig) -> Result<GeneratedCode>;
    
    /// Get default configuration for this target
    fn default_config(&self) -> CodeGenConfig {
        CodeGenConfig::default()
    }
}

/// Generated code output
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// The primary output file content
    pub main_output: String,
    
    /// The language of the generated code
    pub language: String,
    
    /// Additional output files
    pub additional_files: std::collections::HashMap<String, String>,
    
    /// Any metadata about the generation process
    pub metadata: std::collections::HashMap<String, String>,
} 