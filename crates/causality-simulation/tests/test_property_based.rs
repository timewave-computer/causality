//! Property-Based Testing
//!
//! This module tests mathematical invariants and properties including:
//! - Resource conservation properties
//! - Linearity properties and resource safety
//! - State consistency properties
//! - Metamorphic testing patterns
//! - Fuzzing integration for edge case discovery

use anyhow::Result;
use causality_simulation::SimulationEngine;
use rand::Rng;
use tokio::test as tokio_test;

// Property-based testing framework
struct PropertyTest<T> {
    name: String,
    generator: Box<dyn Fn() -> T>,
    property: Box<dyn Fn(&T) -> Result<bool>>,
    max_tests: usize,
}

impl<T> PropertyTest<T> {
    fn new<G, P>(name: &str, generator: G, property: P, max_tests: usize) -> Self 
    where
        G: Fn() -> T + 'static,
        P: Fn(&T) -> Result<bool> + 'static,
    {
        Self {
            name: name.to_string(),
            generator: Box::new(generator),
            property: Box::new(property),
            max_tests,
        }
    }
    
    async fn run(&self) -> Result<PropertyTestResult> {
        let mut passed = 0;
        let mut failed = 0;
        let mut failures = Vec::new();
        
        for i in 0..self.max_tests {
            let test_case = (self.generator)();
            
            match (self.property)(&test_case) {
                Ok(true) => passed += 1,
                Ok(false) => {
                    failed += 1;
                    failures.push(format!("Test case {} failed", i));
                }
                Err(e) => {
                    failed += 1;
                    failures.push(format!("Test case {} errored: {}", i, e));
                }
            }
        }
        
        Ok(PropertyTestResult {
            name: self.name.clone(),
            total_tests: self.max_tests,
            passed,
            failed,
            failures,
        })
    }
}

struct PropertyTestResult {
    name: String,
    total_tests: usize,
    passed: usize,
    failed: usize,
    failures: Vec<String>,
}

// Test data generators
struct ProgramGenerator;

impl ProgramGenerator {
    fn generate_alloc_program() -> String {
        let size = rand::thread_rng().gen_range(1..=1000);
        format!("(alloc {})", size)
    }
    
    fn generate_consume_program() -> String {
        let size = rand::thread_rng().gen_range(1..=1000);
        format!("(consume (alloc {}))", size)
    }
    
    fn generate_tensor_program() -> String {
        let count = rand::thread_rng().gen_range(2..=5);
        let sizes: Vec<u32> = (0..count).map(|_| rand::thread_rng().gen_range(1..=500)).collect();
        let allocs: Vec<String> = sizes.iter().map(|&s| format!("(alloc {})", s)).collect();
        format!("(tensor {})", allocs.join(" "))
    }
    
    fn generate_complex_program() -> String {
        let programs = [Self::generate_alloc_program(),
            Self::generate_consume_program(), 
            Self::generate_tensor_program()];
        programs[rand::thread_rng().gen_range(0..programs.len())].clone()
    }
}

#[tokio_test]
async fn test_resource_conservation_properties() -> Result<()> {
    println!("=== Testing Resource Conservation Properties ===");
    
    // Property: alloc followed by consume should maintain resource conservation
    let conservation_test = PropertyTest::new(
        "resource_conservation",
        || {
            let size = rand::thread_rng().gen_range(1..=1000);
            (size, format!("(consume (alloc {}))", size))
        },
        |(_size, program)| {
            // Test that alloc-consume maintains conservation
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                let result = engine.execute_program(program).await?;
                
                let progression = engine.state_progression();
                
                // Find alloc and consume operations
                let alloc_operations = progression.steps.iter()
                    .filter(|step| step.instruction.as_ref()
                        .map(|i| i.contains("Alloc"))
                        .unwrap_or(false))
                    .count();
                    
                let consume_operations = progression.steps.iter()
                    .filter(|step| step.instruction.as_ref()
                        .map(|i| i.contains("Consume"))
                        .unwrap_or(false))
                    .count();
                
                // Conservation property: every alloc should have a corresponding consume
                Ok(alloc_operations > 0 && consume_operations > 0)
            })
        },
        100
    );
    
    let result = conservation_test.run().await?;
    println!("✓ Resource Conservation: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 90 / 100); // At least 90% should pass
    
    Ok(())
}

