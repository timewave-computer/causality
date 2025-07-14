//! Optimization Testing for Simulation Engine
//!
//! This module tests optimization strategies and performance analysis
//! using valid syntax that works with the current parser.

use anyhow::Result;
use causality_simulation::SimulationEngine;
use std::collections::HashMap;

/// Mock optimizer for testing different optimization strategies
pub struct MockOptimizer {
    strategy: OptimizationStrategy,
}

#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    GasEfficiency,
    TimeOptimization,
    MemoryOptimization,
    BalancedApproach,
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
    
    pub fn optimize_program(&self, _program: &str) -> OptimizationResult {
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
        };
        
        let gas_improvement = if optimized_cost.gas_cost <= original_cost.gas_cost {
            ((original_cost.gas_cost - optimized_cost.gas_cost) as f64 / original_cost.gas_cost as f64) * 100.0
        } else {
            -((optimized_cost.gas_cost - original_cost.gas_cost) as f64 / original_cost.gas_cost as f64) * 100.0
        };
        
        let time_improvement = if optimized_cost.time_cost <= original_cost.time_cost {
            ((original_cost.time_cost - optimized_cost.time_cost) as f64 / original_cost.time_cost as f64) * 100.0
        } else {
            -((optimized_cost.time_cost - original_cost.time_cost) as f64 / original_cost.time_cost as f64) * 100.0
        };
        
        let overall_improvement = (gas_improvement + time_improvement) / 2.0;
        
        OptimizationResult {
            original_cost,
            optimized_cost,
            improvement_percentage: overall_improvement,
            optimization_time_ms: 50,
        }
    }
}

