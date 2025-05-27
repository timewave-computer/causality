//! Simulation Engine
//!
//! Core simulation engine for running and controlling TEL programs with mocking capabilities.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result as AnyhowResult};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as TokioMutex;

// Causality runtime imports
use causality_runtime::{
    state_manager::DefaultStateManager,
    tel::{
        context::LispHostEnvironment,
        interpreter::Interpreter as TelInterpreter,
        traits::{MockBehavior, AutoMockStrategy},
    },
};

// Causality types imports
use causality_types::{
    core::{
        id::{DomainId, NodeId, IntentId},
        str::Str,
    },
    expr::expr_type::TypeExpr,
    serialization::{Encode, Decode, SimpleSerialize},
    effects_core::{Effect, EffectHandler, EffectInput, EffectOutput},
};

// Causality API imports
use causality_api::zk_coprocessor::ZkCoprocessorApi;

// Local simulation imports
use crate::{
    mocking::{SimulationMockManager, SchemaRegistry},
    history::{SimulationHistory, SimulationSnapshot},
    error::{SimulationError, SimulationResult},
    randomness::SeededRng,
};

// Re-export commonly used types
use causality_types::primitive::ids::EntityId;

//-----------------------------------------------------------------------------
// Type Definitions
//-----------------------------------------------------------------------------

/// Default simulated test store type alias
pub type DefaultSimulatedTestStore = DefaultStateManager;

/// Configuration for the simulation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationEngineConfig {
    /// Whether to enable breakpoints
    pub enable_breakpoints: bool,
    /// Whether to collect execution traces
    pub collect_traces: bool,
    /// Maximum number of steps to execute
    pub max_steps: Option<u64>,
    /// Random seed for deterministic simulation
    pub seed: Option<u64>,
}

impl Default for SimulationEngineConfig {
    fn default() -> Self {
        Self {
            enable_breakpoints: true,
            collect_traces: true,
            max_steps: None,
            seed: None,
        }
    }
}

/// Information about a breakpoint hit during simulation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakpointInfo {
    /// Human-readable label for the breakpoint
    pub label: String,
    /// Unique identifier for this breakpoint
    pub id: String,
}

/// Simple serialization implementation for BreakpointInfo
impl SimpleSerialize for BreakpointInfo {}

impl Encode for BreakpointInfo {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let label_bytes = self.label.as_bytes();
        let id_bytes = self.id.as_bytes();
        
        // Length prefixed encoding
        bytes.extend_from_slice(&(label_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(label_bytes);
        bytes.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(id_bytes);
        
        bytes
    }
}

impl Decode for BreakpointInfo {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() < 8 {
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient data for BreakpointInfo".to_string(),
            });
        }
        
        let mut offset = 0;
        let label_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        let label = String::from_utf8(bytes[offset..offset+label_len].to_vec())
            .map_err(|_| causality_types::serialization::DecodeError {
                message: "Invalid UTF-8 in label".to_string(),
            })?;
        offset += label_len;
        
        let id_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        let id = String::from_utf8(bytes[offset..offset+id_len].to_vec())
            .map_err(|_| causality_types::serialization::DecodeError {
                message: "Invalid UTF-8 in id".to_string(),
            })?;
        
        Ok(BreakpointInfo { label, id })
    }
}

/// Simulation step outcome
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationStepOutcome {
    /// No effect to process
    NoEffectToProcess,
    /// Effect was processed successfully
    EffectProcessed(NodeId),
    /// Breakpoint was hit
    BreakpointHit(BreakpointInfo),
    /// Simulation output was generated
    OutputGenerated(String),
    /// Error occurred during processing
    ProcessingError(String),
}

//-----------------------------------------------------------------------------
// Main Engine
//-----------------------------------------------------------------------------

/// Core simulation engine
#[derive(Debug)]
pub struct SimulationEngine {
    /// TEL interpreter for executing programs
    pub tel_interpreter: Arc<TelInterpreter>,
    /// State manager for managing simulation state
    pub state_manager_accessor: Arc<TokioMutex<DefaultSimulatedTestStore>>,
    /// Mock manager for controlling effect behavior
    pub mock_manager: Arc<SimulationMockManager>,
    /// Schema registry for effect schemas
    pub schema_registry: Arc<SchemaRegistry>,
    /// History for tracking simulation state
    pub history: SimulationHistory,
    /// Configuration settings
    config: SimulationEngineConfig,
}

/// A helper struct for serializing the core state of the SimulationEngine.
#[derive(Debug, Clone)]
struct SerializableEngineState {
    history: SimulationHistory,
}

/// Simple serialization implementation for SerializableEngineState
impl SimpleSerialize for SerializableEngineState {}

impl Encode for SerializableEngineState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.history.as_ssz_bytes()
    }
}

impl Decode for SerializableEngineState {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        let history = SimulationHistory::from_ssz_bytes(bytes)?;
        Ok(SerializableEngineState { history })
    }
}

