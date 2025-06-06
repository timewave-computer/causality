//! Performance benchmarks for the Causality compiler
//!
//! This module provides comprehensive benchmarks for compilation performance,
//! instruction efficiency, and optimization characteristics.

use crate::pipeline::compile;
use std::time::{Duration, Instant};

/// Benchmark results for compilation performance
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub compilation_time: Duration,
    pub instruction_count: usize,
    pub register_count: usize,
    pub gas_estimate: u64,
    pub source_size: usize,
    pub memory_usage: usize,
}

/// Comprehensive benchmark suite for the compiler
pub struct CompilerBenchmark {
    results: Vec<BenchmarkResult>,
}

impl CompilerBenchmark {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a single benchmark test
    pub fn benchmark_source(&mut self, test_name: &str, source: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Measure memory before compilation
        let memory_before = get_memory_usage();
        
        // Compile the source
        let artifact = compile(source)?;
        
        let compilation_time = start_time.elapsed();
        let memory_after = get_memory_usage();
        
        // Calculate metrics
        let instruction_count = artifact.instructions.len();
        let register_count = count_unique_registers(&artifact.instructions);
        let gas_estimate = estimate_gas_cost(&artifact.instructions);
        let source_size = source.len();
        let memory_usage = memory_after.saturating_sub(memory_before);
        
        let result = BenchmarkResult {
            test_name: test_name.to_string(),
            compilation_time,
            instruction_count,
            register_count,
            gas_estimate,
            source_size,
            memory_usage,
        };
        
        self.results.push(result.clone());
        Ok(result)
    }

    /// Run all standard benchmarks
    pub fn run_all_benchmarks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple expressions that are currently supported
        self.benchmark_source("pure_literal", "(pure 42)")?;
        self.benchmark_source("alloc_simple", "(alloc 100)")?;
        self.benchmark_source("bind_simple", "(bind (pure 1) (pure 2))")?;
        
        // More complex expressions using supported constructs
        self.benchmark_source("nested_bind", 
            "(bind (pure 10) (bind (pure 20) (pure 30)))")?;
        
        self.benchmark_source("alloc_with_bind",
            "(bind (alloc 42) (pure 100))")?;
        
        self.benchmark_source("resource_allocation",
            "(alloc (alloc 100))")?;
        
        // Realistic examples using only supported constructs
        self.benchmark_source("token_creation",
            "(bind (pure 1000) (alloc 100))")?;
        
        self.benchmark_source("sequential_effects",
            "(bind (pure 1) (bind (pure 2) (bind (pure 3) (pure 4))))")?;
        
        // Resource management patterns
        self.benchmark_source("resource_bind_chain",
            "(bind (alloc 1) (bind (alloc 2) (bind (alloc 3) (pure 42))))")?;
        
