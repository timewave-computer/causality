//! Testing Framework for Code Generation
//!
//! This module provides testing utilities for verifying code generation
//! for different target platforms.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::TemporalEffectGraph;
use super::{CodeGenConfig, GeneratedCode, CodeGenerator};

/// Test case for code generation
pub struct CodeGenTestCase {
    /// Name of the test
    pub name: String,
    
    /// Description of what the test verifies
    pub description: String,
    
    /// Input TEG for the test
    pub input_teg: TemporalEffectGraph,
    
    /// Expected outputs for different targets
    pub expected_outputs: HashMap<String, ExpectedOutput>,
}

/// Expected output for a specific target
pub struct ExpectedOutput {
    /// Expected main output content (patterns to match)
    pub main_output_patterns: Vec<String>,
    
    /// Expected files to be present
    pub expected_files: Vec<String>,
    
    /// Expected language
    pub expected_language: String,
    
    /// Expected metadata keys and values
    pub expected_metadata: HashMap<String, String>,
}

/// Test runner for code generation
pub struct CodeGenTestRunner {
    /// The code generators to test
    generators: HashMap<String, Box<dyn CodeGenerator>>,
    
    /// Test cases
    test_cases: Vec<CodeGenTestCase>,
    
    /// Configuration for code generation
    config: CodeGenConfig,
}

impl CodeGenTestRunner {
    /// Create a new test runner
    pub fn new(config: CodeGenConfig) -> Self {
        Self {
            generators: HashMap::new(),
            test_cases: Vec::new(),
            config,
        }
    }
    
