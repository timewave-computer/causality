// Purpose: Defines the execution context for the Temporal Effect Language (TEL) graph interpreter.

use crate::primitive::ids::{HandlerId, NodeId, DomainId, ExprId, ResourceId};
use crate::tel::common_refs::ResourceRef;
use crate::tel::mode::{ExecutionMode, InterpreterMode};
use crate::tel::optimization::TypedDomain;
use crate::tel::process_dataflow::ProcessDataflowInstanceState;
use crate::primitive::string::Str as CausalityStr;
use std::collections::{HashSet, HashMap, BTreeMap}; // Corrected import path

/// Placeholder for a Zero-Knowledge Proof related to a graph execution step.
/// This would be defined more concretely based on the specific ZK proving system used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkProof {
    /// Identifier for the statement or sub-circuit this proof corresponds to.
    /// (e.g., an Effect ID, a constraint ExprId, a specific transition)
    pub statement_id: String, // Or a more specific ID type

    /// The actual proof data.
    pub proof_bytes: Vec<u8>,

    /// Public inputs associated with this proof, if any.
    pub public_inputs: Option<Vec<u8>>, // Or a structured type
}

impl ZkProof {
    pub fn new(statement_id: String, proof_bytes: Vec<u8>, public_inputs: Option<Vec<u8>>) -> Self {
        Self { statement_id, proof_bytes, public_inputs }
    }
}

/// Constraints on effect execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionConstraint {
    /// Expression that must be satisfied
    pub expr_id: ExprId,
    
    /// Whether this constraint is required
    pub required: bool,
}

/// Context for executing effects in TEL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    /// Domain this execution belongs to
    pub domain_id: DomainId,
    
    /// Mode of execution
    pub mode: ExecutionMode,
    
    /// Interpreter mode
    pub interpreter_mode: InterpreterMode,
    
    /// Input resources
    pub input_resources: Vec<ResourceId>,
    
    /// Output resources
    pub output_resources: Vec<ResourceId>,
    
    /// Constraints on execution
    pub constraints: Vec<ExecutionConstraint>,
}

/// Strategy configuration for optimization
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionStrategyConfiguration {
    /// Strategy identifier
    pub strategy_id: CausalityStr,
    
    /// Strategy-specific parameters
    pub parameters: BTreeMap<CausalityStr, String>,
    
    /// Weight for this strategy in selection
    pub weight: f64,
    
    /// Whether this strategy is enabled
    pub enabled: bool,
}

/// Performance metrics for execution tracking
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMetrics {
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Number of effects processed
    pub effects_processed: u32,
    
    /// Number of dataflow steps executed
    pub dataflow_steps_executed: u32,
    
    /// Resource consumption estimates
    pub resource_consumption: BTreeMap<CausalityStr, u64>,
    
    /// Strategy performance data
    pub strategy_performance: BTreeMap<CausalityStr, f64>,
    
    /// Error count
    pub error_count: u32,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            execution_time_ms: 0,
            effects_processed: 0,
            dataflow_steps_executed: 0,
            resource_consumption: BTreeMap::new(),
            strategy_performance: BTreeMap::new(),
            error_count: 0,
        }
    }
}

/// Plan execution tracking information
#[derive(Debug, Clone, PartialEq)]
pub struct PlanExecutionState {
    /// Currently executing plan ID
    pub current_plan_id: Option<ExprId>,
    
    /// Completed plan steps
    pub completed_steps: Vec<CausalityStr>,
    
    /// Failed plan attempts
    pub failed_plans: Vec<ExprId>,
    
    /// Rollback checkpoints
    pub checkpoints: Vec<CausalityStr>,
}

impl Default for PlanExecutionState {
    fn default() -> Self {
        Self {
            current_plan_id: None,
            completed_steps: Vec::new(),
            failed_plans: Vec::new(),
            checkpoints: Vec::new(),
        }
    }
}

/// Defines the state and context for the Temporal Effect Language (TEL) graph interpreter
/// during the evaluation of an Effect Graph.
#[derive(Debug, Clone, Default)]
pub struct GraphExecutionContext {
    /// A stack or list of currently active handlers that influence effect processing.
    pub current_handlers: Vec<HandlerId>,

    /// Set of resources currently visible and accessible within the execution scope.
    pub visible_resources: HashSet<ResourceRef>,

    /// Set of effects that have already been completed in the current execution trace.
    pub completed_effects: HashSet<NodeId>,

    /// Represents the call stack of effects being processed, useful for cycle detection
    /// or context-dependent logic.
    pub call_stack: Vec<NodeId>,

