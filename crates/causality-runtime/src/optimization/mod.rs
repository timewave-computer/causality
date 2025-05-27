//! Runtime optimization framework for TEL-based cross-domain execution
//!
//! This module provides the core optimization framework for runtime strategy selection,
//! plan evaluation, and execution optimization across TypedDomains and ProcessDataflowBlocks.

use anyhow::Result;
use std::collections::HashMap;

use causality_types::{
    core::{
        id::{EntityId, ResourceId, ExprId},
        str::Str,
        time::Timestamp,
    },
    tel::{
        optimization::{
            ResolutionPlan, ScoredPlan, TypedDomain,
        },
        process_dataflow::{ProcessDataflowDefinition, ProcessDataflowInstanceState},
        strategy::{
            StrategyConfiguration, StrategyMetrics, StrategyPreferences, SystemLoadMetrics,
            HistoricalPerformanceData,
        },
    },
};

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
    fn get_configuration(&self) -> StrategyConfiguration;
    
    /// Update strategy configuration
    fn update_configuration(&mut self, config: StrategyConfiguration) -> Result<()>;
    
    /// Get strategy performance metrics
    fn get_metrics(&self) -> StrategyMetrics;
    
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
    pub available_dataflow_definitions: HashMap<ExprId, ProcessDataflowDefinition>,
    
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
    pub preferences: StrategyPreferences,
    
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
            preferences: StrategyPreferences::default(),
            evaluation_timestamp: Timestamp::now(),
            max_evaluation_time_ms: 5000, // 5 second default
            metadata: HashMap::new(),
        }
    }
    
    /// Add a ProcessDataflowDefinition to the context
    pub fn add_dataflow_definition(&mut self, id: ExprId, definition: ProcessDataflowDefinition) {
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
    pub fn update_preferences(&mut self, preferences: StrategyPreferences) {
        self.preferences = preferences;
    }
    
    /// Check if a typed domain is available for execution
    pub fn is_domain_available(&self, domain: &TypedDomain) -> bool {
        self.available_typed_domains.contains(domain)
    }
    
    /// Get ProcessDataflowDefinition by ID
    pub fn get_dataflow_definition(&self, id: &ExprId) -> Option<&ProcessDataflowDefinition> {
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

//-----------------------------------------------------------------------------
// Module Exports
//-----------------------------------------------------------------------------

pub mod registry;
pub mod evaluation;

// Re-export key types for convenience
pub use registry::StrategyRegistry;
pub use evaluation::{PlanEvaluator, EvaluationResult};

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        core::id::DomainId,
        expr::{value::ValueExpr, ValueExprMap},
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_optimization_context_creation() {
        let domain = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let context = OptimizationContext::new(domain.clone());
        
        assert_eq!(context.current_typed_domain, domain);
        assert_eq!(context.available_typed_domains.len(), 1);
        assert!(context.is_domain_available(&domain));
    }
    
    #[test]
    fn test_optimization_context_dataflow_management() {
        let domain = TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]));
        let mut context = OptimizationContext::new(domain);
        
        let df_id = ExprId::new([2u8; 32]);
        let df_def = ProcessDataflowDefinition {
            id: ExprId::new([1u8; 32]),
            name: Str::from("test_dataflow"),
            input_schema: ValueExpr::Map(ValueExprMap(BTreeMap::new())),
            output_schema: ValueExpr::Map(ValueExprMap(BTreeMap::new())),
            state_schema: ValueExpr::Map(ValueExprMap(BTreeMap::new())),
            nodes: vec![],
            edges: vec![],
            conditions: vec![],
            action_templates: vec![],
            domain_policies: HashMap::new(),
            created_at: Timestamp::now(),
        };
        
        context.add_dataflow_definition(df_id, df_def);
        assert!(context.get_dataflow_definition(&df_id).is_some());
    }
} 