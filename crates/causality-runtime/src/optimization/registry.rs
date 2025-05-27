//! Strategy registry for managing optimization strategies
//!
//! This module provides the StrategyRegistry for registering, discovering, and selecting
//! optimization strategies based on TypedDomain, ProcessDataflowBlock context, and user preferences.

use super::{OptimizationStrategy, OptimizationContext, StrategyError};
use anyhow::Result;
use causality_types::tel::{
        optimization::TypedDomain,
        strategy::{StrategyConfiguration, StrategyMetrics},
    };
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Strategy Registry Trait
//-----------------------------------------------------------------------------

/// Trait for managing a registry of optimization strategies
pub trait StrategyRegistry: Send + Sync {
    /// Register a new strategy
    fn register_strategy(&mut self, strategy: Box<dyn OptimizationStrategy>) -> Result<()>;
    
    /// Unregister a strategy by ID
    fn unregister_strategy(&mut self, strategy_id: &str) -> Result<()>;
    
    /// Get a strategy by ID
    fn get_strategy(&self, strategy_id: &str) -> Option<&dyn OptimizationStrategy>;
    
    /// List all registered strategies
    fn list_strategies(&self) -> Vec<String>;
    
    /// Get strategies that support a specific typed domain
    fn get_strategies_for_domain(&self, domain: &TypedDomain) -> Vec<&dyn OptimizationStrategy>;
    
    /// Get the best strategy for a given context
    fn select_best_strategy(&self, context: &OptimizationContext) -> Option<&dyn OptimizationStrategy>;
    
    /// Update strategy configurations
    fn update_configurations(&mut self, configs: Vec<StrategyConfiguration>) -> Result<()>;
    
    /// Get all strategy metrics
    fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics>;
    
    /// Reset all strategies
    fn reset_all(&mut self);
}

//-----------------------------------------------------------------------------
// Default Strategy Registry Implementation
//-----------------------------------------------------------------------------

/// Default implementation of StrategyRegistry
pub struct DefaultStrategyRegistry {
    /// Registered strategies by ID
    strategies: HashMap<String, Box<dyn OptimizationStrategy>>,
    
    /// Strategy selection preferences
    selection_preferences: StrategySelectionPreferences,
    
    /// Strategy performance history for selection
    performance_history: HashMap<String, StrategyPerformanceHistory>,
}

/// Preferences for strategy selection
#[derive(Debug, Clone)]
pub struct StrategySelectionPreferences {
    /// Prefer strategies with higher success rates
    pub prefer_high_success_rate: bool,
    
    /// Prefer strategies with lower average execution time
    pub prefer_fast_execution: bool,
    
    /// Prefer strategies with better cost efficiency
    pub prefer_cost_efficiency: bool,
    
    /// Minimum success rate threshold (0.0 to 1.0)
    pub min_success_rate: f64,
    
    /// Maximum acceptable execution time (milliseconds)
    pub max_execution_time_ms: u64,
    
    /// TypedDomain-specific strategy preferences
    pub domain_preferences: HashMap<TypedDomain, Vec<String>>,
    
    /// Strategy priority overrides (higher = more preferred)
    pub priority_overrides: HashMap<String, u32>,
}

/// Performance history for a strategy
#[derive(Debug, Clone)]
pub struct StrategyPerformanceHistory {
    /// Strategy ID
    pub strategy_id: String,
    
    /// Recent success rate (0.0 to 1.0)
    pub recent_success_rate: f64,
    
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    
    /// Average cost efficiency score
    pub avg_cost_efficiency: f64,
    
    /// Number of recent evaluations
    pub recent_evaluations: u64,
    
    /// Last updated timestamp
    pub last_updated: causality_types::core::time::Timestamp,
}

