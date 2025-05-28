//! Runtime optimization configuration
//!
//! This module provides configuration types and utilities for the runtime optimization
//! framework, including strategy selection, TypedDomain-specific settings, and
//! ProcessDataflowBlock orchestration configuration.

use causality_types::{
    core::str::Str as CausalityStr,
    graph::optimization::TypedDomain,
};
use anyhow::Result;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Main optimization configuration for the runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct OptimizationConfig {
    /// Strategy selection configuration
    pub strategy_selection: StrategySelectionConfig,
    
    /// TypedDomain-specific configuration overrides
    pub typed_domain_overrides: BTreeMap<String, TypedDomainConfig>,
    
    /// Evaluation timeout and limit settings
    pub evaluation_limits: EvaluationLimitsConfig,
    
    /// Strategy weight and preference configuration
    pub strategy_preferences: StrategyPreferencesConfig,
    
    /// ProcessDataflowBlock orchestration configuration
    pub pdb_orchestration: PdbOrchestrationConfig,
    
    /// Performance monitoring configuration
    pub performance_monitoring: PerformanceMonitoringConfig,
}

/// Strategy selection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySelectionConfig {
    /// Default strategy to use when no specific strategy is configured
    pub default_strategy: CausalityStr,
    
    /// Strategy selection mode
    pub selection_mode: StrategySelectionMode,
    
    /// Whether to enable automatic strategy switching based on performance
    pub enable_adaptive_selection: bool,
    
    /// Minimum performance threshold for strategy switching
    pub performance_threshold: f64,
    
    /// Strategy fallback chain in case of failures
    pub fallback_strategies: Vec<CausalityStr>,
}

/// Strategy selection modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategySelectionMode {
    /// Use a single strategy for all operations
    Single,
    /// Select strategy based on TypedDomain
    DomainBased,
    /// Use multiple strategies and compare results
    Competitive,
    /// Use machine learning to select optimal strategy
    Adaptive,
}

/// TypedDomain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedDomainConfig {
    /// Preferred strategies for this domain
    pub preferred_strategies: Vec<CausalityStr>,
    
    /// Strategy weights specific to this domain
    pub strategy_weights: BTreeMap<CausalityStr, f64>,
    
    /// Domain-specific optimization parameters
    pub optimization_parameters: BTreeMap<CausalityStr, serde_json::Value>,
    
    /// Resource limits for this domain
    pub resource_limits: ResourceLimitsConfig,
    
    /// Whether to enable ZK-specific optimizations (for VerifiableDomain)
    pub enable_zk_optimizations: bool,
    
    /// Whether to enable service-specific optimizations (for ServiceDomain)
    pub enable_service_optimizations: bool,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsConfig {
    /// Maximum CPU cycles allowed
    pub max_cpu_cycles: Option<u64>,
    
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    
    /// Maximum network calls allowed
    pub max_network_calls: Option<u32>,
    
    /// Maximum storage operations
    pub max_storage_operations: Option<u32>,
    
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

/// Evaluation timeout and limit settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationLimitsConfig {
    /// Maximum time to spend on strategy evaluation (milliseconds)
    pub max_evaluation_time_ms: u64,
    
    /// Maximum number of plans to evaluate per strategy
    pub max_plans_per_strategy: usize,
    
    /// Maximum number of strategies to evaluate concurrently
    pub max_concurrent_strategies: usize,
    
    /// Timeout for individual plan evaluation (milliseconds)
    pub plan_evaluation_timeout_ms: u64,
    
    /// Maximum memory usage for evaluation (bytes)
    pub max_evaluation_memory_bytes: u64,
    
    /// Whether to stop evaluation on first successful plan
    pub stop_on_first_success: bool,
}

/// Strategy weight and preference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPreferencesConfig {
    /// Global strategy weights
    pub global_weights: BTreeMap<CausalityStr, f64>,
    
    /// Cost vs. speed preference (0.0 = prefer speed, 1.0 = prefer cost efficiency)
    pub cost_vs_speed_preference: f64,
    
    /// Resource efficiency preference weight
    pub resource_efficiency_weight: f64,
    
    /// Domain compatibility preference weight
    pub domain_compatibility_weight: f64,
    
    /// PDB orchestration complexity preference weight
    pub pdb_complexity_weight: f64,
    
    /// Risk tolerance (0.0 = risk averse, 1.0 = risk tolerant)
    pub risk_tolerance: f64,
}

