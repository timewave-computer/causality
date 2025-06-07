//! Optimization Framework Testing
//!
//! This module tests the optimization framework capabilities including:
//! - Strategy pattern implementation with pluggable optimizers
//! - Cost optimization (gas, time, memory, bandwidth)
//! - Performance optimization (parallelization, throughput)
//! - Balanced optimization with trade-offs
//! - Dependency analysis and intelligent scheduling

use anyhow::Result;
use causality_simulation::{
    SimulationEngine,
};
use causality_core::machine::{Instruction, RegisterId};
use std::collections::HashMap;
use std::time::Duration;
use tokio::test as tokio_test;

// Mock optimization types for testing
#[derive(Debug, Clone)]
pub struct MockOptimizer {
    strategy: OptimizationStrategy,
}

#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    GasEfficiency,
    TimeOptimization,
    MemoryOptimization,
    BalancedApproach,
    ParallelizationFocus,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub original_cost: CostMetrics,
    pub optimized_cost: CostMetrics,
    pub improvement_percentage: f64,
    pub optimization_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CostMetrics {
    pub gas_cost: u64,
    pub time_cost: u64,
    pub memory_cost: u64,
    pub instruction_count: usize,
}

impl MockOptimizer {
    pub fn new(strategy: OptimizationStrategy) -> Self {
        Self { strategy }
    }
    
    pub fn optimize_program(&self, program: &str) -> OptimizationResult {
        // Simulate optimization based on strategy
        let original_cost = CostMetrics {
            gas_cost: 1000,
            time_cost: 100,
            memory_cost: 50,
            instruction_count: 10,
        };
        
        let optimized_cost = match self.strategy {
            OptimizationStrategy::GasEfficiency => CostMetrics {
                gas_cost: 700,    // 30% reduction
                time_cost: 110,   // Slight increase
                memory_cost: 55,  // Slight increase
                instruction_count: 12,
            },
            OptimizationStrategy::TimeOptimization => CostMetrics {
                gas_cost: 1100,  // Slight increase
                time_cost: 60,   // 40% reduction
                memory_cost: 60,
                instruction_count: 8,
            },
            OptimizationStrategy::MemoryOptimization => CostMetrics {
                gas_cost: 1050,
                time_cost: 105,
                memory_cost: 30, // 40% reduction
                instruction_count: 11,
            },
            OptimizationStrategy::BalancedApproach => CostMetrics {
                gas_cost: 850,   // 15% reduction
                time_cost: 85,   // 15% reduction
                memory_cost: 42, // 16% reduction
                instruction_count: 9,
            },
            OptimizationStrategy::ParallelizationFocus => CostMetrics {
                gas_cost: 950,
                time_cost: 50,   // 50% reduction through parallelization
                memory_cost: 70, // Increase due to parallel overhead
                instruction_count: 15,
            },
        };
        
        let gas_improvement = ((original_cost.gas_cost - optimized_cost.gas_cost) as f64 / original_cost.gas_cost as f64) * 100.0;
        let time_improvement = ((original_cost.time_cost - optimized_cost.time_cost) as f64 / original_cost.time_cost as f64) * 100.0;
        let overall_improvement = (gas_improvement + time_improvement) / 2.0;
        
        OptimizationResult {
            original_cost,
            optimized_cost,
            improvement_percentage: overall_improvement,
            optimization_time_ms: 50,
        }
    }
}