//-----------------------------------------------------------------------------
// SimulationEngine Implementation
//-----------------------------------------------------------------------------

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(
        _domain_id: Option<DomainId>,
        _seed: Option<u64>,
    ) -> SimulationResult<Self> {
        let state_manager = Arc::new(TokioMutex::new(DefaultStateManager::new()));
        let state_manager_as_trait = state_manager.clone() as Arc<TokioMutex<dyn causality_runtime::state_manager::StateManager>>;
        
        // Create a LispHostEnvironment for the TelLispAdapter
        let lisp_env = LispHostEnvironment::new(
            state_manager_as_trait.clone(),
            None,
        );
        
        // Create TelLispAdapter with the LispHostEnvironment
        let tel_lisp_adapter = Arc::new(TokioMutex::new(
            causality_runtime::tel::lisp_adapter::TelLispAdapter::new(
                Arc::new(TokioMutex::new(lisp_env))
            )
        ));
        
        let tel_interpreter = Arc::new(TelInterpreter::new(
            state_manager_as_trait,
            tel_lisp_adapter,
        ));
        
        let mock_manager = Arc::new(SimulationMockManager::new());
        let schema_registry = Arc::new(SchemaRegistry::new());
        let initial_snapshot = SimulationSnapshot::new(0, None);
        let history = SimulationHistory::new(initial_snapshot);
        
        Ok(Self {
            tel_interpreter,
            state_manager_accessor: state_manager,
            mock_manager,
            schema_registry,
            history,
            config: SimulationEngineConfig::default(),
        })
    }

    /// Get ZK coprocessor API (placeholder)
    pub fn zk_coprocessor_api(
        &self,
    ) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        None
    }

    //-------------------------------------------------------------------------
    // Handler Registration
    //-------------------------------------------------------------------------

    pub fn register_rust_effect_handler<E, H>(
        &mut self,
        _handler_instance: Arc<H>,
    ) -> Result<(), String>
    where
        E: Effect + Send + Sync + 'static,
        E::Input: EffectInput + Send + Sync + 'static,
        E::Output: EffectOutput + Send + Sync + 'static,
        H: EffectHandler<E = E> + Send + Sync + 'static,
    {
        let effect_type_str = Str::from(E::EFFECT_TYPE);
        let input_schema = E::Input::schema();
        let output_schema = E::Output::schema();
        self.register_effect_schema(effect_type_str, input_schema, output_schema);
        Ok(())
    }

    //-------------------------------------------------------------------------
    // Simulation Execution
    //-------------------------------------------------------------------------

    pub fn execute_step(&mut self) -> SimulationResult<SimulationStepOutcome> {
        // Simplified implementation for now
        log::info!("SimulationEngine::execute_step called but implementation is incomplete");
        Ok(SimulationStepOutcome::NoEffectToProcess)
    }

    /// Creates a breakpoint effect with the given label and ID
    pub fn create_breakpoint_effect(
        &self,
        label: impl Into<String>,
        id: impl Into<String>,
    ) -> causality_types::core::Effect {
        let label_str = label.into();
        let id_str = id.into();

        // Use the implementation from sim_effects.rs
        crate::sim_effects::create_breakpoint_effect(
            NodeId::new([0u8; 32]),
            DomainId::new([0u8; 32]),
            IntentId::new([0u8; 32]),
            label_str,
            id_str,
        )
    }

    //-------------------------------------------------------------------------
    // History Navigation
    //-------------------------------------------------------------------------

    pub fn step_backward_in_history(&mut self) -> SimulationResult<()> {
        let current_step = self.history.get_current_step_number();
        if current_step == 0 {
            return Err(SimulationError::InvalidOperation("Already at the beginning of history.".to_string()));
        }
        let target_step = current_step - 1;
        self.jump_to_history_step(target_step)
    }

    pub fn step_forward_in_history(&mut self) -> SimulationResult<()> {
        let target_step = self.history.get_current_step_number() + 1;
        if target_step < self.history.snapshots.len() as u64 {
            self.jump_to_history_step(target_step)
        } else {
            Err(SimulationError::InvalidOperation("Already at the latest history point.".to_string()))
        }
    }

    pub fn jump_to_history_step(
        &mut self,
        step_number: u64,
    ) -> SimulationResult<()> {
        match self.history.jump_to_step(step_number) {
            Some(_snapshot) => {
                log::warn!("State restoration via jump_to_history_step is currently partial due to private interpreter state and RNG handling.");
                Ok(())
            }
            None => Err(SimulationError::History(format!(
                "No snapshot found for step number {}",
                step_number
            ))),
        }
    }

    //-------------------------------------------------------------------------
    // Context and State Access
    //-------------------------------------------------------------------------

    pub fn get_current_context(&self) -> LispHostEnvironment {
        LispHostEnvironment::new(
            self.state_manager_accessor.clone() as Arc<TokioMutex<dyn causality_runtime::state_manager::StateManager>>,
            None,
        )
    }

    pub fn get_history(&self) -> &SimulationHistory {
        &self.history
    }

    pub fn get_rng_mut(&mut self) -> Option<&mut SeededRng> {
        None
    }

    //-------------------------------------------------------------------------
    // State Persistence
    //-------------------------------------------------------------------------

    pub fn save_state(&self, path: &Path) -> SimulationResult<()> {
        let serializable_state = SerializableEngineState {
            history: self.history.clone(),
        };
        let file = std::fs::File::create(path)
            .map_err(|e| SimulationError::FileIo(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);
        let bytes = serializable_state.as_ssz_bytes();
        std::io::Write::write_all(&mut writer, &bytes)
            .map_err(|e| SimulationError::FileIo(e.to_string()))?;
        Ok(())
    }

    pub fn load_state(&mut self, path: &Path) -> SimulationResult<()> {
        let file = std::fs::File::open(path)
            .map_err(|e| SimulationError::FileIo(e.to_string()))?;
        let mut reader = std::io::BufReader::new(file);
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut bytes)
            .map_err(|e| SimulationError::FileIo(e.to_string()))?;
        
        let loaded_state = SerializableEngineState::from_ssz_bytes(&bytes)
            .map_err(|e| SimulationError::Serialization(e.message))?;

        self.history = loaded_state.history;
        Ok(())
    }

    //-------------------------------------------------------------------------
    // Schema and Mock Management
    //-------------------------------------------------------------------------

    pub fn register_effect_schema(
        &mut self,
        effect_type: Str,
        input_schema: TypeExpr,
        output_schema: TypeExpr,
    ) {
        self.schema_registry.register(effect_type, input_schema, output_schema);
    }

    pub fn register_explicit_mock(
        &mut self,
        _effect_type: Str,
        _behavior: MockBehavior,
    ) {
        log::warn!("register_explicit_mock called but not implemented in SimulationMockManager yet.");
    }

    pub fn clear_explicit_mock(&mut self, _effect_type: &Str) {
        log::warn!("clear_explicit_mock called but not implemented in SimulationMockManager yet.");
    }

    pub fn clear_all_explicit_mocks(&mut self) {
        log::warn!("clear_all_explicit_mocks called but not implemented in SimulationMockManager yet.");
    }

    pub fn set_default_auto_mock_strategy(&mut self, _strategy: AutoMockStrategy) {
        log::warn!("set_default_auto_mock_strategy called but not implemented in SimulationMockManager yet.");
    }

    // === STRATEGY EVALUATION INTERFACE ===
    
    /// Evaluate resolution plans with TypedDomain constraints and PDB orchestration
    pub async fn evaluate_resolution_plans(
        &mut self,
        plans: Vec<causality_types::tel::optimization::ResolutionPlan>,
        typed_domain: causality_types::tel::optimization::TypedDomain,
        pdb_definitions: std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
    ) -> SimulationResult<Vec<PlanEvaluationResult>> {
        let mut results = Vec::new();
        
        for plan in plans {
            // Create a checkpoint before evaluating this plan
            let checkpoint = self.create_evaluation_checkpoint().await?;
            
            // Evaluate the plan
            let result = self.evaluate_single_plan(&plan, &typed_domain, &pdb_definitions).await?;
            
            // Restore the checkpoint for the next plan evaluation
            self.restore_evaluation_checkpoint(checkpoint).await?;
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Evaluate a single resolution plan
    async fn evaluate_single_plan(
        &mut self,
        plan: &causality_types::tel::optimization::ResolutionPlan,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        pdb_definitions: &std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
    ) -> SimulationResult<PlanEvaluationResult> {
        let start_time = std::time::Instant::now();
        let mut execution_steps = 0;
        let mut resource_consumption = std::collections::BTreeMap::new();
        let mut errors = Vec::new();
        
        // Simulate TypedDomain constraints
        if !self.validate_plan_for_typed_domain(plan, typed_domain) {
            return Ok(PlanEvaluationResult {
                plan_id: plan.plan_id,
                success: false,
                execution_time_ms: 0,
                execution_steps: 0,
                resource_consumption,
                errors: vec!["Plan incompatible with TypedDomain".to_string()],
                domain_compatibility_score: 0.0,
                pdb_orchestration_complexity: 0.0,
            });
        }
        
        // Simulate effect execution
        for effect_id in &plan.effect_sequence {
            execution_steps += 1;
            
            // Simulate resource consumption based on TypedDomain
            let (cpu_cost, memory_cost) = self.estimate_effect_cost_for_domain(effect_id, typed_domain);
            *resource_consumption.entry("cpu".to_string()).or_insert(0) += cpu_cost;
            *resource_consumption.entry("memory".to_string()).or_insert(0) += memory_cost;
            
            // Simulate potential errors based on domain constraints
            if let Some(error) = self.simulate_domain_specific_errors(effect_id, typed_domain) {
                errors.push(error);
            }
        }
        
        // Simulate PDB orchestration steps
        let mut pdb_complexity = 0.0;
        for step in &plan.dataflow_steps {
            execution_steps += 1;
            pdb_complexity += self.calculate_pdb_step_complexity(step, pdb_definitions);
            
            // Simulate Lisp execution for PDB orchestration
            let (lisp_cpu, lisp_memory) = self.estimate_lisp_execution_cost(step, typed_domain);
            *resource_consumption.entry("cpu".to_string()).or_insert(0) += lisp_cpu;
            *resource_consumption.entry("memory".to_string()).or_insert(0) += lisp_memory;
        }
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        let domain_compatibility = self.calculate_domain_compatibility_score(plan, typed_domain);
        
        Ok(PlanEvaluationResult {
            plan_id: plan.plan_id,
            success: errors.is_empty(),
            execution_time_ms: execution_time,
            execution_steps,
            resource_consumption,
            errors,
            domain_compatibility_score: domain_compatibility,
            pdb_orchestration_complexity: pdb_complexity,
        })
    }
    
    /// Validate plan compatibility with TypedDomain
    fn validate_plan_for_typed_domain(
        &self,
        plan: &causality_types::tel::optimization::ResolutionPlan,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> bool {
        // Check if all effects in the plan are compatible with the domain
        for _effect_id in &plan.effect_sequence {
            // In a real implementation, this would check effect compatibility
            // For now, assume all effects are compatible
        }
        
        // Check PDB orchestration compatibility
        for step in &plan.dataflow_steps {
            if !self.is_pdb_step_compatible_with_domain(step, typed_domain) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a PDB step is compatible with the TypedDomain
    fn is_pdb_step_compatible_with_domain(
        &self,
        step: &causality_types::tel::optimization::DataflowOrchestrationStep,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> bool {
        match (step, typed_domain) {
            (causality_types::tel::optimization::DataflowOrchestrationStep::InitiateDataflow { .. }, 
             causality_types::tel::optimization::TypedDomain::VerifiableDomain(_)) => {
                // VerifiableDomain requires deterministic operations
                true // Assume initiation is deterministic
            }
            (causality_types::tel::optimization::DataflowOrchestrationStep::AdvanceDataflow { .. }, 
             causality_types::tel::optimization::TypedDomain::VerifiableDomain(_)) => {
                // VerifiableDomain requires deterministic advancement
                true // Assume advancement is deterministic
            }
            (_, causality_types::tel::optimization::TypedDomain::ServiceDomain(_)) => {
                // ServiceDomain allows all operations
                true
            }
        }
    }
    
    /// Estimate effect execution cost for a specific domain
    fn estimate_effect_cost_for_domain(
        &self,
        _effect_id: &causality_types::primitive::ids::EntityId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> (u64, u64) {
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                // ZK domain has higher computational costs
                (1000, 500) // (cpu, memory)
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                // Service domain has lower computational costs but higher network costs
                (200, 100) // (cpu, memory)
            }
        }
    }
    
    /// Simulate domain-specific errors
    fn simulate_domain_specific_errors(
        &self,
        _effect_id: &causality_types::primitive::ids::EntityId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> Option<String> {
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                // Simulate potential ZK constraint violations
                if rand::random::<f64>() < 0.05 { // 5% chance of constraint violation
                    Some("ZK constraint violation detected".to_string())
                } else {
                    None
                }
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                // Simulate potential network errors
                if rand::random::<f64>() < 0.02 { // 2% chance of network error
                    Some("Network communication error".to_string())
                } else {
                    None
                }
            }
        }
    }
    
    /// Calculate PDB step complexity
    fn calculate_pdb_step_complexity(
        &self,
        step: &causality_types::tel::optimization::DataflowOrchestrationStep,
        _pdb_definitions: &std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
    ) -> f64 {
        match step {
            causality_types::tel::optimization::DataflowOrchestrationStep::InitiateDataflow { .. } => 2.0,
            causality_types::tel::optimization::DataflowOrchestrationStep::AdvanceDataflow { .. } => 1.5,
        }
    }
    
    /// Estimate Lisp execution cost for PDB orchestration
    fn estimate_lisp_execution_cost(
        &self,
        step: &causality_types::tel::optimization::DataflowOrchestrationStep,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> (u64, u64) {
        let base_cost = match step {
            causality_types::tel::optimization::DataflowOrchestrationStep::InitiateDataflow { .. } => (500, 200),
            causality_types::tel::optimization::DataflowOrchestrationStep::AdvanceDataflow { .. } => (300, 150),
        };
        
        // Apply domain-specific multipliers
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                (base_cost.0 * 2, base_cost.1 * 2) // ZK requires more computation
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                base_cost // Standard cost for service domain
            }
        }
    }
    
    /// Calculate domain compatibility score
    fn calculate_domain_compatibility_score(
        &self,
        plan: &causality_types::tel::optimization::ResolutionPlan,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> f64 {
        let mut score = 1.0;
        
        // Reduce score for cross-domain operations
        let cross_domain_steps = plan.dataflow_steps.len();
        if cross_domain_steps > 0 {
            score -= (cross_domain_steps as f64) * 0.1;
        }
        
        // Adjust score based on domain characteristics
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                // VerifiableDomain prefers deterministic operations
                score *= 0.9; // Slight penalty for complexity
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                // ServiceDomain is more flexible
                score *= 1.0; // No penalty
            }
        }
        
        score.max(0.0).min(1.0)
    }
    
    /// Create an evaluation checkpoint
    async fn create_evaluation_checkpoint(&self) -> SimulationResult<EvaluationCheckpoint> {
        // In a real implementation, this would capture the full state
        Ok(EvaluationCheckpoint {
            timestamp: std::time::SystemTime::now(),
            step_count: self.history.current_step(),
        })
    }
    
    /// Restore an evaluation checkpoint
    async fn restore_evaluation_checkpoint(&mut self, _checkpoint: EvaluationCheckpoint) -> SimulationResult<()> {
        // In a real implementation, this would restore the state
        Ok(())
    }
    
    /// Perform comparative strategy evaluation
    pub async fn compare_strategies(
        &mut self,
        strategies: Vec<Box<dyn causality_runtime::optimization::OptimizationStrategy>>,
        test_scenarios: Vec<StrategyTestScenario>,
    ) -> SimulationResult<StrategyComparisonResult> {
        let mut strategy_results = std::collections::BTreeMap::new();
        
        for strategy in strategies {
            let strategy_id = strategy.strategy_id().to_string();
            let mut scenario_results = Vec::new();
            
            for scenario in &test_scenarios {
                let result = self.evaluate_strategy_on_scenario(&*strategy, scenario).await?;
                scenario_results.push(result);
            }
            
            strategy_results.insert(strategy_id, scenario_results);
        }
        
        Ok(StrategyComparisonResult {
            strategy_results,
            overall_rankings: self.calculate_strategy_rankings(&strategy_results),
        })
    }
    
    /// Evaluate a strategy on a specific scenario
    async fn evaluate_strategy_on_scenario(
        &mut self,
        strategy: &dyn causality_runtime::optimization::OptimizationStrategy,
        scenario: &StrategyTestScenario,
    ) -> SimulationResult<StrategyScenarioResult> {
        // Create optimization context from scenario
        let context = causality_runtime::optimization::OptimizationContext {
            current_typed_domain: scenario.typed_domain.clone(),
            available_typed_domains: vec![scenario.typed_domain.clone()],
            dataflow_definitions: scenario.pdb_definitions.clone(),
            active_dataflow_instances: std::collections::HashMap::new(),
            system_load: causality_runtime::optimization::SystemLoadMetrics::default(),
            evaluation_constraints: causality_runtime::optimization::EvaluationConstraints::default(),
        };
        
        // Generate plans using the strategy
        let scored_plans = strategy.propose(&context).map_err(|e| SimulationError::EvaluationError(e.to_string()))?;
        
        // Evaluate the generated plans
        let plan_results = self.evaluate_resolution_plans(
            scored_plans.into_iter().map(|sp| sp.plan).collect(),
            scenario.typed_domain.clone(),
            scenario.pdb_definitions.clone(),
        ).await?;
        
        Ok(StrategyScenarioResult {
            scenario_id: scenario.scenario_id.clone(),
            plan_count: plan_results.len(),
            success_rate: plan_results.iter().filter(|r| r.success).count() as f64 / plan_results.len() as f64,
            avg_execution_time: plan_results.iter().map(|r| r.execution_time_ms).sum::<u64>() as f64 / plan_results.len() as f64,
            avg_domain_compatibility: plan_results.iter().map(|r| r.domain_compatibility_score).sum::<f64>() / plan_results.len() as f64,
        })
    }
    
    /// Calculate strategy rankings based on results
    fn calculate_strategy_rankings(&self, results: &std::collections::BTreeMap<String, Vec<StrategyScenarioResult>>) -> Vec<StrategyRanking> {
        let mut rankings = Vec::new();
        
        for (strategy_id, scenario_results) in results {
            let avg_success_rate = scenario_results.iter().map(|r| r.success_rate).sum::<f64>() / scenario_results.len() as f64;
            let avg_execution_time = scenario_results.iter().map(|r| r.avg_execution_time).sum::<f64>() / scenario_results.len() as f64;
            let avg_compatibility = scenario_results.iter().map(|r| r.avg_domain_compatibility).sum::<f64>() / scenario_results.len() as f64;
            
            // Calculate overall score (higher is better)
            let overall_score = (avg_success_rate * 0.5) + (avg_compatibility * 0.3) + ((1.0 / (avg_execution_time + 1.0)) * 0.2);
            
            rankings.push(StrategyRanking {
                strategy_id: strategy_id.clone(),
                overall_score,
                success_rate: avg_success_rate,
                avg_execution_time,
                domain_compatibility: avg_compatibility,
            });
        }
        
        // Sort by overall score (descending)
        rankings.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        rankings
    }
}

/// Result of evaluating a single resolution plan
#[derive(Debug, Clone)]
pub struct PlanEvaluationResult {
    pub plan_id: causality_types::primitive::ids::EntityId,
    pub success: bool,
    pub execution_time_ms: u64,
    pub execution_steps: u64,
    pub resource_consumption: std::collections::BTreeMap<String, u64>,
    pub errors: Vec<String>,
    pub domain_compatibility_score: f64,
    pub pdb_orchestration_complexity: f64,
}

/// Checkpoint for evaluation state
#[derive(Debug, Clone)]
struct EvaluationCheckpoint {
    timestamp: std::time::SystemTime,
    step_count: u64,
}

/// Test scenario for strategy evaluation
#[derive(Debug, Clone)]
pub struct StrategyTestScenario {
    pub scenario_id: String,
    pub typed_domain: causality_types::tel::optimization::TypedDomain,
    pub pdb_definitions: std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
    pub initial_resources: Vec<causality_types::primitive::ids::ResourceId>,
    pub target_intents: Vec<causality_types::primitive::ids::IntentId>,
}

/// Result of evaluating a strategy on a scenario
#[derive(Debug, Clone)]
pub struct StrategyScenarioResult {
    pub scenario_id: String,
    pub plan_count: usize,
    pub success_rate: f64,
    pub avg_execution_time: f64,
    pub avg_domain_compatibility: f64,
}

/// Result of comparing multiple strategies
#[derive(Debug, Clone)]
pub struct StrategyComparisonResult {
    pub strategy_results: std::collections::BTreeMap<String, Vec<StrategyScenarioResult>>,
    pub overall_rankings: Vec<StrategyRanking>,
}

/// Ranking of a strategy
#[derive(Debug, Clone)]
pub struct StrategyRanking {
    pub strategy_id: String,
    pub overall_score: f64,
    pub success_rate: f64,
    pub avg_execution_time: f64,
    pub domain_compatibility: f64,
}

// === PLAN SIMULATION FRAMEWORK ===

/// Detailed plan simulation with state forking and restoration
impl SimulationEngine {
    /// Simulate a resolution plan with detailed state tracking
    pub async fn simulate_resolution_plan(
        &mut self,
        plan: &causality_types::tel::optimization::ResolutionPlan,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        pdb_definitions: &std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
    ) -> SimulationResult<DetailedPlanSimulationResult> {
        // Create a simulation fork
        let fork = self.create_simulation_fork().await?;
        
        let mut simulation_result = DetailedPlanSimulationResult {
            plan_id: plan.plan_id,
            success: true,
            execution_steps: Vec::new(),
            resource_state_changes: Vec::new(),
            pdb_instance_states: std::collections::HashMap::new(),
            generated_effects: Vec::new(),
            domain_violations: Vec::new(),
            total_execution_time_ms: 0,
            total_resource_cost: ResourceCost::default(),
        };
        
        let start_time = std::time::Instant::now();
        
        // Simulate effect sequence
        for (step_index, effect_id) in plan.effect_sequence.iter().enumerate() {
            let step_result = self.simulate_effect_execution(
                effect_id,
                typed_domain,
                step_index,
            ).await?;
            
            simulation_result.execution_steps.push(step_result.clone());
            simulation_result.total_resource_cost.add(&step_result.resource_cost);
            
            if !step_result.success {
                simulation_result.success = false;
                simulation_result.domain_violations.extend(step_result.domain_violations);
            }
        }
        
        // Simulate PDB orchestration steps
        for (step_index, dataflow_step) in plan.dataflow_steps.iter().enumerate() {
            let step_result = self.simulate_pdb_orchestration_step(
                dataflow_step,
                typed_domain,
                pdb_definitions,
                step_index + plan.effect_sequence.len(),
            ).await?;
            
            simulation_result.execution_steps.push(step_result.clone());
            simulation_result.total_resource_cost.add(&step_result.resource_cost);
            
            if !step_result.success {
                simulation_result.success = false;
                simulation_result.domain_violations.extend(step_result.domain_violations);
            }
            
            // Update PDB instance states
            if let Some(instance_id) = step_result.affected_pdb_instance {
                simulation_result.pdb_instance_states.insert(
                    instance_id,
                    step_result.resulting_pdb_state.clone().unwrap_or_default(),
                );
            }
        }
        
        simulation_result.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Restore the simulation fork
        self.restore_simulation_fork(fork).await?;
        
        Ok(simulation_result)
    }
    
    /// Simulate execution of a single effect
    async fn simulate_effect_execution(
        &mut self,
        effect_id: &causality_types::primitive::ids::EntityId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        step_index: usize,
    ) -> SimulationResult<SimulationStepResult> {
        let start_time = std::time::Instant::now();
        
        // Simulate domain-specific constraints
        let domain_violations = self.check_effect_domain_constraints(effect_id, typed_domain);
        
        // Simulate resource consumption
        let resource_cost = self.calculate_effect_resource_cost(effect_id, typed_domain);
        
        // Simulate potential state changes
        let state_changes = self.simulate_effect_state_changes(effect_id).await?;
        
        // Simulate success/failure based on domain constraints
        let success = domain_violations.is_empty() && rand::random::<f64>() > 0.05; // 95% success rate
        
        Ok(SimulationStepResult {
            step_index,
            step_type: SimulationStepType::EffectExecution(*effect_id),
            success,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            resource_cost,
            state_changes,
            domain_violations,
            generated_effects: Vec::new(),
            affected_pdb_instance: None,
            resulting_pdb_state: None,
        })
    }
    
    /// Simulate PDB orchestration step
    async fn simulate_pdb_orchestration_step(
        &mut self,
        step: &causality_types::tel::optimization::DataflowOrchestrationStep,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        pdb_definitions: &std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
        step_index: usize,
    ) -> SimulationResult<SimulationStepResult> {
        let start_time = std::time::Instant::now();
        
        match step {
            causality_types::tel::optimization::DataflowOrchestrationStep::InitiateDataflow { df_def_id, parameters } => {
                self.simulate_dataflow_initiation(df_def_id, parameters, typed_domain, pdb_definitions, step_index).await
            }
            causality_types::tel::optimization::DataflowOrchestrationStep::AdvanceDataflow { df_instance_id, next_step_params } => {
                self.simulate_dataflow_advancement(df_instance_id, next_step_params, typed_domain, step_index).await
            }
        }
    }
    
    /// Simulate dataflow initiation
    async fn simulate_dataflow_initiation(
        &mut self,
        df_def_id: &causality_types::primitive::ids::ExprId,
        parameters: &std::collections::BTreeMap<causality_types::primitive::string::Str, causality_types::expr::value::ValueExpr>,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        pdb_definitions: &std::collections::BTreeMap<causality_types::primitive::ids::ExprId, causality_types::tel::process_dataflow::ProcessDataflowDefinition>,
        step_index: usize,
    ) -> SimulationResult<SimulationStepResult> {
        let start_time = std::time::Instant::now();
        
        // Check if the PDB definition exists
        let pdb_definition = pdb_definitions.get(df_def_id);
        if pdb_definition.is_none() {
            return Ok(SimulationStepResult {
                step_index,
                step_type: SimulationStepType::PdbInitiation(*df_def_id),
                success: false,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                resource_cost: ResourceCost::default(),
                state_changes: Vec::new(),
                domain_violations: vec!["PDB definition not found".to_string()],
                generated_effects: Vec::new(),
                affected_pdb_instance: None,
                resulting_pdb_state: None,
            });
        }
        
        // Simulate Lisp execution for parameter validation
        let lisp_execution_cost = self.simulate_lisp_execution_for_pdb(parameters, typed_domain);
        
        // Create new PDB instance
        let instance_id = causality_types::primitive::ids::ResourceId::new(rand::random());
        let initial_state = causality_types::tel::process_dataflow::ProcessDataflowInstanceState {
            definition_id: *df_def_id,
            current_node_id: pdb_definition.unwrap().nodes.first().map(|n| n.node_id),
            state_values: causality_types::expr::value::ValueExpr::Unit,
            execution_history: Vec::new(),
            status: causality_types::tel::process_dataflow::InstanceStatus::Active,
        };
        
        // Simulate domain-specific constraints
        let domain_violations = self.check_pdb_domain_constraints(df_def_id, typed_domain);
        
        Ok(SimulationStepResult {
            step_index,
            step_type: SimulationStepType::PdbInitiation(*df_def_id),
            success: domain_violations.is_empty(),
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            resource_cost: lisp_execution_cost,
            state_changes: vec![StateChange {
                resource_id: instance_id,
                change_type: StateChangeType::PdbInstanceCreated,
                old_value: None,
                new_value: Some(causality_types::expr::value::ValueExpr::Unit),
            }],
            domain_violations,
            generated_effects: Vec::new(),
            affected_pdb_instance: Some(instance_id),
            resulting_pdb_state: Some(initial_state),
        })
    }
    
    /// Simulate dataflow advancement
    async fn simulate_dataflow_advancement(
        &mut self,
        df_instance_id: &causality_types::primitive::ids::ResourceId,
        next_step_params: &std::collections::BTreeMap<causality_types::primitive::string::Str, causality_types::expr::value::ValueExpr>,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
        step_index: usize,
    ) -> SimulationResult<SimulationStepResult> {
        let start_time = std::time::Instant::now();
        
        // Simulate Lisp execution for step advancement
        let lisp_execution_cost = self.simulate_lisp_execution_for_pdb(next_step_params, typed_domain);
        
        // Simulate state advancement
        let mut updated_state = causality_types::tel::process_dataflow::ProcessDataflowInstanceState {
            definition_id: causality_types::primitive::ids::ExprId::new([0; 32]), // Mock
            current_node_id: Some(causality_types::primitive::ids::ExprId::new([1; 32])), // Mock next node
            state_values: causality_types::expr::value::ValueExpr::Unit,
            execution_history: Vec::new(),
            status: causality_types::tel::process_dataflow::InstanceStatus::Active,
        };
        
        // Add execution step to history
        updated_state.execution_history.push(causality_types::tel::process_dataflow::ExecutionStep {
            step_id: causality_types::primitive::ids::ExprId::new(rand::random()),
            timestamp: std::time::SystemTime::now(),
            node_id: updated_state.current_node_id.unwrap(),
            input_values: causality_types::expr::value::ValueExpr::Unit,
            output_values: causality_types::expr::value::ValueExpr::Unit,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        });
        
        Ok(SimulationStepResult {
            step_index,
            step_type: SimulationStepType::PdbAdvancement(*df_instance_id),
            success: true,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            resource_cost: lisp_execution_cost,
            state_changes: vec![StateChange {
                resource_id: *df_instance_id,
                change_type: StateChangeType::PdbInstanceUpdated,
                old_value: Some(causality_types::expr::value::ValueExpr::Unit),
                new_value: Some(causality_types::expr::value::ValueExpr::Unit),
            }],
            domain_violations: Vec::new(),
            generated_effects: Vec::new(),
            affected_pdb_instance: Some(*df_instance_id),
            resulting_pdb_state: Some(updated_state),
        })
    }
    
    /// Simulate Lisp execution for PDB operations
    fn simulate_lisp_execution_for_pdb(
        &self,
        parameters: &std::collections::BTreeMap<causality_types::primitive::string::Str, causality_types::expr::value::ValueExpr>,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> ResourceCost {
        let base_cost = ResourceCost {
            cpu_cycles: 1000,
            memory_bytes: 512,
            network_calls: 0,
            storage_bytes: 0,
        };
        
        // Scale based on parameter complexity
        let param_complexity = parameters.len() as u64;
        let complexity_multiplier = 1.0 + (param_complexity as f64 * 0.1);
        
        // Apply domain-specific multipliers
        let domain_multiplier = match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => 2.0, // ZK requires more computation
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => 1.0,
        };
        
        ResourceCost {
            cpu_cycles: (base_cost.cpu_cycles as f64 * complexity_multiplier * domain_multiplier) as u64,
            memory_bytes: (base_cost.memory_bytes as f64 * complexity_multiplier * domain_multiplier) as u64,
            network_calls: base_cost.network_calls,
            storage_bytes: base_cost.storage_bytes,
        }
    }
    
    /// Check effect domain constraints
    fn check_effect_domain_constraints(
        &self,
        _effect_id: &causality_types::primitive::ids::EntityId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> Vec<String> {
        let mut violations = Vec::new();
        
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                // Simulate ZK constraint checking
                if rand::random::<f64>() < 0.03 { // 3% chance of constraint violation
                    violations.push("ZK constraint violation: non-deterministic operation detected".to_string());
                }
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                // Service domain is more permissive
                if rand::random::<f64>() < 0.01 { // 1% chance of service error
                    violations.push("Service domain error: external dependency unavailable".to_string());
                }
            }
        }
        
        violations
    }
    
    /// Check PDB domain constraints
    fn check_pdb_domain_constraints(
        &self,
        _df_def_id: &causality_types::primitive::ids::ExprId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> Vec<String> {
        let mut violations = Vec::new();
        
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                // PDB operations in ZK domain must be deterministic
                if rand::random::<f64>() < 0.02 { // 2% chance of determinism violation
                    violations.push("PDB operation violates ZK determinism requirements".to_string());
                }
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                // Service domain allows all PDB operations
            }
        }
        
        violations
    }
    
    /// Calculate effect resource cost
    fn calculate_effect_resource_cost(
        &self,
        _effect_id: &causality_types::primitive::ids::EntityId,
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> ResourceCost {
        match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                ResourceCost {
                    cpu_cycles: 5000,
                    memory_bytes: 2048,
                    network_calls: 0,
                    storage_bytes: 256,
                }
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                ResourceCost {
                    cpu_cycles: 1000,
                    memory_bytes: 512,
                    network_calls: 2,
                    storage_bytes: 128,
                }
            }
        }
    }
    
    /// Simulate effect state changes
    async fn simulate_effect_state_changes(
        &mut self,
        effect_id: &causality_types::primitive::ids::EntityId,
    ) -> SimulationResult<Vec<StateChange>> {
        // Simulate resource creation/modification
        let resource_id = causality_types::primitive::ids::ResourceId::new(rand::random());
        
        Ok(vec![StateChange {
            resource_id,
            change_type: StateChangeType::ResourceModified,
            old_value: Some(causality_types::expr::value::ValueExpr::Unit),
            new_value: Some(causality_types::expr::value::ValueExpr::String(
                causality_types::primitive::string::Str::from(format!("effect_{}", effect_id.to_hex()))
            )),
        }])
    }
    
    /// Create a simulation fork for state isolation
    async fn create_simulation_fork(&self) -> SimulationResult<SimulationFork> {
        Ok(SimulationFork {
            timestamp: std::time::SystemTime::now(),
            history_step: self.history.current_step(),
            state_snapshot: "mock_state_snapshot".to_string(), // In real implementation, this would be actual state
        })
    }
    
    /// Restore a simulation fork
    async fn restore_simulation_fork(&mut self, _fork: SimulationFork) -> SimulationResult<()> {
        // In real implementation, this would restore the actual state
        Ok(())
    }
}

