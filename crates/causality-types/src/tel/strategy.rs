//! Optimization strategy traits and types for runtime optimization
//!
//! This module defines the core traits and types for implementing optimization
//! strategies that can evaluate and score resolution plans for TEL execution.

use crate::{
    core::{
        str::Str,
        time::Timestamp,
    },
    tel::{
        optimization::{ResolutionPlan, ScoredPlan, TypedDomain},
        cost_model::ResourceUsageEstimate,
    },
    serialization::{Encode, Decode, DecodeError, SimpleSerialize},
};
use std::collections::HashMap;
use anyhow::Result;

//-----------------------------------------------------------------------------
// Core Strategy Traits
//-----------------------------------------------------------------------------

/// Core trait for optimization strategies that can evaluate resolution plans
pub trait OptimizationStrategy: Send + Sync {
    /// Unique identifier for this strategy
    fn strategy_id(&self) -> &str;
    
    /// Human-readable name for this strategy
    fn strategy_name(&self) -> &str;
    
    /// Description of what this strategy optimizes for
    fn description(&self) -> &str;
    
    /// Evaluate a resolution plan and return a scored plan
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &StrategyContext) -> Result<ScoredPlan>;
    
    /// Check if this strategy can handle the given typed domain
    fn supports_typed_domain(&self, domain: &TypedDomain) -> bool;
    
    /// Get strategy configuration parameters
    fn get_configuration(&self) -> StrategyConfiguration;
    
    /// Update strategy configuration
    fn update_configuration(&mut self, config: StrategyConfiguration) -> Result<()>;
    
    /// Get strategy performance metrics
    fn get_metrics(&self) -> StrategyMetrics;
    
    /// Reset strategy state and metrics
    fn reset(&mut self);
}

/// Context information provided to strategies during evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyContext {
    /// Current execution domain
    pub current_typed_domain: TypedDomain,
    
    /// Available resources in the context
    pub available_resources: HashMap<Str, u64>,
    
    /// Current system load metrics
    pub system_load: SystemLoadMetrics,
    
    /// Historical performance data
    pub historical_data: HistoricalPerformanceData,
    
    /// Strategy preferences and weights
    pub preferences: StrategyPreferences,
    
    /// Timestamp of evaluation
    pub evaluation_timestamp: Timestamp,
}

/// System load metrics for strategy evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct SystemLoadMetrics {
    /// CPU utilization (0.0 to 1.0)
    pub cpu_utilization: f64,
    
    /// Memory utilization (0.0 to 1.0)
    pub memory_utilization: f64,
    
    /// Network utilization (0.0 to 1.0)
    pub network_utilization: f64,
    
    /// Active transaction count
    pub active_transactions: u32,
    
    /// Pending intent count
    pub pending_intents: u32,
    
    /// Cross-domain transfer queue size
    pub cross_domain_queue_size: u32,
}

/// Historical performance data for strategy evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct HistoricalPerformanceData {
    /// Average execution times by typed domain
    pub avg_execution_times: HashMap<TypedDomain, u64>,
    
    /// Success rates by strategy
    pub strategy_success_rates: HashMap<Str, f64>,
    
    /// Resource consumption patterns
    pub resource_consumption_patterns: HashMap<Str, ResourceConsumptionPattern>,
    
    /// Recent performance trends
    pub performance_trends: Vec<PerformanceTrend>,
}

/// Resource consumption pattern data
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceConsumptionPattern {
    /// Average resource usage
    pub avg_usage: u64,
    
    /// Peak resource usage
    pub peak_usage: u64,
    
    /// Usage variance
    pub variance: f64,
    
    /// Trend direction (-1.0 to 1.0)
    pub trend: f64,
}

/// Performance trend data point
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceTrend {
    /// Timestamp of measurement
    pub timestamp: Timestamp,
    
    /// Performance metric value
    pub value: f64,
    
    /// Metric type identifier
    pub metric_type: Str,
    
    /// Associated typed domain
    pub typed_domain: Option<TypedDomain>,
}

/// Strategy preferences and weights
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyPreferences {
    /// Weight for cost optimization (0.0 to 1.0)
    pub cost_weight: f64,
    
    /// Weight for time optimization (0.0 to 1.0)
    pub time_weight: f64,
    
    /// Weight for resource utilization (0.0 to 1.0)
    pub resource_weight: f64,
    
    /// Weight for domain compatibility (0.0 to 1.0)
    pub compatibility_weight: f64,
    
    /// Preference for specific typed domains
    pub domain_preferences: HashMap<TypedDomain, f64>,
    
    /// Risk tolerance (0.0 = risk-averse, 1.0 = risk-seeking)
    pub risk_tolerance: f64,
}