    /// Register a code generator for testing
    pub fn register_generator<T: CodeGenerator + 'static>(&mut self, generator: T) -> &mut Self {
        self.generators.insert(generator.name().to_string(), Box::new(generator));
        self
    }
    
    /// Add a test case
    pub fn add_test_case(&mut self, test_case: CodeGenTestCase) -> &mut Self {
        self.test_cases.push(test_case);
        self
    }
    
    /// Run all tests
    pub fn run_all(&self) -> Result<TestSummary> {
        let mut summary = TestSummary::default();
        
        for test_case in &self.test_cases {
            let test_name = &test_case.name;
            println!("Running test: {}", test_name);
            
            for (target_name, expected) in &test_case.expected_outputs {
                if let Some(generator) = self.generators.get(target_name) {
                    match self.run_single_test(test_case, target_name, generator.as_ref(), expected) {
                        Ok(()) => {
                            println!("  ✅ Target {} passed", target_name);
                            summary.passed += 1;
                        }
                        Err(e) => {
                            println!("  ❌ Target {} failed: {}", target_name, e);
                            summary.failed += 1;
                            summary.failures.push(format!("{} ({}): {}", test_name, target_name, e));
                        }
                    }
                } else {
                    println!("  ⚠️ Generator for target {} not registered, skipping", target_name);
                    summary.skipped += 1;
                }
            }
        }
        
        Ok(summary)
    }
    
    /// Run a single test for a specific target
    fn run_single_test(
        &self,
        test_case: &CodeGenTestCase,
        target_name: &str,
        generator: &dyn CodeGenerator,
        expected: &ExpectedOutput,
    ) -> Result<()> {
        // Generate code
        let result = generator.generate(&test_case.input_teg, &self.config)?;
        
        // Verify output language
        if result.language != expected.expected_language {
            return Err(anyhow!(
                "Language mismatch: expected {}, got {}",
                expected.expected_language, result.language
            ));
        }
        
        // Verify main output contains expected patterns
        for pattern in &expected.main_output_patterns {
            if !result.main_output.contains(pattern) {
                return Err(anyhow!("Main output missing pattern: {}", pattern));
            }
        }
        
        // Verify expected files are present
        for file in &expected.expected_files {
            if !result.additional_files.contains_key(file) {
                return Err(anyhow!("Missing expected file: {}", file));
            }
        }
        
        // Verify metadata
        for (key, value) in &expected.expected_metadata {
            if let Some(actual_value) = result.metadata.get(key) {
                if actual_value != value {
                    return Err(anyhow!(
                        "Metadata mismatch for key {}: expected {}, got {}",
                        key, value, actual_value
                    ));
                }
            } else {
                return Err(anyhow!("Missing expected metadata key: {}", key));
            }
        }
        
        Ok(())
    }
    
    /// Run tests for a specific target only
    pub fn run_for_target(&self, target_name: &str) -> Result<TestSummary> {
        let mut summary = TestSummary::default();
        
        if let Some(generator) = self.generators.get(target_name) {
            for test_case in &self.test_cases {
                let test_name = &test_case.name;
                
                if let Some(expected) = test_case.expected_outputs.get(target_name) {
                    println!("Running test: {} for target {}", test_name, target_name);
                    
                    match self.run_single_test(test_case, target_name, generator.as_ref(), expected) {
                        Ok(()) => {
                            println!("  ✅ Passed");
                            summary.passed += 1;
                        }
                        Err(e) => {
                            println!("  ❌ Failed: {}", e);
                            summary.failed += 1;
                            summary.failures.push(format!("{}: {}", test_name, e));
                        }
                    }
                } else {
                    println!("  ⚠️ No expected output for target {}, skipping test {}", 
                        target_name, test_name);
                    summary.skipped += 1;
                }
            }
        } else {
            return Err(anyhow!("Generator for target {} not registered", target_name));
        }
        
        Ok(summary)
    }
    
    /// Create test case builders for standard code generation scenarios
    pub fn create_standard_test_suite() -> Vec<CodeGenTestCase> {
        use crate::builder::GraphBuilder;
        
        let mut test_cases = Vec::new();
        
        // Test case 1: Simple resource operation
        let mut builder = GraphBuilder::new();
        let effect1 = builder.add_effect("create_resource", "basic");
        let resource = builder.add_resource("test_resource", "basic");
        builder.connect_effect_to_resource(effect1, resource, crate::graph::edge::AccessMode::Write);
        
        let teg = builder.build().unwrap();
        
        let mut expected_outputs = HashMap::new();
        
        // Rust expectations
        expected_outputs.insert("rust".to_string(), ExpectedOutput {
            main_output_patterns: vec![
                "pub fn create_resource".to_string(),
                "impl ResourceOperations".to_string(),
            ],
            expected_files: vec![
                "Cargo.toml".to_string(),
                "src/lib.rs".to_string(),
            ],
            expected_language: "rust".to_string(),
            expected_metadata: {
                let mut map = HashMap::new();
                map.insert("target".to_string(), "rust".to_string());
                map
            },
        });
        
        // CosmWasm expectations
        expected_outputs.insert("cosmwasm".to_string(), ExpectedOutput {
            main_output_patterns: vec![
                "pub struct Contract".to_string(),
                "pub enum ExecuteMsg".to_string(),
            ],
            expected_files: vec![
                "Cargo.toml".to_string(),
                "execute_handlers.rs".to_string(),
            ],
            expected_language: "rust".to_string(),
            expected_metadata: {
                let mut map = HashMap::new();
                map.insert("target".to_string(), "cosmwasm".to_string());
                map
            },
        });
        
        // Ethereum VM expectations
        expected_outputs.insert("evm".to_string(), ExpectedOutput {
            main_output_patterns: vec![
                "contract ".to_string(),
                "function create".to_string(),
            ],
            expected_files: vec![
                "package.json".to_string(),
                "hardhat.config.js".to_string(),
            ],
            expected_language: "solidity".to_string(),
            expected_metadata: {
                let mut map = HashMap::new();
                map.insert("target".to_string(), "evm".to_string());
                map
            },
        });
        
        test_cases.push(CodeGenTestCase {
            name: "simple_resource_operation".to_string(),
            description: "Tests code generation for a simple resource operation".to_string(),
            input_teg: teg,
            expected_outputs,
        });
        
        // Test case 2: Effect composition
        let mut builder = GraphBuilder::new();
        let effect1 = builder.add_effect("read_data", "basic");
        let effect2 = builder.add_effect("process_data", "basic");
        let effect3 = builder.add_effect("write_result", "basic");
        
        builder.connect_effects(effect1, effect2);
        builder.connect_effects(effect2, effect3);
        
        let teg = builder.build().unwrap();
        
        let mut expected_outputs = HashMap::new();
        
        // Only add Rust expectations for this test
        expected_outputs.insert("rust".to_string(), ExpectedOutput {
            main_output_patterns: vec![
                "pub fn read_data".to_string(),
                "pub fn process_data".to_string(),
                "pub fn write_result".to_string(),
            ],
            expected_files: vec![
                "Cargo.toml".to_string(),
                "src/lib.rs".to_string(),
            ],
            expected_language: "rust".to_string(),
            expected_metadata: {
                let mut map = HashMap::new();
                map.insert("target".to_string(), "rust".to_string());
                map
            },
        });
        
        test_cases.push(CodeGenTestCase {
            name: "effect_composition".to_string(),
            description: "Tests code generation for composed effects".to_string(),
            input_teg: teg,
            expected_outputs,
        });
        
        test_cases
    }
    
    /// Run performance benchmarks for each generator
    pub fn run_performance_benchmarks(&self) -> Result<BenchmarkResults> {
        use std::time::Instant;
        
        let mut results = BenchmarkResults {
            target_timings: HashMap::new(),
        };
        
        // Create a large TEG for benchmarking
        let large_teg = self.create_benchmark_teg()?;
        
        for (target_name, generator) in &self.generators {
            println!("Benchmarking target: {}", target_name);
            
            let start = Instant::now();
            let _ = generator.generate(&large_teg, &self.config)?;
            let elapsed = start.elapsed();
            
            println!("  ⏱️ Time: {:?}", elapsed);
            results.target_timings.insert(target_name.clone(), elapsed.as_millis() as u64);
        }
        
        Ok(results)
    }
    
    /// Create a large TEG for benchmarking purposes
    fn create_benchmark_teg(&self) -> Result<TemporalEffectGraph> {
        use crate::builder::GraphBuilder;
        
        let mut builder = GraphBuilder::new();
        
        // Create a more complex graph for benchmarking
        // Add many effects and resources with connections
        
        for i in 0..100 {
            let effect = builder.add_effect(&format!("effect_{}", i), "benchmark");
            
            // Connect to previous effects to create a chain
            if i > 0 {
                let prev_effect = format!("effect_{}", i-1);
                builder.connect_effects(&prev_effect, &effect);
            }
            
            // Add some resources
            if i % 5 == 0 {
                let resource = builder.add_resource(&format!("resource_{}", i/5), "benchmark_type");
                builder.connect_effect_to_resource(&effect, &resource, crate::graph::edge::AccessMode::ReadWrite);
            }
        }
        
        builder.build()
    }
    
    /// Verify resource handling across different targets
    pub fn verify_resource_handling(&self) -> Result<ResourceVerificationResults> {
        let mut results = ResourceVerificationResults {
            target_results: HashMap::new(),
        };
        
        // Create a TEG with various resource operations
        let resource_teg = self.create_resource_operation_teg()?;
        
        for (target_name, generator) in &self.generators {
            println!("Verifying resource handling for target: {}", target_name);
            
            let result = generator.generate(&resource_teg, &self.config)?;
            
            // Verify resource operations are properly implemented
            let mut resource_operations = Vec::new();
            
            // Check for resource operation patterns in generated code
            if result.main_output.contains("create_resource") {
                resource_operations.push("create".to_string());
            }
            
            if result.main_output.contains("read_resource") {
                resource_operations.push("read".to_string());
            }
            
            if result.main_output.contains("update_resource") {
                resource_operations.push("update".to_string());
            }
            
            if result.main_output.contains("delete_resource") {
                resource_operations.push("delete".to_string());
            }
            
            results.target_results.insert(target_name.clone(), resource_operations);
        }
        
        Ok(results)
    }
    
    /// Create a TEG with various resource operations for testing
    fn create_resource_operation_teg(&self) -> Result<TemporalEffectGraph> {
        use crate::builder::GraphBuilder;
        
        let mut builder = GraphBuilder::new();
        
        // Create operations
        let create_effect = builder.add_effect("create_resource", "basic");
        let read_effect = builder.add_effect("read_resource", "basic");
        let update_effect = builder.add_effect("update_resource", "basic");
        let delete_effect = builder.add_effect("delete_resource", "basic");
        
        // Add resource
        let resource = builder.add_resource("test_resource", "basic_type");
        
        // Connect effects to resource with appropriate access modes
        builder.connect_effect_to_resource(&create_effect, &resource, crate::graph::edge::AccessMode::Write);
        builder.connect_effect_to_resource(&read_effect, &resource, crate::graph::edge::AccessMode::Read);
        builder.connect_effect_to_resource(&update_effect, &resource, crate::graph::edge::AccessMode::ReadWrite);
        builder.connect_effect_to_resource(&delete_effect, &resource, crate::graph::edge::AccessMode::Write);
        
        // Create a sequence for operations
        builder.connect_effects(&create_effect, &read_effect);
        builder.connect_effects(&read_effect, &update_effect);
        builder.connect_effects(&update_effect, &delete_effect);
        
        builder.build()
    }
}

