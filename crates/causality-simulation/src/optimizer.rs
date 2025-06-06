//! Effect execution optimizer for simulation performance
//!
//! This module provides optimization strategies for reordering and scheduling
//! effects to minimize execution cost and maximize parallelization opportunities.

use std::collections::HashMap;
use crate::engine::MockEffect;
use causality_core::machine::instruction::RegisterId;

/// Cost metric for effect execution
#[derive(Debug, Clone, PartialEq)]
pub struct EffectCost {
    /// Gas cost for execution
    pub gas_cost: u64,
    
    /// Time cost in simulated milliseconds
    pub time_cost: u64,
    
    /// Memory usage in bytes
    pub memory_cost: u64,
    
    /// Network bandwidth usage
    pub bandwidth_cost: u64,
}

impl EffectCost {
    /// Create a new effect cost
    pub fn new(gas: u64, time: u64, memory: u64, bandwidth: u64) -> Self {
        Self {
            gas_cost: gas,
            time_cost: time,
            memory_cost: memory,
            bandwidth_cost: bandwidth,
        }
    }
    
    /// Calculate total weighted cost
    pub fn total_cost(&self, weights: &CostWeights) -> f64 {
        (self.gas_cost as f64 * weights.gas_weight) +
        (self.time_cost as f64 * weights.time_weight) +
        (self.memory_cost as f64 * weights.memory_weight) +
        (self.bandwidth_cost as f64 * weights.bandwidth_weight)
    }
}

/// Weights for different cost components
#[derive(Debug, Clone, PartialEq)]
pub struct CostWeights {
    pub gas_weight: f64,
    pub time_weight: f64,
    pub memory_weight: f64,
    pub bandwidth_weight: f64,
}

impl Default for CostWeights {
    fn default() -> Self {
        Self {
            gas_weight: 1.0,
            time_weight: 0.8,
            memory_weight: 0.5,
            bandwidth_weight: 0.3,
        }
    }
}

/// Effect with optimization metadata
#[derive(Debug, Clone)]
pub struct OptimizableEffect {
    /// The effect to execute
    pub effect: MockEffect,
    
    /// Cost metrics for this effect
    pub cost: EffectCost,
    
    /// Dependencies on other effects
    pub dependencies: Vec<usize>,
    
    /// Priority level (higher = more important)
    pub priority: u32,
    
    /// Whether this effect can be parallelized
    pub parallelizable: bool,
}

/// Optimization strategy for effect scheduling
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Minimize total gas cost
    MinimizeGasCost,
    
    /// Minimize total execution time
    MinimizeTime,
    
    /// Maximize parallelization
    MaximizeParallelism,
    
    /// Balance all factors
    Balanced,
    
    /// Custom weighted optimization
    Custom(CostWeights),
}

/// Configuration for the optimizer
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Optimization strategy to use
    pub strategy: OptimizationStrategy,
    
    /// Maximum number of parallel effects
    pub max_parallel_effects: usize,
    
    /// Whether to enable dependency analysis
    pub enable_dependency_analysis: bool,
    
    /// Whether to enable cost prediction
    pub enable_cost_prediction: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            strategy: OptimizationStrategy::Balanced,
            max_parallel_effects: 4,
            enable_dependency_analysis: true,
            enable_cost_prediction: true,
        }
    }
}

/// Effect execution optimizer
pub struct EffectOptimizer {
    /// Configuration for optimization
    config: OptimizerConfig,
    
    /// Cost database for effect types
    cost_database: HashMap<String, EffectCost>,
    
    /// Statistics from previous optimizations
    optimization_stats: OptimizationStats,
}

/// Statistics about optimization performance
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub total_optimizations: usize,
    pub total_effects_optimized: usize,
    pub average_cost_reduction: f64,
    pub average_time_reduction: f64,
}

/// Result of effect optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Optimized effect execution order
    pub execution_order: Vec<usize>,
    
    /// Parallel execution batches
    pub parallel_batches: Vec<Vec<usize>>,
    
    /// Estimated cost savings
    pub cost_savings: EffectCost,
    
    /// Optimization strategy used
    pub strategy_used: OptimizationStrategy,
}

impl EffectOptimizer {
    /// Create a new optimizer with default configuration
    pub fn new() -> Self {
        Self {
            config: OptimizerConfig::default(),
            cost_database: Self::create_default_cost_database(),
            optimization_stats: OptimizationStats::default(),
        }
    }