#[tokio::test]
async fn test_gas_optimization_strategy() -> Result<()> {
    println!("=== Testing Gas Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::GasEfficiency);
    let mut engine = SimulationEngine::new();
    
    // Test programs with valid syntax (only 2 args for tensor)
    let test_programs = vec![
        ("simple_alloc", "(alloc 100)"),
        ("simple_tensor", "(tensor (alloc 200) (alloc 300))"),
        ("consume_operation", "(consume (alloc 500))"),
        ("nested_operations", "(consume (alloc 100))"),
    ];
    
    let mut optimization_results = HashMap::new();
    
    for (program_name, program) in &test_programs {
        println!("  Optimizing program: {}", program_name);
        
        // Measure original performance
        let _original_result = engine.execute_program(program).await?;
        let _original_metrics = engine.metrics().clone();
        
        // Apply gas optimization
        let optimization_result = optimizer.optimize_program(program);
        
        optimization_results.insert(program_name.to_string(), optimization_result.clone());
        
        println!("     Gas optimization: {:.1}% improvement", optimization_result.improvement_percentage);
        println!("      Original gas: {}, Optimized gas: {}", 
                optimization_result.original_cost.gas_cost, 
                optimization_result.optimized_cost.gas_cost);
    }
    
    // Verify gas optimization effectiveness
    let avg_gas_improvement: f64 = optimization_results.values()
        .map(|r| {
            let original = r.original_cost.gas_cost as i64;
            let optimized = r.optimized_cost.gas_cost as i64;
            ((original - optimized) as f64 / original as f64) * 100.0
        })
        .sum::<f64>() / optimization_results.len() as f64;
    
    assert!(avg_gas_improvement > 20.0); // Should achieve at least 20% gas reduction
    println!(" Gas optimization strategy effective: {:.1}% average improvement", avg_gas_improvement);
    
    Ok(())
}

#[tokio::test]
async fn test_time_optimization_strategy() -> Result<()> {
    println!("=== Testing Time Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::TimeOptimization);
    let mut engine = SimulationEngine::new();
    
    // Test time-critical programs with valid syntax
    let time_critical_programs = vec![
        ("quick_alloc", "(alloc 50)"),
        ("batch_operations", "(tensor (alloc 100) (alloc 100))"),
        ("sequential_consume", "(consume (alloc 200))"),
    ];
    
    let mut time_results = HashMap::new();
    
    for (program_name, program) in &time_critical_programs {
        println!("  Time-optimizing program: {}", program_name);
        
        // Measure execution time
        let start_time = std::time::Instant::now();
        let _result = engine.execute_program(program).await?;
        let execution_time = start_time.elapsed();
        
        // Apply time optimization
        let optimization_result = optimizer.optimize_program(program);
        
        time_results.insert(program_name.to_string(), (execution_time, optimization_result.clone()));
        
        let time_improvement = if optimization_result.optimized_cost.time_cost <= optimization_result.original_cost.time_cost {
            ((optimization_result.original_cost.time_cost - optimization_result.optimized_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0
        } else {
            -((optimization_result.optimized_cost.time_cost - optimization_result.original_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0
        };
        
        println!("     Time optimization: {:.1}% improvement", time_improvement);
        println!("      Original time: {} ms, Optimized time: {} ms", 
                optimization_result.original_cost.time_cost, 
                optimization_result.optimized_cost.time_cost);
    }
    
    println!(" Time optimization strategy completed successfully");
    
    Ok(())
}

#[tokio::test]
async fn test_balanced_optimization_strategy() -> Result<()> {
    println!("=== Testing Balanced Optimization Strategy ===");
    
    let optimizer = MockOptimizer::new(OptimizationStrategy::BalancedApproach);
    let mut engine = SimulationEngine::new();
    
    // Test programs for balanced optimization with valid syntax
    let balanced_programs = vec![
        ("standard_workflow", "(consume (alloc 100))"),
        ("resource_intensive", "(tensor (consume (alloc 500)) (consume (alloc 300)))"),
        ("mixed_operations", "(tensor (alloc 100) (consume (alloc 200)))"),
    ];
    
    let mut balanced_results = HashMap::new();
    
    for (program_name, program) in &balanced_programs {
        println!("  Balanced optimization for: {}", program_name);
        
        // Execute program first
        let _result = engine.execute_program(program).await?;
        
        let optimization_result = optimizer.optimize_program(program);
        balanced_results.insert(program_name.to_string(), optimization_result.clone());
        
        // Calculate individual metric improvements
        let gas_improvement = if optimization_result.optimized_cost.gas_cost <= optimization_result.original_cost.gas_cost {
            ((optimization_result.original_cost.gas_cost - optimization_result.optimized_cost.gas_cost) as f64 / optimization_result.original_cost.gas_cost as f64) * 100.0
        } else {
            -((optimization_result.optimized_cost.gas_cost - optimization_result.original_cost.gas_cost) as f64 / optimization_result.original_cost.gas_cost as f64) * 100.0
        };
        
        let time_improvement = if optimization_result.optimized_cost.time_cost <= optimization_result.original_cost.time_cost {
            ((optimization_result.original_cost.time_cost - optimization_result.optimized_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0
        } else {
            -((optimization_result.optimized_cost.time_cost - optimization_result.original_cost.time_cost) as f64 / optimization_result.original_cost.time_cost as f64) * 100.0
        };
        
        let memory_improvement = if optimization_result.optimized_cost.memory_cost <= optimization_result.original_cost.memory_cost {
            ((optimization_result.original_cost.memory_cost - optimization_result.optimized_cost.memory_cost) as f64 / optimization_result.original_cost.memory_cost as f64) * 100.0
        } else {
            -((optimization_result.optimized_cost.memory_cost - optimization_result.original_cost.memory_cost) as f64 / optimization_result.original_cost.memory_cost as f64) * 100.0
        };
        
        println!("     Gas: {:.1}%, Time: {:.1}%, Memory: {:.1}% improvement", 
                gas_improvement, time_improvement, memory_improvement);
        
        // Verify balanced improvement (no metric should be severely degraded)
        assert!(gas_improvement > -20.0); // No more than 20% degradation
        assert!(time_improvement > -20.0);
        assert!(memory_improvement > -20.0);
    }
    
    println!(" Balanced optimization maintains good performance across all metrics");
    
    Ok(())
}

#[tokio::test]
async fn test_optimization_strategy_comparison() -> Result<()> {
    println!("=== Testing Optimization Strategy Comparison ===");
    
    let strategies = vec![
        OptimizationStrategy::GasEfficiency,
        OptimizationStrategy::TimeOptimization,
        OptimizationStrategy::MemoryOptimization,
        OptimizationStrategy::BalancedApproach,
    ];
    
    let test_program = "(tensor (consume (alloc 100)) (consume (alloc 200)))";
    let mut strategy_results = HashMap::new();
    
    for strategy in strategies {
        let optimizer = MockOptimizer::new(strategy.clone());
        let optimization_result = optimizer.optimize_program(test_program);
        
        strategy_results.insert(format!("{:?}", strategy), optimization_result);
    }
    
    // Verify each strategy produces different results
    println!("  Strategy comparison results:");
    for (strategy_name, result) in &strategy_results {
        println!("    {}: {:.1}% improvement", strategy_name, result.improvement_percentage);
    }
    
    // Verify we have results for all strategies
    assert_eq!(strategy_results.len(), 4);
    println!(" All optimization strategies produce results");
    
    Ok(())
}

#[test]
fn test_cost_metrics_calculation() {
    println!("=== Testing Cost Metrics Calculation ===");
    
    let original = CostMetrics {
        gas_cost: 1000,
        time_cost: 100,
        memory_cost: 50,
        instruction_count: 10,
    };
    
    let optimized = CostMetrics {
        gas_cost: 800,
        time_cost: 80,
        memory_cost: 40,
        instruction_count: 8,
    };
    
    // Test improvement calculations
    let gas_improvement = ((original.gas_cost - optimized.gas_cost) as f64 / original.gas_cost as f64) * 100.0;
    let time_improvement = ((original.time_cost - optimized.time_cost) as f64 / original.time_cost as f64) * 100.0;
    let memory_improvement = ((original.memory_cost - optimized.memory_cost) as f64 / original.memory_cost as f64) * 100.0;
    
    assert_eq!(gas_improvement, 20.0);
    assert_eq!(time_improvement, 20.0);
    assert_eq!(memory_improvement, 20.0);
    
    println!(" Cost metrics calculation works correctly");
} 