/// Strategy configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyConfiguration {
    /// Strategy identifier
    pub strategy_id: Str,
    
    /// Configuration parameters
    pub parameters: HashMap<Str, ConfigurationValue>,
    
    /// Enabled typed domains
    pub enabled_domains: Vec<TypedDomain>,
    
    /// Strategy priority (higher = more preferred)
    pub priority: u32,
    
    /// Maximum evaluation time in milliseconds
    pub max_evaluation_time_ms: u64,
    
    /// Configuration version
    pub version: u32,
    
    /// Last updated timestamp
    pub updated_at: Timestamp,
}

/// Configuration value types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationValue {
    /// String value
    String(Str),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigurationValue>),
    /// Nested configuration
    Object(HashMap<Str, ConfigurationValue>),
}

/// Strategy performance metrics
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyMetrics {
    /// Strategy identifier
    pub strategy_id: Str,
    
    /// Total evaluations performed
    pub total_evaluations: u64,
    
    /// Successful evaluations
    pub successful_evaluations: u64,
    
    /// Average evaluation time in milliseconds
    pub avg_evaluation_time_ms: f64,
    
    /// Average plan score
    pub avg_plan_score: f64,
    
    /// Plans selected for execution
    pub plans_selected: u64,
    
    /// Plans successfully executed
    pub plans_executed_successfully: u64,
    
    /// Resource consumption metrics
    pub resource_consumption: ResourceUsageEstimate,
    
    /// Performance by typed domain
    pub domain_performance: HashMap<TypedDomain, DomainPerformanceMetrics>,
    
    /// Last updated timestamp
    pub last_updated: Timestamp,
}

/// Performance metrics for a specific typed domain
#[derive(Debug, Clone, PartialEq)]
pub struct DomainPerformanceMetrics {
    /// Evaluations in this domain
    pub evaluations: u64,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Average execution time
    pub avg_execution_time_ms: f64,
    
    /// Average cost
    pub avg_cost: u64,
    
    /// Average resource utilization
    pub avg_resource_utilization: f64,
}

//-----------------------------------------------------------------------------
// Strategy Registry
//-----------------------------------------------------------------------------

/// Registry for managing optimization strategies
pub trait StrategyRegistry: Send + Sync {
    /// Register a new strategy
    fn register_strategy(&mut self, strategy: Box<dyn OptimizationStrategy>) -> Result<()>;
    
    /// Unregister a strategy by ID
    fn unregister_strategy(&mut self, strategy_id: &str) -> Result<()>;
    
    /// Get a strategy by ID
    fn get_strategy(&self, strategy_id: &str) -> Option<&dyn OptimizationStrategy>;
    
    /// Get a mutable strategy by ID
    fn get_strategy_mut(&mut self, strategy_id: &str) -> Option<&mut dyn OptimizationStrategy>;
    
    /// List all registered strategies
    fn list_strategies(&self) -> Vec<&str>;
    
    /// Get strategies that support a specific typed domain
    fn get_strategies_for_domain(&self, domain: &TypedDomain) -> Vec<&dyn OptimizationStrategy>;
    
    /// Get the best strategy for a given context
    fn select_best_strategy(&self, context: &StrategyContext) -> Option<&dyn OptimizationStrategy>;
    
    /// Update strategy configurations
    fn update_configurations(&mut self, configs: Vec<StrategyConfiguration>) -> Result<()>;
    
    /// Get all strategy metrics
    fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics>;
    
    /// Reset all strategies
    fn reset_all(&mut self);
}

//-----------------------------------------------------------------------------
// Default Implementations
//-----------------------------------------------------------------------------

impl Default for StrategyPreferences {
    fn default() -> Self {
        Self {
            cost_weight: 0.3,
            time_weight: 0.3,
            resource_weight: 0.2,
            compatibility_weight: 0.2,
            domain_preferences: HashMap::new(),
            risk_tolerance: 0.5,
        }
    }
}

impl Default for SystemLoadMetrics {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            network_utilization: 0.0,
            active_transactions: 0,
            pending_intents: 0,
            cross_domain_queue_size: 0,
        }
    }
}