    /// Create an optimizer with custom configuration
    pub fn with_config(config: OptimizerConfig) -> Self {
        Self {
            config,
            cost_database: Self::create_default_cost_database(),
            optimization_stats: OptimizationStats::default(),
        }
    }
    
    /// Set the optimization strategy
    pub fn set_strategy(&mut self, strategy: OptimizationStrategy) {
        self.config.strategy = strategy;
    }
    
    /// Optimize a list of effects for execution
    pub fn optimize_effects(&mut self, effects: Vec<OptimizableEffect>) -> OptimizationResult {
        let original_cost = self.calculate_total_cost(&effects);
        
        // Apply optimization strategy
        let (execution_order, parallel_batches) = match &self.config.strategy {
            OptimizationStrategy::MinimizeGasCost => self.optimize_for_gas_cost(&effects),
            OptimizationStrategy::MinimizeTime => self.optimize_for_time(&effects),
            OptimizationStrategy::MaximizeParallelism => self.optimize_for_parallelism(&effects),
            OptimizationStrategy::Balanced => self.optimize_balanced(&effects),
            OptimizationStrategy::Custom(weights) => self.optimize_custom(&effects, weights),
        };
        
        // Calculate cost savings
        let optimized_cost = self.calculate_optimized_cost(&effects, &execution_order);
        let cost_savings = EffectCost {
            gas_cost: original_cost.gas_cost.saturating_sub(optimized_cost.gas_cost),
            time_cost: original_cost.time_cost.saturating_sub(optimized_cost.time_cost),
            memory_cost: original_cost.memory_cost.saturating_sub(optimized_cost.memory_cost),
            bandwidth_cost: original_cost.bandwidth_cost.saturating_sub(optimized_cost.bandwidth_cost),
        };
        
        // Update statistics
        self.optimization_stats.total_optimizations += 1;
        self.optimization_stats.total_effects_optimized += effects.len();
        
        OptimizationResult {
            execution_order,
            parallel_batches,
            cost_savings,
            strategy_used: self.config.strategy.clone(),
        }
    }
    
    /// Estimate cost for an effect based on its type
    pub fn estimate_effect_cost(&self, effect_tag: &str) -> EffectCost {
        self.cost_database.get(effect_tag)
            .cloned()
            .unwrap_or_else(|| self.get_default_cost())
    }
    
    /// Add or update cost information for an effect type
    pub fn update_cost_database(&mut self, effect_tag: String, cost: EffectCost) {
        self.cost_database.insert(effect_tag, cost);
    }
    
    /// Get optimization statistics
    pub fn get_statistics(&self) -> &OptimizationStats {
        &self.optimization_stats
    }
    
    /// Create default cost database with common effect types
    fn create_default_cost_database() -> HashMap<String, EffectCost> {
        let mut db = HashMap::new();
        
        // Basic computation effects
        db.insert("compute".to_string(), EffectCost::new(10, 50, 1024, 0));
        db.insert("hash".to_string(), EffectCost::new(5, 20, 512, 0));
        db.insert("sort".to_string(), EffectCost::new(15, 100, 2048, 0));
        
        // Storage effects
        db.insert("storage_read".to_string(), EffectCost::new(3, 10, 256, 0));
        db.insert("storage_write".to_string(), EffectCost::new(5, 20, 512, 0));
        db.insert("storage_delete".to_string(), EffectCost::new(2, 5, 128, 0));
        
        // Network effects
        db.insert("network_request".to_string(), EffectCost::new(8, 200, 1024, 1024));
        db.insert("network_upload".to_string(), EffectCost::new(12, 300, 2048, 2048));
        db.insert("network_download".to_string(), EffectCost::new(6, 150, 1024, 4096));
        
        // Transfer effects
        let transfer_effect = MockEffect {
            call: crate::engine::MockEffectCall {
                tag: "transfer".to_string(),
                args: vec!["sender".to_string(), "receiver".to_string(), "100".to_string()],
                return_type: Some("TransferReceipt".to_string()),
            },
            result_register: Some(causality_core::RegisterId(0)),
        };
        db.insert("transfer".to_string(), EffectCost::new(20, 100, 512, 256));
        db.insert("mint".to_string(), EffectCost::new(25, 80, 768, 128));
        db.insert("burn".to_string(), EffectCost::new(15, 60, 512, 128));
        
        // Validation effects
        db.insert("validation".to_string(), EffectCost::new(30, 150, 1024, 512));
        db.insert("signature_verify".to_string(), EffectCost::new(40, 200, 2048, 256));
        
        db
    }
    