impl DefaultStrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            selection_preferences: StrategySelectionPreferences::default(),
            performance_history: HashMap::new(),
        }
    }
    
    /// Update strategy selection preferences
    pub fn update_selection_preferences(&mut self, preferences: StrategySelectionPreferences) {
        self.selection_preferences = preferences;
    }
    
    /// Update performance history for a strategy
    pub fn update_performance_history(&mut self, strategy_id: &str, history: StrategyPerformanceHistory) {
        self.performance_history.insert(strategy_id.to_string(), history);
    }
    
    /// Calculate strategy score for selection
    fn calculate_strategy_score(&self, strategy: &dyn OptimizationStrategy, context: &OptimizationContext) -> f64 {
        let mut score = 0.0;
        let strategy_id = strategy.strategy_id();
        
        // Base score from strategy metrics
        let metrics = strategy.get_metrics();
        if metrics.total_evaluations > 0 {
            let success_rate = metrics.successful_evaluations as f64 / metrics.total_evaluations as f64;
            score += success_rate * 0.4; // 40% weight for success rate
            
            // Penalize slow strategies
            if metrics.avg_evaluation_time_ms > 0.0 {
                let time_score = 1.0 / (1.0 + metrics.avg_evaluation_time_ms / 1000.0);
                score += time_score * 0.2; // 20% weight for speed
            }
            
            // Reward high average scores
            score += metrics.avg_plan_score * 0.3; // 30% weight for plan quality
        }
        
        // Apply performance history if available
        if let Some(history) = self.performance_history.get(strategy_id) {
            score += history.recent_success_rate * 0.1; // 10% weight for recent performance
        }
        
        // Apply domain-specific preferences
        if let Some(preferred_strategies) = self.selection_preferences.domain_preferences.get(&context.current_typed_domain) {
            if preferred_strategies.contains(&strategy_id.to_string()) {
                score += 0.2; // Bonus for domain preference
            }
        }
        
        // Apply priority overrides
        if let Some(priority) = self.selection_preferences.priority_overrides.get(strategy_id) {
            score += (*priority as f64) * 0.01; // Small bonus based on priority
        }
        
        score
    }
}

impl StrategyRegistry for DefaultStrategyRegistry {
    fn register_strategy(&mut self, strategy: Box<dyn OptimizationStrategy>) -> Result<()> {
        let strategy_id = strategy.strategy_id().to_string();
        
        if self.strategies.contains_key(&strategy_id) {
            return Err(StrategyError::InvalidConfiguration {
                reason: format!("Strategy with ID '{}' already registered", strategy_id),
            }.into());
        }
        
        self.strategies.insert(strategy_id, strategy);
        Ok(())
    }
    
    fn unregister_strategy(&mut self, strategy_id: &str) -> Result<()> {
        if self.strategies.remove(strategy_id).is_none() {
            return Err(StrategyError::StrategyNotFound {
                strategy_id: strategy_id.to_string(),
            }.into());
        }
        
        // Clean up performance history
        self.performance_history.remove(strategy_id);
        
        Ok(())
    }
    
    fn get_strategy(&self, strategy_id: &str) -> Option<&dyn OptimizationStrategy> {
        self.strategies.get(strategy_id).map(|s| s.as_ref())
    }
    
    fn list_strategies(&self) -> Vec<String> {
        self.strategies.keys().map(|s| s.clone()).collect()
    }
    
    fn get_strategies_for_domain(&self, domain: &TypedDomain) -> Vec<&dyn OptimizationStrategy> {
        self.strategies
            .values()
            .filter(|strategy| strategy.supports_typed_domain(domain))
            .map(|s| s.as_ref())
            .collect()
    }
    
    fn select_best_strategy(&self, context: &OptimizationContext) -> Option<&dyn OptimizationStrategy> {
        let mut best_strategy: Option<&dyn OptimizationStrategy> = None;
        let mut best_score = -1.0;
        
        for strategy in self.strategies.values() {
            // Check if strategy supports the current domain
            if !strategy.supports_typed_domain(&context.current_typed_domain) {
                continue;
            }
            
            // Check minimum requirements
            let metrics = strategy.get_metrics();
            if metrics.total_evaluations > 0 {
                let success_rate = metrics.successful_evaluations as f64 / metrics.total_evaluations as f64;
                if success_rate < self.selection_preferences.min_success_rate {
                    continue;
                }
                
                if metrics.avg_evaluation_time_ms > self.selection_preferences.max_execution_time_ms as f64 {
                    continue;
                }
            }
            
            let score = self.calculate_strategy_score(strategy.as_ref(), context);
            if score > best_score {
                best_score = score;
                best_strategy = Some(strategy.as_ref());
            }
        }
        
        best_strategy
    }
    
    fn update_configurations(&mut self, configs: Vec<StrategyConfiguration>) -> Result<()> {
        for config in configs {
            let strategy_id = config.strategy_id.to_string();
            if let Some(strategy) = self.strategies.get_mut(&strategy_id) {
                strategy.update_configuration(config)?;
            } else {
                return Err(StrategyError::StrategyNotFound { strategy_id }.into());
            }
        }
        Ok(())
    }
    
    fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics> {
        self.strategies
            .iter()
            .map(|(id, strategy)| (id.clone(), strategy.get_metrics()))
            .collect()
    }
    
    fn reset_all(&mut self) {
        for strategy in self.strategies.values_mut() {
            strategy.reset();
        }
        self.performance_history.clear();
    }
}