impl Default for HistoricalPerformanceData {
    fn default() -> Self {
        Self {
            avg_execution_times: HashMap::new(),
            strategy_success_rates: HashMap::new(),
            resource_consumption_patterns: HashMap::new(),
            performance_trends: Vec::new(),
        }
    }
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self {
            current_typed_domain: TypedDomain::default(),
            available_resources: HashMap::new(),
            system_load: SystemLoadMetrics::default(),
            historical_data: HistoricalPerformanceData::default(),
            preferences: StrategyPreferences::default(),
            evaluation_timestamp: Timestamp::now(),
        }
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for StrategyConfiguration {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encode strategy_id with length prefix
        let strategy_id_bytes = self.strategy_id.as_ssz_bytes();
        bytes.extend_from_slice(&(strategy_id_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&strategy_id_bytes);
        
        // Convert HashMap to Vec of pairs for serialization (simplified - skip complex ConfigurationValue for now)
        let param_count = self.parameters.len() as u32;
        bytes.extend_from_slice(&param_count.to_le_bytes());
        
        // Encode enabled_domains with length prefix
        let enabled_domains_bytes = self.enabled_domains.as_ssz_bytes();
        bytes.extend_from_slice(&(enabled_domains_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&enabled_domains_bytes);
        
        // Encode priority
        bytes.extend_from_slice(&self.priority.to_le_bytes());
        
        // Encode max_evaluation_time_ms
        bytes.extend_from_slice(&self.max_evaluation_time_ms.to_le_bytes());
        
        // Encode version
        bytes.extend_from_slice(&self.version.to_le_bytes());
        
        // Encode updated_at with length prefix
        let updated_at_bytes = self.updated_at.as_ssz_bytes();
        bytes.extend_from_slice(&(updated_at_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&updated_at_bytes);
        
        bytes
    }
}

impl Decode for StrategyConfiguration {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode strategy_id with length prefix
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for strategy_id length"));
        }
        let strategy_id_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + strategy_id_len > bytes.len() {
            return Err(DecodeError::new("Invalid strategy_id length"));
        }
        let strategy_id = Str::from_ssz_bytes(&bytes[offset..offset + strategy_id_len])?;
        offset += strategy_id_len;
        
        // Decode parameters count (skip actual parameters for now)
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for parameters count"));
        }
        let _params_count = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;
        
        // Decode enabled_domains with length prefix
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for enabled_domains length"));
        }
        let enabled_domains_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + enabled_domains_len > bytes.len() {
            return Err(DecodeError::new("Invalid enabled_domains length"));
        }
        let enabled_domains = Vec::<TypedDomain>::from_ssz_bytes(&bytes[offset..offset + enabled_domains_len])?;
        offset += enabled_domains_len;
        
        // Decode priority
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for priority"));
        }
        let priority = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;
        
        // Decode max_evaluation_time_ms
        if offset + 8 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for max_evaluation_time_ms"));
        }
        let max_evaluation_time_ms = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]
        ]);
        offset += 8;
        
        // Decode version
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for version"));
        }
        let version = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;
        
        // Decode updated_at with length prefix
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Insufficient data for updated_at length"));
        }
        let updated_at_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + updated_at_len > bytes.len() {
            return Err(DecodeError::new("Invalid updated_at length"));
        }
        let updated_at = Timestamp::from_ssz_bytes(&bytes[offset..offset + updated_at_len])?;
        
        Ok(StrategyConfiguration {
            strategy_id,
            parameters: HashMap::new(), // Simplified for now
            enabled_domains,
            priority,
            max_evaluation_time_ms,
            version,
            updated_at,
        })
    }
}

impl SimpleSerialize for StrategyConfiguration {}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::DomainId;

    #[test]
    
    fn test_strategy_configuration_serialization() {
        let config = StrategyConfiguration {
            strategy_id: Str::from("test_strategy"),
            parameters: HashMap::new(),
            enabled_domains: vec![
                TypedDomain::VerifiableDomain(DomainId::new([1u8; 32])),
                TypedDomain::ServiceDomain(DomainId::new([2u8; 32])),
            ],
            priority: 10,
            max_evaluation_time_ms: 5000,
            version: 1,
            updated_at: Timestamp::now(),
        };

        let bytes = config.as_ssz_bytes();
        let decoded = StrategyConfiguration::from_ssz_bytes(&bytes)
            .expect("Failed to decode StrategyConfiguration");

        assert_eq!(config.strategy_id, decoded.strategy_id);
        assert_eq!(config.enabled_domains, decoded.enabled_domains);
        assert_eq!(config.priority, decoded.priority);
        assert_eq!(config.max_evaluation_time_ms, decoded.max_evaluation_time_ms);
        assert_eq!(config.version, decoded.version);
    }

    #[test]
    fn test_strategy_preferences_default() {
        let prefs = StrategyPreferences::default();
        
        assert_eq!(prefs.cost_weight, 0.3);
        assert_eq!(prefs.time_weight, 0.3);
        assert_eq!(prefs.resource_weight, 0.2);
        assert_eq!(prefs.compatibility_weight, 0.2);
        assert_eq!(prefs.risk_tolerance, 0.5);
    }

    #[test]
    fn test_strategy_context_creation() {
        let context = StrategyContext {
            current_typed_domain: TypedDomain::VerifiableDomain(DomainId::new([1u8; 32])),
            available_resources: {
                let mut resources = HashMap::new();
                resources.insert(Str::from("compute"), 1000);
                resources.insert(Str::from("storage"), 500);
                resources
            },
            system_load: SystemLoadMetrics {
                cpu_utilization: 0.7,
                memory_utilization: 0.5,
                network_utilization: 0.3,
                active_transactions: 10,
                pending_intents: 5,
                cross_domain_queue_size: 2,
            },
            historical_data: HistoricalPerformanceData::default(),
            preferences: StrategyPreferences::default(),
            evaluation_timestamp: Timestamp::now(),
        };

        assert_eq!(context.available_resources.len(), 2);
        assert_eq!(context.system_load.cpu_utilization, 0.7);
        assert!(context.current_typed_domain.is_verifiable());
    }
} 