/// ProcessDataflowBlock orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdbOrchestrationConfig {
    /// Whether to enable PDB orchestration
    pub enable_pdb_orchestration: bool,
    
    /// Maximum PDB nesting depth
    pub max_nesting_depth: u32,
    
    /// Maximum number of concurrent PDB instances
    pub max_concurrent_instances: u32,
    
    /// PDB execution timeout (milliseconds)
    pub pdb_execution_timeout_ms: u64,
    
    /// Whether to enable PDB state caching
    pub enable_state_caching: bool,
    
    /// PDB state cache size limit
    pub state_cache_size_limit: usize,
    
    /// Lisp execution configuration for PDB combinators
    pub lisp_execution: LispExecutionConfig,
    
    /// PDB-specific strategy preferences
    pub pdb_strategy_preferences: BTreeMap<CausalityStr, f64>,
}

/// Lisp execution configuration for PDB combinators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LispExecutionConfig {
    /// Maximum Lisp execution time per combinator call (milliseconds)
    pub max_execution_time_ms: u64,
    
    /// Maximum memory usage for Lisp execution (bytes)
    pub max_memory_bytes: u64,
    
    /// Maximum recursion depth for Lisp evaluation
    pub max_recursion_depth: u32,
    
    /// Whether to enable Lisp expression caching
    pub enable_expression_caching: bool,
    
    /// Lisp expression cache size limit
    pub expression_cache_size: usize,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringConfig {
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Metrics collection interval (milliseconds)
    pub collection_interval_ms: u64,
    
    /// Whether to collect detailed execution traces
    pub collect_execution_traces: bool,
    
    /// Whether to collect resource usage metrics
    pub collect_resource_metrics: bool,
    
    /// Whether to collect strategy performance metrics
    pub collect_strategy_metrics: bool,
    
    /// Whether to collect PDB orchestration metrics
    pub collect_pdb_metrics: bool,
    
    /// Maximum number of metrics to retain in memory
    pub max_metrics_retention: usize,
    
    /// Whether to export metrics to external systems
    pub enable_metrics_export: bool,
    
    /// Metrics export configuration
    pub metrics_export: MetricsExportConfig,
}

/// Metrics export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExportConfig {
    /// Export format (json, csv, prometheus)
    pub format: String,
    
    /// Export endpoint URL
    pub endpoint: Option<String>,
    
    /// Export interval (milliseconds)
    pub export_interval_ms: u64,
    
    /// Whether to include detailed breakdowns
    pub include_detailed_breakdown: bool,
    
    /// Custom export headers
    pub custom_headers: BTreeMap<String, String>,
}


impl Default for StrategySelectionConfig {
    fn default() -> Self {
        Self {
            default_strategy: CausalityStr::from("capital_efficiency"),
            selection_mode: StrategySelectionMode::DomainBased,
            enable_adaptive_selection: true,
            performance_threshold: 0.8,
            fallback_strategies: vec![
                CausalityStr::from("priority_based"),
                CausalityStr::from("expression_based"),
            ],
        }
    }
}

impl Default for TypedDomainConfig {
    fn default() -> Self {
        Self {
            preferred_strategies: vec![CausalityStr::from("capital_efficiency")],
            strategy_weights: BTreeMap::new(),
            optimization_parameters: BTreeMap::new(),
            resource_limits: ResourceLimitsConfig::default(),
            enable_zk_optimizations: false,
            enable_service_optimizations: false,
        }
    }
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_cpu_cycles: Some(1_000_000),
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            max_network_calls: Some(100),
            max_storage_operations: Some(1000),
            max_execution_time_ms: Some(30_000), // 30 seconds
        }
    }
}

impl Default for EvaluationLimitsConfig {
    fn default() -> Self {
        Self {
            max_evaluation_time_ms: 5_000, // 5 seconds
            max_plans_per_strategy: 10,
            max_concurrent_strategies: 4,
            plan_evaluation_timeout_ms: 1_000, // 1 second
            max_evaluation_memory_bytes: 50 * 1024 * 1024, // 50MB
            stop_on_first_success: false,
        }
    }
}