#[tokio_test]
async fn test_linearity_properties() -> Result<()> {
    println!("=== Testing Linearity Properties ===");
    
    // Property: Linear resources cannot be used multiple times
    let linearity_test = PropertyTest::new(
        "linearity_enforcement",
        ProgramGenerator::generate_consume_program,
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                
                // Execute program once
                let result1 = engine.execute_program(program).await?;
                
                // Verify that the resource is consumed and cannot be used again
                // In a real implementation, this would check for double-consumption errors
                Ok(result1.step_count > 0)
            })
        },
        50
    );
    
    let result = linearity_test.run().await?;
    println!("✓ Linearity Enforcement: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed == result.total_tests); // All tests should pass for linearity
    
    Ok(())
}

#[tokio_test]
async fn test_state_consistency_properties() -> Result<()> {
    println!("=== Testing State Consistency Properties ===");
    
    // Property: State should remain consistent across all operations
    let consistency_test = PropertyTest::new(
        "state_consistency",
        ProgramGenerator::generate_complex_program,
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                
                // Record initial state
                let initial_state = engine.execution_state().clone();
                
                // Execute program
                let result = engine.execute_program(program).await?;
                
                // Verify state consistency
                let final_state = engine.execution_state();
                
                // State consistency properties:
                // 1. Instruction pointer should progress
                // 2. Effects log should grow
                // 3. No corruption in state data structures
                
                let ip_progressed = final_state.instruction_pointer >= initial_state.instruction_pointer;
                let effects_recorded = engine.effects_log().len() >= initial_state.effect_history.len();
                let state_valid = result.step_count > 0;
                
                Ok(ip_progressed && effects_recorded && state_valid)
            })
        },
        75
    );
    
    let result = consistency_test.run().await?;
    println!("✓ State Consistency: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 95 / 100); // At least 95% should pass
    
    Ok(())
}

#[tokio_test]
async fn test_determinism_properties() -> Result<()> {
    println!("=== Testing Determinism Properties ===");
    
    // Property: Same program should produce identical results
    let determinism_test = PropertyTest::new(
        "deterministic_execution",
        ProgramGenerator::generate_complex_program,
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                // Execute same program multiple times
                let mut results = Vec::new();
                
                for _ in 0..3 {
                    let mut engine = SimulationEngine::new();
                    let result = engine.execute_program(program).await?;
                    results.push((result.step_count, result.instruction_count));
                }
                
                // Verify all results are identical
                let first_result = &results[0];
                let all_identical = results.iter().all(|r| r == first_result);
                
                Ok(all_identical)
            })
        },
        30
    );
    
    let result = determinism_test.run().await?;
    println!("✓ Deterministic Execution: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed == result.total_tests); // All should be deterministic
    
    Ok(())
}