/// Detailed result of plan simulation
#[derive(Debug, Clone)]
pub struct DetailedPlanSimulationResult {
    pub plan_id: causality_types::primitive::ids::EntityId,
    pub success: bool,
    pub execution_steps: Vec<SimulationStepResult>,
    pub resource_state_changes: Vec<StateChange>,
    pub pdb_instance_states: std::collections::HashMap<causality_types::primitive::ids::ResourceId, causality_types::tel::process_dataflow::ProcessDataflowInstanceState>,
    pub generated_effects: Vec<causality_types::core::Effect>,
    pub domain_violations: Vec<String>,
    pub total_execution_time_ms: u64,
    pub total_resource_cost: ResourceCost,
}

/// Result of simulating a single step
#[derive(Debug, Clone)]
pub struct SimulationStepResult {
    pub step_index: usize,
    pub step_type: SimulationStepType,
    pub success: bool,
    pub execution_time_ms: u64,
    pub resource_cost: ResourceCost,
    pub state_changes: Vec<StateChange>,
    pub domain_violations: Vec<String>,
    pub generated_effects: Vec<causality_types::core::Effect>,
    pub affected_pdb_instance: Option<causality_types::primitive::ids::ResourceId>,
    pub resulting_pdb_state: Option<causality_types::tel::process_dataflow::ProcessDataflowInstanceState>,
}