impl Default for StrategyPreferencesConfig {
    fn default() -> Self {
        let mut global_weights = BTreeMap::new();
        global_weights.insert(CausalityStr::from("capital_efficiency"), 1.0);
        global_weights.insert(CausalityStr::from("priority_based"), 0.8);
        global_weights.insert(CausalityStr::from("expression_based"), 0.9);
        global_weights.insert(CausalityStr::from("pdb_orchestration"), 1.0);
        
        Self {
            global_weights,
            cost_vs_speed_preference: 0.6, // Slightly prefer cost efficiency
            resource_efficiency_weight: 0.8,
            domain_compatibility_weight: 0.9,
            pdb_complexity_weight: 0.7,
            risk_tolerance: 0.5, // Moderate risk tolerance
        }
    }
}

impl Default for PdbOrchestrationConfig {
    fn default() -> Self {
        Self {
            enable_pdb_orchestration: true,
            max_nesting_depth: 5,
            max_concurrent_instances: 10,
            pdb_execution_timeout_ms: 10_000, // 10 seconds
            enable_state_caching: true,
            state_cache_size_limit: 1000,
            lisp_execution: LispExecutionConfig::default(),
            pdb_strategy_preferences: BTreeMap::new(),
        }
    }
}

impl Default for LispExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 2_000, // 2 seconds
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            max_recursion_depth: 100,
            enable_expression_caching: true,
            expression_cache_size: 500,
        }
    }
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            collection_interval_ms: 1_000, // 1 second
            collect_execution_traces: true,
            collect_resource_metrics: true,
            collect_strategy_metrics: true,
            collect_pdb_metrics: true,
            max_metrics_retention: 10_000,
            enable_metrics_export: false,
            metrics_export: MetricsExportConfig::default(),
        }
    }
}

impl Default for MetricsExportConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            endpoint: None,
            export_interval_ms: 60_000, // 1 minute
            include_detailed_breakdown: true,
            custom_headers: BTreeMap::new(),
        }
    }
}

impl OptimizationConfig {
    /// Load configuration from a TOML file
    pub fn from_toml_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: OptimizationConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn to_toml_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Get TypedDomain-specific configuration
    pub fn get_typed_domain_config(&self, domain: &TypedDomain) -> TypedDomainConfig {
        let domain_key = domain.domain_type.as_str();
        
        self.typed_domain_overrides
            .get(domain_key)
            .cloned()
            .unwrap_or_else(|| {
                let mut config = TypedDomainConfig::default();
                match domain.domain_type.as_str() {
                    "verifiable" => {
                        config.enable_zk_optimizations = true;
                        config.preferred_strategies = vec![
                            CausalityStr::from("capital_efficiency"),
                            CausalityStr::from("expression_based"),
                        ];
                    }
                    "service" => {
                        config.enable_service_optimizations = true;
                        config.preferred_strategies = vec![
                            CausalityStr::from("priority_based"),
                            CausalityStr::from("pdb_orchestration"),
                        ];
                    }
                    _ => {
                        // For other domain types, config remains TypedDomainConfig::default()
                    }
                };
                config
            })
    }
    
    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        // Validate evaluation limits
        if self.evaluation_limits.max_evaluation_time_ms == 0 {
            return Err(anyhow::anyhow!("max_evaluation_time_ms must be greater than 0"));
        }
        
        if self.evaluation_limits.max_plans_per_strategy == 0 {
            return Err(anyhow::anyhow!("max_plans_per_strategy must be greater than 0"));
        }
        
        // Validate strategy preferences
        if self.strategy_preferences.cost_vs_speed_preference < 0.0 
            || self.strategy_preferences.cost_vs_speed_preference > 1.0 {
            return Err(anyhow::anyhow!("cost_vs_speed_preference must be between 0.0 and 1.0"));
        }
        
        if self.strategy_preferences.risk_tolerance < 0.0 
            || self.strategy_preferences.risk_tolerance > 1.0 {
            return Err(anyhow::anyhow!("risk_tolerance must be between 0.0 and 1.0"));
        }
        