    /// Get default cost for unknown effect types
    fn get_default_cost(&self) -> EffectCost {
        EffectCost::new(10, 50, 1024, 256) // Default moderate cost
    }
    
    /// Calculate total cost for a list of effects
    fn calculate_total_cost(&self, effects: &[OptimizableEffect]) -> EffectCost {
        effects.iter().fold(EffectCost::new(0, 0, 0, 0), |acc, effect| {
            EffectCost {
                gas_cost: acc.gas_cost + effect.cost.gas_cost,
                time_cost: acc.time_cost + effect.cost.time_cost,
                memory_cost: acc.memory_cost + effect.cost.memory_cost,
                bandwidth_cost: acc.bandwidth_cost + effect.cost.bandwidth_cost,
            }
        })
    }
    
    /// Calculate cost for optimized execution order
    fn calculate_optimized_cost(&self, effects: &[OptimizableEffect], order: &[usize]) -> EffectCost {
        // For sequential execution, cost is just the sum
        // For parallel execution, we could model overlapping costs
        order.iter().fold(EffectCost::new(0, 0, 0, 0), |acc, &idx| {
            let effect = &effects[idx];
            EffectCost {
                gas_cost: acc.gas_cost + effect.cost.gas_cost,
                time_cost: acc.time_cost + effect.cost.time_cost,
                memory_cost: acc.memory_cost.max(effect.cost.memory_cost), // Peak memory
                bandwidth_cost: acc.bandwidth_cost + effect.cost.bandwidth_cost,
            }
        })
    }
    
    /// Optimize for minimum gas cost
    fn optimize_for_gas_cost(&self, effects: &[OptimizableEffect]) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut indices: Vec<usize> = (0..effects.len()).collect();
        indices.sort_by_key(|&i| effects[i].cost.gas_cost);
        
        // Simple sequential execution for gas optimization
        let batches = indices.iter().map(|&i| vec![i]).collect();
        (indices, batches)
    }
    
    /// Optimize for minimum execution time
    fn optimize_for_time(&self, effects: &[OptimizableEffect]) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut indices: Vec<usize> = (0..effects.len()).collect();
        indices.sort_by_key(|&i| effects[i].cost.time_cost);
        
        // Group fast effects together for parallel execution
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        
        for &idx in &indices {
            if current_batch.len() < self.config.max_parallel_effects && 
               effects[idx].parallelizable {
                current_batch.push(idx);
            } else {
                if !current_batch.is_empty() {
                    batches.push(current_batch);
                    current_batch = Vec::new();
                }
                current_batch.push(idx);
            }
        }
        
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        
        (indices, batches)
    }
    
    /// Optimize for maximum parallelization
    fn optimize_for_parallelism(&self, effects: &[OptimizableEffect]) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut parallel_effects = Vec::new();
        let mut sequential_effects = Vec::new();
        
        for (i, effect) in effects.iter().enumerate() {
            if effect.parallelizable {
                parallel_effects.push(i);
            } else {
                sequential_effects.push(i);
            }
        }
        
        let mut batches = Vec::new();
        
        // Create batches of parallel effects
        for chunk in parallel_effects.chunks(self.config.max_parallel_effects) {
            batches.push(chunk.to_vec());
        }
        
        // Add sequential effects as individual batches
        for &idx in &sequential_effects {
            batches.push(vec![idx]);
        }
        
        let execution_order = parallel_effects.into_iter()
            .chain(sequential_effects.into_iter())
            .collect();
        
        (execution_order, batches)
    }
    
    /// Optimize with balanced approach
    fn optimize_balanced(&self, effects: &[OptimizableEffect]) -> (Vec<usize>, Vec<Vec<usize>>) {
        let weights = CostWeights::default();
        self.optimize_custom(effects, &weights)
    }
    
    /// Optimize with custom weights
    fn optimize_custom(&self, effects: &[OptimizableEffect], weights: &CostWeights) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut indices: Vec<usize> = (0..effects.len()).collect();
        
        // Sort by weighted total cost
        indices.sort_by(|&a, &b| {
            let cost_a = effects[a].cost.total_cost(weights);
            let cost_b = effects[b].cost.total_cost(weights);
            cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Create balanced batches considering both cost and parallelization
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_batch_cost = 0.0;
        let max_batch_cost = 100.0; // Arbitrary threshold
        
        for &idx in &indices {
            let effect_cost = effects[idx].cost.total_cost(weights);
            let can_parallelize = effects[idx].parallelizable && 
                                 current_batch.len() < self.config.max_parallel_effects;
            
            if can_parallelize && current_batch_cost + effect_cost <= max_batch_cost {
                current_batch.push(idx);
                current_batch_cost += effect_cost;
            } else {
                if !current_batch.is_empty() {
                    batches.push(current_batch);
                }
                current_batch = vec![idx];
                current_batch_cost = effect_cost;
            }
        }
        
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        
        (indices, batches)
    }
}