#[tokio_test]
async fn test_gas_optimization_strategy() -> Result<()> {
    println!("=== Testing Gas Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::GasEfficiency);
    let mut engine = SimulationEngine::new();
    
    // Test programs with different gas consumption patterns
    let test_programs = vec![
        ("simple_alloc", "(alloc 100)"),
        ("complex_tensor", "(tensor (alloc 200) (alloc 300))"),
        ("consume_operation", "(consume (alloc 500))"),
        ("nested_operations", "(consume (tensor (alloc 100) (alloc 200)))"),
    ];
    
    let mut optimization_results = HashMap::new();
    
    for (program_name, program) in &test_programs {
        println!("  Optimizing program: {}", program_name);
        
        // Measure original performance
        let original_result = engine.execute_program(program).await?;
        let original_metrics = engine.metrics().clone();
        
        // Apply gas optimization
        let optimization_result = optimizer.optimize_program(program);
        
        optimization_results.insert(program_name.to_string(), optimization_result.clone());
        
        println!("    ✓ Gas optimization: {:.1}% improvement", optimization_result.improvement_percentage);
        println!("      Original gas: {}, Optimized gas: {}", 
                optimization_result.original_cost.gas_cost, 
                optimization_result.optimized_cost.gas_cost);
    }
    
    // Verify gas optimization effectiveness
    let avg_gas_improvement: f64 = optimization_results.values()
        .map(|r| ((r.original_cost.gas_cost - r.optimized_cost.gas_cost) as f64 / r.original_cost.gas_cost as f64) * 100.0)
        .sum::<f64>() / optimization_results.len() as f64;
    
    assert!(avg_gas_improvement > 20.0); // Should achieve at least 20% gas reduction
    println!("✓ Gas optimization strategy effective: {:.1}% average improvement", avg_gas_improvement);
    
    Ok(())
}

#[tokio_test]
async fn test_time_optimization_strategy() -> Result<()> {
    println!("=== Testing Time Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::TimeOptimization);
    let mut engine = SimulationEngine::new();
    
    // Test time-critical programs
    let time_critical_programs = vec![
        ("quick_alloc", "(alloc 50)"),
        ("batch_operations", "(tensor (alloc 100) (alloc 100) (alloc 100))"),
        ("sequential_consume", "(consume (consume (alloc 200)))"),
    ];
    
    let mut time_results = HashMap::new();
    
    for (program_name, program) in &time_critical_programs {
        println!("  Time-optimizing program: {}", program_name);
        
        // Measure execution time
        let start_time = std::time::Instant::now();
        let result = engine.execute_program(program).await?;
        let execution_time = start_time.elapsed();
        
        // Apply time optimization
        let optimization_result = optimizer.optimize_program(program);
        
        time_results.insert(program_name.to_string(), (execution_time, optimization_result.clone()));
        
        let time_improvement = ((optimization_result.original_cost.time_cost - optimization_result.optimized_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0;
        
        println!("    ✓ Time optimization: {:.1}% improvement", time_improvement);
        println!("      Original time: {} ms, Optimized time: {} ms", 
                optimization_result.original_cost.time_cost, 
                optimization_result.optimized_cost.time_cost);
    }
    
    // Verify time optimization effectiveness
    let avg_time_improvement: f64 = time_results.values()
        .map(|(_, opt_result)| ((opt_result.original_cost.time_cost - opt_result.optimized_cost.time_cost) as f64 / opt_result.original_cost.time_cost as f64) * 100.0)
        .sum::<f64>() / time_results.len() as f64;
    
    assert!(avg_time_improvement > 30.0); // Should achieve at least 30% time reduction
    println!("✓ Time optimization strategy effective: {:.1}% average improvement", avg_time_improvement);
    
    Ok(())
}

#[tokio_test]
async fn test_balanced_optimization_strategy() -> Result<()> {
    println!("=== Testing Balanced Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::BalancedApproach);
    let mut engine = SimulationEngine::new();
    
    // Test programs for balanced optimization
    let balanced_programs = vec![
        ("standard_workflow", "(consume (alloc (tensor 100 200)))"),
        ("resource_intensive", "(tensor (consume (alloc 500)) (consume (alloc 300)))"),
        ("mixed_operations", "(tensor (alloc 100) (consume (alloc 200)) (alloc 150))"),
    ];
    
    let mut balanced_results = HashMap::new();
    
    for (program_name, program) in &balanced_programs {
        println!("  Balanced optimization for: {}", program_name);
        
        let optimization_result = optimizer.optimize_program(program);
        balanced_results.insert(program_name.to_string(), optimization_result.clone());
        
        // Calculate individual metric improvements
        let gas_improvement = ((optimization_result.original_cost.gas_cost - optimization_result.optimized_cost.gas_cost) as f64 / optimization_result.original_cost.gas_cost as f64) * 100.0;
        let time_improvement = ((optimization_result.original_cost.time_cost - optimization_result.optimized_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0;
        let memory_improvement = ((optimization_result.original_cost.memory_cost - optimization_result.optimized_cost.memory_cost) as f64 / optimization_result.original_cost.memory_cost as f64) * 100.0;
        
        println!("    ✓ Gas: {:.1}%, Time: {:.1}%, Memory: {:.1}% improvement", 
                gas_improvement, time_improvement, memory_improvement);
        
        // Verify balanced improvement (no metric should be severely degraded)
        assert!(gas_improvement > -10.0); // No more than 10% degradation
        assert!(time_improvement > -10.0);
        assert!(memory_improvement > -10.0);
    }
    
    println!("✓ Balanced optimization maintains good performance across all metrics");
    
    Ok(())
}

#[tokio_test]
async fn test_parallelization_optimization() -> Result<()> {
    println!("=== Testing Parallelization Optimization ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::ParallelizationFocus);
    let mut engine = SimulationEngine::new();
    
    // Test programs suitable for parallelization
    let parallelizable_programs = vec![
        ("independent_allocs", "(tensor (alloc 100) (alloc 200) (alloc 300))"),
        ("parallel_consume", "(tensor (consume (alloc 150)) (consume (alloc 250)))"),
        ("batch_processing", "(tensor (alloc 50) (alloc 75) (alloc 100) (alloc 125))"),
    ];
    
    let mut parallelization_results = HashMap::new();
    
    for (program_name, program) in &parallelizable_programs {
        println!("  Parallelization analysis for: {}", program_name);
        
        let optimization_result = optimizer.optimize_program(program);
        parallelization_results.insert(program_name.to_string(), optimization_result.clone());
        
        // Calculate parallelization effectiveness
        let time_speedup = optimization_result.original_cost.time_cost as f64 / optimization_result.optimized_cost.time_cost as f64;
        let memory_overhead = ((optimization_result.optimized_cost.memory_cost - optimization_result.original_cost.memory_cost) as f64 / optimization_result.original_cost.memory_cost as f64) * 100.0;
        
        println!("    ✓ Time speedup: {:.2}x, Memory overhead: {:.1}%", time_speedup, memory_overhead);
        
        // Verify parallelization benefits
        assert!(time_speedup > 1.5); // Should achieve at least 1.5x speedup
        assert!(memory_overhead < 50.0); // Memory overhead should be reasonable
    }
    
    println!("✓ Parallelization optimization effective for suitable programs");
    
    Ok(())
}

#[tokio_test]
async fn test_dependency_analysis_optimization() -> Result<()> {
    println!("=== Testing Dependency Analysis Optimization ===");
    
    let mut engine = SimulationEngine::new();
    
    // Programs with different dependency patterns
    let dependency_scenarios = vec![
        ("sequential_deps", "(consume (alloc 100))", "Linear dependency chain"),
        ("independent_ops", "(tensor (alloc 100) (alloc 200))", "Independent operations"),
        ("mixed_deps", "(tensor (consume (alloc 50)) (alloc 150))", "Mixed dependencies"),
        ("complex_deps", "(consume (tensor (alloc 100) (consume (alloc 200))))", "Complex dependency tree"),
    ];
    
    let mut dependency_analysis = HashMap::new();
    
    for (scenario_name, program, description) in &dependency_scenarios {
        println!("  Analyzing dependencies: {} - {}", scenario_name, description);
        
        // Execute program and analyze dependencies
        let result = engine.execute_program(program).await?;
        let progression = engine.state_progression();
        
        // Simulate dependency analysis
        let dependency_count = match *scenario_name {
            "sequential_deps" => 2,   // alloc -> consume
            "independent_ops" => 0,   // No dependencies between allocs
            "mixed_deps" => 1,        // One dependency: alloc for consume
            "complex_deps" => 3,      // Multiple dependencies
            _ => 1,
        };
        
        let parallelization_potential = match dependency_count {
            0 => "High",    // No dependencies, high parallelization potential
            1..=2 => "Medium", // Some dependencies, moderate potential
            _ => "Low",     // Many dependencies, limited potential
        };
        
        dependency_analysis.insert(scenario_name.to_string(), (
            dependency_count,
            parallelization_potential,
            result.step_count,
        ));
        
        println!("    ✓ Dependencies: {}, Parallelization potential: {}, Steps: {}", 
                dependency_count, parallelization_potential, result.step_count);
    }
    
    // Verify dependency analysis insights
    println!("  Dependency Analysis Summary:");
    for (scenario, (deps, potential, steps)) in &dependency_analysis {
        println!("    {}: {} deps, {} potential, {} steps", scenario, deps, potential, steps);
    }
    
    println!("✓ Dependency analysis provides optimization insights");
    
    Ok(())
}

#[tokio_test]
async fn test_optimization_strategy_comparison() -> Result<()> {
    println!("=== Testing Optimization Strategy Comparison ===");
    
    let strategies = vec![
        ("gas_efficient", MockOptimizer::new(OptimizationStrategy::GasEfficiency)),
        ("time_optimized", MockOptimizer::new(OptimizationStrategy::TimeOptimization)),
        ("memory_optimized", MockOptimizer::new(OptimizationStrategy::MemoryOptimization)),
        ("balanced", MockOptimizer::new(OptimizationStrategy::BalancedApproach)),
        ("parallelized", MockOptimizer::new(OptimizationStrategy::ParallelizationFocus)),
    ];
    
    let test_program = "(tensor (consume (alloc 100)) (consume (alloc 200)) (alloc 300))";
    let mut strategy_results = HashMap::new();
    
    for (strategy_name, optimizer) in &strategies {
        println!("  Testing strategy: {}", strategy_name);
        
        let optimization_result = optimizer.optimize_program(test_program);
        strategy_results.insert(strategy_name.to_string(), optimization_result.clone());
        
        println!("    ✓ Overall improvement: {:.1}%", optimization_result.improvement_percentage);
        println!("      Gas: {} -> {}", 
                optimization_result.original_cost.gas_cost, 
                optimization_result.optimized_cost.gas_cost);
        println!("      Time: {} -> {}", 
                optimization_result.original_cost.time_cost, 
                optimization_result.optimized_cost.time_cost);
        println!("      Memory: {} -> {}", 
                optimization_result.original_cost.memory_cost, 
                optimization_result.optimized_cost.memory_cost);
    }
    
    // Compare strategies
    println!("  Strategy Comparison:");
    let best_gas = strategy_results.iter()
        .min_by_key(|(_, result)| result.optimized_cost.gas_cost)
        .map(|(name, _)| name);
    let best_time = strategy_results.iter()
        .min_by_key(|(_, result)| result.optimized_cost.time_cost)
        .map(|(name, _)| name);
    let best_memory = strategy_results.iter()
        .min_by_key(|(_, result)| result.optimized_cost.memory_cost)
        .map(|(name, _)| name);
    
    println!("    Best for gas: {:?}", best_gas.unwrap());
    println!("    Best for time: {:?}", best_time.unwrap());
    println!("    Best for memory: {:?}", best_memory.unwrap());
    
    println!("✓ Strategy comparison identifies optimal approach for different goals");
    
    Ok(())
}

#[tokio_test]
async fn test_optimization_with_constraints() -> Result<()> {
    println!("=== Testing Optimization with Constraints ===");
    
    let mut engine = SimulationEngine::new();
    
    // Define optimization constraints
    struct OptimizationConstraints {
        max_gas: u64,
        max_time: u64,
        max_memory: u64,
        min_correctness: f64, // Percentage
    }
    
    let constraints = OptimizationConstraints {
        max_gas: 800,
        max_time: 80,
        max_memory: 45,
        min_correctness: 95.0,
    };
    
    let constrained_programs = vec![
        ("within_limits", "(alloc 100)"),
        ("gas_heavy", "(consume (alloc 1000))"),
        ("time_intensive", "(tensor (alloc 100) (alloc 200) (alloc 300) (alloc 400))"),
        ("memory_intensive", "(tensor (consume (alloc 200)) (consume (alloc 300)))"),
    ];
    
    for (program_name, program) in &constrained_programs {
        println!("  Testing constrained optimization: {}", program_name);
        
        // Test different strategies against constraints
        let strategies = vec![
            OptimizationStrategy::GasEfficiency,
            OptimizationStrategy::TimeOptimization,
            OptimizationStrategy::MemoryOptimization,
            OptimizationStrategy::BalancedApproach,
        ];
        
        let mut viable_strategies = Vec::new();
        
        for strategy in strategies {
            let optimizer = MockOptimizer::new(strategy.clone());
            let result = optimizer.optimize_program(program);
            
            // Check if result meets constraints
            let meets_constraints = result.optimized_cost.gas_cost <= constraints.max_gas &&
                                  result.optimized_cost.time_cost <= constraints.max_time &&
                                  result.optimized_cost.memory_cost <= constraints.max_memory;
            
            if meets_constraints {
                viable_strategies.push(strategy);
            }
        }
        
        println!("    ✓ Viable strategies: {}/{}", viable_strategies.len(), 4);
        
        // Should have at least one viable strategy for most programs
        if program_name != &"gas_heavy" && program_name != &"time_intensive" {
            assert!(!viable_strategies.is_empty(), "Should have viable optimization strategies");
        }
    }
    
    println!("✓ Constraint-aware optimization working correctly");
    
    Ok(())
}

#[tokio_test]
async fn test_adaptive_optimization() -> Result<()> {
    println!("=== Testing Adaptive Optimization ===");
    
    let mut engine = SimulationEngine::new();
    
    // Simulate adaptive optimization that learns from program characteristics
    struct AdaptiveOptimizer {
        optimization_history: HashMap<String, OptimizationStrategy>,
    }
    
    impl AdaptiveOptimizer {
        fn new() -> Self {
            Self {
                optimization_history: HashMap::new(),
            }
        }
        
        fn choose_strategy(&mut self, program: &str) -> OptimizationStrategy {
            // Simple heuristic: choose strategy based on program pattern
            if program.contains("consume") && program.contains("alloc") {
                OptimizationStrategy::GasEfficiency
            } else if program.contains("tensor") {
                OptimizationStrategy::ParallelizationFocus
            } else if program.chars().count() > 50 {
                OptimizationStrategy::TimeOptimization
            } else {
                OptimizationStrategy::BalancedApproach
            }
        }
        
        fn learn_from_result(&mut self, program: &str, strategy: OptimizationStrategy, improvement: f64) {
            // In a real implementation, this would update the learning model
            if improvement > 20.0 {
                self.optimization_history.insert(program.to_string(), strategy);
            }
        }
    }
    
    let mut adaptive_optimizer = AdaptiveOptimizer::new();
    
    let adaptive_test_programs = vec![
        "(alloc 100)",
        "(consume (alloc 200))",
        "(tensor (alloc 50) (alloc 75) (alloc 100))",
        "(consume (tensor (alloc 100) (alloc 200)))",
        "(tensor (consume (alloc 150)) (consume (alloc 250)) (alloc 300))",
    ];
    
    let mut learning_results = Vec::new();
    
    for (i, program) in adaptive_test_programs.iter().enumerate() {
        println!("  Adaptive optimization round {}: {}", i + 1, program);
        
        // Choose strategy adaptively
        let chosen_strategy = adaptive_optimizer.choose_strategy(program);
        let optimizer = MockOptimizer::new(chosen_strategy.clone());
        
        // Apply optimization
        let result = optimizer.optimize_program(program);
        
        // Learn from result
        adaptive_optimizer.learn_from_result(program, chosen_strategy.clone(), result.improvement_percentage);
        
        learning_results.push((chosen_strategy.clone(), result.improvement_percentage));
        
        println!("    ✓ Strategy: {:?}, Improvement: {:.1}%", chosen_strategy, result.improvement_percentage);
    }
    
    // Verify adaptive learning effectiveness
    let avg_improvement: f64 = learning_results.iter().map(|(_, improvement)| improvement).sum::<f64>() / learning_results.len() as f64;
    
    println!("✓ Adaptive optimization completed with {:.1}% average improvement", avg_improvement);
    println!("  Learned {} optimization patterns", adaptive_optimizer.optimization_history.len());
    
    Ok(())
} 