        // Validate PDB orchestration settings
        if self.pdb_orchestration.max_nesting_depth == 0 {
            return Err(anyhow::anyhow!("max_nesting_depth must be greater than 0"));
        }
        
        if self.pdb_orchestration.max_concurrent_instances == 0 {
            return Err(anyhow::anyhow!("max_concurrent_instances must be greater than 0"));
        }
        
        Ok(())
    }
    
    /// Merge with another configuration, with other taking precedence
    pub fn merge_with(&mut self, other: OptimizationConfig) {
        // Merge typed domain overrides
        for (key, value) in other.typed_domain_overrides {
            self.typed_domain_overrides.insert(key, value);
        }
        
        // Merge strategy preferences global weights
        for (key, value) in other.strategy_preferences.global_weights {
            self.strategy_preferences.global_weights.insert(key, value);
        }
        
        // Merge PDB strategy preferences
        for (key, value) in other.pdb_orchestration.pdb_strategy_preferences {
            self.pdb_orchestration.pdb_strategy_preferences.insert(key, value);
        }
        
        // Replace other fields with values from other config
        self.strategy_selection = other.strategy_selection;
        self.evaluation_limits = other.evaluation_limits;
        self.strategy_preferences.cost_vs_speed_preference = other.strategy_preferences.cost_vs_speed_preference;
        self.strategy_preferences.resource_efficiency_weight = other.strategy_preferences.resource_efficiency_weight;
        self.strategy_preferences.domain_compatibility_weight = other.strategy_preferences.domain_compatibility_weight;
        self.strategy_preferences.pdb_complexity_weight = other.strategy_preferences.pdb_complexity_weight;
        self.strategy_preferences.risk_tolerance = other.strategy_preferences.risk_tolerance;
        self.pdb_orchestration.enable_pdb_orchestration = other.pdb_orchestration.enable_pdb_orchestration;
        self.pdb_orchestration.max_nesting_depth = other.pdb_orchestration.max_nesting_depth;
        self.pdb_orchestration.max_concurrent_instances = other.pdb_orchestration.max_concurrent_instances;
        self.pdb_orchestration.pdb_execution_timeout_ms = other.pdb_orchestration.pdb_execution_timeout_ms;
        self.pdb_orchestration.enable_state_caching = other.pdb_orchestration.enable_state_caching;
        self.pdb_orchestration.state_cache_size_limit = other.pdb_orchestration.state_cache_size_limit;
        self.pdb_orchestration.lisp_execution = other.pdb_orchestration.lisp_execution;
        self.performance_monitoring = other.performance_monitoring;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::core::id::DomainId;

    #[test]
    fn test_default_config() {
        let config = OptimizationConfig::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_typed_domain_config() {
        let config = OptimizationConfig::default();
        let domain_id = DomainId::new([1u8; 32]);
        
        let verifiable_config = config.get_typed_domain_config(&TypedDomain::new(domain_id, "verifiable".into()));
        assert!(verifiable_config.enable_zk_optimizations);
        assert!(!verifiable_config.enable_service_optimizations);
        
        let service_config = config.get_typed_domain_config(&TypedDomain::new(domain_id, "service".into()));
        assert!(!service_config.enable_zk_optimizations);
        assert!(service_config.enable_service_optimizations);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = OptimizationConfig::default();
        
        // Test invalid cost_vs_speed_preference
        config.strategy_preferences.cost_vs_speed_preference = 1.5;
        assert!(config.validate().is_err());
        
        // Test invalid risk_tolerance
        config.strategy_preferences.cost_vs_speed_preference = 0.5;
        config.strategy_preferences.risk_tolerance = -0.1;
        assert!(config.validate().is_err());
        
        // Test valid config
        config.strategy_preferences.risk_tolerance = 0.5;
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_merge() {
        let mut config1 = OptimizationConfig::default();
        let mut config2 = OptimizationConfig::default();
        
        config2.strategy_preferences.cost_vs_speed_preference = 0.8;
        config2.pdb_orchestration.max_nesting_depth = 10;
        
        config1.merge_with(config2);
        
        assert_eq!(config1.strategy_preferences.cost_vs_speed_preference, 0.8);
        assert_eq!(config1.pdb_orchestration.max_nesting_depth, 10);
    }
}