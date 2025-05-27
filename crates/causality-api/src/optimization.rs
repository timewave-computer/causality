//! Optimization API endpoints
//!
//! This module provides REST API endpoints for managing optimization strategies,
//! evaluating plans, and monitoring performance in the Causality runtime.

use causality_types::{
    core::str::Str,
    anyhow::Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Simple TypedDomain representation for API serialization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiTypedDomain {
    /// Domain enforcing ZK-compatibility and determinism
    VerifiableDomain(String),
    /// Domain facilitating interactions with external services
    ServiceDomain(String),
}

/// Simple plan representation for API serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiScoredPlan {
    /// Plan identifier
    pub plan_id: String,
    /// Overall score (0.0 to 1.0, higher is better)
    pub overall_score: f64,
    /// Cost efficiency score
    pub cost_efficiency_score: f64,
    /// Time efficiency score
    pub time_efficiency_score: f64,
    /// Resource utilization score
    pub resource_utilization_score: f64,
    /// TypedDomain compatibility score
    pub domain_compatibility_score: f64,
    /// Strategy that generated this plan
    pub strategy_name: String,
    /// Evaluation timestamp
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

/// Strategy management endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyManagementApi {
    /// Available strategies
    pub strategies: BTreeMap<Str, StrategyInfo>,
    
    /// Current configuration
    pub current_config: serde_json::Value,
}

/// Information about a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    /// Strategy identifier
    pub strategy_id: Str,
    
    /// Human-readable name
    pub name: String,
    
    /// Strategy description
    pub description: String,
    
    /// Whether the strategy is currently enabled
    pub enabled: bool,
    
    /// Supported TypedDomains
    pub supported_domains: Vec<ApiTypedDomain>,
    
    /// Strategy-specific configuration parameters
    pub parameters: BTreeMap<String, serde_json::Value>,
    
    /// Performance metrics
    pub performance_metrics: StrategyPerformanceMetrics,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformanceMetrics {
    /// Number of times this strategy has been invoked
    pub invocation_count: u64,
    
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Average plan quality score
    pub average_plan_quality_score: f64,
    
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Strategy evaluation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEvaluationRequest {
    /// Strategy to evaluate
    pub strategy_id: Str,
    
    /// TypedDomain context
    pub typed_domain: ApiTypedDomain,
    
    /// Test scenario configuration
    pub scenario: TestScenarioConfig,
    
    /// Evaluation parameters
    pub evaluation_params: EvaluationParameters,
}

/// Test scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenarioConfig {
    /// Scenario identifier
    pub scenario_id: String,
    
    /// Scenario description
    pub description: String,
    
    /// Number of effects to include
    pub effect_count: usize,
    
    /// Number of resources to include
    pub resource_count: usize,
    
    /// Complexity score (1.0 = simple, 10.0 = very complex)
    pub complexity_score: f64,
    
    /// Whether to include PDB orchestration
    pub include_pdb_orchestration: bool,
    
    /// Whether to include cross-domain operations
    pub include_cross_domain: bool,
}

/// Evaluation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationParameters {
    /// Maximum evaluation time in milliseconds
    pub max_evaluation_time_ms: u64,
    
    /// Maximum number of plans to evaluate
    pub max_plans: usize,
    
    /// Whether to collect detailed metrics
    pub collect_detailed_metrics: bool,
    
    /// Whether to include execution traces
    pub include_execution_traces: bool,
}

/// Strategy evaluation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEvaluationResponse {
    /// Strategy that was evaluated
    pub strategy_id: Str,
    
    /// Evaluation success status
    pub success: bool,
    
    /// Generated plans with scores
    pub scored_plans: Vec<ApiScoredPlan>,
    
    /// Evaluation metrics
    pub evaluation_metrics: EvaluationMetrics,
    
    /// Error message if evaluation failed
    pub error_message: Option<String>,
    
    /// Execution traces if requested
    pub execution_traces: Option<Vec<ExecutionTrace>>,
}

/// Evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    /// Total evaluation time in milliseconds
    pub total_evaluation_time_ms: u64,
    
    /// Number of plans generated
    pub plans_generated: usize,
    
    /// Average plan quality score
    pub average_plan_quality: f64,
    
    /// Resource utilization metrics
    pub resource_utilization: ResourceUtilizationMetrics,
    
    /// Domain compatibility score
    pub domain_compatibility_score: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationMetrics {
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    
    /// Network calls made
    pub network_calls: u32,
    
    /// Storage operations performed
    pub storage_operations: u32,
}

/// Execution trace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Step index
    pub step_index: usize,
    
    /// Step type
    pub step_type: String,
    
    /// Step duration in milliseconds
    pub duration_ms: u64,
    
    /// Step success status
    pub success: bool,
    
    /// Step details
    pub details: BTreeMap<String, serde_json::Value>,
}