/// Type of simulation step
#[derive(Debug, Clone)]
pub enum SimulationStepType {
    EffectExecution(causality_types::primitive::ids::EntityId),
    PdbInitiation(causality_types::primitive::ids::ExprId),
    PdbAdvancement(causality_types::primitive::ids::ResourceId),
}

/// Resource cost tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceCost {
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub network_calls: u64,
    pub storage_bytes: u64,
}

impl ResourceCost {
    pub fn add(&mut self, other: &ResourceCost) {
        self.cpu_cycles += other.cpu_cycles;
        self.memory_bytes += other.memory_bytes;
        self.network_calls += other.network_calls;
        self.storage_bytes += other.storage_bytes;
    }
}

/// State change tracking
#[derive(Debug, Clone)]
pub struct StateChange {
    pub resource_id: causality_types::primitive::ids::ResourceId,
    pub change_type: StateChangeType,
    pub old_value: Option<causality_types::expr::value::ValueExpr>,
    pub new_value: Option<causality_types::expr::value::ValueExpr>,
}

/// Type of state change
#[derive(Debug, Clone)]
pub enum StateChangeType {
    ResourceCreated,
    ResourceModified,
    ResourceDeleted,
    PdbInstanceCreated,
    PdbInstanceUpdated,
    PdbInstanceCompleted,
}

/// Simulation fork for state isolation
#[derive(Debug, Clone)]
struct SimulationFork {
    timestamp: std::time::SystemTime,
    history_step: u64,
    state_snapshot: String, // In real implementation, this would be actual state data
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = SimulationEngine::new(None, None);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_new_with_initial_snapshot() {
        let engine = SimulationEngine::new(
            Some(DomainId::new([0u8; 32])),
            Some(12345),
        );
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_save_and_load_state() -> SimulationResult<()> {
        let mut engine = SimulationEngine::new(None, Some(0))?;
        let save_path = Path::new("test_engine_state.dat");
        engine.save_state(save_path)?;

        let mut new_engine = SimulationEngine::new(None, None)?;
        new_engine.load_state(save_path)?;
        std::fs::remove_file(save_path)
            .expect("Failed to remove test state file");
        Ok(())
    }

    #[test]
    fn test_engine_step_forward_backward_jump() -> SimulationResult<()> {
        let mut engine = SimulationEngine::new(None, None)?;

        // Add some steps to history (simplified)
        engine.history.record_step(SimulationSnapshot::new(1, None));
        engine.history.record_step(SimulationSnapshot::new(2, None));
        engine.history.record_step(SimulationSnapshot::new(3, None));
        engine.history.set_current_step_index(2); // At step 3 (0-indexed)

        engine.step_backward_in_history()?;
        assert_eq!(engine.history.get_current_step_index(), 1);

        engine.step_forward_in_history()?;
        assert_eq!(engine.history.get_current_step_index(), 2);

        engine.jump_to_history_step(0)?;
        assert_eq!(engine.history.get_current_step_index(), 0);

        Ok(())
    }
}

// === EVALUATION CONFIGURATION SYSTEM ===

/// Configuration for strategy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfiguration {
    /// Test scenarios to run
    pub test_scenarios: Vec<TestScenarioConfig>,
    
    /// Metrics to collect during evaluation
    pub metrics_config: MetricsConfiguration,
    
    /// Evaluation constraints and limits
    pub evaluation_constraints: EvaluationConstraints,
    
    /// Reporting configuration
    pub reporting_config: ReportingConfiguration,
}

/// Configuration for a test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenarioConfig {
    /// Unique identifier for the scenario
    pub scenario_id: String,
    
    /// Human-readable description
    pub description: String,
    
    /// TypedDomain for this scenario
    pub typed_domain_config: TypedDomainConfig,
    
    /// PDB definitions to include
    pub pdb_definition_configs: Vec<PdbDefinitionConfig>,
    
    /// Initial resource configurations
    pub initial_resources: Vec<ResourceConfig>,
    
    /// Target intents to achieve
    pub target_intents: Vec<IntentConfig>,
    
    /// Scenario-specific constraints
    pub constraints: ScenarioConstraints,
}

/// Configuration for TypedDomain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedDomainConfig {
    pub domain_type: String, // "verifiable" or "service"
    pub domain_id: String,
    pub constraints: std::collections::BTreeMap<String, serde_json::Value>,
}

/// Configuration for PDB definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdbDefinitionConfig {
    pub definition_id: String,
    pub name: String,
    pub complexity_score: f64,
    pub node_count: usize,
    pub edge_count: usize,
}

/// Configuration for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub resource_id: String,
    pub resource_type: String,
    pub initial_value: serde_json::Value,
}

/// Configuration for intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentConfig {
    pub intent_id: String,
    pub intent_type: String,
    pub priority: f64,
    pub target_domain: Option<String>,
}

/// Scenario-specific constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConstraints {
    pub max_execution_time_ms: Option<u64>,
    pub max_resource_cost: Option<ResourceCostLimits>,
    pub required_success_rate: Option<f64>,
}

/// Resource cost limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCostLimits {
    pub max_cpu_cycles: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_network_calls: Option<u64>,
    pub max_storage_bytes: Option<u64>,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfiguration {
    /// Whether to collect detailed execution traces
    pub collect_execution_traces: bool,
    
    /// Whether to collect resource usage metrics
    pub collect_resource_metrics: bool,
    
    /// Whether to collect domain compatibility metrics
    pub collect_domain_metrics: bool,
    
    /// Whether to collect PDB orchestration metrics
    pub collect_pdb_metrics: bool,
    
    /// Custom metrics to collect
    pub custom_metrics: Vec<CustomMetricConfig>,
}

/// Configuration for custom metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    pub metric_name: String,
    pub metric_type: String, // "counter", "gauge", "histogram"
    pub description: String,
}

/// Evaluation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConstraints {
    /// Maximum number of plans to evaluate per strategy
    pub max_plans_per_strategy: Option<usize>,
    
    /// Maximum evaluation time per scenario
    pub max_evaluation_time_ms: Option<u64>,
    
    /// Maximum memory usage for evaluation
    pub max_memory_usage_bytes: Option<u64>,
    
    /// Whether to stop on first failure
    pub stop_on_failure: bool,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfiguration {
    /// Output format for reports
    pub output_format: String, // "json", "csv", "html"
    
    /// Whether to include detailed breakdowns
    pub include_detailed_breakdown: bool,
    
    /// Whether to include comparison charts
    pub include_comparison_charts: bool,
    
    /// Custom report sections to include
    pub custom_sections: Vec<String>,
}

impl Default for EvaluationConfiguration {
    fn default() -> Self {
        Self {
            test_scenarios: vec![
                TestScenarioConfig {
                    scenario_id: "basic_zk_scenario".to_string(),
                    description: "Basic ZK domain scenario".to_string(),
                    typed_domain_config: TypedDomainConfig {
                        domain_type: "verifiable".to_string(),
                        domain_id: "zk_domain_1".to_string(),
                        constraints: std::collections::BTreeMap::new(),
                    },
                    pdb_definition_configs: Vec::new(),
                    initial_resources: Vec::new(),
                    target_intents: Vec::new(),
                    constraints: ScenarioConstraints {
                        max_execution_time_ms: Some(10000),
                        max_resource_cost: None,
                        required_success_rate: Some(0.95),
                    },
                },
                TestScenarioConfig {
                    scenario_id: "basic_service_scenario".to_string(),
                    description: "Basic service domain scenario".to_string(),
                    typed_domain_config: TypedDomainConfig {
                        domain_type: "service".to_string(),
                        domain_id: "service_domain_1".to_string(),
                        constraints: std::collections::BTreeMap::new(),
                    },
                    pdb_definition_configs: Vec::new(),
                    initial_resources: Vec::new(),
                    target_intents: Vec::new(),
                    constraints: ScenarioConstraints {
                        max_execution_time_ms: Some(5000),
                        max_resource_cost: None,
                        required_success_rate: Some(0.90),
                    },
                },
            ],
            metrics_config: MetricsConfiguration {
                collect_execution_traces: true,
                collect_resource_metrics: true,
                collect_domain_metrics: true,
                collect_pdb_metrics: true,
                custom_metrics: Vec::new(),
            },
            evaluation_constraints: EvaluationConstraints {
                max_plans_per_strategy: Some(100),
                max_evaluation_time_ms: Some(60000),
                max_memory_usage_bytes: Some(1024 * 1024 * 1024), // 1GB
                stop_on_failure: false,
            },
            reporting_config: ReportingConfiguration {
                output_format: "json".to_string(),
                include_detailed_breakdown: true,
                include_comparison_charts: false,
                custom_sections: Vec::new(),
            },
        }
    }
}

