//! Runtime optimization framework for TEL-based cross-domain execution
//!
//! This module provides the core optimization framework for runtime strategy selection,
//! plan evaluation, and execution optimization across TypedDomains and ProcessDataflowBlocks.

use anyhow::Result;
use std::collections::HashMap;

pub mod evaluation; 
pub mod registry; 

use causality_types::{
    primitive::{
        ids::{EntityId, ExprId, ResourceId},
        time::Timestamp,
        string::Str,
    },
    graph::{
        optimization::{TypedDomain, ScoredPlan, ResolutionPlan, DataflowOrchestrationStep},
        dataflow::{LegacyProcessDataflowDefinition, ProcessDataflowInstanceState},
    },
};

// Import StrategyPerformanceHistory from the registry module
use self::registry::StrategyPerformanceHistory;

// TODO: Ensure StrategyConfiguration, StrategyMetrics, StrategyPreferences are in scope
// SystemLoadMetrics and HistoricalPerformanceData are defined below.

//-----------------------------------------------------------------------------
// Helper Structs for OptimizationContext
//-----------------------------------------------------------------------------

/// Represents system load metrics at a point in time.
#[derive(Debug, Clone)]
pub struct SystemLoadMetrics {
    /// CPU load, typically as a percentage (e.g., 0.0 to 1.0 or 0 to 100).
    pub cpu_load: f64,
    /// Memory usage in megabytes.
    pub memory_usage_mb: u64,
    /// Network throughput in Mbps (optional).
    pub network_throughput_mbps: Option<f64>,
    /// Disk I/O operations per second (optional).
    pub disk_io_ops_sec: Option<u64>,
}

impl Default for SystemLoadMetrics {
    fn default() -> Self {
        Self {
            cpu_load: 0.0,
            memory_usage_mb: 0,
            network_throughput_mbps: None,
            disk_io_ops_sec: None,
        }
    }
}

/// Container for historical performance data of strategies.
#[derive(Debug, Clone)]
pub struct HistoricalPerformanceData {
    /// Maps strategy ID to its performance history.
    pub data: HashMap<String, StrategyPerformanceHistory>,
}

impl Default for HistoricalPerformanceData {
    fn default() -> Self { Self { data: HashMap::new() } }
}

//-----------------------------------------------------------------------------
// Core Strategy Traits
//-----------------------------------------------------------------------------

/// Core trait for optimization strategies that can evaluate and score resolution plans
pub trait OptimizationStrategy: Send + Sync {
    /// Unique identifier for this strategy
    fn strategy_id(&self) -> &str;
    
    /// Human-readable name for this strategy
    fn strategy_name(&self) -> &str;
    
    /// Description of what this strategy optimizes for
    fn description(&self) -> &str;
    
    /// Propose resolution plans for the given optimization context
    fn propose(&self, context: &OptimizationContext) -> Result<Vec<ScoredPlan>>;

    /// Evaluate a specific resolution plan and return a scored plan
    fn evaluate_plan(&self, plan: &ResolutionPlan, context: &OptimizationContext) -> Result<ScoredPlan>;
    
    /// Check if this strategy can handle the given typed domain
    fn supports_typed_domain(&self, domain: &TypedDomain) -> bool;
    
    /// Get strategy configuration parameters
    fn get_configuration(&self) -> evaluation::EvaluationConfig;
    
    /// Update strategy configuration
    fn update_configuration(&mut self, config: evaluation::EvaluationConfig) -> Result<()>;
    
    /// Get strategy performance metrics
    fn get_metrics(&self) -> evaluation::EvaluationMetrics;
    
    /// Reset strategy state and metrics
    fn reset(&mut self);
}

//-----------------------------------------------------------------------------
// Optimization Context
//-----------------------------------------------------------------------------

/// Context information provided to optimization strategies for decision making
#[derive(Debug, Clone)]
pub struct OptimizationContext {
    /// Current execution domain
    pub current_typed_domain: TypedDomain,
    
    /// Available typed domains for execution
    pub available_typed_domains: Vec<TypedDomain>,
    
    /// Available ProcessDataflowDefinitions
    pub available_dataflow_definitions: HashMap<ExprId, LegacyProcessDataflowDefinition>,
    
    /// Active ProcessDataflowInstances
    pub active_dataflow_instances: HashMap<ResourceId, ProcessDataflowInstanceState>,
    
    /// Pending intents to be resolved
    pub pending_intents: Vec<EntityId>,
    
    /// Available resources in the context
    pub available_resources: HashMap<Str, u64>,
    
    /// Current system load metrics
    pub system_load: SystemLoadMetrics,
    
    /// Historical performance data
    pub historical_data: HistoricalPerformanceData,
    
    /// Strategy preferences and weights
    pub preferences: evaluation::ScoringWeights,
    
    /// Timestamp of evaluation
    pub evaluation_timestamp: Timestamp,
    
