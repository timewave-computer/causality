//! Execution context and modes for TEL graph interpretation
//!
//! Defines execution modes, contexts, and related types for the Temporal Effect Language (TEL) 
//! graph interpreter during the evaluation of Effect Graphs.

use crate::primitive::ids::{HandlerId, NodeId, DomainId, ExprId, ResourceId};
use crate::primitive::string::Str;
use crate::system::serialization::{Decode, Encode, SimpleSerialize, DecodeError};
use std::collections::{HashSet, HashMap, BTreeMap};

//-----------------------------------------------------------------------------
// Execution Modes
//-----------------------------------------------------------------------------

/// The mode in which a TEL effect should be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ExecutionMode {
    /// The effect should be executed eagerly, as soon as possible.
    #[default]
    Eager = 0,
    /// The effect should be executed only once all dependencies are satisfied.
    Strict = 1,
    /// The effect should be executed lazily, only when needed.
    Lazy = 2,
}

// Manually implement Encode for ExecutionMode
impl Encode for ExecutionMode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let value = match self {
            ExecutionMode::Eager => 0u8,
            ExecutionMode::Strict => 1u8,
            ExecutionMode::Lazy => 2u8,
        };
        vec![value]
    }
}

// Manually implement Decode for ExecutionMode
impl Decode for ExecutionMode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for ExecutionMode".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(ExecutionMode::Eager),
            1 => Ok(ExecutionMode::Strict),
            2 => Ok(ExecutionMode::Lazy),
            _ => Err(DecodeError {
                message: format!("Invalid ExecutionMode value: {}", bytes[0]),
            }),
        }
    }
}

// Implement SimpleSerialize for ExecutionMode
impl SimpleSerialize for ExecutionMode {}

/// The mode in which the TEL interpreter should operate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum InterpreterMode {
    /// The interpreter should execute effects sequentially.
    #[default]
    Sequential = 0,
    /// The interpreter should execute effects in parallel when possible.
    Parallel = 1,
    /// The interpreter should optimize for ZK proof generation.
    ZkOptimized = 2,
    /// The interpreter should evaluate effects (compute results).
    Evaluate = 3,
    /// The interpreter should simulate effects (dry run).
    Simulate = 4,
}

// Manually implement Encode for InterpreterMode
impl Encode for InterpreterMode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let value = match self {
            InterpreterMode::Sequential => 0u8,
            InterpreterMode::Parallel => 1u8,
            InterpreterMode::ZkOptimized => 2u8,
            InterpreterMode::Evaluate => 3u8,
            InterpreterMode::Simulate => 4u8,
        };
        vec![value]
    }
}

// Manually implement Decode for InterpreterMode
impl Decode for InterpreterMode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for InterpreterMode".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(InterpreterMode::Sequential),
            1 => Ok(InterpreterMode::Parallel),
            2 => Ok(InterpreterMode::ZkOptimized),
            3 => Ok(InterpreterMode::Evaluate),
            4 => Ok(InterpreterMode::Simulate),
            _ => Err(DecodeError {
                message: format!("Invalid InterpreterMode value: {}", bytes[0]),
            }),
        }
    }
}

// Implement SimpleSerialize for InterpreterMode
impl SimpleSerialize for InterpreterMode {}

//-----------------------------------------------------------------------------
// Execution Context Types
//-----------------------------------------------------------------------------

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
    pub strategy_id: Str,
    
    /// Strategy-specific parameters
    pub parameters: BTreeMap<Str, String>,
    
    /// Weight for this strategy in selection
    pub weight: f64,
    
    /// Whether this strategy is enabled
    pub enabled: bool,
}

/// Performance metrics for execution tracking
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExecutionMetrics {
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Number of effects processed
    pub effects_processed: u32,
    
    /// Number of dataflow steps executed
    pub dataflow_steps_executed: u32,
    
    /// Resource consumption estimates
    pub resource_consumption: BTreeMap<Str, u64>,
    
    /// Strategy performance data
    pub strategy_performance: BTreeMap<Str, f64>,
    
    /// Error count
    pub error_count: u32,
}



/// Plan execution tracking information
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanExecutionState {
    /// Currently executing plan ID
    pub current_plan_id: Option<ExprId>,
    
    /// Completed plan steps
    pub completed_steps: Vec<Str>,
    
    /// Failed plan attempts
    pub failed_plans: Vec<ExprId>,
    
    /// Rollback checkpoints
    pub checkpoints: Vec<Str>,
}



/// A reference to a resource within the TEL system (simplified version)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    /// The ID of the resource being referenced
    pub resource_id: ResourceId,
    /// The domain where this resource exists
    pub domain_id: DomainId,
    /// Optional type information for the resource
    pub resource_type: Option<Str>,
}

/// Typed domain for optimization (simplified version)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedDomain {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Domain type
    pub domain_type: Str,
}

/// Process dataflow instance state (simplified version)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDataflowInstanceState {
    /// Instance identifier
    pub instance_id: ResourceId,
    /// Current state
    pub state: Str,
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
    pub metadata: BTreeMap<Str, String>,
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

    /// Creates a new execution context with optimization support
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

    /// Get a strategy configuration by ID
    pub fn get_strategy_configuration(&self, strategy_id: &Str) -> Option<&ExecutionStrategyConfiguration> {
        self.strategy_configuration.iter().find(|c| c.strategy_id == *strategy_id)
    }

    /// Update execution metrics
    pub fn update_metrics<F>(&mut self, updater: F) 
    where 
        F: FnOnce(&mut ExecutionMetrics)
    {
        updater(&mut self.execution_metrics);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: Str, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &Str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Add a ProcessDataflowBlock instance
    pub fn add_pdb_instance(&mut self, instance_id: ResourceId, state: ProcessDataflowInstanceState) {
        self.active_pdb_instances.insert(instance_id, state);
    }

    /// Get a ProcessDataflowBlock instance
    pub fn get_pdb_instance(&self, instance_id: &ResourceId) -> Option<&ProcessDataflowInstanceState> {
        self.active_pdb_instances.get(instance_id)
    }

    /// Update a ProcessDataflowBlock instance
    pub fn update_pdb_instance(&mut self, instance_id: &ResourceId, state: ProcessDataflowInstanceState) {
        self.active_pdb_instances.insert(*instance_id, state);
    }

    /// Remove a ProcessDataflowBlock instance
    pub fn remove_pdb_instance(&mut self, instance_id: &ResourceId) -> Option<ProcessDataflowInstanceState> {
        self.active_pdb_instances.remove(instance_id)
    }
} 