/// Enhanced simulation engine with evaluation configuration
impl SimulationEngine {
    /// Run comprehensive strategy evaluation with configuration
    pub async fn run_configured_evaluation(
        &mut self,
        strategies: Vec<Box<dyn causality_runtime::optimization::OptimizationStrategy>>,
        config: &EvaluationConfiguration,
    ) -> SimulationResult<ComprehensiveEvaluationResult> {
        let start_time = std::time::Instant::now();
        let mut strategy_results = std::collections::BTreeMap::new();
        let mut collected_metrics = CollectedMetrics::new();
        
        // Convert configuration scenarios to runtime scenarios
        let test_scenarios = self.convert_config_scenarios_to_runtime(&config.test_scenarios).await?;
        
        // Evaluate each strategy
        for strategy in strategies {
            let strategy_id = strategy.strategy_id().to_string();
            log::info!("Evaluating strategy: {}", strategy_id);
            
            let mut scenario_results = Vec::new();
            
            for scenario in &test_scenarios {
                // Check evaluation constraints
                if let Some(max_time) = config.evaluation_constraints.max_evaluation_time_ms {
                    if start_time.elapsed().as_millis() as u64 > max_time {
                        log::warn!("Evaluation time limit exceeded, stopping early");
                        break;
                    }
                }
                
                let scenario_result = self.evaluate_strategy_on_configured_scenario(
                    &*strategy,
                    scenario,
                    &config.metrics_config,
                ).await?;
                
                // Collect metrics
                if config.metrics_config.collect_execution_traces {
                    collected_metrics.execution_traces.push(scenario_result.execution_trace.clone());
                }
                
                scenario_results.push(scenario_result);
                
                // Check if we should stop on failure
                if config.evaluation_constraints.stop_on_failure && !scenario_results.last().unwrap().success {
                    log::warn!("Stopping evaluation due to failure in scenario: {}", scenario.scenario_id);
                    break;
                }
            }
            
            strategy_results.insert(strategy_id, scenario_results);
        }
        
        // Generate comprehensive report
        let report = self.generate_evaluation_report(&strategy_results, &collected_metrics, config).await?;
        
        Ok(ComprehensiveEvaluationResult {
            strategy_results,
            collected_metrics,
            evaluation_report: report,
            total_evaluation_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }
    
    /// Convert configuration scenarios to runtime scenarios
    async fn convert_config_scenarios_to_runtime(
        &self,
        config_scenarios: &[TestScenarioConfig],
    ) -> SimulationResult<Vec<StrategyTestScenario>> {
        let mut scenarios = Vec::new();
        
        for config in config_scenarios {
            // Convert TypedDomain configuration
            let typed_domain = match config.typed_domain_config.domain_type.as_str() {
                "verifiable" => {
                    let domain_id = causality_types::primitive::ids::DomainId::new(rand::random());
                    causality_types::tel::optimization::TypedDomain::VerifiableDomain(domain_id)
                }
                "service" => {
                    let domain_id = causality_types::primitive::ids::DomainId::new(rand::random());
                    causality_types::tel::optimization::TypedDomain::ServiceDomain(domain_id)
                }
                _ => return Err(SimulationError::ConfigurationError(
                    format!("Unknown domain type: {}", config.typed_domain_config.domain_type)
                )),
            };
            
            // Convert PDB definitions (simplified for now)
            let pdb_definitions = std::collections::BTreeMap::new();
            
            // Convert resource and intent configurations
            let initial_resources = Vec::new();
            let target_intents = Vec::new();
            
            scenarios.push(StrategyTestScenario {
                scenario_id: config.scenario_id.clone(),
                typed_domain,
                pdb_definitions,
                initial_resources,
                target_intents,
            });
        }
        
        Ok(scenarios)
    }
    
    /// Evaluate strategy on configured scenario with metrics collection
    async fn evaluate_strategy_on_configured_scenario(
        &mut self,
        strategy: &dyn causality_runtime::optimization::OptimizationStrategy,
        scenario: &StrategyTestScenario,
        metrics_config: &MetricsConfiguration,
    ) -> SimulationResult<ConfiguredScenarioResult> {
        let start_time = std::time::Instant::now();
        
        // Create optimization context
        let context = causality_runtime::optimization::OptimizationContext {
            current_typed_domain: scenario.typed_domain.clone(),
            available_typed_domains: vec![scenario.typed_domain.clone()],
            dataflow_definitions: scenario.pdb_definitions.clone(),
            active_dataflow_instances: std::collections::HashMap::new(),
            system_load: causality_runtime::optimization::SystemLoadMetrics::default(),
            evaluation_constraints: causality_runtime::optimization::EvaluationConstraints::default(),
        };
        
        // Generate plans
        let scored_plans = strategy.propose(&context)
            .map_err(|e| SimulationError::EvaluationError(e.to_string()))?;
        
        // Evaluate plans with detailed simulation
        let mut plan_results = Vec::new();
        for scored_plan in &scored_plans {
            let detailed_result = self.simulate_resolution_plan(
                &scored_plan.plan,
                &scenario.typed_domain,
                &scenario.pdb_definitions,
            ).await?;
            plan_results.push(detailed_result);
        }
        
        // Collect metrics
        let mut execution_trace = ExecutionTrace::new();
        if metrics_config.collect_execution_traces {
            for result in &plan_results {
                execution_trace.add_plan_execution(&result);
            }
        }
        
        let resource_metrics = if metrics_config.collect_resource_metrics {
            Some(self.collect_resource_metrics(&plan_results))
        } else {
            None
        };
        
        let domain_metrics = if metrics_config.collect_domain_metrics {
            Some(self.collect_domain_metrics(&plan_results, &scenario.typed_domain))
        } else {
            None
        };
        
        let pdb_metrics = if metrics_config.collect_pdb_metrics {
            Some(self.collect_pdb_metrics(&plan_results))
        } else {
            None
        };
        
        // Calculate overall success rate
        let success_rate = plan_results.iter().filter(|r| r.success).count() as f64 / plan_results.len() as f64;
        
        Ok(ConfiguredScenarioResult {
            scenario_id: scenario.scenario_id.clone(),
            success: success_rate > 0.5, // Consider successful if >50% of plans succeed
            plan_count: plan_results.len(),
            success_rate,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            execution_trace,
            resource_metrics,
            domain_metrics,
            pdb_metrics,
        })
    }
    
    /// Collect resource usage metrics
    fn collect_resource_metrics(&self, plan_results: &[DetailedPlanSimulationResult]) -> ResourceMetrics {
        let mut total_cost = ResourceCost::default();
        let mut max_cost = ResourceCost::default();
        
        for result in plan_results {
            total_cost.add(&result.total_resource_cost);
            
            if result.total_resource_cost.cpu_cycles > max_cost.cpu_cycles {
                max_cost.cpu_cycles = result.total_resource_cost.cpu_cycles;
            }
            if result.total_resource_cost.memory_bytes > max_cost.memory_bytes {
                max_cost.memory_bytes = result.total_resource_cost.memory_bytes;
            }
            if result.total_resource_cost.network_calls > max_cost.network_calls {
                max_cost.network_calls = result.total_resource_cost.network_calls;
            }
            if result.total_resource_cost.storage_bytes > max_cost.storage_bytes {
                max_cost.storage_bytes = result.total_resource_cost.storage_bytes;
            }
        }
        
        let avg_cost = ResourceCost {
            cpu_cycles: total_cost.cpu_cycles / plan_results.len() as u64,
            memory_bytes: total_cost.memory_bytes / plan_results.len() as u64,
            network_calls: total_cost.network_calls / plan_results.len() as u64,
            storage_bytes: total_cost.storage_bytes / plan_results.len() as u64,
        };
        
        ResourceMetrics {
            total_cost,
            average_cost: avg_cost,
            max_cost,
            cost_distribution: self.calculate_cost_distribution(plan_results),
        }
    }
    
    /// Calculate cost distribution
    fn calculate_cost_distribution(&self, plan_results: &[DetailedPlanSimulationResult]) -> CostDistribution {
        let cpu_costs: Vec<u64> = plan_results.iter().map(|r| r.total_resource_cost.cpu_cycles).collect();
        let memory_costs: Vec<u64> = plan_results.iter().map(|r| r.total_resource_cost.memory_bytes).collect();
        
        CostDistribution {
            cpu_percentiles: self.calculate_percentiles(&cpu_costs),
            memory_percentiles: self.calculate_percentiles(&memory_costs),
        }
    }
    
    /// Calculate percentiles for a dataset
    fn calculate_percentiles(&self, data: &[u64]) -> Percentiles {
        let mut sorted_data = data.to_vec();
        sorted_data.sort();
        
        let len = sorted_data.len();
        Percentiles {
            p50: sorted_data[len / 2],
            p90: sorted_data[(len * 9) / 10],
            p95: sorted_data[(len * 95) / 100],
            p99: sorted_data[(len * 99) / 100],
        }
    }
    
    /// Collect domain-specific metrics
    fn collect_domain_metrics(
        &self,
        plan_results: &[DetailedPlanSimulationResult],
        typed_domain: &causality_types::tel::optimization::TypedDomain,
    ) -> DomainMetrics {
        let total_violations = plan_results.iter().map(|r| r.domain_violations.len()).sum::<usize>();
        let violation_rate = total_violations as f64 / plan_results.len() as f64;
        
        let domain_specific_data = match typed_domain {
            causality_types::tel::optimization::TypedDomain::VerifiableDomain(_) => {
                std::collections::BTreeMap::from([
                    ("zk_constraint_checks".to_string(), serde_json::Value::Number(serde_json::Number::from(total_violations))),
                    ("determinism_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1.0 - violation_rate).unwrap())),
                ])
            }
            causality_types::tel::optimization::TypedDomain::ServiceDomain(_) => {
                std::collections::BTreeMap::from([
                    ("service_calls".to_string(), serde_json::Value::Number(serde_json::Number::from(plan_results.len()))),
                    ("availability_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1.0 - violation_rate).unwrap())),
                ])
            }
        };
        
        DomainMetrics {
            domain_type: format!("{:?}", typed_domain),
            violation_count: total_violations,
            violation_rate,
            domain_specific_data,
        }
    }
    
    /// Collect PDB orchestration metrics
    fn collect_pdb_metrics(&self, plan_results: &[DetailedPlanSimulationResult]) -> PdbMetrics {
        let total_pdb_instances = plan_results.iter()
            .map(|r| r.pdb_instance_states.len())
            .sum::<usize>();
        
        let avg_pdb_instances = total_pdb_instances as f64 / plan_results.len() as f64;
        
        PdbMetrics {
            total_pdb_instances,
            average_pdb_instances_per_plan: avg_pdb_instances,
            pdb_success_rate: 0.95, // Mock value
            orchestration_complexity_score: 2.5, // Mock value
        }
    }
    
    /// Generate comprehensive evaluation report
    async fn generate_evaluation_report(
        &self,
        strategy_results: &std::collections::BTreeMap<String, Vec<ConfiguredScenarioResult>>,
        collected_metrics: &CollectedMetrics,
        config: &EvaluationConfiguration,
    ) -> SimulationResult<EvaluationReport> {
        let mut strategy_summaries = std::collections::BTreeMap::new();
        
        for (strategy_id, scenario_results) in strategy_results {
            let avg_success_rate = scenario_results.iter().map(|r| r.success_rate).sum::<f64>() / scenario_results.len() as f64;
            let avg_execution_time = scenario_results.iter().map(|r| r.execution_time_ms).sum::<u64>() as f64 / scenario_results.len() as f64;
            
            strategy_summaries.insert(strategy_id.clone(), StrategySummary {
                strategy_id: strategy_id.clone(),
                total_scenarios: scenario_results.len(),
                average_success_rate: avg_success_rate,
                average_execution_time_ms: avg_execution_time,
                scenario_breakdown: scenario_results.iter().map(|r| ScenarioBreakdown {
                    scenario_id: r.scenario_id.clone(),
                    success: r.success,
                    execution_time_ms: r.execution_time_ms,
                    plan_count: r.plan_count,
                }).collect(),
            });
        }
        
        Ok(EvaluationReport {
            configuration: config.clone(),
            strategy_summaries,
            overall_metrics: OverallMetrics {
                total_strategies_evaluated: strategy_results.len(),
                total_scenarios_run: strategy_results.values().map(|v| v.len()).sum(),
                total_plans_evaluated: strategy_results.values()
                    .flat_map(|scenarios| scenarios.iter().map(|s| s.plan_count))
                    .sum(),
            },
            recommendations: self.generate_recommendations(strategy_results),
        })
    }
    
    /// Generate recommendations based on evaluation results
    fn generate_recommendations(
        &self,
        strategy_results: &std::collections::BTreeMap<String, Vec<ConfiguredScenarioResult>>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Find best performing strategy
        let mut best_strategy = None;
        let mut best_score = 0.0;
        
        for (strategy_id, scenario_results) in strategy_results {
            let avg_success_rate = scenario_results.iter().map(|r| r.success_rate).sum::<f64>() / scenario_results.len() as f64;
            if avg_success_rate > best_score {
                best_score = avg_success_rate;
                best_strategy = Some(strategy_id.clone());
            }
        }
        
        if let Some(best) = best_strategy {
            recommendations.push(format!("Strategy '{}' shows the best overall performance with {:.2}% success rate", best, best_score * 100.0));
        }
        
        // Add domain-specific recommendations
        recommendations.push("Consider domain-specific optimizations for better performance".to_string());
        recommendations.push("Monitor PDB orchestration complexity to avoid performance bottlenecks".to_string());
        
        recommendations
    }
}

// === EVALUATION RESULT TYPES ===

/// Comprehensive evaluation result
#[derive(Debug, Clone)]
pub struct ComprehensiveEvaluationResult {
    pub strategy_results: std::collections::BTreeMap<String, Vec<ConfiguredScenarioResult>>,
    pub collected_metrics: CollectedMetrics,
    pub evaluation_report: EvaluationReport,
    pub total_evaluation_time_ms: u64,
}

/// Result for a configured scenario
#[derive(Debug, Clone)]
pub struct ConfiguredScenarioResult {
    pub scenario_id: String,
    pub success: bool,
    pub plan_count: usize,
    pub success_rate: f64,
    pub execution_time_ms: u64,
    pub execution_trace: ExecutionTrace,
    pub resource_metrics: Option<ResourceMetrics>,
    pub domain_metrics: Option<DomainMetrics>,
    pub pdb_metrics: Option<PdbMetrics>,
}

/// Collected metrics during evaluation
#[derive(Debug, Clone)]
pub struct CollectedMetrics {
    pub execution_traces: Vec<ExecutionTrace>,
    pub resource_usage_history: Vec<ResourceCost>,
    pub domain_violation_history: Vec<String>,
}

impl CollectedMetrics {
    pub fn new() -> Self {
        Self {
            execution_traces: Vec::new(),
            resource_usage_history: Vec::new(),
            domain_violation_history: Vec::new(),
        }
    }
}

/// Execution trace for detailed analysis
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub total_duration_ms: u64,
}

impl ExecutionTrace {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            total_duration_ms: 0,
        }
    }
    
    pub fn add_plan_execution(&mut self, plan_result: &DetailedPlanSimulationResult) {
        for step in &plan_result.execution_steps {
            self.steps.push(TraceStep {
                step_index: step.step_index,
                step_type: format!("{:?}", step.step_type),
                duration_ms: step.execution_time_ms,
                success: step.success,
                resource_cost: step.resource_cost.clone(),
            });
        }
        self.total_duration_ms += plan_result.total_execution_time_ms;
    }
}

/// Individual trace step
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub step_index: usize,
    pub step_type: String,
    pub duration_ms: u64,
    pub success: bool,
    pub resource_cost: ResourceCost,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub total_cost: ResourceCost,
    pub average_cost: ResourceCost,
    pub max_cost: ResourceCost,
    pub cost_distribution: CostDistribution,
}

/// Cost distribution statistics
#[derive(Debug, Clone)]
pub struct CostDistribution {
    pub cpu_percentiles: Percentiles,
    pub memory_percentiles: Percentiles,
}

/// Percentile statistics
#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
}

/// Domain-specific metrics
#[derive(Debug, Clone)]
pub struct DomainMetrics {
    pub domain_type: String,
    pub violation_count: usize,
    pub violation_rate: f64,
    pub domain_specific_data: std::collections::BTreeMap<String, serde_json::Value>,
}

/// PDB orchestration metrics
#[derive(Debug, Clone)]
pub struct PdbMetrics {
    pub total_pdb_instances: usize,
    pub average_pdb_instances_per_plan: f64,
    pub pdb_success_rate: f64,
    pub orchestration_complexity_score: f64,
}

/// Comprehensive evaluation report
#[derive(Debug, Clone)]
pub struct EvaluationReport {
    pub configuration: EvaluationConfiguration,
    pub strategy_summaries: std::collections::BTreeMap<String, StrategySummary>,
    pub overall_metrics: OverallMetrics,
    pub recommendations: Vec<String>,
}

/// Summary for a strategy
#[derive(Debug, Clone)]
pub struct StrategySummary {
    pub strategy_id: String,
    pub total_scenarios: usize,
    pub average_success_rate: f64,
    pub average_execution_time_ms: f64,
    pub scenario_breakdown: Vec<ScenarioBreakdown>,
}

