//! Plan evaluation framework for optimization strategies
//!
//! This module provides utilities for scoring, ranking, and analyzing resolution plans,
//! considering TypedDomain constraints and ProcessDataflowBlock orchestration complexity.

use super::{OptimizationContext, OptimizationStrategy};
use anyhow::Result;
use causality_types::{
    core::{
        str::Str,
        time::Timestamp,
    },
    tel::optimization::{ResolutionPlan, ScoredPlan, TypedDomain},
};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Plan Evaluator
//-----------------------------------------------------------------------------

/// Framework for evaluating and comparing resolution plans
pub struct PlanEvaluator {
    /// Evaluation configuration
    config: EvaluationConfig,
    
    /// Performance metrics collection
    metrics: EvaluationMetrics,
    
    /// Cached evaluation results
    evaluation_cache: HashMap<String, CachedEvaluationResult>,
}

/// Configuration for plan evaluation
#[derive(Debug, Clone)]
pub struct EvaluationConfig {
    /// Maximum evaluation time per plan (milliseconds)
    pub max_evaluation_time_ms: u64,
    
    /// Maximum number of plans to evaluate concurrently
    pub max_concurrent_evaluations: usize,
    
    /// Enable caching of evaluation results
    pub enable_caching: bool,
    
    /// Cache expiration time (milliseconds)
    pub cache_expiration_ms: u64,
    
    /// Weights for different scoring criteria
    pub scoring_weights: ScoringWeights,
    
    /// TypedDomain-specific evaluation parameters
    pub domain_parameters: HashMap<TypedDomain, DomainEvaluationParameters>,
    
    /// Enable detailed performance analysis
    pub enable_detailed_analysis: bool,
}

/// Weights for different aspects of plan scoring
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Weight for cost efficiency (0.0 to 1.0)
    pub cost_efficiency: f64,
    
    /// Weight for time efficiency (0.0 to 1.0)
    pub time_efficiency: f64,
    
    /// Weight for resource utilization (0.0 to 1.0)
    pub resource_utilization: f64,
    
    /// Weight for domain compatibility (0.0 to 1.0)
    pub domain_compatibility: f64,
    
    /// Weight for ProcessDataflowBlock orchestration complexity (0.0 to 1.0)
    pub dataflow_complexity: f64,
    
    /// Weight for risk assessment (0.0 to 1.0)
    pub risk_assessment: f64,
}

/// TypedDomain-specific evaluation parameters
#[derive(Debug, Clone)]
pub struct DomainEvaluationParameters {
    /// Cost multiplier for this domain
    pub cost_multiplier: f64,
    
    /// Time multiplier for this domain
    pub time_multiplier: f64,
    
    /// Complexity penalty for cross-domain operations
    pub cross_domain_penalty: f64,
    
    /// Preferred resource types for this domain
    pub preferred_resources: Vec<Str>,
    
    /// Maximum acceptable risk level (0.0 to 1.0)
    pub max_risk_level: f64,
}

/// Metrics collected during plan evaluation
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    /// Total plans evaluated
    pub total_evaluations: u64,
    
    /// Successful evaluations
    pub successful_evaluations: u64,
    
    /// Failed evaluations
    pub failed_evaluations: u64,
    
    /// Average evaluation time (milliseconds)
    pub avg_evaluation_time_ms: f64,
    
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    
    /// Evaluation performance by TypedDomain
    pub domain_performance: HashMap<TypedDomain, DomainEvaluationMetrics>,
    
    /// Last updated timestamp
    pub last_updated: Timestamp,
}

/// Evaluation metrics for a specific TypedDomain
#[derive(Debug, Clone)]
pub struct DomainEvaluationMetrics {
    /// Evaluations in this domain
    pub evaluations: u64,
    
    /// Average evaluation time for this domain
    pub avg_time_ms: f64,
    
    /// Average plan score for this domain
    pub avg_score: f64,
    
    /// Success rate for this domain
    pub success_rate: f64,
}

/// Cached evaluation result
#[derive(Debug, Clone)]
struct CachedEvaluationResult {
    /// The evaluation result
    result: EvaluationResult,
    
    /// Timestamp when cached
    cached_at: Timestamp,
    
    /// Cache key used
    cache_key: String,
}

/// Result of plan evaluation
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The evaluated plan
    pub plan: ResolutionPlan,
    
    /// Overall evaluation score (0.0 to 1.0)
    pub overall_score: f64,
    
    /// Detailed scoring breakdown
    pub score_breakdown: ScoreBreakdown,
    
    /// Evaluation metadata
    pub metadata: EvaluationMetadata,
    
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
}

