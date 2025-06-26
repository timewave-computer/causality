//! Effect execution optimizer for simulation performance
//!
//! This module provides optimization strategies for reordering and scheduling
//! effects to minimize execution cost and maximize parallelization opportunities.

use crate::{
    engine::{SessionEffect, SessionOperation, SessionParticipantState},
    error::SimulationResult,
};
use causality_core::lambda::base::{SessionType, TypeInner};
use std::collections::BTreeMap;

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
    pub effect: SessionEffect,
    
    /// Cost metrics for this effect
    pub cost: EffectCost,
    
    /// Dependencies on other effects
    pub dependencies: Vec<usize>,
    
    /// Priority level (higher = more important)
    pub priority: u32,
    
    /// Whether this effect can be parallelized
    pub parallelizable: bool,
}

/// Enhanced optimization strategy including session-aware patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStrategy {
    /// Minimize gas/resource consumption
    GasEfficiency,
    /// Minimize execution time
    Speed,
    /// Balance between gas and speed
    Balanced,
    /// Session protocol optimization
    SessionOptimized,
    /// Communication pattern optimization
    CommunicationOptimized,
    /// Multi-party protocol optimization
    MultiPartyOptimized,
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
    cost_database: BTreeMap<String, EffectCost>,
    
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

/// Session-aware performance optimization engine
#[derive(Debug, Clone)]
pub struct SessionAwareOptimizer {
    /// Session type analysis cache
    session_analysis_cache: BTreeMap<String, SessionAnalysisResult>,
    /// Communication pattern optimization patterns
    communication_patterns: Vec<CommunicationOptimizationPattern>,
    /// Performance prediction models
    performance_models: BTreeMap<String, PerformancePredictionModel>,
    /// Optimization statistics
    optimization_stats: SessionOptimizationStats,
}

/// Result of session type analysis for optimization
#[derive(Debug, Clone)]
pub struct SessionAnalysisResult {
    /// Session type being analyzed
    pub session_type: SessionType,
    /// Communication complexity score
    pub communication_complexity: u32,
    /// Expected message count
    pub estimated_message_count: u32,
    /// Parallelization opportunities
    pub parallelization_opportunities: Vec<ParallelizationOpportunity>,
    /// Critical path operations
    pub critical_path: Vec<SessionOperation>,
    /// Resource usage prediction
    pub resource_prediction: ResourceUsagePrediction,
}

/// Communication optimization pattern for session protocols
#[derive(Debug, Clone)]
pub struct CommunicationOptimizationPattern {
    /// Pattern name
    pub name: String,
    /// Session type pattern this applies to
    pub session_pattern: SessionType,
    /// Optimization technique
    pub optimization: CommunicationOptimization,
    /// Expected improvement factor
    pub improvement_factor: f64,
}

/// Communication optimization techniques
#[derive(Debug, Clone)]
pub enum CommunicationOptimization {
    /// Batch multiple sends into a single message
    MessageBatching {
        max_batch_size: usize,
        timeout_ms: u64,
    },
    /// Pipeline communication operations
    Pipelining {
        pipeline_depth: usize,
    },
    /// Parallel execution of independent branches
    ParallelBranches {
        max_parallel_branches: usize,
    },
    /// Choice prediction and pre-execution
    ChoicePrediction {
        prediction_accuracy: f64,
    },
    /// Message compression for large data types
    MessageCompression {
        compression_ratio: f64,
    },
}

/// Parallelization opportunity in session protocol
#[derive(Debug, Clone)]
pub struct ParallelizationOpportunity {
    /// Operations that can be run in parallel
    pub parallel_operations: Vec<SessionOperation>,
    /// Expected speedup factor
    pub speedup_factor: f64,
    /// Resource requirements for parallel execution
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for optimization
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub network_bandwidth_mbps: u64,
}

/// Resource usage prediction based on session type
#[derive(Debug, Clone)]
pub struct ResourceUsagePrediction {
    /// Expected gas consumption
    pub gas_usage: u64,
    /// Expected execution time in milliseconds
    pub execution_time_ms: u64,
    /// Expected memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Expected network bandwidth usage in bytes
    pub network_usage_bytes: u64,
    /// Confidence level of prediction (0.0 to 1.0)
    pub confidence: f64,
}