impl Default for StrategySelectionPreferences {
    fn default() -> Self {
        Self {
            prefer_high_success_rate: true,
            prefer_fast_execution: true,
            prefer_cost_efficiency: true,
            min_success_rate: 0.7,
            max_execution_time_ms: 10000, // 10 seconds
            domain_preferences: HashMap::new(),
            priority_overrides: HashMap::new(),
        }
    }
}

impl Default for DefaultStrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        core::{id::DomainId, str::Str, time::Timestamp},
        tel::cost_model::ResourceUsageEstimate,
    };

    // Mock strategy for testing
    struct MockStrategy {
        id: String,
        supports_domain: TypedDomain,
    }

    impl OptimizationStrategy for MockStrategy {
        fn strategy_id(&self) -> &str {
            &self.id
        }
        
        fn strategy_name(&self) -> &str {
            "Mock Strategy"
        }
        
        fn description(&self) -> &str {
            "A mock strategy for testing"
        }
        
        fn propose(&self, _context: &OptimizationContext) -> Result<Vec<causality_types::tel::optimization::ScoredPlan>> {
            Ok(vec![])
        }
        
        fn evaluate_plan(&self, plan: &causality_types::tel::optimization::ResolutionPlan, _context: &OptimizationContext) -> Result<causality_types::tel::optimization::ScoredPlan> {
            // Mock implementation: create a scored plan with default scores
            Ok(causality_types::tel::optimization::ScoredPlan {
                plan: plan.clone(),
                overall_score: 0.5,
                cost_efficiency_score: 0.6,
                time_efficiency_score: 0.7,
                resource_utilization_score: 0.8,
                domain_compatibility_score: 0.9,
                strategy_name: causality_types::primitive::string::Str::from("MockStrategy"),
                evaluated_at: causality_types::core::time::Timestamp::now(),
            })
        }
        
        fn supports_typed_domain(&self, domain: &TypedDomain) -> bool {
            domain == &self.supports_domain
        }
        
        fn get_configuration(&self) -> StrategyConfiguration {
            StrategyConfiguration {
                strategy_id: Str::from(self.id.as_str()),
                parameters: HashMap::new(),
                enabled_domains: vec![self.supports_domain.clone()],
                priority: 1,
                max_evaluation_time_ms: 5000,
                version: 1,
                updated_at: Timestamp::now(),
            }
        }
        
        fn update_configuration(&mut self, _config: StrategyConfiguration) -> Result<()> {
            Ok(())
        }
        
        fn get_metrics(&self) -> StrategyMetrics {
            StrategyMetrics {
                strategy_id: Str::from(self.id.as_str()),
                total_evaluations: 10,
                successful_evaluations: 8,
                avg_evaluation_time_ms: 100.0,
                avg_plan_score: 0.8,
                plans_selected: 5,
                plans_executed_successfully: 4,
                resource_consumption: ResourceUsageEstimate::default(),
                domain_performance: HashMap::new(),
                last_updated: Timestamp::now(),
            }
        }
        
        fn reset(&mut self) {
            // Nothing to reset for mock
        }
    }

    #[test]
    fn test_strategy_registry_registration() {
        let mut registry = DefaultStrategyRegistry::new();
        let domain = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        
        let strategy = Box::new(MockStrategy {
            id: "test_strategy".to_string(),
            supports_domain: domain.clone(),
        });
        
        assert!(registry.register_strategy(strategy).is_ok());
        assert!(registry.get_strategy("test_strategy").is_some());
        assert_eq!(registry.list_strategies().len(), 1);
    }
    
    #[test]
    fn test_strategy_selection_by_domain() {
        let mut registry = DefaultStrategyRegistry::new();
        let domain1 = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let domain2 = TypedDomain::ServiceDomain(DomainId::new([2u8; 32]));
        
        let strategy1 = Box::new(MockStrategy {
            id: "verifiable_strategy".to_string(),
            supports_domain: domain1.clone(),
        });
        
        let strategy2 = Box::new(MockStrategy {
            id: "service_strategy".to_string(),
            supports_domain: domain2.clone(),
        });
        
        registry.register_strategy(strategy1).unwrap();
        registry.register_strategy(strategy2).unwrap();
        
        let verifiable_strategies = registry.get_strategies_for_domain(&domain1);
        assert_eq!(verifiable_strategies.len(), 1);
        assert_eq!(verifiable_strategies[0].strategy_id(), "verifiable_strategy");
        
        let service_strategies = registry.get_strategies_for_domain(&domain2);
        assert_eq!(service_strategies.len(), 1);
        assert_eq!(service_strategies[0].strategy_id(), "service_strategy");
    }
} 