/// Test summary
#[derive(Debug, Default)]
pub struct TestSummary {
    /// Number of tests passed
    pub passed: usize,
    
    /// Number of tests failed
    pub failed: usize,
    
    /// Number of tests skipped
    pub skipped: usize,
    
    /// Failure messages
    pub failures: Vec<String>,
}

impl TestSummary {
    /// Print the summary
    pub fn print(&self) {
        println!("Test Summary:");
        println!("  Passed: {}", self.passed);
        println!("  Failed: {}", self.failed);
        println!("  Skipped: {}", self.skipped);
        
        if !self.failures.is_empty() {
            println!("Failures:");
            for (i, failure) in self.failures.iter().enumerate() {
                println!("  {}. {}", i + 1, failure);
            }
        }
    }
    
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Benchmark results
#[derive(Debug)]
pub struct BenchmarkResults {
    /// Execution time in milliseconds for each target
    pub target_timings: HashMap<String, u64>,
}

/// Resource verification results
#[derive(Debug)]
pub struct ResourceVerificationResults {
    /// Resource operations supported by each target
    pub target_results: HashMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_generation::target::BaseTarget;
    
    /// A mock code generator for testing
    #[derive(Debug)]
    struct MockGenerator {
        name: String,
        language: String,
    }
    
    impl MockGenerator {
        fn new(name: &str, language: &str) -> Self {
            Self {
                name: name.to_string(),
                language: language.to_string(),
            }
        }
    }
    