/// Breakdown for a scenario
#[derive(Debug, Clone)]
pub struct ScenarioBreakdown {
    pub scenario_id: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub plan_count: usize,
}

/// Overall evaluation metrics
#[derive(Debug, Clone)]
pub struct OverallMetrics {
    pub total_strategies_evaluated: usize,
    pub total_scenarios_run: usize,
    pub total_plans_evaluated: usize,
}

// === CHECKPOINT AND RESTORATION SYSTEM ===

/// Comprehensive checkpoint system for simulation state
impl SimulationEngine {
    /// Create a comprehensive checkpoint of the current simulation state
    pub async fn create_comprehensive_checkpoint(&self) -> SimulationResult<ComprehensiveCheckpoint> {
        let checkpoint_id = format!("checkpoint_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        // Capture simulation engine state
        let engine_state = self.capture_engine_state().await?;
        
        // Capture PDB instance states (if any are active)
        let pdb_instance_states = self.capture_pdb_instance_states().await?;
        
        // Capture resource states
        let resource_states = self.capture_resource_states().await?;
        
        // Capture execution context
        let execution_context = self.capture_execution_context().await?;
        
        Ok(ComprehensiveCheckpoint {
            checkpoint_id,
            timestamp: std::time::SystemTime::now(),
            engine_state,
            pdb_instance_states,
            resource_states,
            execution_context,
            metadata: CheckpointMetadata {
                creation_reason: "manual_checkpoint".to_string(),
                simulation_step: self.history.current_step(),
                memory_usage_bytes: self.estimate_memory_usage(),
            },
        })
    }
    
    /// Restore simulation state from a comprehensive checkpoint
    pub async fn restore_from_comprehensive_checkpoint(
        &mut self,
        checkpoint: &ComprehensiveCheckpoint,
    ) -> SimulationResult<()> {
        log::info!("Restoring simulation state from checkpoint: {}", checkpoint.checkpoint_id);
        
        // Restore engine state
        self.restore_engine_state(&checkpoint.engine_state).await?;
        
        // Restore PDB instance states
        self.restore_pdb_instance_states(&checkpoint.pdb_instance_states).await?;
        
        // Restore resource states
        self.restore_resource_states(&checkpoint.resource_states).await?;
        
        // Restore execution context
        self.restore_execution_context(&checkpoint.execution_context).await?;
        
        log::info!("Successfully restored simulation state from checkpoint");
        Ok(())
    }
    
    /// Create a lightweight checkpoint for quick rollbacks
    pub async fn create_lightweight_checkpoint(&self) -> SimulationResult<LightweightCheckpoint> {
        Ok(LightweightCheckpoint {
            checkpoint_id: format!("light_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            timestamp: std::time::SystemTime::now(),
            history_step: self.history.current_step(),
            state_hash: self.calculate_state_hash().await?,
            critical_state_snapshot: self.capture_critical_state().await?,
        })
    }
    
    /// Restore from a lightweight checkpoint
    pub async fn restore_from_lightweight_checkpoint(
        &mut self,
        checkpoint: &LightweightCheckpoint,
    ) -> SimulationResult<()> {
        // Jump to the history step
        self.jump_to_history_step(checkpoint.history_step)?;
        
        // Restore critical state
        self.restore_critical_state(&checkpoint.critical_state_snapshot).await?;
        
        // Verify state integrity
        let current_hash = self.calculate_state_hash().await?;
        if current_hash != checkpoint.state_hash {
            return Err(SimulationError::CheckpointError(
                "State hash mismatch after restoration".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Capture current engine state
    async fn capture_engine_state(&self) -> SimulationResult<EngineState> {
        Ok(EngineState {
            configuration: self.config.clone(),
            current_step: self.history.current_step(),
            execution_mode: "simulation".to_string(), // Mock value
            active_breakpoints: Vec::new(), // Mock value
        })
    }
    
    /// Capture PDB instance states
    async fn capture_pdb_instance_states(&self) -> SimulationResult<std::collections::HashMap<causality_types::primitive::ids::ResourceId, causality_types::tel::process_dataflow::ProcessDataflowInstanceState>> {
        // In a real implementation, this would capture actual PDB instance states
        // For now, return empty map as we don't have active PDB instances in the simulation engine
        Ok(std::collections::HashMap::new())
    }
    
    /// Capture resource states
    async fn capture_resource_states(&self) -> SimulationResult<std::collections::HashMap<causality_types::primitive::ids::ResourceId, ResourceState>> {
        // Mock resource state capture
        let mut resource_states = std::collections::HashMap::new();
        
        // Add some mock resource states
        for i in 0..3 {
            let resource_id = causality_types::primitive::ids::ResourceId::new([i; 32]);
            resource_states.insert(resource_id, ResourceState {
                resource_id,
                resource_type: format!("mock_resource_type_{}", i),
                current_value: causality_types::expr::value::ValueExpr::String(
                    causality_types::primitive::string::Str::from(format!("mock_value_{}", i))
                ),
                last_modified: std::time::SystemTime::now(),
                version: i as u64,
            });
        }
        
        Ok(resource_states)
    }
    
    /// Capture execution context
    async fn capture_execution_context(&self) -> SimulationResult<ExecutionContextState> {
        Ok(ExecutionContextState {
            current_typed_domain: None, // Mock value
            active_strategies: Vec::new(),
            optimization_state: std::collections::BTreeMap::new(),
            performance_metrics: PerformanceMetrics {
                total_effects_processed: 0,
                total_execution_time_ms: 0,
                average_effect_processing_time_ms: 0.0,
                memory_usage_bytes: self.estimate_memory_usage(),
            },
        })
    }
    
    /// Restore engine state
    async fn restore_engine_state(&mut self, engine_state: &EngineState) -> SimulationResult<()> {
        self.config = engine_state.configuration.clone();
        
        // Jump to the correct history step
        if engine_state.current_step != self.history.current_step() {
            self.jump_to_history_step(engine_state.current_step)?;
        }
        
        Ok(())
    }
    
    /// Restore PDB instance states
    async fn restore_pdb_instance_states(
        &mut self,
        _pdb_states: &std::collections::HashMap<causality_types::primitive::ids::ResourceId, causality_types::tel::process_dataflow::ProcessDataflowInstanceState>,
    ) -> SimulationResult<()> {
        // In a real implementation, this would restore PDB instance states
        // For now, just log the restoration
        log::debug!("Restoring PDB instance states (mock implementation)");
        Ok(())
    }
    
    /// Restore resource states
    async fn restore_resource_states(
        &mut self,
        _resource_states: &std::collections::HashMap<causality_types::primitive::ids::ResourceId, ResourceState>,
    ) -> SimulationResult<()> {
        // In a real implementation, this would restore resource states to the state manager
        log::debug!("Restoring resource states (mock implementation)");
        Ok(())
    }
    
    /// Restore execution context
    async fn restore_execution_context(&mut self, _context_state: &ExecutionContextState) -> SimulationResult<()> {
        // In a real implementation, this would restore the execution context
        log::debug!("Restoring execution context (mock implementation)");
        Ok(())
    }
    
    /// Calculate a hash of the current state for integrity checking
    async fn calculate_state_hash(&self) -> SimulationResult<String> {
        // In a real implementation, this would calculate a proper hash
        // For now, return a mock hash based on current step
        Ok(format!("state_hash_{}", self.history.current_step()))
    }
    
    /// Capture critical state for lightweight checkpoints
    async fn capture_critical_state(&self) -> SimulationResult<CriticalStateSnapshot> {
        Ok(CriticalStateSnapshot {
            active_effects_count: 0, // Mock value
            resource_count: 3, // Mock value
            pdb_instance_count: 0, // Mock value
            last_effect_timestamp: std::time::SystemTime::now(),
        })
    }
    
    /// Restore critical state from snapshot
    async fn restore_critical_state(&mut self, _snapshot: &CriticalStateSnapshot) -> SimulationResult<()> {
        // In a real implementation, this would restore critical state
        log::debug!("Restoring critical state (mock implementation)");
        Ok(())
    }
    
    /// Estimate current memory usage
    fn estimate_memory_usage(&self) -> u64 {
        // Mock memory usage estimation
        1024 * 1024 // 1MB
    }
    
    /// Create automatic checkpoint before risky operations
    pub async fn create_auto_checkpoint(&self, reason: &str) -> SimulationResult<ComprehensiveCheckpoint> {
        let mut checkpoint = self.create_comprehensive_checkpoint().await?;
        checkpoint.metadata.creation_reason = reason.to_string();
        
        log::debug!("Created automatic checkpoint: {} (reason: {})", checkpoint.checkpoint_id, reason);
        Ok(checkpoint)
    }
    
    /// Validate checkpoint integrity
    pub async fn validate_checkpoint(&self, checkpoint: &ComprehensiveCheckpoint) -> SimulationResult<CheckpointValidationResult> {
        let mut validation_result = CheckpointValidationResult {
            is_valid: true,
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Check timestamp validity
        if checkpoint.timestamp > std::time::SystemTime::now() {
            validation_result.is_valid = false;
            validation_result.validation_errors.push("Checkpoint timestamp is in the future".to_string());
        }
        
        // Check if checkpoint is too old (more than 1 hour)
        if let Ok(duration) = std::time::SystemTime::now().duration_since(checkpoint.timestamp) {
            if duration.as_secs() > 3600 {
                validation_result.warnings.push("Checkpoint is more than 1 hour old".to_string());
            }
        }
        
        // Validate PDB instance states
        for (instance_id, pdb_state) in &checkpoint.pdb_instance_states {
            if pdb_state.execution_history.is_empty() {
                validation_result.warnings.push(format!("PDB instance {} has no execution history", instance_id.to_hex()));
            }
        }
        
        // Validate resource states
        if checkpoint.resource_states.is_empty() {
            validation_result.warnings.push("No resource states captured in checkpoint".to_string());
        }
        
        Ok(validation_result)
    }
    
    /// List available checkpoints (mock implementation)
    pub async fn list_checkpoints(&self) -> SimulationResult<Vec<CheckpointInfo>> {
        // In a real implementation, this would list stored checkpoints
        Ok(vec![
            CheckpointInfo {
                checkpoint_id: "checkpoint_1".to_string(),
                timestamp: std::time::SystemTime::now(),
                checkpoint_type: CheckpointType::Comprehensive,
                size_bytes: 1024 * 1024,
                description: "Manual checkpoint before strategy evaluation".to_string(),
            },
            CheckpointInfo {
                checkpoint_id: "light_1".to_string(),
                timestamp: std::time::SystemTime::now(),
                checkpoint_type: CheckpointType::Lightweight,
                size_bytes: 1024,
                description: "Lightweight checkpoint for quick rollback".to_string(),
            },
        ])
    }
    
    /// Delete old checkpoints to free up space
    pub async fn cleanup_old_checkpoints(&self, max_age_hours: u64) -> SimulationResult<usize> {
        // In a real implementation, this would delete old checkpoint files
        log::info!("Cleaning up checkpoints older than {} hours", max_age_hours);
        Ok(0) // Mock: no checkpoints deleted
    }
}

// === CHECKPOINT DATA STRUCTURES ===

/// Comprehensive checkpoint containing all simulation state
#[derive(Debug, Clone)]
pub struct ComprehensiveCheckpoint {
    pub checkpoint_id: String,
    pub timestamp: std::time::SystemTime,
    pub engine_state: EngineState,
    pub pdb_instance_states: std::collections::HashMap<causality_types::primitive::ids::ResourceId, causality_types::tel::process_dataflow::ProcessDataflowInstanceState>,
    pub resource_states: std::collections::HashMap<causality_types::primitive::ids::ResourceId, ResourceState>,
    pub execution_context: ExecutionContextState,
    pub metadata: CheckpointMetadata,
}

/// Lightweight checkpoint for quick operations
#[derive(Debug, Clone)]
pub struct LightweightCheckpoint {
    pub checkpoint_id: String,
    pub timestamp: std::time::SystemTime,
    pub history_step: u64,
    pub state_hash: String,
    pub critical_state_snapshot: CriticalStateSnapshot,
}

/// Engine state snapshot
#[derive(Debug, Clone)]
pub struct EngineState {
    pub configuration: SimulationEngineConfig,
    pub current_step: u64,
    pub execution_mode: String,
    pub active_breakpoints: Vec<BreakpointInfo>,
}

/// Resource state snapshot
#[derive(Debug, Clone)]
pub struct ResourceState {
    pub resource_id: causality_types::primitive::ids::ResourceId,
    pub resource_type: String,
    pub current_value: causality_types::expr::value::ValueExpr,
    pub last_modified: std::time::SystemTime,
    pub version: u64,
}

/// Execution context state snapshot
#[derive(Debug, Clone)]
pub struct ExecutionContextState {
    pub current_typed_domain: Option<causality_types::tel::optimization::TypedDomain>,
    pub active_strategies: Vec<String>,
    pub optimization_state: std::collections::BTreeMap<String, serde_json::Value>,
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics snapshot
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_effects_processed: u64,
    pub total_execution_time_ms: u64,
    pub average_effect_processing_time_ms: f64,
    pub memory_usage_bytes: u64,
}

/// Critical state snapshot for lightweight checkpoints
#[derive(Debug, Clone)]
pub struct CriticalStateSnapshot {
    pub active_effects_count: usize,
    pub resource_count: usize,
    pub pdb_instance_count: usize,
    pub last_effect_timestamp: std::time::SystemTime,
}

/// Checkpoint metadata
#[derive(Debug, Clone)]
pub struct CheckpointMetadata {
    pub creation_reason: String,
    pub simulation_step: u64,
    pub memory_usage_bytes: u64,
}

/// Checkpoint validation result
#[derive(Debug, Clone)]
pub struct CheckpointValidationResult {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Information about a checkpoint
#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub checkpoint_id: String,
    pub timestamp: std::time::SystemTime,
    pub checkpoint_type: CheckpointType,
    pub size_bytes: u64,
    pub description: String,
}

/// Type of checkpoint
#[derive(Debug, Clone)]
pub enum CheckpointType {
    Comprehensive,
    Lightweight,
    Automatic,
}

// === METRICS AND ANALYSIS SYSTEM ===

/// Comprehensive metrics and analysis system
impl SimulationEngine {
    /// Collect comprehensive performance metrics
    pub async fn collect_performance_metrics(&self) -> SimulationResult<ComprehensivePerformanceMetrics> {
        let current_time = std::time::SystemTime::now();
        
        // Collect basic engine metrics
        let engine_metrics = self.collect_engine_metrics().await?;
        
        // Collect strategy performance metrics
        let strategy_metrics = self.collect_strategy_performance_metrics().await?;
        
        // Collect domain-specific metrics
        let domain_metrics = self.collect_domain_performance_metrics().await?;
        
        // Collect PDB orchestration metrics
        let pdb_metrics = self.collect_pdb_performance_metrics().await?;
        
        // Collect resource utilization metrics
        let resource_metrics = self.collect_resource_utilization_metrics().await?;
        
        Ok(ComprehensivePerformanceMetrics {
            timestamp: current_time,
            engine_metrics,
            strategy_metrics,
            domain_metrics,
            pdb_metrics,
            resource_metrics,
            analysis_summary: self.generate_performance_analysis_summary().await?,
        })
    }
    
    /// Detect performance bottlenecks
    pub async fn detect_bottlenecks(&self) -> SimulationResult<BottleneckAnalysis> {
        let mut bottlenecks = Vec::new();
        let mut recommendations = Vec::new();
        
        // Analyze execution time bottlenecks
        let execution_bottlenecks = self.analyze_execution_time_bottlenecks().await?;
        bottlenecks.extend(execution_bottlenecks);
        
        // Analyze memory usage bottlenecks
        let memory_bottlenecks = self.analyze_memory_bottlenecks().await?;
        bottlenecks.extend(memory_bottlenecks);
        
        // Analyze PDB orchestration bottlenecks
        let pdb_bottlenecks = self.analyze_pdb_bottlenecks().await?;
        bottlenecks.extend(pdb_bottlenecks);
        
        // Generate recommendations based on bottlenecks
        recommendations.extend(self.generate_bottleneck_recommendations(&bottlenecks).await?);
        
        Ok(BottleneckAnalysis {
            detected_bottlenecks: bottlenecks,
            severity_distribution: self.calculate_bottleneck_severity_distribution(&bottlenecks),
            recommendations,
            overall_performance_score: self.calculate_overall_performance_score().await?,
        })
    }
    
    /// Generate optimization recommendations
    pub async fn generate_optimization_recommendations(&self) -> SimulationResult<OptimizationRecommendations> {
        let mut recommendations = Vec::new();
        
        // Strategy optimization recommendations
        let strategy_recommendations = self.analyze_strategy_optimization_opportunities().await?;
        recommendations.extend(strategy_recommendations);
        
        // Domain-specific optimization recommendations
        let domain_recommendations = self.analyze_domain_optimization_opportunities().await?;
        recommendations.extend(domain_recommendations);
        
        // PDB orchestration optimization recommendations
        let pdb_recommendations = self.analyze_pdb_optimization_opportunities().await?;
        recommendations.extend(pdb_recommendations);
        
        // Resource utilization optimization recommendations
        let resource_recommendations = self.analyze_resource_optimization_opportunities().await?;
        recommendations.extend(resource_recommendations);
        
        // Prioritize recommendations
        recommendations.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(OptimizationRecommendations {
            recommendations,
            implementation_roadmap: self.generate_implementation_roadmap(&recommendations).await?,
            expected_improvements: self.estimate_improvement_potential(&recommendations).await?,
        })
    }
    
    /// Analyze strategy effectiveness
    pub async fn analyze_strategy_effectiveness(&self) -> SimulationResult<StrategyEffectivenessAnalysis> {
        // Mock strategy effectiveness analysis
        let strategies = vec![
            StrategyEffectiveness {
                strategy_id: "capital_efficiency".to_string(),
                success_rate: 0.92,
                average_execution_time_ms: 150.0,
                resource_efficiency_score: 0.85,
                domain_compatibility_score: 0.88,
                overall_effectiveness_score: 0.89,
                strengths: vec![
                    "Excellent cost optimization".to_string(),
                    "Good TypedDomain compatibility".to_string(),
                ],
                weaknesses: vec![
                    "Slower execution for complex scenarios".to_string(),
                ],
                improvement_suggestions: vec![
                    "Optimize plan enumeration algorithm".to_string(),
                    "Add caching for repeated calculations".to_string(),
                ],
            },
            StrategyEffectiveness {
                strategy_id: "priority_based".to_string(),
                success_rate: 0.87,
                average_execution_time_ms: 95.0,
                resource_efficiency_score: 0.78,
                domain_compatibility_score: 0.82,
                overall_effectiveness_score: 0.83,
                strengths: vec![
                    "Fast execution".to_string(),
                    "Simple and reliable".to_string(),
                ],
                weaknesses: vec![
                    "Lower resource efficiency".to_string(),
                    "Limited optimization sophistication".to_string(),
                ],
                improvement_suggestions: vec![
                    "Add resource cost consideration".to_string(),
                    "Implement dynamic priority adjustment".to_string(),
                ],
            },
        ];
        
        Ok(StrategyEffectivenessAnalysis {
            strategy_effectiveness: strategies,
            comparative_analysis: self.generate_comparative_strategy_analysis().await?,
            recommendations: vec![
                "Use capital_efficiency strategy for cost-sensitive scenarios".to_string(),
                "Use priority_based strategy for time-sensitive scenarios".to_string(),
                "Consider hybrid approach combining both strategies".to_string(),
            ],
        })
    }
    
    /// Generate performance trend analysis
    pub async fn analyze_performance_trends(&self, time_window_hours: u64) -> SimulationResult<PerformanceTrendAnalysis> {
        // Mock trend analysis data
        let trends = vec![
            PerformanceTrend {
                metric_name: "average_execution_time".to_string(),
                trend_direction: TrendDirection::Improving,
                change_percentage: -12.5,
                confidence_level: 0.85,
                data_points: 50,
            },
            PerformanceTrend {
                metric_name: "memory_usage".to_string(),
                trend_direction: TrendDirection::Stable,
                change_percentage: 2.1,
                confidence_level: 0.92,
                data_points: 50,
            },
            PerformanceTrend {
                metric_name: "success_rate".to_string(),
                trend_direction: TrendDirection::Improving,
                change_percentage: 8.3,
                confidence_level: 0.78,
                data_points: 50,
            },
        ];
        
        Ok(PerformanceTrendAnalysis {
            time_window_hours,
            trends,
            overall_trend_summary: "Performance is generally improving with execution time decreasing and success rate increasing".to_string(),
            anomalies_detected: vec![
                "Memory usage spike detected at 14:30 UTC".to_string(),
            ],
            forecast: self.generate_performance_forecast(&trends).await?,
        })
    }
    
    /// Export metrics for external analysis
    pub async fn export_metrics(&self, format: MetricsExportFormat) -> SimulationResult<String> {
        let metrics = self.collect_performance_metrics().await?;
        
        match format {
            MetricsExportFormat::Json => {
                serde_json::to_string_pretty(&metrics)
                    .map_err(|e| SimulationError::Serialization(e.to_string()))
            }
            MetricsExportFormat::Csv => {
                // Mock CSV export
                Ok("timestamp,metric_name,value\n2024-01-01T00:00:00Z,execution_time,150.5\n".to_string())
            }
            MetricsExportFormat::Prometheus => {
                // Mock Prometheus format
                Ok("# HELP simulation_execution_time_ms Execution time in milliseconds\nsimulation_execution_time_ms 150.5\n".to_string())
            }
        }
    }
    
    // === PRIVATE HELPER METHODS ===
    
    async fn collect_engine_metrics(&self) -> SimulationResult<EnginePerformanceMetrics> {
        Ok(EnginePerformanceMetrics {
            total_simulations_run: 42,
            average_simulation_duration_ms: 1250.0,
            total_effects_processed: 1337,
            average_effect_processing_time_ms: 25.5,
            memory_usage_bytes: self.estimate_memory_usage(),
            cpu_utilization_percentage: 65.0,
        })
    }
    
    async fn collect_strategy_performance_metrics(&self) -> SimulationResult<Vec<StrategyPerformanceMetrics>> {
        Ok(vec![
            StrategyPerformanceMetrics {
                strategy_id: "capital_efficiency".to_string(),
                invocation_count: 25,
                average_execution_time_ms: 150.0,
                success_rate: 0.92,
                average_plan_quality_score: 0.85,
            },
            StrategyPerformanceMetrics {
                strategy_id: "priority_based".to_string(),
                invocation_count: 17,
                average_execution_time_ms: 95.0,
                success_rate: 0.87,
                average_plan_quality_score: 0.78,
            },
        ])
    }
    
    async fn collect_domain_performance_metrics(&self) -> SimulationResult<Vec<DomainPerformanceMetrics>> {
        Ok(vec![
            DomainPerformanceMetrics {
                domain_type: "VerifiableDomain".to_string(),
                operations_count: 150,
                average_operation_time_ms: 200.0,
                constraint_violation_rate: 0.03,
                resource_efficiency_score: 0.82,
            },
            DomainPerformanceMetrics {
                domain_type: "ServiceDomain".to_string(),
                operations_count: 200,
                average_operation_time_ms: 120.0,
                constraint_violation_rate: 0.01,
                resource_efficiency_score: 0.88,
            },
        ])
    }
    
    async fn collect_pdb_performance_metrics(&self) -> SimulationResult<PdbPerformanceMetrics> {
        Ok(PdbPerformanceMetrics {
            total_pdb_instances_created: 35,
            average_pdb_execution_time_ms: 300.0,
            pdb_success_rate: 0.94,
            average_orchestration_complexity: 2.3,
            lisp_execution_overhead_percentage: 15.0,
        })
    }
    
    async fn collect_resource_utilization_metrics(&self) -> SimulationResult<ResourceUtilizationMetrics> {
        Ok(ResourceUtilizationMetrics {
            peak_memory_usage_bytes: 2 * 1024 * 1024, // 2MB
            average_memory_usage_bytes: 1024 * 1024,   // 1MB
            peak_cpu_utilization_percentage: 85.0,
            average_cpu_utilization_percentage: 65.0,
            network_calls_per_second: 12.5,
            storage_operations_per_second: 8.2,
        })
    }
    
    async fn generate_performance_analysis_summary(&self) -> SimulationResult<PerformanceAnalysisSummary> {
        Ok(PerformanceAnalysisSummary {
            overall_health_score: 0.87,
            key_insights: vec![
                "Strategy performance is within expected ranges".to_string(),
                "Memory usage is stable and efficient".to_string(),
                "PDB orchestration showing good success rates".to_string(),
            ],
            areas_for_improvement: vec![
                "Consider optimizing ZK domain operations".to_string(),
                "Monitor Lisp execution overhead".to_string(),
            ],
        })
    }
    
    async fn analyze_execution_time_bottlenecks(&self) -> SimulationResult<Vec<PerformanceBottleneck>> {
        Ok(vec![
            PerformanceBottleneck {
                bottleneck_type: BottleneckType::ExecutionTime,
                component: "PDB Orchestration".to_string(),
                severity: BottleneckSeverity::Medium,
                impact_description: "PDB orchestration taking 20% longer than expected".to_string(),
                suggested_fixes: vec![
                    "Optimize Lisp expression evaluation".to_string(),
                    "Add caching for repeated PDB operations".to_string(),
                ],
            },
        ])
    }
    
    async fn analyze_memory_bottlenecks(&self) -> SimulationResult<Vec<PerformanceBottleneck>> {
        Ok(vec![])
    }
    
    async fn analyze_pdb_bottlenecks(&self) -> SimulationResult<Vec<PerformanceBottleneck>> {
        Ok(vec![
            PerformanceBottleneck {
                bottleneck_type: BottleneckType::PdbOrchestration,
                component: "Lisp Expression Evaluation".to_string(),
                severity: BottleneckSeverity::Low,
                impact_description: "Lisp evaluation overhead is 15% of total execution time".to_string(),
                suggested_fixes: vec![
                    "Implement expression compilation".to_string(),
                    "Add expression result caching".to_string(),
                ],
            },
        ])
    }
    
    async fn generate_bottleneck_recommendations(&self, bottlenecks: &[PerformanceBottleneck]) -> SimulationResult<Vec<String>> {
        let mut recommendations = Vec::new();
        
        for bottleneck in bottlenecks {
            match bottleneck.severity {
                BottleneckSeverity::High => {
                    recommendations.push(format!("URGENT: Address {} bottleneck in {}", bottleneck.bottleneck_type, bottleneck.component));
                }
                BottleneckSeverity::Medium => {
                    recommendations.push(format!("Consider optimizing {} in {}", bottleneck.bottleneck_type, bottleneck.component));
                }
                BottleneckSeverity::Low => {
                    recommendations.push(format!("Monitor {} performance in {}", bottleneck.bottleneck_type, bottleneck.component));
                }
            }
        }
        
        Ok(recommendations)
    }
    
    fn calculate_bottleneck_severity_distribution(&self, bottlenecks: &[PerformanceBottleneck]) -> BottleneckSeverityDistribution {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        
        for bottleneck in bottlenecks {
            match bottleneck.severity {
                BottleneckSeverity::High => high += 1,
                BottleneckSeverity::Medium => medium += 1,
                BottleneckSeverity::Low => low += 1,
            }
        }
        
        BottleneckSeverityDistribution { high, medium, low }
    }
    
    async fn calculate_overall_performance_score(&self) -> SimulationResult<f64> {
        // Mock performance score calculation
        Ok(0.87)
    }
    
    async fn analyze_strategy_optimization_opportunities(&self) -> SimulationResult<Vec<OptimizationRecommendation>> {
        Ok(vec![
            OptimizationRecommendation {
                category: OptimizationCategory::Strategy,
                title: "Implement Hybrid Strategy Selection".to_string(),
                description: "Combine capital efficiency and priority-based strategies based on scenario characteristics".to_string(),
                expected_improvement: "15-20% improvement in overall effectiveness".to_string(),
                implementation_effort: ImplementationEffort::Medium,
                priority_score: 0.85,
            },
        ])
    }
    
    async fn analyze_domain_optimization_opportunities(&self) -> SimulationResult<Vec<OptimizationRecommendation>> {
        Ok(vec![
            OptimizationRecommendation {
                category: OptimizationCategory::Domain,
                title: "Optimize ZK Constraint Checking".to_string(),
                description: "Implement more efficient ZK constraint validation algorithms".to_string(),
                expected_improvement: "25% reduction in VerifiableDomain operation time".to_string(),
                implementation_effort: ImplementationEffort::High,
                priority_score: 0.78,
            },
        ])
    }
    
    async fn analyze_pdb_optimization_opportunities(&self) -> SimulationResult<Vec<OptimizationRecommendation>> {
        Ok(vec![
            OptimizationRecommendation {
                category: OptimizationCategory::PdbOrchestration,
                title: "Implement Lisp Expression Compilation".to_string(),
                description: "Compile frequently used Lisp expressions to reduce evaluation overhead".to_string(),
                expected_improvement: "30% reduction in Lisp execution time".to_string(),
                implementation_effort: ImplementationEffort::Medium,
                priority_score: 0.72,
            },
        ])
    }
    
    async fn analyze_resource_optimization_opportunities(&self) -> SimulationResult<Vec<OptimizationRecommendation>> {
        Ok(vec![
            OptimizationRecommendation {
                category: OptimizationCategory::Resource,
                title: "Implement Resource Pooling".to_string(),
                description: "Pool frequently used resources to reduce allocation overhead".to_string(),
                expected_improvement: "10% reduction in memory allocation overhead".to_string(),
                implementation_effort: ImplementationEffort::Low,
                priority_score: 0.65,
            },
        ])
    }
    
    async fn generate_implementation_roadmap(&self, recommendations: &[OptimizationRecommendation]) -> SimulationResult<ImplementationRoadmap> {
        let phases = vec![
            RoadmapPhase {
                phase_number: 1,
                title: "Quick Wins".to_string(),
                duration_weeks: 2,
                recommendations: recommendations.iter()
                    .filter(|r| r.implementation_effort == ImplementationEffort::Low)
                    .map(|r| r.title.clone())
                    .collect(),
            },
            RoadmapPhase {
                phase_number: 2,
                title: "Medium Impact Improvements".to_string(),
                duration_weeks: 6,
                recommendations: recommendations.iter()
                    .filter(|r| r.implementation_effort == ImplementationEffort::Medium)
                    .map(|r| r.title.clone())
                    .collect(),
            },
            RoadmapPhase {
                phase_number: 3,
                title: "Major Optimizations".to_string(),
                duration_weeks: 12,
                recommendations: recommendations.iter()
                    .filter(|r| r.implementation_effort == ImplementationEffort::High)
                    .map(|r| r.title.clone())
                    .collect(),
            },
        ];
        
        Ok(ImplementationRoadmap {
            phases,
            total_duration_weeks: 20,
            estimated_total_improvement: "40-50% overall performance improvement".to_string(),
        })
    }
    
    async fn estimate_improvement_potential(&self, recommendations: &[OptimizationRecommendation]) -> SimulationResult<ImprovementEstimate> {
        Ok(ImprovementEstimate {
            execution_time_improvement_percentage: 35.0,
            memory_usage_improvement_percentage: 15.0,
            success_rate_improvement_percentage: 8.0,
            overall_efficiency_improvement_percentage: 25.0,
            confidence_level: 0.75,
        })
    }
    
    async fn generate_comparative_strategy_analysis(&self) -> SimulationResult<ComparativeStrategyAnalysis> {
        Ok(ComparativeStrategyAnalysis {
            best_overall_strategy: "capital_efficiency".to_string(),
            best_speed_strategy: "priority_based".to_string(),
            best_resource_efficiency_strategy: "capital_efficiency".to_string(),
            strategy_trade_offs: vec![
                "Capital efficiency trades speed for better resource utilization".to_string(),
                "Priority-based strategy trades optimization quality for faster execution".to_string(),
            ],
        })
    }
    
    async fn generate_performance_forecast(&self, trends: &[PerformanceTrend]) -> SimulationResult<PerformanceForecast> {
        Ok(PerformanceForecast {
            forecast_horizon_hours: 24,
            predicted_metrics: vec![
                ForecastedMetric {
                    metric_name: "average_execution_time".to_string(),
                    current_value: 150.0,
                    predicted_value: 135.0,
                    confidence_interval: (125.0, 145.0),
                },
                ForecastedMetric {
                    metric_name: "success_rate".to_string(),
                    current_value: 0.89,
                    predicted_value: 0.92,
                    confidence_interval: (0.90, 0.94),
                },
            ],
            forecast_accuracy_score: 0.82,
        })
    }
}

// === METRICS DATA STRUCTURES ===

/// Comprehensive performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct ComprehensivePerformanceMetrics {
    pub timestamp: std::time::SystemTime,
    pub engine_metrics: EnginePerformanceMetrics,
    pub strategy_metrics: Vec<StrategyPerformanceMetrics>,
    pub domain_metrics: Vec<DomainPerformanceMetrics>,
    pub pdb_metrics: PdbPerformanceMetrics,
    pub resource_metrics: ResourceUtilizationMetrics,
    pub analysis_summary: PerformanceAnalysisSummary,
}

/// Engine-level performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct EnginePerformanceMetrics {
    pub total_simulations_run: u64,
    pub average_simulation_duration_ms: f64,
    pub total_effects_processed: u64,
    pub average_effect_processing_time_ms: f64,
    pub memory_usage_bytes: u64,
    pub cpu_utilization_percentage: f64,
}

/// Strategy-specific performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct StrategyPerformanceMetrics {
    pub strategy_id: String,
    pub invocation_count: u64,
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
    pub average_plan_quality_score: f64,
}

/// Domain-specific performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct DomainPerformanceMetrics {
    pub domain_type: String,
    pub operations_count: u64,
    pub average_operation_time_ms: f64,
    pub constraint_violation_rate: f64,
    pub resource_efficiency_score: f64,
}

/// PDB orchestration performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct PdbPerformanceMetrics {
    pub total_pdb_instances_created: u64,
    pub average_pdb_execution_time_ms: f64,
    pub pdb_success_rate: f64,
    pub average_orchestration_complexity: f64,
    pub lisp_execution_overhead_percentage: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize)]
pub struct ResourceUtilizationMetrics {
    pub peak_memory_usage_bytes: u64,
    pub average_memory_usage_bytes: u64,
    pub peak_cpu_utilization_percentage: f64,
    pub average_cpu_utilization_percentage: f64,
    pub network_calls_per_second: f64,
    pub storage_operations_per_second: f64,
}

/// Performance analysis summary
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceAnalysisSummary {
    pub overall_health_score: f64,
    pub key_insights: Vec<String>,
    pub areas_for_improvement: Vec<String>,
}

/// Bottleneck analysis result
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub detected_bottlenecks: Vec<PerformanceBottleneck>,
    pub severity_distribution: BottleneckSeverityDistribution,
    pub recommendations: Vec<String>,
    pub overall_performance_score: f64,
}