/// Detailed breakdown of evaluation scores
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    /// Cost efficiency score (0.0 to 1.0)
    pub cost_efficiency: f64,
    
    /// Time efficiency score (0.0 to 1.0)
    pub time_efficiency: f64,
    
    /// Resource utilization score (0.0 to 1.0)
    pub resource_utilization: f64,
    
    /// Domain compatibility score (0.0 to 1.0)
    pub domain_compatibility: f64,
    
    /// ProcessDataflowBlock complexity score (0.0 to 1.0)
    pub dataflow_complexity: f64,
    
    /// Risk score (0.0 to 1.0, lower is better)
    pub risk_score: f64,
}

/// Metadata about the evaluation process
#[derive(Debug, Clone)]
pub struct EvaluationMetadata {
    /// Strategy that performed the evaluation
    pub strategy_id: String,
    
    /// Evaluation timestamp
    pub evaluated_at: Timestamp,
    
    /// Evaluation duration (milliseconds)
    pub evaluation_duration_ms: u64,
    
    /// TypedDomain context
    pub typed_domain: TypedDomain,
    
    /// Whether result was cached
    pub was_cached: bool,
    
    /// Additional metadata
    pub additional_data: HashMap<String, String>,
}

/// Risk assessment for a plan
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// Overall risk level (0.0 to 1.0)
    pub overall_risk: f64,
    
    /// Identified risk factors
    pub risk_factors: Vec<RiskFactor>,
    
    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
    
    /// Confidence in risk assessment (0.0 to 1.0)
    pub confidence: f64,
}

/// Individual risk factor
#[derive(Debug, Clone)]
pub struct RiskFactor {
    /// Risk factor type
    pub factor_type: String,
    
    /// Risk level for this factor (0.0 to 1.0)
    pub risk_level: f64,
    
    /// Description of the risk
    pub description: String,
    
    /// Impact if risk materializes
    pub impact: String,
}

impl PlanEvaluator {
    /// Create a new plan evaluator
    pub fn new(config: EvaluationConfig) -> Self {
        Self {
            config,
            metrics: EvaluationMetrics::default(),
            evaluation_cache: HashMap::new(),
        }
    }
    
    /// Evaluate a plan using the configured strategy
    pub fn evaluate_plan(
        &mut self,
        plan: &ResolutionPlan,
        context: &OptimizationContext,
        strategy: &dyn OptimizationStrategy,
    ) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = self.generate_cache_key(plan, context);
        let cached_result = if let Some(cached) = self.get_cached_result(&cache_key) {
            Some(cached.result.clone())
        } else {
            None
        };
        
        if let Some(cached) = cached_result {
            self.metrics.total_evaluations += 1;
            return Ok(cached);
        }
        
        // Perform evaluation
        let result = self.perform_evaluation(plan, context, strategy)?;
        
        // Update metrics
        let duration = start_time.elapsed().as_millis() as u64;
        self.update_metrics(duration, &context.current_typed_domain, true);
        