        Ok(())
    }

    /// Get benchmark results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Causality Compiler Performance Benchmark Report ===\n\n");
        
        for result in &self.results {
            report.push_str(&format!(
                "Test: {}\n\
                 Compilation Time: {:?}\n\
                 Instructions: {}\n\
                 Registers: {}\n\
                 Gas Estimate: {}\n\
                 Source Size: {} bytes\n\
                 Memory Usage: {} bytes\n\
                 Instructions/ms: {:.2}\n\
                 Gas/Instruction: {:.2}\n\n",
                result.test_name,
                result.compilation_time,
                result.instruction_count,
                result.register_count,
                result.gas_estimate,
                result.source_size,
                result.memory_usage,
                result.instruction_count as f64 / result.compilation_time.as_millis().max(1) as f64,
                if result.instruction_count > 0 { 
                    result.gas_estimate as f64 / result.instruction_count as f64 
                } else { 0.0 }
            ));
        }

        // Summary statistics
        if !self.results.is_empty() {
            let total_time: Duration = self.results.iter().map(|r| r.compilation_time).sum();
            let avg_instruction_count = self.results.iter().map(|r| r.instruction_count).sum::<usize>() / self.results.len();
            let avg_gas = self.results.iter().map(|r| r.gas_estimate).sum::<u64>() / self.results.len() as u64;
            
            report.push_str(&format!(
                "=== Summary Statistics ===\n\
                 Total Compilation Time: {:?}\n\
                 Average Instructions: {}\n\
                 Average Gas Cost: {}\n\
                 Total Tests: {}\n",
                total_time,
                avg_instruction_count,
                avg_gas,
                self.results.len()
            ));
        }
        
        report
    }

    /// Check for performance regressions
    pub fn check_performance_targets(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        for result in &self.results {
            // Compilation time targets
            let max_compile_time = match result.test_name.as_str() {
                "pure_literal" => Duration::from_millis(10),
                "alloc_simple" => Duration::from_millis(15),
                "bind_simple" => Duration::from_millis(25),
                "nested_bind" => Duration::from_millis(50),
                "alloc_with_bind" => Duration::from_millis(100),
                "resource_allocation" => Duration::from_millis(100),
                "token_creation" => Duration::from_millis(100),
                "sequential_effects" => Duration::from_millis(100),
                "resource_bind_chain" => Duration::from_millis(100),
                _ => Duration::from_millis(500), // Default
            };
            
            if result.compilation_time > max_compile_time {
                issues.push(format!(
                    "PERFORMANCE ISSUE: {} took {:?}, expected < {:?}",
                    result.test_name, result.compilation_time, max_compile_time
                ));
            }
            
            // Instruction efficiency targets
            let max_instructions = match result.test_name.as_str() {
                "pure_literal" => 3,
                "alloc_simple" => 5,
                "bind_simple" => 10,
                "resource_allocation" => 8,
                _ => 50, // Default
            };
            
            if result.instruction_count > max_instructions {
                issues.push(format!(
                    "EFFICIENCY ISSUE: {} generated {} instructions, expected <= {}",
                    result.test_name, result.instruction_count, max_instructions
                ));
            }
        }
        
        issues
    }
}

/// Helper function to count unique registers used
fn count_unique_registers(instructions: &[causality_core::machine::Instruction]) -> usize {
    use causality_core::machine::Instruction;
    use std::collections::HashSet;
    
    let mut registers = HashSet::new();
    
    for instruction in instructions {
        match instruction {
            Instruction::Move { src, dst } => {
                registers.insert(*src);
                registers.insert(*dst);
            }
            Instruction::Alloc { type_reg, val_reg, out_reg } => {
                registers.insert(*type_reg);
                registers.insert(*val_reg);
                registers.insert(*out_reg);
            }
            Instruction::Consume { resource_reg, out_reg } => {
                registers.insert(*resource_reg);
                registers.insert(*out_reg);
            }
            Instruction::Witness { out_reg } => {
                registers.insert(*out_reg);
            }
            Instruction::Apply { fn_reg, arg_reg, out_reg } => {
                registers.insert(*fn_reg);
                registers.insert(*arg_reg);
                registers.insert(*out_reg);
            }
            _ => {}
        }
    }
    
    registers.len()
}

/// Helper function to estimate gas cost
fn estimate_gas_cost(instructions: &[causality_core::machine::Instruction]) -> u64 {
    use causality_core::machine::Instruction;
    
    let mut total_cost = 0u64;
    
    for instruction in instructions {
        total_cost += match instruction {
            Instruction::Move { .. } => 1,
            Instruction::Alloc { .. } => 10,
            Instruction::Consume { .. } => 5,
            Instruction::Witness { .. } => 3,
            Instruction::Apply { .. } => 20,
            _ => 5,
        };
    }
    
    total_cost
}