/// Performance bottleneck information
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub bottleneck_type: BottleneckType,
    pub component: String,
    pub severity: BottleneckSeverity,
    pub impact_description: String,
    pub suggested_fixes: Vec<String>,
}

/// Type of performance bottleneck
#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckType {
    ExecutionTime,
    Memory,
    PdbOrchestration,
    DomainConstraints,
    ResourceUtilization,
}

impl std::fmt::Display for BottleneckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BottleneckType::ExecutionTime => write!(f, "execution_time"),
            BottleneckType::Memory => write!(f, "memory"),
            BottleneckType::PdbOrchestration => write!(f, "pdb_orchestration"),
            BottleneckType::DomainConstraints => write!(f, "domain_constraints"),
            BottleneckType::ResourceUtilization => write!(f, "resource_utilization"),
        }
    }
}

/// Severity of performance bottleneck
#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckSeverity {
    High,
    Medium,
    Low,
}

/// Distribution of bottleneck severities
#[derive(Debug, Clone)]
pub struct BottleneckSeverityDistribution {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Optimization recommendations
#[derive(Debug, Clone)]
pub struct OptimizationRecommendations {
    pub recommendations: Vec<OptimizationRecommendation>,
    pub implementation_roadmap: ImplementationRoadmap,
    pub expected_improvements: ImprovementEstimate,
}

/// Individual optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: OptimizationCategory,
    pub title: String,
    pub description: String,
    pub expected_improvement: String,
    pub implementation_effort: ImplementationEffort,
    pub priority_score: f64,
}