        // Cache result if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(plan, context);
            self.cache_result(cache_key, result.clone());
        }
        
        Ok(result)
    }
    
    /// Compare multiple plans and return them ranked by score
    pub fn compare_plans(
        &mut self,
        plans: &[ResolutionPlan],
        context: &OptimizationContext,
        strategy: &dyn OptimizationStrategy,
    ) -> Result<Vec<EvaluationResult>> {
        let mut results = Vec::new();
        
        for plan in plans {
            match self.evaluate_plan(plan, context, strategy) {
                Ok(result) => results.push(result),
                Err(e) => {
                    log::warn!("Failed to evaluate plan {}: {}", plan.plan_id, e);
                    self.update_metrics(0, &context.current_typed_domain, false);
                }
            }
        }
        
        // Sort by overall score (descending)
        results.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }
    
    /// Get evaluation metrics
    pub fn get_metrics(&self) -> &EvaluationMetrics {
        &self.metrics
    }
    
    /// Reset metrics and cache
    pub fn reset(&mut self) {
        self.metrics = EvaluationMetrics::default();
        self.evaluation_cache.clear();
    }
    
    /// Perform the actual evaluation logic
    fn perform_evaluation(
        &self,
        plan: &ResolutionPlan,
        context: &OptimizationContext,
        strategy: &dyn OptimizationStrategy,
    ) -> Result<EvaluationResult> {
        let mut score_breakdown = ScoreBreakdown {
            cost_efficiency: 0.0,
            time_efficiency: 0.0,
            resource_utilization: 0.0,
            domain_compatibility: 0.0,
            dataflow_complexity: 0.0,
            risk_score: 0.0,
        };
        
        // Evaluate cost efficiency
        score_breakdown.cost_efficiency = self.evaluate_cost_efficiency(plan, context)?;
        
        // Evaluate time efficiency
        score_breakdown.time_efficiency = self.evaluate_time_efficiency(plan, context)?;
        
        // Evaluate resource utilization
        score_breakdown.resource_utilization = self.evaluate_resource_utilization(plan, context)?;
        
        // Evaluate domain compatibility
        score_breakdown.domain_compatibility = self.evaluate_domain_compatibility(plan, context)?;
        
        // Evaluate ProcessDataflowBlock complexity
        score_breakdown.dataflow_complexity = self.evaluate_dataflow_complexity(plan, context)?;
        
        // Perform risk assessment
        let risk_assessment = self.assess_risk(plan, context)?;
        score_breakdown.risk_score = risk_assessment.overall_risk;
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(&score_breakdown);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(plan, &score_breakdown, context);
        
        Ok(EvaluationResult {
            plan: plan.clone(),
            overall_score,
            score_breakdown,
            metadata: EvaluationMetadata {
                strategy_id: strategy.strategy_id().to_string(),
                evaluated_at: Timestamp::now(),
                evaluation_duration_ms: 0, // Will be set by caller
                typed_domain: context.current_typed_domain.clone(),
                was_cached: false,
                additional_data: HashMap::new(),
            },
            recommendations,
            risk_assessment,
        })
    }
    
    /// Evaluate cost efficiency of a plan
    fn evaluate_cost_efficiency(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<f64> {
        // Simple cost efficiency based on estimated cost vs available resources
        let total_cost = plan.estimated_cost;
        let available_budget = context.available_resources.values().sum::<u64>();
        
        if available_budget == 0 {
            return Ok(0.0);
        }
        
        let efficiency = 1.0 - (total_cost as f64 / available_budget as f64).min(1.0);
        Ok(efficiency.max(0.0))
    }
    
    /// Evaluate time efficiency of a plan
    fn evaluate_time_efficiency(&self, plan: &ResolutionPlan, _context: &OptimizationContext) -> Result<f64> {
        // Simple time efficiency based on estimated execution time
        let max_acceptable_time = 60000; // 1 minute in milliseconds
        let efficiency = 1.0 - (plan.estimated_time_ms as f64 / max_acceptable_time as f64).min(1.0);
        Ok(efficiency.max(0.0))
    }
    
    /// Evaluate resource utilization efficiency
    fn evaluate_resource_utilization(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<f64> {
        // Evaluate how well the plan utilizes available resources
        let mut utilization_score = 0.0;
        let mut resource_count = 0;
        
        for transfer in &plan.resource_transfers {
            // Check if required resources are available
            // This is a simplified implementation
            utilization_score += 0.8; // Placeholder score
            resource_count += 1;
        }
        
        if resource_count == 0 {
            Ok(1.0) // No resource requirements = perfect utilization
        } else {
            Ok(utilization_score / resource_count as f64)
        }
    }
    
    /// Evaluate domain compatibility
    fn evaluate_domain_compatibility(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<f64> {
        // Check if plan's target domain matches current domain
        if plan.target_typed_domain == context.current_typed_domain {
            Ok(1.0)
        } else if context.is_domain_available(&plan.target_typed_domain) {
            Ok(0.7) // Available but not current domain
        } else {
            Ok(0.3) // Domain not available
        }
    }
    
    /// Evaluate ProcessDataflowBlock complexity
    fn evaluate_dataflow_complexity(&self, plan: &ResolutionPlan, _context: &OptimizationContext) -> Result<f64> {
        // Lower complexity is better (higher score)
        let complexity_factor = plan.dataflow_steps.len() as f64;
        let max_complexity = 10.0; // Arbitrary maximum
        
        let complexity_score = 1.0 - (complexity_factor / max_complexity).min(1.0);
        Ok(complexity_score.max(0.0))
    }
    
    /// Assess risk factors for a plan
    fn assess_risk(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut overall_risk = 0.0;
        
        // Risk from high cost
        if plan.estimated_cost > 1000000 { // Arbitrary threshold
            risk_factors.push(RiskFactor {
                factor_type: "high_cost".to_string(),
                risk_level: 0.7,
                description: "Plan has very high estimated cost".to_string(),
                impact: "May exceed budget constraints".to_string(),
            });
            overall_risk += 0.7;
        }
        
        // Risk from long execution time
        if plan.estimated_time_ms > 300000 { // 5 minutes
            risk_factors.push(RiskFactor {
                factor_type: "long_execution".to_string(),
                risk_level: 0.5,
                description: "Plan has long estimated execution time".to_string(),
                impact: "May cause timeouts or user dissatisfaction".to_string(),
            });
            overall_risk += 0.5;
        }
        
        // Risk from cross-domain operations
        if plan.target_typed_domain != context.current_typed_domain {
            risk_factors.push(RiskFactor {
                factor_type: "cross_domain".to_string(),
                risk_level: 0.4,
                description: "Plan requires cross-domain execution".to_string(),
                impact: "Additional complexity and potential failure points".to_string(),
            });
            overall_risk += 0.4;
        }
        
        // Normalize overall risk
        overall_risk = (overall_risk / risk_factors.len() as f64).min(1.0);
        
        Ok(RiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_strategies: vec![
                "Monitor execution closely".to_string(),
                "Have rollback plan ready".to_string(),
            ],
            confidence: 0.8,
        })
    }
    
    /// Calculate overall score from breakdown
    fn calculate_overall_score(&self, breakdown: &ScoreBreakdown) -> f64 {
        let weights = &self.config.scoring_weights;
        
        weights.cost_efficiency * breakdown.cost_efficiency +
        weights.time_efficiency * breakdown.time_efficiency +
        weights.resource_utilization * breakdown.resource_utilization +
        weights.domain_compatibility * breakdown.domain_compatibility +
        weights.dataflow_complexity * breakdown.dataflow_complexity +
        weights.risk_assessment * (1.0 - breakdown.risk_score) // Lower risk is better
    }
    
    /// Generate recommendations for plan improvement
    fn generate_recommendations(&self, plan: &ResolutionPlan, breakdown: &ScoreBreakdown, _context: &OptimizationContext) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if breakdown.cost_efficiency < 0.5 {
            recommendations.push("Consider optimizing resource usage to reduce costs".to_string());
        }
        
        if breakdown.time_efficiency < 0.5 {
            recommendations.push("Look for opportunities to parallelize operations".to_string());
        }
        
        if breakdown.domain_compatibility < 0.8 {
            recommendations.push("Consider executing in the current domain to avoid cross-domain overhead".to_string());
        }
        
        if breakdown.dataflow_complexity < 0.6 {
            recommendations.push("Simplify ProcessDataflowBlock orchestration if possible".to_string());
        }
        
        recommendations
    }
    
    /// Generate cache key for a plan and context
    fn generate_cache_key(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> String {
        format!("{}:{}:{}", plan.plan_id, context.current_typed_domain.domain_id(), context.evaluation_timestamp.as_millis())
    }
    
    /// Get cached result if available and not expired
    fn get_cached_result(&self, cache_key: &str) -> Option<&CachedEvaluationResult> {
        if let Some(cached) = self.evaluation_cache.get(cache_key) {
            let now = Timestamp::now();
            let age_ms = now.as_millis() - cached.cached_at.as_millis();
            
            if age_ms < self.config.cache_expiration_ms {
                return Some(cached);
            }
        }
        None
    }
    
    /// Cache an evaluation result
    fn cache_result(&mut self, cache_key: String, result: EvaluationResult) {
        let cached = CachedEvaluationResult {
            result,
            cached_at: Timestamp::now(),
            cache_key: cache_key.clone(),
        };
        
        self.evaluation_cache.insert(cache_key, cached);
        
        // Clean up expired entries periodically
        if self.evaluation_cache.len() > 1000 {
            self.cleanup_expired_cache();
        }
    }
    
    /// Clean up expired cache entries
    fn cleanup_expired_cache(&mut self) {
        let now = Timestamp::now();
        let expiration_ms = self.config.cache_expiration_ms;
        
        self.evaluation_cache.retain(|_, cached| {
            let age_ms = now.as_millis() - cached.cached_at.as_millis();
            age_ms < expiration_ms
        });
    }
    
    /// Update evaluation metrics
    fn update_metrics(&mut self, duration_ms: u64, domain: &TypedDomain, success: bool) {
        self.metrics.total_evaluations += 1;
        
        if success {
            self.metrics.successful_evaluations += 1;
        } else {
            self.metrics.failed_evaluations += 1;
        }
        
        // Update average evaluation time
        let total_time = self.metrics.avg_evaluation_time_ms * (self.metrics.total_evaluations - 1) as f64;
        self.metrics.avg_evaluation_time_ms = (total_time + duration_ms as f64) / self.metrics.total_evaluations as f64;
        
        // Update domain-specific metrics
        let domain_metrics = self.metrics.domain_performance.entry(domain.clone()).or_insert(DomainEvaluationMetrics {
            evaluations: 0,
            avg_time_ms: 0.0,
            avg_score: 0.0,
            success_rate: 0.0,
        });
        
        domain_metrics.evaluations += 1;
        let domain_total_time = domain_metrics.avg_time_ms * (domain_metrics.evaluations - 1) as f64;
        domain_metrics.avg_time_ms = (domain_total_time + duration_ms as f64) / domain_metrics.evaluations as f64;
        
        self.metrics.last_updated = Timestamp::now();
    }

    pub fn evaluate_plan_with_cache(&mut self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan> {
        let cache_key = self.generate_cache_key(plan, context);
        
        // Check cache first and clone the result to avoid borrowing issues
        let cached_result = {
            let cached = self.get_cached_result(&cache_key);
            cached.map(|c| c.result.clone())
        };
        
        if let Some(cached) = cached_result {
            self.metrics.total_evaluations += 1;
            // Convert EvaluationResult to ScoredPlan
            let scored_plan = ScoredPlan {
                plan: plan.clone(),
                overall_score: cached.overall_score,
                cost_efficiency_score: 0.8, // Default values for now
                time_efficiency_score: 0.7,
                resource_utilization_score: 0.6,
                domain_compatibility_score: 0.9,
                strategy_name: Str::from("cached"),
                evaluated_at: Timestamp::now(),
            };
            return Ok(scored_plan);
        }
        
        // Increment metrics before evaluation
        self.metrics.total_evaluations += 1;
        
        // For now, create a simple scored plan without using a strategy
        let scored_plan = ScoredPlan {
            plan: plan.clone(),
            overall_score: 0.8, // Default score
            cost_efficiency_score: 0.8,
            time_efficiency_score: 0.7,
            resource_utilization_score: 0.6,
            domain_compatibility_score: 0.9,
            strategy_name: Str::from("default"),
            evaluated_at: Timestamp::now(),
        };
        
        // Create a simple evaluation result for caching
        let eval_result = EvaluationResult {
            plan: plan.clone(),
            overall_score: scored_plan.overall_score,
            score_breakdown: ScoreBreakdown {
                cost_efficiency: scored_plan.cost_efficiency_score,
                time_efficiency: scored_plan.time_efficiency_score,
                resource_utilization: scored_plan.resource_utilization_score,
                domain_compatibility: scored_plan.domain_compatibility_score,
                dataflow_complexity: 0.5,
                risk_score: 0.2,
            },
            metadata: EvaluationMetadata {
                strategy_id: "default".to_string(),
                evaluated_at: Timestamp::now(),
                evaluation_duration_ms: 0,
                typed_domain: context.current_typed_domain.clone(),
                was_cached: false,
                additional_data: HashMap::new(),
            },
            recommendations: vec![],
            risk_assessment: RiskAssessment {
                overall_risk: 0.2,
                risk_factors: vec![],
                mitigation_strategies: vec![],
                confidence: 0.8,
            },
        };
        
        self.cache_result(cache_key, eval_result);
        Ok(scored_plan)
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            max_evaluation_time_ms: 5000,
            max_concurrent_evaluations: 4,
            enable_caching: true,
            cache_expiration_ms: 300000, // 5 minutes
            scoring_weights: ScoringWeights::default(),
            domain_parameters: HashMap::new(),
            enable_detailed_analysis: true,
        }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            cost_efficiency: 0.25,
            time_efficiency: 0.25,
            resource_utilization: 0.20,
            domain_compatibility: 0.15,
            dataflow_complexity: 0.10,
            risk_assessment: 0.05,
        }
    }
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self {
            total_evaluations: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
            avg_evaluation_time_ms: 0.0,
            cache_hit_rate: 0.0,
            domain_performance: HashMap::new(),
            last_updated: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::DomainId;

    #[test]
    fn test_plan_evaluator_creation() {
        let config = EvaluationConfig::default();
        let evaluator = PlanEvaluator::new(config);
        
        assert_eq!(evaluator.metrics.total_evaluations, 0);
        assert!(evaluator.evaluation_cache.is_empty());
    }
    
    #[test]
    fn test_scoring_weights_sum() {
        let weights = ScoringWeights::default();
        let sum = weights.cost_efficiency + weights.time_efficiency + weights.resource_utilization +
                  weights.domain_compatibility + weights.dataflow_complexity + weights.risk_assessment;
        
        // Should sum to approximately 1.0
        assert!((sum - 1.0).abs() < 0.01);
    }
} 