/// Performance prediction model for session types
#[derive(Debug, Clone)]
pub struct PerformancePredictionModel {
    /// Model name
    pub name: String,
    /// Session type this model applies to
    pub applicable_session_type: SessionType,
    /// Base performance characteristics
    pub base_performance: ResourceUsagePrediction,
    /// Scaling factors for session complexity
    pub scaling_factors: ScalingFactors,
}

/// Scaling factors for performance prediction
#[derive(Debug, Clone)]
pub struct ScalingFactors {
    /// Factor for number of participants
    pub participant_factor: f64,
    /// Factor for message count
    pub message_factor: f64,
    /// Factor for choice complexity
    pub choice_factor: f64,
    /// Factor for nesting depth
    pub nesting_factor: f64,
}

/// Session optimization statistics
#[derive(Debug, Clone, Default)]
pub struct SessionOptimizationStats {
    pub session_analyses_performed: usize,
    pub communication_patterns_optimized: usize,
    pub average_performance_improvement: f64,
    pub total_gas_savings: u64,
    pub total_time_savings_ms: u64,
}

/// Result of communication optimization
#[derive(Debug, Clone)]
pub struct CommunicationOptimizationResult {
    pub optimization_type: String,
    pub improvement_factor: f64,
    pub resource_savings: ResourceUsagePrediction,
    pub optimized_operations: Vec<SessionOperation>,
}

/// Performance prediction result
#[derive(Debug, Clone)]
pub struct PerformancePrediction {
    pub predicted_performance: ResourceUsagePrediction,
    pub bottlenecks: Vec<String>,
    pub optimization_recommendations: Vec<String>,
    pub scaling_behavior: ScalingFactors,
}

/// Enhanced effect optimization and scheduling engine with session awareness
#[derive(Debug, Clone)]
pub struct SimulationOptimizer {
    /// Default optimization strategy
    default_strategy: OptimizationStrategy,
    /// Cache for optimization results
    optimization_cache: BTreeMap<String, String>,
    /// Session-aware optimizer for protocol optimization
    session_optimizer: SessionAwareOptimizer,
}

impl SimulationOptimizer {
    /// Create a new simulation optimizer
    pub fn new() -> Self {
        Self {
            default_strategy: OptimizationStrategy::Balanced,
            optimization_cache: BTreeMap::new(),
            session_optimizer: SessionAwareOptimizer::new(),
        }
    }
    
    /// Create optimizer with specific default strategy  
    pub fn with_strategy(strategy: OptimizationStrategy) -> Self {
        Self {
            default_strategy: strategy,
            optimization_cache: BTreeMap::new(),
            session_optimizer: SessionAwareOptimizer::new(),
        }
    }
    
    /// Optimize session protocol for performance
    pub fn optimize_session_protocol(
        &mut self,
        session_type: &SessionType,
        participants: &BTreeMap<String, SessionParticipantState>,
    ) -> SimulationResult<CommunicationOptimizationResult> {
        self.session_optimizer.optimize_communication_pattern(session_type, participants)
    }
    
    /// Analyze session type for optimization opportunities
    pub fn analyze_session_type(&mut self, session_type: &SessionType) -> SimulationResult<SessionAnalysisResult> {
        self.session_optimizer.analyze_session_type(session_type)
    }
    
    /// Predict performance for session protocol
    pub fn predict_session_performance(
        &mut self,
        session_type: &SessionType,
        participant_count: usize,
    ) -> SimulationResult<PerformancePrediction> {
        self.session_optimizer.predict_performance(session_type, participant_count)
    }
    
    /// Get session optimization statistics
    pub fn get_session_optimization_stats(&self) -> &SessionOptimizationStats {
        self.session_optimizer.get_optimization_statistics()
    }
    
