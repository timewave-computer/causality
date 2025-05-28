//! ProcessDataflow orchestration optimization strategy
//!
//! This strategy focuses on optimizing ProcessDataflowBlock orchestration,
//! parameter generation, and step decision-making across TypedDomains.

use super::super::optimization::{OptimizationStrategy, OptimizationContext};
use crate::optimization::evaluation::{StrategyConfiguration, StrategyMetrics, ResourceUsageEstimate};
use anyhow::Result;
use causality_types::{
    core::{
        id::{EntityId},
        str::Str,
        time::Timestamp,
    },
    graph::{
        optimization::{
            ResolutionPlan, ScoredPlan, TypedDomain, DataflowOrchestrationStep,
        },
    },
};
use causality_types::AsIdConverter; // Added import
use std::collections::HashMap;

/// Strategy that focuses on optimizing ProcessDataflowBlock orchestration
pub struct ProcessDataflowOrchestrationStrategy {
    /// Strategy configuration
    config: StrategyConfiguration,
    
    /// Performance metrics
    metrics: StrategyMetrics,
    
    /// Maximum dataflow steps to consider
    max_dataflow_steps: usize,
    
    /// Prefer parallel vs sequential execution
    _prefer_parallel_execution: bool,
}

impl Default for ProcessDataflowOrchestrationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessDataflowOrchestrationStrategy {
    /// Create a new ProcessDataflow orchestration strategy
    pub fn new() -> Self {
        Self {
            config: StrategyConfiguration {
                strategy_id: Str::from("dataflow_orchestration"),
                parameters: HashMap::new(),
                enabled_domains: Vec::new(),
                priority: 12,
                max_evaluation_time_ms: 4000,
                version: 1,
                updated_at: Timestamp::now(),
            },
            metrics: StrategyMetrics {
                strategy_id: Str::from("dataflow_orchestration"),
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
            max_dataflow_steps: 8,
            _prefer_parallel_execution: true,
        }
    }
    
    /// Generate optimized dataflow steps for the given context
    fn generate_dataflow_steps(&self, context: &OptimizationContext) -> Vec<DataflowOrchestrationStep> {
        let mut steps = Vec::new();
        
        // Look for available ProcessDataflowDefinitions
        for _df_def in context.dataflow_definitions.values() {
            if steps.len() >= 10 { // Use a reasonable default limit
                break;
            }
            
            // Create an initiation step
            steps.push(DataflowOrchestrationStep {
                step_id: EntityId::default(), // TODO: Replace with proper unique ID generation
                step_type: Str::from("InitiateDataflow"),
                required_resources: Vec::new(),
                produced_resources: Vec::new(),
                dependencies: Vec::new(),
                estimated_duration_ms: 0, // Added field
            });
        }
        
        // Add advancement steps for active instances
        for _instance_state in context.dataflow_instances.values() {
            if steps.len() >= 10 { // Use a reasonable default limit
                break;
            }
            
            steps.push(DataflowOrchestrationStep {
                step_id: EntityId::default(), // TODO: Replace with proper unique ID generation
                step_type: Str::from("AdvanceDataflow"),
                required_resources: Vec::new(),
                produced_resources: Vec::new(),
                dependencies: Vec::new(),
                estimated_duration_ms: 0, // Added field
            });
        }
        
        steps
    }
    
    /// Calculate orchestration efficiency score
    fn calculate_orchestration_score(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> f64 {
        let mut score = 0.0;
        
        // Base score from dataflow utilization
        let dataflow_utilization = if context.dataflow_definitions.is_empty() {
            1.0 // No dataflows available, so not using them is optimal
        } else {
            plan.dataflow_steps.len() as f64 / context.dataflow_definitions.len() as f64
        };
        
        score += dataflow_utilization * 0.4;
        
        // Efficiency bonus for reasonable number of steps
        let step_efficiency = if plan.dataflow_steps.len() <= self.max_dataflow_steps {
            1.0 - (plan.dataflow_steps.len() as f64 / self.max_dataflow_steps as f64) * 0.3
        } else {
            0.5 // Penalty for too many steps
        };
        
        score += step_efficiency * 0.3;
        
        // Domain compatibility bonus
        let domain_bonus = match plan.target_typed_domain.domain_type.as_str() {
            "verifiable" => 0.2, // Good for deterministic dataflows
            "service" => 0.1,   // Less ideal but workable
            _ => 0.05, // Default case
        };
        
        score += domain_bonus;
        
        // Resource efficiency
        let resource_efficiency = 1.0 / (1.0 + plan.resource_transfers.len() as f64 * 0.1);
        score += resource_efficiency * 0.1;
        
        score.clamp(0.0, 1.0)
    }
}

impl OptimizationStrategy for ProcessDataflowOrchestrationStrategy {
    fn strategy_id(&self) -> &str {
        "dataflow_orchestration"
    }
    
    fn strategy_name(&self) -> &str {
        "ProcessDataflow Orchestration Strategy"
    }
    
    fn description(&self) -> &str {
        "Focuses on optimizing ProcessDataflowBlock orchestration, parameter generation, and step decision-making"
    }
    
    fn propose(&self, context: &OptimizationContext) -> Result<Vec<ScoredPlan>> {
        let mut plans = Vec::new();
        
        for domain in &context.available_typed_domains {
            let dataflow_steps = self.generate_dataflow_steps(context);
            
            let plan = ResolutionPlan {
                plan_id: EntityId::new(rand::random()),
                intent_bundles: context.pending_intents.iter().map(|id| id.to_id()).collect(), // Re-applied AsIdConverter import and EntityId to ExprId conversion
                effect_sequence: vec![],
                dataflow_steps,
                resource_transfers: vec![],
                target_typed_domain: domain.clone(),
                estimated_cost: 1200, // Higher cost due to orchestration complexity
                estimated_time_ms: 6000, // More time for orchestration
                metadata: HashMap::new(),
            };
            
            let orchestration_score = self.calculate_orchestration_score(&plan, context);
            
            let scored_plan = ScoredPlan {
                plan,
                overall_score: orchestration_score,
                cost_efficiency_score: 0.7, // Lower due to orchestration overhead
                time_efficiency_score: 0.6, // Lower due to coordination time
                resource_utilization_score: 0.9, // High due to dataflow optimization
                domain_compatibility_score: match domain.domain_type.as_str() {
                    "verifiable" => 0.9,
                    "service" => 0.7,
                    _ => 0.5,
                },
                strategy_name: Str::from(self.strategy_name()),
                evaluated_at: Timestamp::now(),
            };
            
            plans.push(scored_plan);
        }
        
        // Sort by orchestration score
        plans.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(plans)
    }
    
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan> {
        let orchestration_score = self.calculate_orchestration_score(plan, context);
        
        Ok(ScoredPlan {
            plan: plan.clone(),
            overall_score: orchestration_score,
            cost_efficiency_score: 0.7,
            time_efficiency_score: 0.6,
            resource_utilization_score: 0.9,
            domain_compatibility_score: match plan.target_typed_domain.domain_type.as_str() {
                "verifiable" => 0.9,
                "service" => 0.7,
                _ => 0.5,
            },
            strategy_name: Str::from(self.strategy_name()),
            evaluated_at: Timestamp::now(),
        })
    }
    
    fn supports_typed_domain(&self, domain: &TypedDomain) -> bool {
        // ProcessDataflow orchestration works better with VerifiableDomain but supports both
        match domain.domain_type.as_str() {
            "verifiable" => true,
            "service" => true,
            _ => false, // Only support known domain types
        }
    }
    
    fn get_configuration(&self) -> crate::optimization::evaluation::EvaluationConfig {
        // Convert our internal config to EvaluationConfig
        crate::optimization::evaluation::EvaluationConfig {
            max_evaluation_time_ms: self.config.max_evaluation_time_ms,
            max_concurrent_evaluations: 4,
            enable_caching: true,
            cache_expiration_ms: 3_600_000,
            scoring_weights: crate::optimization::evaluation::ScoringWeights::default(),
            domain_parameters: std::collections::HashMap::new(),
            enable_detailed_analysis: false,
        }
    }
    
    fn update_configuration(&mut self, config: crate::optimization::evaluation::EvaluationConfig) -> Result<()> {
        self.config.max_evaluation_time_ms = config.max_evaluation_time_ms;
        Ok(())
    }
    
    fn get_metrics(&self) -> crate::optimization::evaluation::EvaluationMetrics {
        // Convert our internal metrics to EvaluationMetrics
        crate::optimization::evaluation::EvaluationMetrics {
            total_evaluations: self.metrics.total_evaluations,
            successful_evaluations: self.metrics.successful_evaluations,
            failed_evaluations: 0, // Not tracked in our internal metrics
            avg_evaluation_time_ms: self.metrics.avg_evaluation_time_ms,
            cache_hit_rate: 0.0, // Not tracked in our internal metrics
            domain_performance: std::collections::HashMap::new(),
            last_updated: self.metrics.last_updated,
        }
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