//! Code Generation Framework for TEG
//!
//! This module provides a framework for generating code from Temporal Effect Graphs
//! for various target platforms.

pub mod target;
pub mod ethereum;
pub mod cosmwasm;
pub mod rust;
pub mod testing;

use anyhow::Result;
use crate::TemporalEffectGraph;

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

/// Get a code generator for the specified target
pub fn get_generator(target: &str) -> Option<Box<dyn CodeGenerator>> {
    match target {
        "ethereum" => Some(Box::new(ethereum::EthereumGenerator::new())),
        "cosmwasm" => Some(Box::new(cosmwasm::CosmWasmGenerator::new())),
        "rust" => Some(Box::new(rust::RustGenerator::new())),
        _ => None,
    }
}

/// Generate code for a specific target
pub fn generate_code(teg: &TemporalEffectGraph, target: &str, config: Option<CodeGenConfig>) -> Result<GeneratedCode> {
    let generator = get_generator(target)
        .ok_or_else(|| anyhow::anyhow!("Unsupported target: {}", target))?;
    
    let config = config.unwrap_or_else(|| generator.default_config());
    
    generator.generate(teg, &config)
}

/// Run tests for all supported targets
pub fn run_tests() -> Result<testing::TestSummary> {
    let mut runner = testing::CodeGenTestRunner::new(CodeGenConfig::default());
    
    // Register all supported generators
    runner
        .register_generator(ethereum::EthereumGenerator::new())
        .register_generator(cosmwasm::CosmWasmGenerator::new())
        .register_generator(rust::RustGenerator::new());
    
    // Add standard test cases
    for test_case in testing::CodeGenTestRunner::create_standard_test_suite() {
        runner.add_test_case(test_case);
    }
    
    // Run all tests
    let summary = runner.run_all()?;
    summary.print();
    
    Ok(summary)
}

/// Run performance benchmarks for all supported targets
pub fn run_benchmarks() -> Result<testing::BenchmarkResults> {
    let mut runner = testing::CodeGenTestRunner::new(CodeGenConfig::default());
    
    // Register all supported generators
    runner
        .register_generator(ethereum::EthereumGenerator::new())
        .register_generator(cosmwasm::CosmWasmGenerator::new())
        .register_generator(rust::RustGenerator::new());
    
    // Run benchmarks
    runner.run_performance_benchmarks()
} 