    /// Optimize program for gas efficiency
    pub fn optimize_for_gas_efficiency(&self, program: &str) -> String {
        // Mock optimization for gas efficiency
        // In reality, this would analyze the program and:
        // - Merge adjacent allocations
        // - Eliminate redundant operations
        // - Use more efficient instruction sequences
        
        if program.contains("alloc") && program.contains("consume") {
            // Optimize alloc-consume patterns
            format!("(optimized-gas {})", program)
        } else if program.contains("tensor") {
            // Optimize tensor operations
            format!("(tensor-opt-gas {})", program)
        } else {
            // Apply general gas optimizations
            format!("(gas-opt {})", program)
        }
    }
    
    /// Optimize program for execution speed
    pub fn optimize_for_speed(&self, program: &str) -> String {
        // Mock optimization for speed
        // In reality, this would:
        // - Parallelize independent operations
        // - Inline function calls
        // - Use faster instruction variants
        
        if program.contains("lambda") {
            // Inline lambda functions
            format!("(inlined {})", program)
        } else if program.contains("tensor") {
            // Use vectorized operations
            format!("(vectorized {})", program)
        } else {
            // Apply general speed optimizations
            format!("(speed-opt {})", program)
        }
    }
    
    /// Optimize for parallelism (internal method for balanced optimization)
    fn _optimize_for_parallelism(&self, program: &str) -> String {
        // Mock parallelism optimization
        // In reality, this would:
        // - Identify independent operations
        // - Create parallel execution paths
        // - Balance workload across threads
        
        if program.contains("lambda") && program.contains("tensor") {
            // Parallelize both lambda and tensor operations
            format!("(parallel-all {})", program)
        } else if program.contains("lambda") {
            // Parallelize lambda operations
            format!("(parallel-lambda {})", program)
        } else if program.contains("tensor") {
            // Parallelize tensor operations
            format!("(parallel-tensor {})", program)
        } else {
            // Apply general parallelization
            format!("(parallel {})", program)
        }
    }
    
    /// Apply balanced optimization
    pub fn optimize_balanced(&self, program: &str) -> String {
        // Mock balanced optimization
        // Combines gas and speed optimizations with reasonable trade-offs
        
        let gas_optimized = self.optimize_for_gas_efficiency(program);
        let speed_hints = if program.contains("lambda") { 
            "-fast" 
        } else { 
            "" 
        };
        
        format!("(balanced{} {})", speed_hints, gas_optimized)
    }
    
    /// Apply optimization based on strategy
    pub fn optimize_with_strategy(&self, program: &str, strategy: OptimizationStrategy) -> String {
        match strategy {
            OptimizationStrategy::GasEfficiency => self.optimize_for_gas_efficiency(program),
            OptimizationStrategy::Speed => self.optimize_for_speed(program),
            OptimizationStrategy::Balanced => self.optimize_balanced(program),
            _ => self.optimize_balanced(program),
        }
    }
    
    /// Apply default optimization
    pub fn optimize(&self, program: &str) -> String {
        self.optimize_with_strategy(program, self.default_strategy)
    }
    
    /// Get optimization recommendations for a program
    pub fn analyze_program(&self, program: &str) -> OptimizationAnalysis {
        let has_allocations = program.contains("alloc");
        let has_consumption = program.contains("consume");
        let has_lambdas = program.contains("lambda");
        let has_tensors = program.contains("tensor");
        
        let mut recommendations = Vec::new();
        let mut estimated_savings = 0;
        
        if has_allocations && has_consumption {
            recommendations.push("Consider merging allocation-consumption patterns".to_string());
            estimated_savings += 15;
        }
        
        if has_lambdas {
            recommendations.push("Lambda inlining available for speed optimization".to_string());
            estimated_savings += 25;
        }
        
        if has_tensors {
            recommendations.push("Tensor operations can be vectorized".to_string());
            estimated_savings += 20;
        }
        
        OptimizationAnalysis {
            complexity_score: program.len() as u32,
            optimization_potential: estimated_savings,
            recommended_strategy: if has_lambdas {
                OptimizationStrategy::Speed
            } else if has_allocations {
                OptimizationStrategy::GasEfficiency
            } else {
                OptimizationStrategy::Balanced
            },
            recommendations,
        }
    }
    