/// Configuration management request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdateRequest {
    /// Configuration section to update
    pub section: ConfigurationSection,
    
    /// Updated configuration data
    pub config_data: serde_json::Value,
    
    /// Whether to validate before applying
    pub validate_before_apply: bool,
}

/// Configuration sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationSection {
    /// Strategy selection configuration
    StrategySelection,
    
    /// TypedDomain-specific configuration
    TypedDomain(String),
    
    /// Evaluation limits configuration
    EvaluationLimits,
    
    /// Strategy preferences configuration
    StrategyPreferences,
    
    /// PDB orchestration configuration
    PdbOrchestration,
    
    /// Performance monitoring configuration
    PerformanceMonitoring,
}

/// Configuration update response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdateResponse {
    /// Update success status
    pub success: bool,
    
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    
    /// Error message if update failed
    pub error_message: Option<String>,
    
    /// Updated configuration
    pub updated_config: Option<serde_json::Value>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation rule name
    pub rule_name: String,
    
    /// Validation success status
    pub success: bool,
    
    /// Validation message
    pub message: String,
    
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Information only
    Info,
    
    /// Warning - configuration will work but may not be optimal
    Warning,
    
    /// Error - configuration is invalid and will not work
    Error,
}

/// Performance monitoring request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringRequest {
    /// Time range for metrics collection
    pub time_range: TimeRange,
    
    /// Metrics to include
    pub metrics_filter: MetricsFilter,
    
    /// Aggregation level
    pub aggregation_level: AggregationLevel,
}

/// Time range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time
    pub end_time: chrono::DateTime<chrono::Utc>,
}

/// Metrics filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsFilter {
    /// Strategy IDs to include
    pub strategy_ids: Option<Vec<Str>>,
    
    /// TypedDomains to include
    pub typed_domains: Option<Vec<ApiTypedDomain>>,
    
    /// Metric types to include
    pub metric_types: Vec<MetricType>,
}

/// Metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Strategy performance metrics
    StrategyPerformance,
    
    /// Resource utilization metrics
    ResourceUtilization,
    
    /// Domain compatibility metrics
    DomainCompatibility,
    
    /// PDB orchestration metrics
    PdbOrchestration,
    
    /// Execution time metrics
    ExecutionTime,
    
    /// Error rate metrics
    ErrorRate,
}

/// Aggregation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationLevel {
    /// Raw data points
    Raw,
    
    /// Aggregated by minute
    Minute,
    
    /// Aggregated by hour
    Hour,
    
    /// Aggregated by day
    Day,
}

/// Performance monitoring response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringResponse {
    /// Time range of the data
    pub time_range: TimeRange,
    
    /// Collected metrics
    pub metrics: BTreeMap<String, Vec<MetricDataPoint>>,
    
    /// Summary statistics
    pub summary: PerformanceSummary,
    
    /// Recommendations based on the data
    pub recommendations: Vec<String>,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Metric value
    pub value: f64,
    
    /// Additional metadata
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Overall health score (0.0 to 1.0)
    pub overall_health_score: f64,
    
    /// Average strategy performance
    pub average_strategy_performance: f64,
    
    /// Resource efficiency score
    pub resource_efficiency_score: f64,
    
    /// Domain compatibility score
    pub domain_compatibility_score: f64,
    
    /// Key insights
    pub key_insights: Vec<String>,
    
    /// Areas for improvement
    pub areas_for_improvement: Vec<String>,
}

impl Default for StrategyPerformanceMetrics {
    fn default() -> Self {
        Self {
            invocation_count: 0,
            average_execution_time_ms: 0.0,
            success_rate: 0.0,
            average_plan_quality_score: 0.0,
            last_updated: chrono::Utc::now(),
        }
    }
}

impl Default for EvaluationParameters {
    fn default() -> Self {
        Self {
            max_evaluation_time_ms: 5_000,
            max_plans: 10,
            collect_detailed_metrics: true,
            include_execution_traces: false,
        }
    }
}

impl Default for TestScenarioConfig {
    fn default() -> Self {
        Self {
            scenario_id: "default".to_string(),
            description: "Default test scenario".to_string(),
            effect_count: 5,
            resource_count: 10,
            complexity_score: 1.0,
            include_pdb_orchestration: false,
            include_cross_domain: false,
        }
    }
}

/// API endpoint implementations
impl StrategyManagementApi {
    /// Create a new strategy management API instance
    pub fn new(config: serde_json::Value) -> Self {
        Self {
            strategies: BTreeMap::new(),
            current_config: config,
        }
    }
    