    /// The current operational mode of the interpreter (e.g., Evaluate, Validate, Prove).
    pub interpreter_mode: InterpreterMode,

    /// A collection of ZK proofs accumulated or relevant during the execution.
    pub proofs: Vec<ZkProof>,

    /// The domain ID associated with this execution context
    pub domain_id: Option<DomainId>,
    
    // === OPTIMIZATION ENHANCEMENTS ===
    
    /// Current typed domain for optimization decisions
    pub current_typed_domain: TypedDomain,
    
    /// Active ProcessDataflowBlock instances
    pub active_pdb_instances: HashMap<ResourceId, ProcessDataflowInstanceState>,
    
    /// Strategy configuration for optimization
    pub strategy_configuration: Vec<ExecutionStrategyConfiguration>,
    
    /// Plan execution tracking
    pub plan_execution_state: PlanExecutionState,
    
    /// Performance metrics collection
    pub execution_metrics: ExecutionMetrics,
    
    /// General metadata for optimization and tracking
    pub metadata: BTreeMap<CausalityStr, String>,
    
    // TODO: Consider adding other relevant fields as per ADR or further design:
    // - current_effect_id: Option<NodeId>
    // - current_intent_id: Option<IntentId>
    // - error_log: Vec<GraphError>
    // - execution_trace: SomeTraceType
}

impl GraphExecutionContext {
    /// Creates a new, empty execution context with a specific interpreter mode.
    pub fn new(interpreter_mode: InterpreterMode) -> Self {
        Self {
            interpreter_mode,
            current_handlers: Vec::new(),
            visible_resources: HashSet::new(),
            completed_effects: HashSet::new(),
            call_stack: Vec::new(),
            proofs: Vec::new(),
            domain_id: None,
            current_typed_domain: TypedDomain::default(),
            active_pdb_instances: HashMap::new(),
            strategy_configuration: Vec::new(),
            plan_execution_state: PlanExecutionState::default(),
            execution_metrics: ExecutionMetrics::default(),
            metadata: BTreeMap::new(),
        }
    }
    
    /// Creates a new execution context with optimization settings
    pub fn new_with_optimization(
        interpreter_mode: InterpreterMode,
        typed_domain: TypedDomain,
    ) -> Self {
        Self {
            interpreter_mode,
            current_typed_domain: typed_domain,
            current_handlers: Vec::new(),
            visible_resources: HashSet::new(),
            completed_effects: HashSet::new(),
            call_stack: Vec::new(),
            proofs: Vec::new(),
            domain_id: None,
            active_pdb_instances: HashMap::new(),
            strategy_configuration: Vec::new(),
            plan_execution_state: PlanExecutionState::default(),
            execution_metrics: ExecutionMetrics::default(),
            metadata: BTreeMap::new(),
        }
    }
    
    /// Add a strategy configuration
    pub fn add_strategy_configuration(&mut self, config: ExecutionStrategyConfiguration) {
        self.strategy_configuration.push(config);
    }
    
    /// Get strategy configuration by ID
    pub fn get_strategy_configuration(&self, strategy_id: &CausalityStr) -> Option<&ExecutionStrategyConfiguration> {
        self.strategy_configuration.iter().find(|config| config.strategy_id == *strategy_id)
    }
    
    /// Update execution metrics
    pub fn update_metrics<F>(&mut self, updater: F) 
    where 
        F: FnOnce(&mut ExecutionMetrics)
    {
        updater(&mut self.execution_metrics);
    }
    
    /// Add metadata entry
    pub fn add_metadata(&mut self, key: CausalityStr, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata entry
    pub fn get_metadata(&self, key: &CausalityStr) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Add active PDB instance
    pub fn add_pdb_instance(&mut self, instance_id: ResourceId, state: ProcessDataflowInstanceState) {
        self.active_pdb_instances.insert(instance_id, state);
    }
    
    /// Get active PDB instance
    pub fn get_pdb_instance(&self, instance_id: &ResourceId) -> Option<&ProcessDataflowInstanceState> {
        self.active_pdb_instances.get(instance_id)
    }
    
    /// Update PDB instance state
    pub fn update_pdb_instance(&mut self, instance_id: &ResourceId, state: ProcessDataflowInstanceState) {
        self.active_pdb_instances.insert(*instance_id, state);
    }
    
    /// Remove PDB instance
    pub fn remove_pdb_instance(&mut self, instance_id: &ResourceId) -> Option<ProcessDataflowInstanceState> {
        self.active_pdb_instances.remove(instance_id)
    }
}