    /// Clear optimization cache
    pub fn clear_cache(&mut self) {
        self.optimization_cache.clear();
    }
}

/// Analysis results for optimization potential
#[derive(Debug, Clone)]
pub struct OptimizationAnalysis {
    /// Complexity score of the program (higher = more complex)
    pub complexity_score: u32,
    /// Estimated optimization potential as percentage
    pub optimization_potential: u32,
    /// Recommended optimization strategy
    pub recommended_strategy: OptimizationStrategy,
    /// Specific optimization recommendations
    pub recommendations: Vec<String>,
}

impl Default for SimulationOptimizer {
    fn default() -> Self {
        Self::new()
    }
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
            OptimizationStrategy::GasEfficiency => self.optimize_for_gas_cost(&effects),
            OptimizationStrategy::Speed => self.optimize_for_time(&effects),
            OptimizationStrategy::Balanced => self.optimize_balanced(&effects),
            _ => self.optimize_balanced(&effects),
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
            strategy_used: self.config.strategy,
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
    fn create_default_cost_database() -> BTreeMap<String, EffectCost> {
        let mut db = BTreeMap::new();
        
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
        let _transfer_effect = SessionEffect {
            operation: SessionOperation::Send {
                value_type: causality_core::lambda::base::TypeInner::Base(causality_core::lambda::base::BaseType::Int),
                target_participant: "receiver".to_string(),
                value: Some(causality_core::lambda::base::Value::Int(100)),
            },
            timestamp: crate::clock::SimulatedTimestamp::new(0),
            gas_consumed: 20,
            success: true,
            result: None,
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

/// Custom optimization strategy with weights
#[derive(Debug, Clone, PartialEq)]
pub struct CustomOptimizationWeights {
    /// Weight for gas efficiency (0.0 to 1.0)
    pub gas_weight: f64,
    /// Weight for speed (0.0 to 1.0)
    pub speed_weight: f64,
    /// Weight for parallelism (0.0 to 1.0)
    pub parallelism_weight: f64,
}

impl SessionAwareOptimizer {
    /// Create a new session-aware optimizer
    pub fn new() -> Self {
        let mut optimizer = Self {
            session_analysis_cache: BTreeMap::new(),
            communication_patterns: Vec::new(),
            performance_models: BTreeMap::new(),
            optimization_stats: SessionOptimizationStats::default(),
        };
        
        optimizer.initialize_communication_patterns();
        optimizer.initialize_performance_models();
        optimizer
    }
    
    /// Analyze session type for optimization opportunities
    pub fn analyze_session_type(&mut self, session_type: &SessionType) -> SimulationResult<SessionAnalysisResult> {
        let session_key = format!("{:?}", session_type);
        
        // Check cache first
        if let Some(cached_result) = self.session_analysis_cache.get(&session_key) {
            return Ok(cached_result.clone());
        }
        
        // Perform analysis
        let communication_complexity = self.calculate_communication_complexity(session_type);
        let estimated_message_count = self.estimate_message_count(session_type);
        let parallelization_opportunities = self.identify_parallelization_opportunities(session_type);
        let critical_path = self.extract_critical_path(session_type);
        let resource_prediction = self.predict_resource_usage(session_type)?;
        
        let result = SessionAnalysisResult {
            session_type: session_type.clone(),
            communication_complexity,
            estimated_message_count,
            parallelization_opportunities,
            critical_path,
            resource_prediction,
        };
        
        // Cache the result
        self.session_analysis_cache.insert(session_key, result.clone());
        self.optimization_stats.session_analyses_performed += 1;
        
        Ok(result)
    }
    
    /// Optimize communication patterns for session protocol
    pub fn optimize_communication_pattern(
        &mut self,
        session_type: &SessionType,
        participants: &BTreeMap<String, SessionParticipantState>,
    ) -> SimulationResult<CommunicationOptimizationResult> {
        let _analysis = self.analyze_session_type(session_type)?;
        let mut optimizations = Vec::new();
        
        // Check for applicable patterns
        for pattern in &self.communication_patterns {
            if self.pattern_matches_session(&pattern.session_pattern, session_type) {
                let optimization_result = self.apply_communication_optimization(
                    &pattern.optimization,
                    session_type,
                    participants,
                )?;
                optimizations.push(optimization_result);
            }
        }
        
        // Select best optimization
        let best_optimization = optimizations
            .into_iter()
            .max_by(|a, b| a.improvement_factor.partial_cmp(&b.improvement_factor).unwrap())
            .unwrap_or_else(|| CommunicationOptimizationResult {
                optimization_type: "none".to_string(),
                improvement_factor: 1.0,
                resource_savings: ResourceUsagePrediction {
                    gas_usage: 0,
                    execution_time_ms: 0,
                    memory_usage_bytes: 0,
                    network_usage_bytes: 0,
                    confidence: 1.0,
                },
                optimized_operations: Vec::new(),
            });
        
        self.optimization_stats.communication_patterns_optimized += 1;
        self.optimization_stats.total_gas_savings += best_optimization.resource_savings.gas_usage;
        self.optimization_stats.total_time_savings_ms += best_optimization.resource_savings.execution_time_ms;
        
        Ok(best_optimization)
    }
    
    /// Predict performance for a session protocol
    pub fn predict_performance(
        &mut self,
        session_type: &SessionType,
        participant_count: usize,
    ) -> SimulationResult<PerformancePrediction> {
        let analysis = self.analyze_session_type(session_type)?;
        
        // Find applicable performance model
        let model = self.performance_models.values()
            .find(|model| self.pattern_matches_session(&model.applicable_session_type, session_type))
            .cloned()
            .unwrap_or_else(|| self.create_default_performance_model(session_type));
        
        // Apply scaling factors
        let base = &model.base_performance;
        let factors = &model.scaling_factors;
        
        let predicted_gas = (base.gas_usage as f64 * 
            (participant_count as f64).powf(factors.participant_factor) *
            (analysis.estimated_message_count as f64).powf(factors.message_factor)) as u64;
        
        let predicted_time = (base.execution_time_ms as f64 *
            (participant_count as f64).powf(factors.participant_factor) *
            (analysis.estimated_message_count as f64).powf(factors.message_factor)) as u64;
        
        let predicted_memory = (base.memory_usage_bytes as f64 *
            (participant_count as f64).powf(factors.participant_factor)) as u64;
        
        let predicted_network = (base.network_usage_bytes as f64 *
            (analysis.estimated_message_count as f64).powf(factors.message_factor)) as u64;
        
        Ok(PerformancePrediction {
            predicted_performance: ResourceUsagePrediction {
                gas_usage: predicted_gas,
                execution_time_ms: predicted_time,
                memory_usage_bytes: predicted_memory,
                network_usage_bytes: predicted_network,
                confidence: base.confidence * 0.9, // Slightly lower confidence for predictions
            },
            bottlenecks: self.identify_performance_bottlenecks(&analysis),
            optimization_recommendations: self.generate_optimization_recommendations(&analysis),
            scaling_behavior: model.scaling_factors.clone(),
        })
    }
    
    /// Get optimization statistics
    pub fn get_optimization_statistics(&self) -> &SessionOptimizationStats {
        &self.optimization_stats
    }
    
    /// Initialize common communication optimization patterns
    fn initialize_communication_patterns(&mut self) {
        // Message batching pattern for sequential sends
        self.communication_patterns.push(CommunicationOptimizationPattern {
            name: "sequential_send_batching".to_string(),
            session_pattern: SessionType::Send(
                Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                Box::new(SessionType::Send(
                    Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                    Box::new(SessionType::End)
                ))
            ),
            optimization: CommunicationOptimization::MessageBatching {
                max_batch_size: 10,
                timeout_ms: 100,
            },
            improvement_factor: 2.5,
        });
        
        // Pipeline optimization for request-response patterns
        self.communication_patterns.push(CommunicationOptimizationPattern {
            name: "request_response_pipeline".to_string(),
            session_pattern: SessionType::Send(
                Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                Box::new(SessionType::Receive(
                    Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                    Box::new(SessionType::End)
                ))
            ),
            optimization: CommunicationOptimization::Pipelining {
                pipeline_depth: 3,
            },
            improvement_factor: 1.8,
        });
        
        // Parallel branches for choice operations
        self.communication_patterns.push(CommunicationOptimizationPattern {
            name: "parallel_choice_branches".to_string(),
            session_pattern: SessionType::InternalChoice(vec![
                ("branch1".to_string(), SessionType::End),
                ("branch2".to_string(), SessionType::End),
            ]),
            optimization: CommunicationOptimization::ParallelBranches {
                max_parallel_branches: 4,
            },
            improvement_factor: 3.0,
        });
    }
    
    /// Initialize performance prediction models
    fn initialize_performance_models(&mut self) {
        // Simple send-receive model
        self.performance_models.insert(
            "send_receive".to_string(),
            PerformancePredictionModel {
                name: "send_receive".to_string(),
                applicable_session_type: SessionType::Send(
                    Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(causality_core::lambda::base::BaseType::Unit)),
                        Box::new(SessionType::End)
                    ))
                ),
                base_performance: ResourceUsagePrediction {
                    gas_usage: 50,
                    execution_time_ms: 100,
                    memory_usage_bytes: 1024,
                    network_usage_bytes: 256,
                    confidence: 0.95,
                },
                scaling_factors: ScalingFactors {
                    participant_factor: 1.2,
                    message_factor: 1.1,
                    choice_factor: 1.0,
                    nesting_factor: 1.3,
                },
            }
        );
        
        // Multi-party coordination model
        self.performance_models.insert(
            "multi_party".to_string(),
            PerformancePredictionModel {
                name: "multi_party".to_string(),
                applicable_session_type: SessionType::InternalChoice(vec![
                    ("broadcast".to_string(), SessionType::End),
                    ("gather".to_string(), SessionType::End),
                ]),
                base_performance: ResourceUsagePrediction {
                    gas_usage: 200,
                    execution_time_ms: 500,
                    memory_usage_bytes: 4096,
                    network_usage_bytes: 1024,
                    confidence: 0.85,
                },
                scaling_factors: ScalingFactors {
                    participant_factor: 2.0,
                    message_factor: 1.5,
                    choice_factor: 1.8,
                    nesting_factor: 2.2,
                },
            }
        );
    }
    