    /// Register a new strategy
    pub fn register_strategy(&mut self, strategy_info: StrategyInfo) -> Result<()> {
        self.strategies.insert(strategy_info.strategy_id.clone(), strategy_info);
        Ok(())
    }
    
    /// Get all registered strategies
    pub fn get_strategies(&self) -> &BTreeMap<Str, StrategyInfo> {
        &self.strategies
    }
    
    /// Get a specific strategy by ID
    pub fn get_strategy(&self, strategy_id: &Str) -> Option<&StrategyInfo> {
        self.strategies.get(strategy_id)
    }
    
    /// Enable a strategy
    pub fn enable_strategy(&mut self, strategy_id: &Str) -> Result<()> {
        if let Some(strategy) = self.strategies.get_mut(strategy_id) {
            strategy.enabled = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Strategy not found: {}", strategy_id))
        }
    }
    
    /// Disable a strategy
    pub fn disable_strategy(&mut self, strategy_id: &Str) -> Result<()> {
        if let Some(strategy) = self.strategies.get_mut(strategy_id) {
            strategy.enabled = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Strategy not found: {}", strategy_id))
        }
    }
    
    /// Update strategy configuration
    pub fn update_strategy_config(
        &mut self,
        strategy_id: &Str,
        parameters: BTreeMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(strategy) = self.strategies.get_mut(strategy_id) {
            strategy.parameters = parameters;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Strategy not found: {}", strategy_id))
        }
    }
    
    /// Get strategies for a specific TypedDomain
    pub fn get_strategies_for_domain(&self, domain: &ApiTypedDomain) -> Vec<&StrategyInfo> {
        self.strategies
            .values()
            .filter(|strategy| {
                strategy.enabled && strategy.supported_domains.contains(domain)
            })
            .collect()
    }
    
    /// Update performance metrics for a strategy
    pub fn update_strategy_metrics(
        &mut self,
        strategy_id: &Str,
        metrics: StrategyPerformanceMetrics,
    ) -> Result<()> {
        if let Some(strategy) = self.strategies.get_mut(strategy_id) {
            strategy.performance_metrics = metrics;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Strategy not found: {}", strategy_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strategy_management_api() {
        let config = serde_json::json!({});
        let mut api = StrategyManagementApi::new(config);
        
        let strategy_info = StrategyInfo {
            strategy_id: Str::from("test_strategy"),
            name: "Test Strategy".to_string(),
            description: "A test strategy".to_string(),
            enabled: true,
            supported_domains: vec![ApiTypedDomain::VerifiableDomain("verifiable_domain".to_string())],
            parameters: BTreeMap::new(),
            performance_metrics: StrategyPerformanceMetrics::default(),
        };
        
        assert!(api.register_strategy(strategy_info).is_ok());
        assert_eq!(api.get_strategies().len(), 1);
        
        let strategy_id = Str::from("test_strategy");
        assert!(api.get_strategy(&strategy_id).is_some());
        
        assert!(api.disable_strategy(&strategy_id).is_ok());
        assert!(!api.get_strategy(&strategy_id).unwrap().enabled);
    }
    
    #[test]
    fn test_domain_filtering() {
        let config = serde_json::json!({});
        let mut api = StrategyManagementApi::new(config);
        
        let verifiable_strategy = StrategyInfo {
            strategy_id: Str::from("verifiable_strategy"),
            name: "Verifiable Strategy".to_string(),
            description: "For verifiable domain".to_string(),
            enabled: true,
            supported_domains: vec![ApiTypedDomain::VerifiableDomain("verifiable_domain".to_string())],
            parameters: BTreeMap::new(),
            performance_metrics: StrategyPerformanceMetrics::default(),
        };
        
        let service_strategy = StrategyInfo {
            strategy_id: Str::from("service_strategy"),
            name: "Service Strategy".to_string(),
            description: "For service domain".to_string(),
            enabled: true,
            supported_domains: vec![ApiTypedDomain::ServiceDomain("service_domain".to_string())],
            parameters: BTreeMap::new(),
            performance_metrics: StrategyPerformanceMetrics::default(),
        };
        
        api.register_strategy(verifiable_strategy).unwrap();
        api.register_strategy(service_strategy).unwrap();
        
        let verifiable_strategies = api.get_strategies_for_domain(&ApiTypedDomain::VerifiableDomain("verifiable_domain".to_string()));
        assert_eq!(verifiable_strategies.len(), 1);
        assert_eq!(verifiable_strategies[0].strategy_id, Str::from("verifiable_strategy"));
        
        let service_strategies = api.get_strategies_for_domain(&ApiTypedDomain::ServiceDomain("service_domain".to_string()));
        assert_eq!(service_strategies.len(), 1);
        assert_eq!(service_strategies[0].strategy_id, Str::from("service_strategy"));
    }
} 