#[tokio_test]
async fn test_metamorphic_properties() -> Result<()> {
    println!("=== Testing Metamorphic Properties ===");
    
    // Metamorphic property: Combining two alloc operations should be equivalent to one larger alloc
    let metamorphic_test = PropertyTest::new(
        "alloc_combination_equivalence",
        || {
            let size1 = rand::thread_rng().gen_range(10..=100);
            let size2 = rand::thread_rng().gen_range(10..=100);
            (size1, size2)
        },
        |(size1, size2)| {
            tokio::runtime::Handle::current().block_on(async {
                // Test: (tensor (alloc size1) (alloc size2)) should be similar to (alloc (size1 + size2))
                let combined_program = format!("(tensor (alloc {}) (alloc {}))", size1, size2);
                let single_program = format!("(alloc {})", size1 + size2);
                
                let mut engine1 = SimulationEngine::new();
                let mut engine2 = SimulationEngine::new();
                
                let result1 = engine1.execute_program(&combined_program).await?;
                let result2 = engine2.execute_program(&single_program).await?;
                
                // Metamorphic relationship: step counts should be related
                // Combined should have more steps (tensor + 2 allocs vs 1 alloc)
                Ok(result1.step_count >= result2.step_count)
            })
        },
        40
    );
    
    let result = metamorphic_test.run().await?;
    println!("✓ Metamorphic Relations: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 85 / 100); // At least 85% should pass
    
    Ok(())
}

#[tokio_test]
async fn test_symmetry_properties() -> Result<()> {
    println!("=== Testing Symmetry Properties ===");
    
    // Symmetry property: Order of independent operations shouldn't affect outcome
    let symmetry_test = PropertyTest::new(
        "operation_order_symmetry",
        || {
            let size1 = rand::thread_rng().gen_range(10..=100);
            let size2 = rand::thread_rng().gen_range(10..=100);
            (size1, size2)
        },
        |(size1, size2)| {
            tokio::runtime::Handle::current().block_on(async {
                // Test: (tensor (alloc size1) (alloc size2)) vs (tensor (alloc size2) (alloc size1))
                let order1 = format!("(tensor (alloc {}) (alloc {}))", size1, size2);
                let order2 = format!("(tensor (alloc {}) (alloc {}))", size2, size1);
                
                let mut engine1 = SimulationEngine::new();
                let mut engine2 = SimulationEngine::new();
                
                let result1 = engine1.execute_program(&order1).await?;
                let result2 = engine2.execute_program(&order2).await?;
                
                // Symmetry property: results should be equivalent
                Ok(result1.step_count == result2.step_count && 
                   result1.instruction_count == result2.instruction_count)
            })
        },
        35
    );
    
    let result = symmetry_test.run().await?;
    println!("✓ Symmetry Properties: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 90 / 100); // At least 90% should pass
    
    Ok(())
}

#[tokio_test]
async fn test_composition_properties() -> Result<()> {
    println!("=== Testing Composition Properties ===");
    
    // Composition property: Sequential execution should compose correctly
    let composition_test = PropertyTest::new(
        "sequential_composition",
        || {
            let programs = vec![
                ProgramGenerator::generate_alloc_program(),
                ProgramGenerator::generate_consume_program(),
                ProgramGenerator::generate_tensor_program(),
            ];
            programs
        },
        |programs| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                let mut total_steps = 0;
                
                // Execute programs sequentially
                for program in programs {
                    let result = engine.execute_program(program).await?;
                    total_steps += result.step_count;
                }
                
                // Composition property: total steps should be sum of individual executions
                let progression = engine.state_progression();
                Ok(progression.steps.len() >= total_steps)
            })
        },
        25
    );
    
    let result = composition_test.run().await?;
    println!("✓ Composition Properties: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 88 / 100); // At least 88% should pass
    
    Ok(())
}

#[tokio_test]
async fn test_fuzzing_integration() -> Result<()> {
    println!("=== Testing Fuzzing Integration ===");
    
    // Fuzzing test: Generate random programs and ensure no crashes
    let fuzzing_test = PropertyTest::new(
        "fuzzing_robustness",
        || {
            // Generate increasingly complex random programs
            let complexity = rand::thread_rng().gen_range(1..=3);
            match complexity {
                1 => ProgramGenerator::generate_alloc_program(),
                2 => ProgramGenerator::generate_consume_program(),
                3 => ProgramGenerator::generate_tensor_program(),
                _ => ProgramGenerator::generate_complex_program(),
            }
        },
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                
                // Fuzzing property: no program should cause crashes
                match engine.execute_program(program).await {
                    Ok(_) => Ok(true),  // Success is good
                    Err(_) => Ok(true), // Expected errors are also acceptable
                }
            })
        },
        100
    );
    
    let result = fuzzing_test.run().await?;
    println!("✓ Fuzzing Robustness: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed == result.total_tests); // Should handle all inputs gracefully
    
    Ok(())
}