    impl CodeGenerator for MockGenerator {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            "Mock generator for testing"
        }
        
        fn generate(&self, _teg: &TemporalEffectGraph, _config: &CodeGenConfig) -> Result<GeneratedCode> {
            // Generate a simple mock output
            let mut main_output = String::new();
            
            // Add some patterns for testing
            main_output.push_str("pub fn create_resource() {}\n");
            main_output.push_str("pub fn read_resource() {}\n");
            main_output.push_str("pub fn update_resource() {}\n");
            main_output.push_str("pub fn delete_resource() {}\n");
            main_output.push_str("impl ResourceOperations {}\n");
            
            // Additional files
            let mut additional_files = HashMap::new();
            additional_files.insert("Cargo.toml".to_string(), "name = \"test\"".to_string());
            additional_files.insert("src/lib.rs".to_string(), "// Test lib".to_string());
            
            // Metadata
            let mut metadata = HashMap::new();
            metadata.insert("target".to_string(), self.name.clone());
            
            Ok(GeneratedCode {
                main_output,
                language: self.language.clone(),
                additional_files,
                metadata,
            })
        }
    }
    
    #[test]
    fn test_register_generator() {
        let mut runner = CodeGenTestRunner::new(CodeGenConfig::default());
        
        // Register some mock generators
        runner.register_generator(MockGenerator::new("rust", "rust"));
        runner.register_generator(MockGenerator::new("cosmwasm", "rust"));
        
        assert_eq!(runner.generators.len(), 2);
        assert!(runner.generators.contains_key("rust"));
        assert!(runner.generators.contains_key("cosmwasm"));
    }
    
    #[test]
    fn test_add_test_case() {
        let mut runner = CodeGenTestRunner::new(CodeGenConfig::default());
        
        // Create a test case
        let test_cases = CodeGenTestRunner::create_standard_test_suite();
        
        for test_case in test_cases {
            runner.add_test_case(test_case);
        }
        
        assert!(!runner.test_cases.is_empty());
    }
    
    #[test]
    fn test_run_single_test() {
        let mut runner = CodeGenTestRunner::new(CodeGenConfig::default());
        
        // Register a mock generator
        runner.register_generator(MockGenerator::new("rust", "rust"));
        
        // Create and add a test case
        let test_cases = CodeGenTestRunner::create_standard_test_suite();
        assert!(!test_cases.is_empty());
        
        for test_case in test_cases {
            runner.add_test_case(test_case);
        }
        
        // Run the test for the rust target
        let result = runner.run_for_target("rust");
        assert!(result.is_ok());
        
        let summary = result.unwrap();
        println!("Test summary: {:?}", summary);
        
        // At least one test should have passed
        assert!(summary.passed > 0);
    }
} 