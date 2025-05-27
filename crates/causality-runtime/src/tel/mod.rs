//! Temporal Effect Language (TEL) Module
//!
//! This module provides the core implementation of the Temporal Effect Language,
//! which is used for expressing causal relationships and temporal logic in the
//! Causality system.
//!
//! ## Runtime Optimization Framework
//!
//! The TEL module includes a sophisticated runtime optimization framework that supports:
//!
//! ### TypedDomain-Aware Execution
//! - **VerifiableDomain**: Enforces ZK-compatibility and deterministic execution
//! - **ServiceDomain**: Facilitates interactions with external, non-deterministic services
//! - Domain-specific cost models and optimization strategies
//! - Cross-domain execution planning and resource management
//!
//! ### ProcessDataflowBlock (PDB) Orchestration
//! - Higher-level declarative workflow structures
//! - Multi-step conditional execution with Lisp-based combinators
//! - Dynamic PDB instantiation and advancement
//! - State management and orchestration complexity analysis
//!
//! ### Optimization Strategies
//! - Pluggable strategy framework with multiple implementations
//! - Capital efficiency optimization across domains
//! - Priority-based selection with domain preferences
//! - Expression-based dynamic optimization using TEL expressions
//! - PDB-focused orchestration strategies
//!
//! ### Performance and Analysis
//! - Comprehensive metrics collection and analysis
//! - Strategy effectiveness evaluation and comparison
//! - Bottleneck detection and optimization recommendations
//! - Simulation-based performance forecasting

//-----------------------------------------------------------------------------
// TEL Components
//-----------------------------------------------------------------------------

pub mod context;
pub mod graph;
pub mod interpreter;
pub mod lisp_adapter;
pub mod lisp_bridge;
pub mod graph_executor;
pub mod intent_processor;

#[cfg(test)] // Only compile the tests module when running tests
pub mod tests;

pub mod traits;
pub mod types;

// Re-export core TEL types
pub use graph::TelGraph;
pub use interpreter::Interpreter;
pub use graph_executor::EffectGraphExecutor;
pub use causality_types::tel::InterpreterMode;
pub use causality_types::tel::execution_context::GraphExecutionContext as TelExecutionContext;
pub use types::{TelEdgeTypes, TelNodeTypes};
pub use crate::state_manager::StateManager;
pub use intent_processor::IntentProcessor;

// Re-export optimization framework types
pub use crate::optimization::{
    OptimizationStrategy, OptimizationContext, StrategyRegistry, 
    PlanEvaluator
};
pub use crate::optimization::registry::DefaultStrategyRegistry;
pub use crate::strategies::{
    CapitalEfficiencyStrategy, PriorityBasedStrategy, 
    ProcessDataflowOrchestrationStrategy, ExpressionBasedStrategy
};

// Re-export key optimization types from causality-types
pub use causality_types::tel::{
    optimization::{
        TypedDomain, ResolutionPlan, ScoredPlan, DataflowOrchestrationStep,
        EffectCompatibility, ResourcePreference, ProcessDataflowInitiationHint
    },
    cost_model::{
        EffectCostModel, ResourceUsageEstimate, DomainSpecificEstimate
    },
    process_dataflow::{
        ProcessDataflowDefinition, ProcessDataflowInstanceState, 
        DataflowNode, DataflowEdge, ProcessDataflowReference
    }
};

// Re-export configuration types
pub use crate::config::{
    OptimizationConfig, StrategySelectionConfig, TypedDomainConfig,
    PdbOrchestrationConfig, PerformanceMonitoringConfig
};