    // Helper methods for analysis
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_communication_complexity(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) => 1 + self.calculate_communication_complexity(continuation),
            SessionType::Receive(_, continuation) => 1 + self.calculate_communication_complexity(continuation),
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                1 + branches.iter().map(|(_, branch)| self.calculate_communication_complexity(branch)).max().unwrap_or(0)
            }
            SessionType::Recursive(_, body) => 2 + self.calculate_communication_complexity(body), // Add recursion overhead
            SessionType::Variable(_) => 1, // Variable reference has minimal complexity
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_message_count(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                1 + self.estimate_message_count(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.iter().map(|(_, branch)| self.estimate_message_count(branch)).sum::<u32>() / (branches.len() as u32).max(1)
            }
            SessionType::Recursive(_, body) => {
                // Assume recursive types execute at least twice on average
                2 * self.estimate_message_count(body)
            }
            SessionType::Variable(_) => 0, // Variable doesn't directly produce messages
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn identify_parallelization_opportunities(&self, session_type: &SessionType) -> Vec<ParallelizationOpportunity> {
        match session_type {
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) if branches.len() > 1 => {
                vec![ParallelizationOpportunity {
                    parallel_operations: branches.iter().enumerate().map(|(i, _)| SessionOperation::InternalChoice {
                        chosen_branch: format!("branch_{}", i),
                        branch_operations: Vec::new(),
                    }).collect(),
                    speedup_factor: branches.len() as f64 * 0.8, // 80% efficiency
                    resource_requirements: ResourceRequirements {
                        cpu_cores: branches.len() as u32,
                        memory_mb: (branches.len() * 256) as u64,
                        network_bandwidth_mbps: (branches.len() * 10) as u64,
                    },
                }]
            }
            SessionType::Recursive(_, body) => {
                // Recursive types might have parallelization opportunities in their body
                self.identify_parallelization_opportunities(body)
            }
            SessionType::Variable(_) => Vec::new(), // Variables don't have parallelization opportunities
            _ => Vec::new(),
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn extract_critical_path(&self, session_type: &SessionType) -> Vec<SessionOperation> {
        match session_type {
            SessionType::Send(value_type, continuation) => {
                let mut path = vec![SessionOperation::Send {
                    value_type: value_type.as_ref().clone(),
                    target_participant: "unknown".to_string(),
                    value: None,
                }];
                path.extend(self.extract_critical_path(continuation));
                path
            }
            SessionType::Receive(value_type, continuation) => {
                let mut path = vec![SessionOperation::Receive {
                    value_type: value_type.as_ref().clone(),
                    source_participant: "unknown".to_string(),
                    expected_value: None,
                }];
                path.extend(self.extract_critical_path(continuation));
                path
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                // Take the longest branch as critical path
                branches.iter()
                    .map(|(branch_name, branch_session)| {
                        let mut path = vec![SessionOperation::InternalChoice {
                            chosen_branch: branch_name.clone(),
                            branch_operations: Vec::new(),
                        }];
                        path.extend(self.extract_critical_path(branch_session));
                        path
                    })
                    .max_by_key(|path| path.len())
                    .unwrap_or_default()
            }
            SessionType::Recursive(var_name, body) => {
                // For recursive types, include the recursion in the critical path
                let mut path = vec![SessionOperation::InternalChoice {
                    chosen_branch: format!("recursive_{}", var_name),
                    branch_operations: Vec::new(),
                }];
                path.extend(self.extract_critical_path(body));
                path
            }
            SessionType::Variable(var_name) => {
                // Variable reference represents a recursive call
                vec![SessionOperation::InternalChoice {
                    chosen_branch: format!("variable_{}", var_name),
                    branch_operations: Vec::new(),
                }]
            }
            SessionType::End => vec![SessionOperation::End],
        }
    }
    
    fn predict_resource_usage(&self, session_type: &SessionType) -> SimulationResult<ResourceUsagePrediction> {
        let complexity = self.calculate_communication_complexity(session_type);
        let message_count = self.estimate_message_count(session_type);
        
        // Base resource usage calculation
        let base_gas = 10u64;
        let base_time = 50u64;
        let base_memory = 512u64;
        let base_network = 128u64;
        
        Ok(ResourceUsagePrediction {
            gas_usage: base_gas * (complexity as u64) + (message_count as u64) * 5,
            execution_time_ms: base_time * (complexity as u64) + (message_count as u64) * 20,
            memory_usage_bytes: base_memory * (complexity as u64),
            network_usage_bytes: base_network * (message_count as u64),
            confidence: 0.8, // Medium confidence for basic model
        })
    }
    
    fn pattern_matches_session(&self, pattern: &SessionType, session: &SessionType) -> bool {
        // Simplified pattern matching - in practice would be more sophisticated
        std::mem::discriminant(pattern) == std::mem::discriminant(session)
    }
    
    fn apply_communication_optimization(
        &self,
        optimization: &CommunicationOptimization,
        _session_type: &SessionType,
        _participants: &BTreeMap<String, SessionParticipantState>,
    ) -> SimulationResult<CommunicationOptimizationResult> {
        match optimization {
            CommunicationOptimization::MessageBatching { max_batch_size, .. } => {
                Ok(CommunicationOptimizationResult {
                    optimization_type: "message_batching".to_string(),
                    improvement_factor: 2.0,
                    resource_savings: ResourceUsagePrediction {
                        gas_usage: (*max_batch_size as u64) * 10,
                        execution_time_ms: (*max_batch_size as u64) * 25,
                        memory_usage_bytes: 0,
                        network_usage_bytes: (*max_batch_size as u64) * 50,
                        confidence: 0.9,
                    },
                    optimized_operations: Vec::new(),
                })
            }
            CommunicationOptimization::Pipelining { pipeline_depth } => {
                Ok(CommunicationOptimizationResult {
                    optimization_type: "pipelining".to_string(),
                    improvement_factor: (*pipeline_depth as f64) * 0.6,
                    resource_savings: ResourceUsagePrediction {
                        gas_usage: 0,
                        execution_time_ms: (*pipeline_depth as u64) * 100,
                        memory_usage_bytes: (*pipeline_depth as u64) * 256,
                        network_usage_bytes: 0,
                        confidence: 0.85,
                    },
                    optimized_operations: Vec::new(),
                })
            }
            _ => Ok(CommunicationOptimizationResult {
                optimization_type: "generic".to_string(),
                improvement_factor: 1.2,
                resource_savings: ResourceUsagePrediction {
                    gas_usage: 20,
                    execution_time_ms: 50,
                    memory_usage_bytes: 0,
                    network_usage_bytes: 0,
                    confidence: 0.7,
                },
                optimized_operations: Vec::new(),
            }),
        }
    }
    
    fn create_default_performance_model(&self, session_type: &SessionType) -> PerformancePredictionModel {
        PerformancePredictionModel {
            name: "default".to_string(),
            applicable_session_type: session_type.clone(),
            base_performance: ResourceUsagePrediction {
                gas_usage: 100,
                execution_time_ms: 200,
                memory_usage_bytes: 2048,
                network_usage_bytes: 512,
                confidence: 0.6,
            },
            scaling_factors: ScalingFactors {
                participant_factor: 1.5,
                message_factor: 1.2,
                choice_factor: 1.4,
                nesting_factor: 1.6,
            },
        }
    }
    
    fn identify_performance_bottlenecks(&self, analysis: &SessionAnalysisResult) -> Vec<String> {
        let mut bottlenecks = Vec::new();
        
        if analysis.communication_complexity > 10 {
            bottlenecks.push("High communication complexity".to_string());
        }
        
        if analysis.estimated_message_count > 50 {
            bottlenecks.push("High message count".to_string());
        }
        
        if analysis.parallelization_opportunities.is_empty() {
            bottlenecks.push("Limited parallelization opportunities".to_string());
        }
        
        bottlenecks
    }
    
    fn generate_optimization_recommendations(&self, analysis: &SessionAnalysisResult) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if analysis.estimated_message_count > 10 {
            recommendations.push("Consider message batching to reduce communication overhead".to_string());
        }
        
        if !analysis.parallelization_opportunities.is_empty() {
            recommendations.push("Enable parallel execution for choice branches".to_string());
        }
        
        if analysis.communication_complexity > 5 {
            recommendations.push("Consider protocol simplification or decomposition".to_string());
        }
        
        recommendations
    }
}

impl Default for SessionAwareOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    fn create_test_effect(tag: &str, gas: u64, time: u64, parallelizable: bool) -> OptimizableEffect {
        use causality_core::lambda::base::{TypeInner, BaseType, Value};
        
        let operation = match tag {
            "transfer" => SessionOperation::Send {
                value_type: TypeInner::Base(BaseType::Int),
                target_participant: "receiver".to_string(),
                value: Some(Value::Int(100)),
            },
            _ => SessionOperation::Send {
                value_type: TypeInner::Base(BaseType::Unit),
                target_participant: "other".to_string(),
                value: Some(Value::Unit),
            },
        };
        
        OptimizableEffect {
            effect: SessionEffect {
                operation,
                timestamp: crate::clock::SimulatedTimestamp::new(0),
                gas_consumed: gas,
                success: true,
                result: None,
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
        
        optimizer.config.strategy = OptimizationStrategy::GasEfficiency;
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
        
        optimizer.config.strategy = OptimizationStrategy::Speed;
        let result = optimizer.optimize_effects(effects);
        
        // Should order by time cost: fast (10), medium (100), slow (200)
        assert_eq!(result.execution_order, vec![1, 2, 0]);
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