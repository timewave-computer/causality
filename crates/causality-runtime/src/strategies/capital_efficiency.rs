//! Capital efficiency optimization strategy
//!
//! This strategy optimizes for capital efficiency across TypedDomains, considering
//! domain-specific cost multipliers and ProcessDataflowBlock complexity.

use crate::optimization::{OptimizationStrategy, OptimizationContext};
use anyhow::{anyhow, Result};
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
        strategy::{StrategyConfiguration, StrategyMetrics},
    },
};
use std::collections::HashMap;

/// Strategy that optimizes for capital efficiency across TypedDomains
pub struct CapitalEfficiencyStrategy {
    /// Strategy configuration
    config: StrategyConfiguration,
    
    /// Performance metrics
    metrics: StrategyMetrics,
    
    /// TypedDomain-specific cost multipliers
    domain_cost_multipliers: HashMap<TypedDomain, f64>,
    
    /// Minimum acceptable efficiency threshold
    min_efficiency_threshold: f64,
}

impl CapitalEfficiencyStrategy {
    /// Create a new capital efficiency strategy
    pub fn new() -> Self {
        let mut domain_cost_multipliers = HashMap::new();
        // VerifiableDomain typically has higher computational costs but lower trust costs
        domain_cost_multipliers.insert(
            TypedDomain::VerifiableDomain(causality_types::primitive::ids::DomainId::new([0u8; 32])), 
            1.2
        );
        // ServiceDomain has variable costs depending on external services
        domain_cost_multipliers.insert(
            TypedDomain::ServiceDomain(causality_types::primitive::ids::DomainId::new([0u8; 32])), 
            0.8
        );
        
        Self {
            config: StrategyConfiguration {
                strategy_id: Str::from("capital_efficiency"),
                parameters: HashMap::new(),
                enabled_domains: vec![],
                priority: 10,
                max_evaluation_time_ms: 3000,
                version: 1,
                updated_at: Timestamp::now(),
            },
            metrics: StrategyMetrics {
                strategy_id: Str::from("capital_efficiency"),
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
            domain_cost_multipliers,
            min_efficiency_threshold: 0.6,
        }
    }
    
    /// Calculate cost efficiency score for a plan
    fn calculate_cost_efficiency(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> f64 {
        let base_cost = plan.estimated_cost as f64;
        
        // Apply domain-specific cost multiplier
        let domain_multiplier = self.domain_cost_multipliers
            .get(&plan.target_typed_domain)
            .copied()
            .unwrap_or(1.0);
        
        let adjusted_cost = base_cost * domain_multiplier;
        
        // Calculate efficiency based on available resources
        let available_budget = context.available_resources.values().sum::<u64>() as f64;
        
        if available_budget == 0.0 {
            return 0.0;
        }
        
        // Efficiency is inversely related to cost ratio
        let cost_ratio = adjusted_cost / available_budget;
        let efficiency = 1.0 / (1.0 + cost_ratio);
        
        // Apply ProcessDataflowBlock complexity penalty
        let dataflow_penalty = plan.dataflow_steps.len() as f64 * 0.05;
        let final_efficiency = (efficiency - dataflow_penalty).max(0.0);
        
        final_efficiency
    }

    /// Estimate cost for a specific domain
    fn estimate_domain_cost(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        let base_cost = 1000u64; // Base cost
        let multiplier = self.domain_cost_multipliers.get(domain).copied().unwrap_or(1.0);
        let intent_count = context.pending_intents.len() as u64;
        
        (base_cost as f64 * multiplier * intent_count as f64) as u64
    }
    
    /// Estimate execution time for a specific domain
    fn estimate_domain_time(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        match domain {
            TypedDomain::VerifiableDomain(_) => {
                // ZK proof generation takes longer
                let base_time = 5000u64; // 5 seconds base
                let intent_count = context.pending_intents.len() as u64;
                base_time * intent_count
            },
            TypedDomain::ServiceDomain(_) => {
                // Service calls are typically faster but variable
                let base_time = 1000u64; // 1 second base
                let intent_count = context.pending_intents.len() as u64;
                base_time * intent_count
            },
        }
    }
}

impl OptimizationStrategy for CapitalEfficiencyStrategy {
    fn strategy_id(&self) -> &str {
        "capital_efficiency"
    }
    
    fn strategy_name(&self) -> &str {
        "Capital Efficiency Strategy"
    }
    
    fn description(&self) -> &str {
        "Optimizes for capital efficiency across TypedDomains, considering domain-specific cost multipliers and ProcessDataflowBlock complexity"
    }
    
    fn propose(&self, context: &OptimizationContext) -> Result<Vec<ScoredPlan>> {
        let mut plans = Vec::new();
        
        // Generate plans for each available domain
        for domain in &context.available_typed_domains {
            // Create a basic resolution plan for this domain
            let plan = ResolutionPlan {
                plan_id: EntityId::new(rand::random()),
                intent_bundles: context.pending_intents.clone(),
                effect_sequence: vec![], // Would be populated by actual planning logic
                dataflow_steps: vec![], // Minimal dataflow for efficiency
                resource_transfers: vec![],
                target_typed_domain: domain.clone(),
                estimated_cost: self.estimate_domain_cost(domain, context),
                estimated_time_ms: self.estimate_domain_time(domain, context),
                metadata: HashMap::new(),
            };
            
            // Score the plan
            let efficiency_score = self.calculate_cost_efficiency(&plan, context);
            
            if efficiency_score >= self.min_efficiency_threshold {
                let scored_plan = ScoredPlan {
                    plan,
                    overall_score: efficiency_score,
                    cost_efficiency_score: efficiency_score,
                    time_efficiency_score: 0.7, // Placeholder
                    resource_utilization_score: 0.8, // Placeholder
                    domain_compatibility_score: if domain == &context.current_typed_domain { 1.0 } else { 0.7 },
                    strategy_name: Str::from(self.strategy_name()),
                    evaluated_at: Timestamp::now(),
                };
                
                plans.push(scored_plan);
            }
        }
        
        // Sort by efficiency score
        plans.sort_by(|a, b| b.cost_efficiency_score.partial_cmp(&a.cost_efficiency_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(plans)
    }
    
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan> {
        let efficiency_score = self.calculate_cost_efficiency(plan, context);
        
        Ok(ScoredPlan {
            plan: plan.clone(),
            overall_score: efficiency_score,
            cost_efficiency_score: efficiency_score,
            time_efficiency_score: 0.7, // Would be calculated properly
            resource_utilization_score: 0.8, // Would be calculated properly
            domain_compatibility_score: if plan.target_typed_domain == context.current_typed_domain { 1.0 } else { 0.7 },
            strategy_name: Str::from(self.strategy_name()),
            evaluated_at: Timestamp::now(),
        })
    }
    
    fn supports_typed_domain(&self, _domain: &TypedDomain) -> bool {
        true // Capital efficiency applies to all domains
    }
    
    fn get_configuration(&self) -> StrategyConfiguration {
        self.config.clone()
    }
    
    fn update_configuration(&mut self, config: StrategyConfiguration) -> Result<()> {
        if config.strategy_id != self.config.strategy_id {
            return Err(anyhow!("Strategy ID mismatch"));
        }
        self.config = config;
        Ok(())
    }
    
    fn get_metrics(&self) -> StrategyMetrics {
        self.metrics.clone()
    }
    
    fn reset(&mut self) {
        self.metrics = StrategyMetrics {
            strategy_id: Str::from("capital_efficiency"),
            total_evaluations: 0,
            successful_evaluations: 0,
            avg_evaluation_time_ms: 0.0,
            avg_plan_score: 0.0,
            plans_selected: 0,
            plans_executed_successfully: 0,
            resource_consumption: ResourceUsageEstimate::default(),
            domain_performance: HashMap::new(),
            last_updated: Timestamp::now(),
        };
    }
} 