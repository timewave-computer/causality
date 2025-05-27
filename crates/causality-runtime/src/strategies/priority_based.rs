//! Priority-based optimization strategy
//!
//! This strategy optimizes based on intent priorities and TypedDomain preferences,
//! favoring simpler execution paths and current domain efficiency.

use crate::optimization::{OptimizationStrategy, OptimizationContext};
use anyhow::Result;
use causality_types::{
    core::{
        id::{EntityId, AsId},
        str::Str,
        time::Timestamp,
    },
    tel::{
        optimization::{
            ResolutionPlan, ScoredPlan, TypedDomain,
        },
        cost_model::ResourceUsageEstimate,
        strategy::{StrategyConfiguration, StrategyMetrics, ConfigurationValue},
    },
};
use std::collections::HashMap;

/// Strategy that optimizes based on intent priorities and TypedDomain preferences
pub struct PriorityBasedStrategy {
    /// Strategy configuration
    config: StrategyConfiguration,
    
    /// Performance metrics
    metrics: StrategyMetrics,
    
    /// Domain priority preferences (higher = more preferred)
    domain_priorities: HashMap<TypedDomain, u32>,
    
    /// Whether to prefer current domain for efficiency
    prefer_current_domain: bool,
}

impl PriorityBasedStrategy {
    /// Create a new priority-based strategy
    pub fn new() -> Self {
        let mut domain_priorities = HashMap::new();
        // Default priorities - can be configured
        domain_priorities.insert(
            TypedDomain::VerifiableDomain(causality_types::primitive::ids::DomainId::new([0u8; 32])), 
            10
        );
        domain_priorities.insert(
            TypedDomain::ServiceDomain(causality_types::primitive::ids::DomainId::new([0u8; 32])), 
            8
        );
        
        Self {
            config: StrategyConfiguration {
                strategy_id: Str::from("priority_based"),
                parameters: HashMap::new(),
                enabled_domains: vec![],
                priority: 8,
                max_evaluation_time_ms: 2000,
                version: 1,
                updated_at: Timestamp::now(),
            },
            metrics: StrategyMetrics {
                strategy_id: Str::from("priority_based"),
                total_evaluations: 0,
                successful_evaluations: 0,
                avg_evaluation_time_ms: 0.0,
                avg_plan_score: 0.0,
                plans_selected: 0,
                plans_executed_successfully: 0,
                resource_consumption: ResourceUsageEstimate::default(),
                domain_performance: HashMap::new(),
                last_updated: Timestamp::now(),
            },
            domain_priorities,
            prefer_current_domain: true,
        }
    }
    
    /// Calculate priority score for a plan
    fn calculate_priority_score(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> f64 {
        let mut score = 0.0;
        
        // Base score from domain priority
        let domain_priority = self.domain_priorities.get(&plan.target_typed_domain).copied().unwrap_or(5);
        score += (domain_priority as f64) / 10.0; // Normalize to 0-1
        
        // Bonus for current domain if preferred
        if self.prefer_current_domain && plan.target_typed_domain == context.current_typed_domain {
            score += 0.2;
        }
        
        // Penalty for complex ProcessDataflowBlock orchestration
        let dataflow_complexity_penalty = (plan.dataflow_steps.len() as f64) * 0.05;
        score -= dataflow_complexity_penalty;
        
        // Bonus for fewer resource transfers (simpler execution)
        let transfer_bonus = 1.0 / (1.0 + plan.resource_transfers.len() as f64);
        score += transfer_bonus * 0.1;
        
        score.max(0.0).min(1.0)
    }
}

impl OptimizationStrategy for PriorityBasedStrategy {
    fn strategy_id(&self) -> &str {
        "priority_based"
    }
    
    fn strategy_name(&self) -> &str {
        "Priority Based Strategy"
    }
    
    fn description(&self) -> &str {
        "Optimizes based on intent priorities and TypedDomain preferences, favoring simpler execution paths"
    }
    
    fn propose(&self, context: &OptimizationContext) -> Result<Vec<ScoredPlan>> {
        let mut plans = Vec::new();
        
        // Prioritize current domain first
        let mut domains_to_try = vec![context.current_typed_domain.clone()];
        for domain in &context.available_typed_domains {
            if domain != &context.current_typed_domain {
                domains_to_try.push(domain.clone());
            }
        }
        
        for domain in domains_to_try {
            let plan = ResolutionPlan {
                plan_id: EntityId::new(rand::random()),
                intent_bundles: context.pending_intents.clone(),
                effect_sequence: vec![],
                dataflow_steps: vec![], // Keep simple for priority-based approach
                resource_transfers: vec![],
                target_typed_domain: domain.clone(),
                estimated_cost: 800, // Lower cost estimate for priority approach
                estimated_time_ms: 3000, // Faster execution estimate
                metadata: HashMap::new(),
            };
            
            let priority_score = self.calculate_priority_score(&plan, context);
            
            let scored_plan = ScoredPlan {
                plan,
                overall_score: priority_score,
                cost_efficiency_score: 0.8, // Placeholder
                time_efficiency_score: 0.9, // Priority strategy aims for speed
                resource_utilization_score: 0.7, // Placeholder
                domain_compatibility_score: if domain == context.current_typed_domain { 1.0 } else { 0.6 },
                strategy_name: Str::from(self.strategy_name()),
                evaluated_at: Timestamp::now(),
            };
            
            plans.push(scored_plan);
        }
        
        // Sort by priority score
        plans.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(plans)
    }
    
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan> {
        let priority_score = self.calculate_priority_score(plan, context);
        
        Ok(ScoredPlan {
            plan: plan.clone(),
            overall_score: priority_score,
            cost_efficiency_score: 0.8,
            time_efficiency_score: 0.9,
            resource_utilization_score: 0.7,
            domain_compatibility_score: if plan.target_typed_domain == context.current_typed_domain { 1.0 } else { 0.6 },
            strategy_name: Str::from(self.strategy_name()),
            evaluated_at: Timestamp::now(),
        })
    }
    
    fn supports_typed_domain(&self, _domain: &TypedDomain) -> bool {
        true // Priority-based strategy works with all domains
    }
    
    fn get_configuration(&self) -> StrategyConfiguration {
        self.config.clone()
    }
    
    fn update_configuration(&mut self, config: StrategyConfiguration) -> Result<()> {
        // Update prefer_current_domain if provided
        if let Some(ConfigurationValue::Boolean(prefer)) = config.parameters.get(&Str::from("prefer_current_domain")) {
            self.prefer_current_domain = *prefer;
        }
        
        self.config = config;
        Ok(())
    }
    
    fn get_metrics(&self) -> StrategyMetrics {
        self.metrics.clone()
    }
    
    fn reset(&mut self) {
        self.metrics.total_evaluations = 0;
        self.metrics.successful_evaluations = 0;
        self.metrics.avg_evaluation_time_ms = 0.0;
        self.metrics.avg_plan_score = 0.0;
        self.metrics.plans_selected = 0;
        self.metrics.plans_executed_successfully = 0;
        self.metrics.domain_performance.clear();
        self.metrics.last_updated = Timestamp::now();
    }
} 