impl Default for EffectOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::machine::instruction::RegisterId;
    
    fn create_test_effect(tag: &str, gas: u64, time: u64, parallelizable: bool) -> OptimizableEffect {
        OptimizableEffect {
            effect: MockEffect {
                call: crate::engine::MockEffectCall {
                    tag: tag.to_string(),
                    args: vec![],
                    return_type: Some("Value".to_string()),
                },
                result_register: Some(causality_core::RegisterId(0)),
            },
            cost: EffectCost::new(gas, time, 1024, 256),
            dependencies: vec![],
            priority: 1,
            parallelizable,
        }
    }
    
    #[test]
    fn test_optimizer_creation() {
        let optimizer = EffectOptimizer::new();
        assert_eq!(optimizer.config.strategy, OptimizationStrategy::Balanced);
        assert!(!optimizer.cost_database.is_empty());
    }
    
    #[test]
    fn test_gas_cost_optimization() {
        let mut optimizer = EffectOptimizer::new();
        
        let effects = vec![
            create_test_effect("expensive", 100, 50, true),
            create_test_effect("cheap", 10, 100, true),
            create_test_effect("medium", 50, 75, true),
        ];
        
        optimizer.config.strategy = OptimizationStrategy::MinimizeGasCost;
        let result = optimizer.optimize_effects(effects);
        
        // Should order by gas cost: cheap (10), medium (50), expensive (100)
        assert_eq!(result.execution_order, vec![1, 2, 0]);
    }
    
    #[test]
    fn test_time_optimization() {
        let mut optimizer = EffectOptimizer::new();
        
        let effects = vec![
            create_test_effect("slow", 50, 200, true),
            create_test_effect("fast", 100, 10, true),
            create_test_effect("medium", 75, 100, true),
        ];
        
        optimizer.config.strategy = OptimizationStrategy::MinimizeTime;
        let result = optimizer.optimize_effects(effects);
        
        // Should order by time cost: fast (10), medium (100), slow (200)
        assert_eq!(result.execution_order, vec![1, 2, 0]);
    }
    
    #[test]
    fn test_parallelization_optimization() {
        let mut optimizer = EffectOptimizer::new();
        
        let effects = vec![
            create_test_effect("parallel1", 50, 100, true),
            create_test_effect("sequential", 30, 50, false),
            create_test_effect("parallel2", 40, 80, true),
        ];
        
        optimizer.config.strategy = OptimizationStrategy::MaximizeParallelism;
        optimizer.config.max_parallel_effects = 2;
        
        let result = optimizer.optimize_effects(effects);
        
        // Should group parallel effects together
        assert_eq!(result.execution_order, vec![0, 2, 1]);
        assert_eq!(result.parallel_batches.len(), 2); // One batch for parallel, one for sequential
    }
    
    #[test]
    fn test_cost_estimation() {
        let optimizer = EffectOptimizer::new();
        
        let compute_cost = optimizer.estimate_effect_cost("compute");
        assert_eq!(compute_cost.gas_cost, 10);
        assert_eq!(compute_cost.time_cost, 50);
        
        let unknown_cost = optimizer.estimate_effect_cost("unknown_effect");
        assert_eq!(unknown_cost.gas_cost, 10); // Default cost
    }
} 