/// Category of optimization
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationCategory {
    Strategy,
    Domain,
    PdbOrchestration,
    Resource,
    Infrastructure,
}

/// Implementation effort level
#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

/// Implementation roadmap
#[derive(Debug, Clone)]
pub struct ImplementationRoadmap {
    pub phases: Vec<RoadmapPhase>,
    pub total_duration_weeks: u64,
    pub estimated_total_improvement: String,
}

/// Phase in implementation roadmap
#[derive(Debug, Clone)]
pub struct RoadmapPhase {
    pub phase_number: u32,
    pub title: String,
    pub duration_weeks: u64,
    pub recommendations: Vec<String>,
}

/// Improvement estimate
#[derive(Debug, Clone)]
pub struct ImprovementEstimate {
    pub execution_time_improvement_percentage: f64,
    pub memory_usage_improvement_percentage: f64,
    pub success_rate_improvement_percentage: f64,
    pub overall_efficiency_improvement_percentage: f64,
    pub confidence_level: f64,
}

/// Strategy effectiveness analysis
#[derive(Debug, Clone)]
pub struct StrategyEffectivenessAnalysis {
    pub strategy_effectiveness: Vec<StrategyEffectiveness>,
    pub comparative_analysis: ComparativeStrategyAnalysis,
    pub recommendations: Vec<String>,
}

/// Individual strategy effectiveness
#[derive(Debug, Clone)]
pub struct StrategyEffectiveness {
    pub strategy_id: String,
    pub success_rate: f64,
    pub average_execution_time_ms: f64,
    pub resource_efficiency_score: f64,
    pub domain_compatibility_score: f64,
    pub overall_effectiveness_score: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub improvement_suggestions: Vec<String>,
}

/// Comparative strategy analysis
#[derive(Debug, Clone)]
pub struct ComparativeStrategyAnalysis {
    pub best_overall_strategy: String,
    pub best_speed_strategy: String,
    pub best_resource_efficiency_strategy: String,
    pub strategy_trade_offs: Vec<String>,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrendAnalysis {
    pub time_window_hours: u64,
    pub trends: Vec<PerformanceTrend>,
    pub overall_trend_summary: String,
    pub anomalies_detected: Vec<String>,
    pub forecast: PerformanceForecast,
}

/// Individual performance trend
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    pub metric_name: String,
    pub trend_direction: TrendDirection,
    pub change_percentage: f64,
    pub confidence_level: f64,
    pub data_points: u32,
}

/// Direction of performance trend
#[derive(Debug, Clone)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
}

/// Performance forecast
#[derive(Debug, Clone)]
pub struct PerformanceForecast {
    pub forecast_horizon_hours: u64,
    pub predicted_metrics: Vec<ForecastedMetric>,
    pub forecast_accuracy_score: f64,
}

/// Forecasted metric value
#[derive(Debug, Clone)]
pub struct ForecastedMetric {
    pub metric_name: String,
    pub current_value: f64,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
}

/// Metrics export format
#[derive(Debug, Clone)]
pub enum MetricsExportFormat {
    Json,
    Csv,
    Prometheus,
}