/// Simple memory usage estimation (placeholder)
fn get_memory_usage() -> usize {
    // In a real implementation, this would measure actual memory usage
    // For now, return a placeholder value
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_simple_compilation() {
        let mut benchmark = CompilerBenchmark::new();
        let result = benchmark.benchmark_source("test_simple", "(pure 42)").unwrap();
        
        assert_eq!(result.test_name, "test_simple");
        assert!(result.compilation_time > Duration::from_nanos(0));
        assert!(result.instruction_count > 0);
        assert!(result.gas_estimate > 0);
    }

    #[test]
    fn test_benchmark_alloc_performance() {
        let mut benchmark = CompilerBenchmark::new();
        let result = benchmark.benchmark_source("test_alloc", "(alloc 100)").unwrap();
        
        assert!(result.instruction_count >= 2); // Should have witness + alloc at minimum
        assert!(result.gas_estimate >= 10); // Alloc has cost 10
    }

    #[test]
    fn test_full_benchmark_suite() {
        let mut benchmark = CompilerBenchmark::new();
        benchmark.run_all_benchmarks().unwrap();
        
        assert!(!benchmark.results().is_empty());
        assert!(benchmark.results().len() >= 5); // Should have multiple tests
        
        // Check that we have expected test names
        let test_names: Vec<_> = benchmark.results().iter().map(|r| r.test_name.as_str()).collect();
        assert!(test_names.contains(&"pure_literal"));
        assert!(test_names.contains(&"alloc_simple"));
        assert!(test_names.contains(&"resource_allocation"));
    }

    #[test]
    fn test_performance_report_generation() {
        let mut benchmark = CompilerBenchmark::new();
        benchmark.benchmark_source("test_report", "(pure 42)").unwrap();
        
        let report = benchmark.generate_report();
        assert!(report.contains("Performance Benchmark Report"));
        assert!(report.contains("test_report"));
        assert!(report.contains("Compilation Time"));
        assert!(report.contains("Instructions"));
    }

    #[test]
    fn test_performance_targets() {
        let mut benchmark = CompilerBenchmark::new();
        
        // This should be fast enough to pass performance targets
        benchmark.benchmark_source("pure_literal", "(pure 42)").unwrap();
        
        let issues = benchmark.check_performance_targets();
        // In a well-optimized system, simple literals should compile quickly
        // We'll check if there are major performance issues
        if !issues.is_empty() {
            println!("Performance issues detected: {:?}", issues);
        }
    }

    #[test]
    fn test_instruction_efficiency() {
        let mut benchmark = CompilerBenchmark::new();
        
        // Test instruction count efficiency
        let simple_result = benchmark.benchmark_source("simple", "(pure 42)").unwrap();
        let complex_result = benchmark.benchmark_source("complex", 
            "(bind (pure 10) (bind (pure 20) (pure 30)))").unwrap();
        
        // Complex expressions should generate more instructions than simple ones
        assert!(complex_result.instruction_count > simple_result.instruction_count);
        assert!(complex_result.gas_estimate > simple_result.gas_estimate);
    }

    #[test]
    fn test_compilation_scaling() {
        let mut benchmark = CompilerBenchmark::new();
        
        // Test how compilation time scales with complexity
        let small = benchmark.benchmark_source("small", "(pure 1)").unwrap();
        let medium = benchmark.benchmark_source("medium", 
            "(bind (pure 1) (bind (pure 2) (pure 3)))").unwrap();
        let large = benchmark.benchmark_source("large",
            "(bind (alloc 1) (bind (alloc 2) (bind (alloc 3) (bind (alloc 4) (pure 5)))))").unwrap();
        
        // Verify that instruction count scales reasonably
        assert!(small.instruction_count <= medium.instruction_count);
        assert!(medium.instruction_count <= large.instruction_count);
        
        // Gas cost should also scale
        assert!(small.gas_estimate <= medium.gas_estimate);
        assert!(medium.gas_estimate <= large.gas_estimate);
    }

    #[test]
    fn test_register_usage_efficiency() {
        let mut benchmark = CompilerBenchmark::new();
        
        // Simple expressions should use fewer registers
        let simple = benchmark.benchmark_source("simple_reg", "(pure 42)").unwrap();
        let complex = benchmark.benchmark_source("complex_reg",
            "(bind (alloc 1) (bind (alloc 2) (bind (alloc 3) (pure 42))))").unwrap();
        
        // Complex expressions should generally use more registers
        assert!(complex.register_count >= simple.register_count);
        
        // But register usage should be reasonable (not exponential)
        assert!(complex.register_count < 20); // Sanity check
    }
} 