    /// Maximum evaluation time allowed
    pub max_evaluation_time_ms: u64,
    
    /// Additional context metadata
    pub metadata: HashMap<Str, String>,
}

impl OptimizationContext {
    /// Create a new optimization context
    pub fn new(current_typed_domain: TypedDomain) -> Self {
        Self {
            current_typed_domain: current_typed_domain.clone(),
            available_typed_domains: vec![current_typed_domain],
            available_dataflow_definitions: HashMap::new(),
            active_dataflow_instances: HashMap::new(),
            pending_intents: Vec::new(),
            available_resources: HashMap::new(),
            system_load: SystemLoadMetrics::default(),
            historical_data: HistoricalPerformanceData::default(),
            preferences: evaluation::ScoringWeights::default(),
            evaluation_timestamp: Timestamp::now(),
            max_evaluation_time_ms: 5000, // 5 second default
            metadata: HashMap::new(),
        }
    }
    
    /// Add a ProcessDataflowDefinition to the context
    pub fn add_dataflow_definition(&mut self, id: ExprId, definition: LegacyProcessDataflowDefinition) {
        self.available_dataflow_definitions.insert(id, definition);
    }
    
    /// Add an active ProcessDataflowInstance to the context
    pub fn add_dataflow_instance(&mut self, id: ResourceId, instance: ProcessDataflowInstanceState) {
        self.active_dataflow_instances.insert(id, instance);
    }
    
    /// Add a pending intent to be resolved
    pub fn add_pending_intent(&mut self, intent_id: EntityId) {
        self.pending_intents.push(intent_id);
    }
    
    /// Set available resource amounts
    pub fn set_available_resource(&mut self, resource_type: Str, amount: u64) {
        self.available_resources.insert(resource_type, amount);
    }
    
    /// Update system load metrics
    pub fn update_system_load(&mut self, load: SystemLoadMetrics) {
        self.system_load = load;
    }
    
    /// Update strategy preferences
    pub fn update_preferences(&mut self, preferences: evaluation::ScoringWeights) {
        self.preferences = preferences;
    }
    
    /// Check if a typed domain is available for execution
    pub fn is_domain_available(&self, domain: &TypedDomain) -> bool {
        self.available_typed_domains.contains(domain)
    }
    
    /// Get ProcessDataflowDefinition by ID
    pub fn get_dataflow_definition(&self, id: &ExprId) -> Option<&LegacyProcessDataflowDefinition> {
        self.available_dataflow_definitions.get(id)
    }
    
    /// Get ProcessDataflowInstance by ID
    pub fn get_dataflow_instance(&self, id: &ResourceId) -> Option<&ProcessDataflowInstanceState> {
        self.active_dataflow_instances.get(id)
    }
}

//-----------------------------------------------------------------------------
// Strategy Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur during strategy execution
#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    #[error("Strategy not found: {strategy_id}")]
    StrategyNotFound { strategy_id: String },
    
    #[error("Strategy evaluation failed: {reason}")]
    EvaluationFailed { reason: String },
    
    #[error("Strategy configuration invalid: {reason}")]
    InvalidConfiguration { reason: String },
    
    #[error("TypedDomain not supported: {domain:?}")]
    UnsupportedDomain { domain: TypedDomain },
    
    #[error("ProcessDataflowBlock operation failed: {reason}")]
    DataflowOperationFailed { reason: String },
    
    #[error("Evaluation timeout exceeded: {timeout_ms}ms")]
    EvaluationTimeout { timeout_ms: u64 },
    
    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        primitive::ids::{ExprId, ResourceId},
        graph::dataflow::{LegacyProcessDataflowDefinition, DataflowExecutionState},
        primitive::string::Str,
        graph::optimization::TypedDomain,
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_optimization_context_creation() {
        let domain = TypedDomain::default();
        let context = OptimizationContext::new(domain.clone());
        assert_eq!(context.current_typed_domain, domain);
        assert!(context.available_typed_domains.contains(&domain));
    }

    #[test]
    fn test_optimization_context_dataflow_management() {
        let domain = TypedDomain::default();
        let mut context = OptimizationContext::new(domain.clone());

        let df_id = ExprId::new_v4();
        let df_def = LegacyProcessDataflowDefinition::new(df_id.clone(), Str::from("test_df"));
        context.add_dataflow_definition(df_id.clone(), df_def.clone());
        assert!(context.get_dataflow_definition(&df_id).is_some());

        let df_instance_id = ResourceId::new_v4();
        let df_instance = ProcessDataflowInstanceState {
            instance_id: df_instance_id.clone(),
            definition_id: df_id.clone(),
            execution_state: DataflowExecutionState::Running,
            node_states: BTreeMap::new(),
            metadata: BTreeMap::new(),
            initiation_hint: None,
        };
        context.add_dataflow_instance(df_instance_id.clone(), df_instance.clone());
        assert!(context.get_dataflow_instance(&df_instance_id).is_some());
    }
}