//! Expression-based optimization strategy
//!
//! This strategy uses TEL expressions for dynamic optimization decisions,
//! supporting custom scoring, filtering, and domain compatibility logic.

use super::super::optimization::{OptimizationStrategy, OptimizationContext};
use crate::optimization::evaluation::{StrategyConfiguration, StrategyMetrics, ResourceUsageEstimate};
use anyhow::Result;
use causality_types::AsIdConverter;
use causality_types::{
    core::{
        id::{EntityId, ExprId},
        str::Str,
        time::Timestamp,
    },
    graph::{
        optimization::{
            ResolutionPlan, ScoredPlan, TypedDomain, DataflowOrchestrationStep,
        },
    },
};
use std::collections::HashMap;

/// Strategy that uses TEL expressions for dynamic optimization decisions
pub struct ExpressionBasedStrategy {
    /// Strategy configuration
    config: StrategyConfiguration,
    
    /// Performance metrics
    metrics: StrategyMetrics,
    
    /// Expression for scoring plans (returns f64 score)
    scoring_expression: Option<ExprId>,
    
    /// Expression for filtering plans (returns boolean)
    filter_expression: Option<ExprId>,
    
    /// Expression for domain compatibility (returns f64 score)
    domain_compatibility_expression: Option<ExprId>,
    
    /// Default scoring weights when expressions are not provided
    default_weights: HashMap<String, f64>,
}