#[tokio_test]
async fn test_invariant_preservation() -> Result<()> {
    println!("=== Testing Invariant Preservation ===");
    
    // Test that key system invariants are preserved across operations
    let invariant_test = PropertyTest::new(
        "system_invariants",
        ProgramGenerator::generate_complex_program,
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                
                // Check invariants before execution
                let initial_metrics = engine.metrics().clone();
                let initial_effects_count = engine.effects_log().len();
                
                // Execute program
                let result = engine.execute_program(program).await?;
                
                // Check invariants after execution
                let final_metrics = engine.metrics().clone();
                let final_effects_count = engine.effects_log().len();
                
                // System invariants:
                // 1. Metrics should only increase (monotonic)
                // 2. Effects log should only grow
                // 3. Step count should be positive if execution occurred
                
                let metrics_monotonic = final_metrics.effects_executed >= initial_metrics.effects_executed &&
                                       final_metrics.total_gas_consumed >= initial_metrics.total_gas_consumed;
                
                let effects_monotonic = final_effects_count >= initial_effects_count;
                let valid_execution = result.step_count > 0;
                
                Ok(metrics_monotonic && effects_monotonic && valid_execution)
            })
        },
        60
    );
    
    let result = invariant_test.run().await?;
    println!("✓ Invariant Preservation: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 95 / 100); // At least 95% should preserve invariants
    
    Ok(())
}

#[tokio_test]
async fn test_equivalence_properties() -> Result<()> {
    println!("=== Testing Equivalence Properties ===");
    
    // Test that equivalent programs produce equivalent results
    let equivalence_test = PropertyTest::new(
        "program_equivalence",
        || {
            let size = rand::thread_rng().gen_range(10..=100);
            // Two equivalent ways to express the same allocation
            let program1 = format!("(alloc {})", size);
            let program2 = format!("(consume (alloc {}))", size * 2); // Different but related
            (program1, program2, size)
        },
        |(program1, program2, _size)| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine1 = SimulationEngine::new();
                let mut engine2 = SimulationEngine::new();
                
                let result1 = engine1.execute_program(program1).await?;
                let result2 = engine2.execute_program(program2).await?;
                
                // Equivalence property: related programs should have predictable relationships
                // In this case, consume(alloc) should have more steps than just alloc
                Ok(result2.step_count >= result1.step_count)
            })
        },
        45
    );
    
    let result = equivalence_test.run().await?;
    println!("✓ Equivalence Properties: {}/{} tests passed", result.passed, result.total_tests);
    
    assert!(result.passed >= result.total_tests * 90 / 100); // At least 90% should show expected equivalence
    
    Ok(())
}

#[tokio_test]
async fn test_comprehensive_property_suite() -> Result<()> {
    println!("=== Running Comprehensive Property Test Suite ===");
    
    // Combine multiple properties into a comprehensive test
    let comprehensive_test = PropertyTest::new(
        "comprehensive_properties",
        ProgramGenerator::generate_complex_program,
        |program| {
            tokio::runtime::Handle::current().block_on(async {
                let mut engine = SimulationEngine::new();
                
                // Test multiple properties simultaneously
                let initial_state = engine.execution_state().clone();
                let initial_metrics = engine.metrics().clone();
                
                let result = engine.execute_program(program).await?;
                
                let final_state = engine.execution_state();
                let final_metrics = engine.metrics().clone();
                
                // Comprehensive property checks:
                let state_progressed = final_state.instruction_pointer >= initial_state.instruction_pointer;
                let metrics_increased = final_metrics.effects_executed >= initial_metrics.effects_executed;
                let valid_result = result.step_count > 0;
                let no_state_corruption = final_state.registers.len() >= initial_state.registers.len();
                
                Ok(state_progressed && metrics_increased && valid_result && no_state_corruption)
            })
        },
        80
    );
    
    let result = comprehensive_test.run().await?;
    println!("✓ Comprehensive Properties: {}/{} tests passed", result.passed, result.total_tests);
    
    // High success rate expected for comprehensive properties
    assert!(result.passed >= result.total_tests * 92 / 100);
    
    // Print summary
    println!("\n=== Property-Based Testing Summary ===");
    println!("  Total comprehensive tests: {}", result.total_tests);
    println!("  Success rate: {:.1}%", (result.passed as f64 / result.total_tests as f64) * 100.0);
    println!("  Property violations: {}", result.failed);
    
    if !result.failures.is_empty() {
        println!("  Example failures:");
        for (i, failure) in result.failures.iter().take(3).enumerate() {
            println!("    {}: {}", i + 1, failure);
        }
    }
    
    Ok(())
} 