impl Default for ExpressionBasedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionBasedStrategy {
    /// Create a new expression-based strategy
    pub fn new() -> Self {
        let mut default_weights = HashMap::new();
        default_weights.insert("cost_efficiency".to_string(), 0.3);
        default_weights.insert("time_efficiency".to_string(), 0.25);
        default_weights.insert("resource_utilization".to_string(), 0.2);
        default_weights.insert("domain_compatibility".to_string(), 0.15);
        default_weights.insert("dataflow_complexity".to_string(), 0.1);
        
        Self {
            config: StrategyConfiguration {
                strategy_id: Str::from("expression_based"),
                parameters: HashMap::new(),
                enabled_domains: vec![],
                priority: 15,
                max_evaluation_time_ms: 5000,
                version: 1,
                updated_at: Timestamp::now(),
            },
            metrics: StrategyMetrics {
                strategy_id: Str::from("expression_based"),
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
            scoring_expression: None,
            filter_expression: None,
            domain_compatibility_expression: None,
            default_weights,
        }
    }
    
    /// Set the scoring expression for plan evaluation
    pub fn with_scoring_expression(mut self, expr_id: ExprId) -> Self {
        self.scoring_expression = Some(expr_id);
        self
    }
    
    /// Set the filter expression for plan filtering
    pub fn with_filter_expression(mut self, expr_id: ExprId) -> Self {
        self.filter_expression = Some(expr_id);
        self
    }
    
    /// Set the domain compatibility expression
    pub fn with_domain_compatibility_expression(mut self, expr_id: ExprId) -> Self {
        self.domain_compatibility_expression = Some(expr_id);
        self
    }
    
    /// Calculate expression-based score for a plan
    fn calculate_expression_score(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> f64 {
        // For now, implement a sophisticated default scoring algorithm
        // In a full implementation, this would evaluate the TEL expressions
        
        let mut score = 0.0;
        
        // Cost efficiency component
        let cost_efficiency = if context.available_resources.values().sum::<u64>() > 0 {
            let cost_ratio = plan.estimated_cost as f64 / context.available_resources.values().sum::<u64>() as f64;
            1.0 / (1.0 + cost_ratio)
        } else {
            0.5
        };
        score += cost_efficiency * self.default_weights.get("cost_efficiency").copied().unwrap_or(0.3);
        
        // Time efficiency component
        let time_efficiency = if plan.estimated_time_ms > 0 {
            // Prefer faster execution, normalize to 0-1 range
            
            (10000.0 - plan.estimated_time_ms as f64).max(0.0) / 10000.0
        } else {
            0.5
        };
        score += time_efficiency * self.default_weights.get("time_efficiency").copied().unwrap_or(0.25);
        
        // Resource utilization component
        let resource_utilization = if plan.resource_transfers.is_empty() {
            0.9 // High score for no transfers needed
        } else {
            // Penalty for complex resource transfers
            (1.0 / (1.0 + plan.resource_transfers.len() as f64 * 0.2)).max(0.1)
        };
        score += resource_utilization * self.default_weights.get("resource_utilization").copied().unwrap_or(0.2);
        
        // Domain compatibility component
        let domain_compatibility = if plan.target_typed_domain == context.current_typed_domain {
            1.0 // Perfect compatibility
        } else {
            // Cross-domain penalty, but still viable
            match (context.current_typed_domain.domain_type.as_str(), plan.target_typed_domain.domain_type.as_str()) {
                ("verifiable", "service") => 0.6,
                ("service", "verifiable") => 0.7,
                _ => 0.8,
            }
        };
        score += domain_compatibility * self.default_weights.get("domain_compatibility").copied().unwrap_or(0.15);
        
        // ProcessDataflowBlock complexity component
        let dataflow_complexity = if plan.dataflow_steps.is_empty() {
            0.8 // Good score for simple execution
        } else {
            // Score based on reasonable complexity
            let complexity_ratio = plan.dataflow_steps.len() as f64 / 10.0; // Normalize to 10 steps
            (1.0 - complexity_ratio).max(0.1)
        };
        score += dataflow_complexity * self.default_weights.get("dataflow_complexity").copied().unwrap_or(0.1);
        
        score.clamp(0.0, 1.0)
    }
    
    /// Check if plan passes filter expression
    fn passes_filter(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> bool {
        // For now, implement basic filtering logic
        // In a full implementation, this would evaluate the filter expression
        
        // Basic filters:
        // 1. Plan must be affordable
        let affordable = plan.estimated_cost <= context.available_resources.values().sum::<u64>();
        
        // 2. Plan must not be too complex
        let not_too_complex = plan.dataflow_steps.len() <= 15;
        
        // 3. Plan must have reasonable time estimate
        let reasonable_time = plan.estimated_time_ms <= 30000; // 30 seconds max
        
        affordable && not_too_complex && reasonable_time
    }
    
    /// Generate plans using expression-based logic
    fn generate_expression_plans(&self, context: &OptimizationContext) -> Vec<ResolutionPlan> {
        let mut plans = Vec::new();
        
        // Generate plans for each available domain
        for domain in &context.available_typed_domains {
            // Basic plan
            let basic_plan = ResolutionPlan {
                plan_id: EntityId::new(rand::random()),
                intent_bundles: context.pending_intents.iter().map(|id| {
                    let expr_id: ExprId = id.to_id();
                    expr_id
                }).collect(),
                effect_sequence: vec![],
                dataflow_steps: vec![],
                resource_transfers: vec![],
                target_typed_domain: domain.clone(),
                estimated_cost: self.estimate_basic_cost(domain, context),
                estimated_time_ms: self.estimate_basic_time(domain, context),
                metadata: HashMap::new(),
            };
            plans.push(basic_plan);
            
            // ProcessDataflow-enhanced plan if dataflows are available
            if !context.dataflow_definitions.is_empty() {
                let dataflow_steps = self.generate_smart_dataflow_steps(context, domain);
                let dataflow_plan = ResolutionPlan {
                    plan_id: EntityId::new(rand::random()),
                    intent_bundles: context.pending_intents.iter().map(|id| {
                        let expr_id: ExprId = id.to_id();
                        expr_id
                    }).collect(),
                    effect_sequence: vec![],
                    dataflow_steps,
                    resource_transfers: vec![],
                    target_typed_domain: domain.clone(),
                    estimated_cost: self.estimate_dataflow_cost(domain, context),
                    estimated_time_ms: self.estimate_dataflow_time(domain, context),
                    metadata: HashMap::new(),
                };
                plans.push(dataflow_plan);
            }
        }
        
        plans
    }
    
    /// Generate smart dataflow steps based on context
    fn generate_smart_dataflow_steps(&self, context: &OptimizationContext, _domain: &TypedDomain) -> Vec<DataflowOrchestrationStep> {
        let mut steps = Vec::new();
        
        // Prioritize dataflows that match the target domain characteristics
        let mut sorted_dataflows: Vec<_> = context.dataflow_definitions.iter().collect();
        sorted_dataflows.sort_by_key(|(_, _def)| {
            // In a real implementation, we'd check the dataflow definition's domain preferences
            // For now, use a simple heuristic
            match _domain.domain_type.as_str() {
                "verifiable" => 0, // Prefer deterministic dataflows
                "service" => 1,   // Prefer service-oriented dataflows
                _ => 2, // Default case
            }
        });
        
        // Add initiation steps for top dataflows
        for (_df_id, _df_def) in sorted_dataflows.iter().take(3) {
            steps.push(DataflowOrchestrationStep {
                step_id: EntityId::default(), // TODO: Replace with proper unique ID generation
                step_type: "InitiateDataflow".into(),
                required_resources: Vec::new(),
                produced_resources: Vec::new(),
                estimated_duration_ms: 0,
                dependencies: Vec::new(),
            });
        }
        
        // Add advancement steps for active instances
        for _instance_state in context.dataflow_instances.values() {
            if steps.len() >= 10 { // Use a reasonable default limit
                break;
            }
            
            steps.push(DataflowOrchestrationStep {
                step_id: EntityId::default(), // TODO: Replace with proper unique ID generation
                step_type: "AdvanceDataflow".into(),
                required_resources: Vec::new(),
                produced_resources: Vec::new(),
                estimated_duration_ms: 0,
                dependencies: Vec::new(),
            });
        }
        
        steps
    }
    
    /// Estimate cost for basic execution
    fn estimate_basic_cost(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        let base_cost = 500u64;
        let intent_count = context.pending_intents.len() as u64;
        let domain_multiplier = match domain.domain_type.as_str() {
            "verifiable" => 1.1,
            "service" => 0.9,
            _ => 1.0,
        };
        
        ((base_cost * intent_count) as f64 * domain_multiplier) as u64
    }
    
    /// Estimate cost for dataflow execution
    fn estimate_dataflow_cost(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        let base_cost = self.estimate_basic_cost(domain, context);
        let dataflow_overhead = context.dataflow_definitions.len() as u64 * 200;
        base_cost + dataflow_overhead
    }
    
    /// Estimate time for basic execution
    fn estimate_basic_time(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        let base_time = 2000u64; // 2 seconds
        let intent_count = context.pending_intents.len() as u64;
        let domain_multiplier = match domain.domain_type.as_str() {
            "verifiable" => 1.3, // ZK proofs take longer
            "service" => 0.8,   // Service calls can be faster
            _ => 1.0,
        };
        
        ((base_time * intent_count) as f64 * domain_multiplier) as u64
    }
    
    /// Estimate time for dataflow execution
    fn estimate_dataflow_time(&self, domain: &TypedDomain, context: &OptimizationContext) -> u64 {
        let base_time = self.estimate_basic_time(domain, context);
        let dataflow_overhead = context.dataflow_definitions.len() as u64 * 500;
        base_time + dataflow_overhead
    }
}

impl OptimizationStrategy for ExpressionBasedStrategy {
    fn strategy_id(&self) -> &str {
        "expression_based"
    }
    
    fn strategy_name(&self) -> &str {
        "Expression Based Strategy"
    }
    
    fn description(&self) -> &str {
        "Uses TEL expressions for dynamic optimization decisions, supporting custom scoring, filtering, and domain compatibility logic"
    }
    
    fn propose(&self, context: &OptimizationContext) -> Result<Vec<ScoredPlan>> {
        let plans = self.generate_expression_plans(context);
        let mut scored_plans = Vec::new();
        
        for plan in plans {
            // Apply filter if configured
            if !self.passes_filter(&plan, context) {
                continue;
            }
            
            // Calculate score using expression or default logic
            let overall_score = self.calculate_expression_score(&plan, context);
            
            let scored_plan = ScoredPlan {
                plan,
                overall_score,
                cost_efficiency_score: overall_score * 0.4, // Weighted component
                time_efficiency_score: overall_score * 0.3,
                resource_utilization_score: overall_score * 0.2,
                domain_compatibility_score: overall_score * 0.1,
                strategy_name: Str::from(self.strategy_name()),
                evaluated_at: Timestamp::now(),
            };
            
            scored_plans.push(scored_plan);
        }
        
        // Sort by overall score
        scored_plans.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scored_plans)
    }
    
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan> {
        let overall_score = self.calculate_expression_score(plan, context);
        
        Ok(ScoredPlan {
            plan: plan.clone(),
            overall_score,
            cost_efficiency_score: overall_score * 0.4,
            time_efficiency_score: overall_score * 0.3,
            resource_utilization_score: overall_score * 0.2,
            domain_compatibility_score: overall_score * 0.1,
            strategy_name: Str::from(self.strategy_name()),
            evaluated_at: Timestamp::now(),
        })
    }
    
    fn supports_typed_domain(&self, _domain: &TypedDomain) -> bool {
        true // Expression-based strategy is flexible and supports all domains
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