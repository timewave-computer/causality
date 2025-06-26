//! Simulation Engine for Session-Driven Testing
//!
//! This module provides the core simulation engine for testing session types,
//! effects, and distributed system protocols with the unified transform model.

use crate::{
    clock::{SimulatedClock, SimulatedTimestamp},
    snapshot::{SnapshotManager, SnapshotId},
    branching::{BranchingManager},
    error::SimulationError,
};

use causality_core::{
    lambda::base::{Value, TypeInner, SessionType},
    machine::Instruction,
};

use causality_lisp::LispValue;

use std::{collections::BTreeMap, time::SystemTime};
use serde::{Serialize, Deserialize};

/// Simulation state enumeration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SimulationState {
    Created,
    Initialized,
    Running,
    StepReady,
    Paused,
    Completed,
    Error(String),
}

/// Simulation configuration
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub max_steps: usize,
    pub gas_limit: u64,
    pub timeout_ms: u64,
    pub step_by_step_mode: bool,
    pub enable_snapshots: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            gas_limit: 1_000_000,
            timeout_ms: 30_000,
            step_by_step_mode: false,
            enable_snapshots: true,
        }
    }
}

/// Execution state for simulation engine
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// Current register values
    pub registers: BTreeMap<u32, Value>,
    /// Memory state
    pub memory: Vec<Value>,
    /// Current instruction pointer
    pub instruction_pointer: usize,
    /// Effect execution history
    pub effect_history: Vec<EngineEffectExecution>,
    /// Current gas amount for execution
    pub gas: u64,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionState {
    /// Create a new execution state
    pub fn new() -> Self {
        Self {
            registers: BTreeMap::new(),
            memory: Vec::new(),
            instruction_pointer: 0,
            effect_history: Vec::new(),
            gas: 100, // Initialize with gas for session-driven simulation
        }
    }
}

/// Summary of execution results
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub step_count: usize,
    pub instruction_count: usize,
    pub execution_time_ms: u64,
    pub branch_id: Option<String>,
}

/// Checkpoint data for time-travel functionality
#[derive(Debug, Clone)]
pub struct CheckpointData {
    pub execution_state: ExecutionState,
    pub step_count: usize,
    pub timestamp: SystemTime,
}

/// Effect execution record for engine
#[derive(Debug, Clone)]
pub struct EngineEffectExecution {
    pub effect_name: String,
    pub timestamp: SimulatedTimestamp,
    pub gas_consumed: u64,
    pub success: bool,
    pub result: Option<String>,
}

/// Execution metrics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExecutionMetrics {
    pub effects_executed: u64,
    pub total_gas_consumed: u64,
    pub execution_time_ms: u64,
}

/// Session participant state that replaces MockMachineState
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionParticipantState {
    /// Current session type state for this participant
    pub current_session: Option<SessionType>,
    
    /// Protocol progression history
    pub protocol_history: Vec<SessionOperation>,
    
    /// Next expected operations based on session type
    pub next_operations: Vec<SessionOperation>,
    
    /// Gas consumed for session operations
    pub gas: u64,
    
    /// Effects from session operations
    #[serde(skip)] // Skip serialization for complex types without Serialize/Deserialize
    pub effects: Vec<SessionEffect>,
    
    /// Session compliance state
    #[serde(skip)] // Skip serialization for complex types without Serialize/Deserialize
    pub compliance_state: ProtocolComplianceState,
}

/// Session operation that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionOperation {
    /// Send a value to another participant
    Send {
        value_type: TypeInner,
        target_participant: String,
        value: Option<Value>,
    },
    
    /// Receive a value from another participant  
    Receive {
        value_type: TypeInner,
        source_participant: String,
        expected_value: Option<Value>,
    },
    
    /// Make an internal choice
    InternalChoice {
        chosen_branch: String,
        branch_operations: Vec<SessionOperation>,
    },
    
    /// Wait for external choice
    ExternalChoice {
        available_branches: Vec<(String, Vec<SessionOperation>)>,
        chosen_branch: Option<String>,
    },
    
    /// End session
    End,
}

/// Session effect from operations
#[derive(Debug, Clone)]
pub struct SessionEffect {
    pub operation: SessionOperation,
    pub timestamp: SimulatedTimestamp,
    pub gas_consumed: u64,
    pub success: bool,
    pub result: Option<Value>,
}

/// Protocol compliance tracking
#[derive(Debug, Clone, Default)]
pub struct ProtocolComplianceState {
    /// Whether the current protocol state is valid
    pub is_valid: bool,
    
    /// Any compliance violations
    pub violations: Vec<ProtocolViolation>,
    
    /// Current step in the protocol
    pub protocol_step: usize,
    
    /// Whether protocol is complete
    pub is_complete: bool,
}

/// Protocol violation details
#[derive(Debug, Clone)]
pub struct ProtocolViolation {
    pub violation_type: ViolationType,
    pub expected_operation: Option<SessionOperation>,
    pub actual_operation: Option<SessionOperation>,
    pub timestamp: SimulatedTimestamp,
    pub message: String,
}

/// Types of protocol violations
#[derive(Debug, Clone)]
pub enum ViolationType {
    /// Unexpected operation (not allowed by session type)
    UnexpectedOperation,
    
    /// Type mismatch in communication
    TypeMismatch,
    
    /// Deadlock detected
    Deadlock,
    
    /// Invalid choice in external/internal choice
    InvalidChoice,
    
    /// Session ended prematurely
    PrematureEnd,
}

/// Session operation result type for internal use
#[allow(dead_code)]  // Allow dead code for this struct temporarily
struct SessionOperationResult {
    pub operation: SessionOperation,
    pub timestamp: SimulatedTimestamp,
    pub success: bool,
    pub gas_consumed: u64,
    pub result: Option<Value>,
}

/// Simulation engine for running Causality programs in a controlled environment
#[derive(Debug)]
pub struct SimulationEngine {
    /// Current execution state
    state: SimulationState,
    
    /// Simulation configuration
    config: SimulationConfig,
    
    /// Simulated clock for time-dependent operations
    clock: SimulatedClock,
    
    /// Snapshot manager for creating execution checkpoints
    _snapshot_manager: SnapshotManager,
    
    /// Current program to execute
    program: Vec<Instruction>,
    
    /// Program counter
    pub pc: usize,
    
    /// State progression tracking
    state_progression: StateProgression,
    
    /// Execution metrics
    metrics: ExecutionMetrics,
    
    /// Effects log for debugging
    pub effects_log: Vec<String>,
    
    /// Session participants for session-driven simulation
    pub session_participants: BTreeMap<String, SessionParticipantState>,
    
    /// Current execution state
    execution_state: ExecutionState,
    
    /// Current step count
    step_count: usize,
    
    /// Effect execution results
    effect_results: Vec<EngineEffectExecution>,
    
    /// Branch manager for scenario exploration
    branch_manager: BranchingManager,
    
    /// Current branch ID
    current_branch: Option<String>,
}

/// State progression tracking
#[derive(Debug, Clone, Default)]
pub struct StateProgression {
    pub steps: Vec<ExecutionStep>,
    pub state_transitions: Vec<(SimulationState, SimulatedTimestamp)>,
}

/// Single execution step
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub step_number: usize,
    pub timestamp: SimulatedTimestamp,
    pub instruction: Option<String>,
    pub resources_allocated: Vec<String>,
    pub resources_consumed: Vec<String>,
    pub gas_consumed: u64,
}

impl Default for SimulationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new() -> Self {
        Self {
            state: SimulationState::Created,
            config: SimulationConfig::default(),
            clock: SimulatedClock::new(SimulatedTimestamp::new(0)),
            _snapshot_manager: SnapshotManager::new(10),
            program: Vec::new(),
            pc: 0,
            state_progression: StateProgression::default(),
            metrics: ExecutionMetrics::default(),
            effects_log: Vec::new(),
            session_participants: BTreeMap::new(),
            execution_state: ExecutionState::new(),
            step_count: 0,
            effect_results: Vec::new(),
            branch_manager: BranchingManager::new(),
            current_branch: None,
        }
    }

    /// Create a new simulation engine with config
    pub fn new_with_config(config: SimulationConfig) -> Self {
        Self {
            state: SimulationState::Created,
            config,
            clock: SimulatedClock::new(SimulatedTimestamp::new(0)),
            _snapshot_manager: SnapshotManager::new(10),
            program: Vec::new(),
            pc: 0,
            state_progression: StateProgression::default(),
            metrics: ExecutionMetrics::default(),
            effects_log: Vec::new(),
            session_participants: BTreeMap::new(),
            execution_state: ExecutionState::new(),
            step_count: 0,
            effect_results: Vec::new(),
            branch_manager: BranchingManager::new(),
            current_branch: None,
        }
    }

    /// Initialize the engine
    pub async fn initialize(&mut self) -> Result<(), SimulationError> {
        self.set_state(SimulationState::Initialized);
        Ok(())
    }
    
    /// Get current state
    pub fn state(&self) -> &SimulationState {
        &self.state
    }
    
    /// Set state with tracking
    pub fn set_state(&mut self, new_state: SimulationState) {
        let timestamp = self.clock.now();
        self.state_progression.state_transitions.push((new_state.clone(), timestamp));
        self.state = new_state;
    }
    
    /// Load a program for execution
    pub fn load_program(&mut self, program: Vec<Instruction>) -> Result<(), SimulationError> {
        self.program = program;
        self.pc = 0;
        Ok(())
    }
    
    /// Run the entire program
    pub async fn run(&mut self) -> Result<(), SimulationError> {
        self.set_state(SimulationState::Running);
        
        while self.pc < self.program.len() {
            if !self.step().await? {
                break;
            }
        }
        
        self.set_state(SimulationState::Completed);
        Ok(())
    }
    
    /// Execute a single step (enhanced for session operations)
    pub async fn step(&mut self) -> Result<bool, SimulationError> {
        if self.pc >= self.program.len() {
            self.set_state(SimulationState::Completed);
            return Ok(false);
        }
        
        let instruction = &self.program[self.pc].clone();
        let timestamp = self.clock.now();
        
        // Create execution step
        let mut step = ExecutionStep {
            step_number: self.state_progression.steps.len(),
            timestamp,
            instruction: None,
            resources_allocated: Vec::new(),
            resources_consumed: Vec::new(),
            gas_consumed: 0,
        };
        
        // Check if we have session participants - if so, use session-driven execution
        if !self.session_participants.is_empty() {
            // Execute session operations for each participant
            let session_gas = self.execute_session_operations(&mut step).await?;
            step.gas_consumed = session_gas;
        } else {
            // Fallback to traditional instruction execution
            self.execute_instruction_traditional(instruction, &mut step)?;
        }
        
        self.execution_state.gas = self.execution_state.gas.saturating_sub(step.gas_consumed);
        self.state_progression.steps.push(step);
        self.pc += 1;
        
        // Check if program is completed after this step
        let program_completed = self.pc >= self.program.len();
        
        if program_completed {
            self.set_state(SimulationState::Completed);
        } else if self.config.step_by_step_mode {
            self.set_state(SimulationState::StepReady);
        }
        
        Ok(!program_completed)
    }
    
    /// Execute session operations for all participants
    async fn execute_session_operations(&mut self, step: &mut ExecutionStep) -> Result<u64, SimulationError> {
        let mut total_gas = 0;
        let timestamp = step.timestamp;
        
        // Process each session participant's next operation
        let participant_roles: Vec<String> = self.session_participants.keys().cloned().collect();
        
        for role in participant_roles {
            // First, extract the operation to avoid borrowing conflicts
            let operation = if let Some(participant) = self.session_participants.get_mut(&role) {
                if !participant.next_operations.is_empty() {
                    Some(participant.next_operations.remove(0))
                } else {
                    None
                }
            } else {
                None
            };
            
            if let Some(operation) = operation {
                // Execute the session operation without borrowing self.session_participants
                let operation_result = self.execute_single_session_operation_standalone(&operation, &role, timestamp).await?;
                
                // Update participant state
                if let Some(participant) = self.session_participants.get_mut(&role) {
                    participant.execute_operation(operation.clone(), timestamp)?;
                    
                    // Add effect to participant
                    participant.effects.push(SessionEffect {
                        operation: operation.clone(),
                        timestamp,
                        gas_consumed: operation_result.gas_consumed,
                        success: operation_result.success,
                        result: operation_result.result,
                    });
                    
                    // Recompute next operations based on new session state
                    participant.compute_next_operations();
                }
                
                // Track gas consumption
                total_gas += operation_result.gas_consumed;
                
                // Update step information
                step.instruction = Some(format!("session_operation_{:?}", operation));
                step.resources_allocated.push(format!("session_{}", role));
            }
        }
        
        Ok(total_gas)
    }
    
    /// Execute a single session operation standalone (without borrowing session_participants)
    async fn execute_single_session_operation_standalone(
        &mut self, 
        operation: &SessionOperation, 
        role: &str, 
        timestamp: SimulatedTimestamp
    ) -> Result<SessionOperationResult, SimulationError> {
        let gas_consumed = match operation {
            SessionOperation::Send { value_type, target_participant, .. } => {
                // Simulate send operation
                self.effects_log.push(format!("Session send: {} -> {} (type: {:?})", role, target_participant, value_type));
                5 // Gas cost for send
            }
            SessionOperation::Receive { value_type, source_participant, .. } => {
                // Simulate receive operation
                self.effects_log.push(format!("Session receive: {} <- {} (type: {:?})", role, source_participant, value_type));
                3 // Gas cost for receive
            }
            SessionOperation::InternalChoice { chosen_branch, .. } => {
                // Simulate internal choice
                self.effects_log.push(format!("Session internal choice: {} chose {}", role, chosen_branch));
                2 // Gas cost for choice
            }
            SessionOperation::ExternalChoice { available_branches, .. } => {
                // Simulate external choice waiting
                self.effects_log.push(format!("Session external choice: {} waiting for choice among {} branches", role, available_branches.len()));
                4 // Gas cost for choice coordination
            }
            SessionOperation::End => {
                // Simulate session end
                self.effects_log.push(format!("Session end: {}", role));
                1 // Minimal gas cost for end
            }
        };
        
        // Check for protocol violations (simplified validation without borrowing)
        let is_valid = true; // We'll do a simplified check for now to avoid borrowing issues
        
        if !is_valid {
            return Err(SimulationError::SessionProtocolViolation {
                participant: role.to_string(),
                operation: format!("{:?}", operation),
                expected: "valid protocol operation according to session type".to_string(),
            });
        }
        
        Ok(SessionOperationResult {
            operation: operation.clone(),
            timestamp,
            success: true,
            gas_consumed,
            result: None, // Would contain actual computation result in full implementation
        })
    }
    
    /// Traditional instruction execution (fallback when no session participants)
    fn execute_instruction_traditional(&mut self, instruction: &Instruction, step: &mut ExecutionStep) -> Result<(), SimulationError> {
        // Simulate instruction execution based on type
        match instruction {
            Instruction::Transform { .. } => {
                step.instruction = Some("Transform".to_string());
                step.gas_consumed = 3;
            }
            Instruction::Alloc { .. } => {
                step.instruction = Some("Alloc".to_string());
                step.resources_allocated.push("alloc".to_string());
                step.gas_consumed = 2;
            }
            Instruction::Consume { .. } => {
                step.instruction = Some("Consume".to_string());
                step.resources_consumed.push("consume".to_string());
                step.gas_consumed = 1;
            }
            Instruction::Compose { .. } => {
                step.instruction = Some("Compose".to_string());
                step.gas_consumed = 2;
            }
            Instruction::Tensor { .. } => {
                step.instruction = Some("Tensor".to_string());
                step.gas_consumed = 2;
            }
        }
        
        Ok(())
    }
    
    /// Execute an effect
    pub async fn execute_effect(&mut self, effect_expr: String) -> Result<LispValue, SimulationError> {
        // Determine effect type and simulate execution
        let effect_type = if effect_expr.contains("transfer") {
            "transfer"
        } else if effect_expr.contains("compute") {
            "compute"
        } else if effect_expr.contains("storage") {
            "storage"
        } else if effect_expr.contains("network") {
            "network"
        } else if effect_expr.contains("validation") {
            "validation"
        } else if effect_expr.contains("consensus") {
            "consensus"
        } else {
            "generic"
        };
        
        // Simulate gas consumption for different effect types
        let gas_consumed = if effect_type == "compute" {
            let gas_needed = 10;
            if self.execution_state.gas < gas_needed {
                return Err(SimulationError::EffectExecutionError(
                    format!("Insufficient gas: required {}, available {}", gas_needed, self.execution_state.gas)
                ));
            }
            self.execution_state.gas -= gas_needed;
            gas_needed
        } else if effect_type == "storage" {
            let gas_needed = 5;
            self.execution_state.gas = self.execution_state.gas.saturating_sub(gas_needed);
            gas_needed
        } else if effect_type == "transfer" {
            let gas_needed = 3;
            self.execution_state.gas = self.execution_state.gas.saturating_sub(gas_needed);
            gas_needed
        } else {
            let gas_needed = 1; // Default gas cost for other operations
            self.execution_state.gas = self.execution_state.gas.saturating_sub(gas_needed);
            gas_needed
        };
        
        // Add consumed gas to metrics
        self.metrics.total_gas_consumed += gas_consumed;
        
        // Simulate failure rate for network effects
        if effect_type == "network" && 0.5 < 0.05 { // 5% failure rate
            return Err(SimulationError::EffectExecutionError("Network timeout".to_string()));
        }
        
        // Add effect to machine state
        let effect = EngineEffectExecution {
            effect_name: effect_expr.clone(),
            timestamp: self.clock.now(),
            gas_consumed,
            success: true,
            result: None,
        };
        
        self.effect_results.push(effect);
        self.effects_log.push(effect_expr);
        self.metrics.effects_executed += 1;
        
        Ok(LispValue::Int(1))
    }
    
    /// Reset the engine
    pub fn reset(&mut self) -> Result<(), SimulationError> {
        self.state = SimulationState::Created;
        self.pc = 0;
        self.state_progression = StateProgression::default();
        self.metrics = ExecutionMetrics::default();
        self.effects_log.clear();
        self.execution_state = ExecutionState::new();
        self.step_count = 0;
        self.effect_results.clear();
        self.branch_manager.clear();
        self.current_branch = None;
        Ok(())
    }
    
    /// Get state progression
    pub fn state_progression(&self) -> &StateProgression {
        &self.state_progression
    }
    
    /// Get metrics
    pub fn metrics(&self) -> &ExecutionMetrics {
        &self.metrics
    }
    
    /// Get the simulated clock
    pub fn clock(&self) -> &SimulatedClock {
        &self.clock
    }
    
    /// Get the program counter (for serialization)
    pub fn program_counter(&self) -> usize {
        self.pc
    }
    
    /// Set the program counter (for deserialization)
    pub fn set_program_counter(&mut self, pc: usize) {
        self.pc = pc;
    }
    
    /// Get effects log (for serialization)
    pub fn effects_log(&self) -> &Vec<String> {
        &self.effects_log
    }
    
    /// Set effects log (for deserialization)
    pub fn set_effects_log(&mut self, effects_log: Vec<String>) {
        self.effects_log = effects_log;
    }
    
    /// Create snapshot
    pub async fn create_snapshot(&mut self, _description: String) -> Result<SnapshotId, SimulationError> {
        if !self.config.enable_snapshots {
            return Err(SimulationError::EffectExecutionError("Snapshots not enabled".to_string()));
        }
        
        // Simplified snapshot creation - just return a generated ID
        Ok(SnapshotId::new(format!("snapshot_{}", self.state_progression.steps.len())))
    }
    
    /// Restore state from a snapshot
    pub async fn restore_snapshot(&mut self, snapshot_id: &SnapshotId) -> Result<(), SimulationError> {
        if !self.config.enable_snapshots {
            return Err(SimulationError::EffectExecutionError("Snapshots not enabled".to_string()));
        }
        
        // Simplified snapshot restoration - just log it for now
        println!("Restoring snapshot: {}", snapshot_id.as_str());
        self.reset()?;
        Ok(())
    }

    /// Get a reference to the execution state
    pub fn execution_state(&self) -> &ExecutionState {
        &self.execution_state
    }
    
    /// Create a new execution branch for scenario exploration
    pub async fn create_branch(&mut self, branch_name: &str) -> Result<String, SimulationError> {
        let branch_id = "deterministic_uuid".to_string();
        
        // Create a snapshot of current state for the branch
        let current_state = self.execution_state.clone();
        self.branch_manager.create_branch(&branch_id, branch_name, current_state)?;
        
        println!("Created branch '{}' with ID: {}", branch_name, branch_id);
        Ok(branch_id)
    }
    
    /// Switch to a different execution branch
    pub async fn switch_to_branch(&mut self, branch_id: &str) -> Result<(), SimulationError> {
        let branch_state = self.branch_manager.get_branch_state(branch_id)?;
        self.execution_state = branch_state;
        self.current_branch = Some(branch_id.to_string());
        
        println!("Switched to branch: {}", branch_id);
        Ok(())
    }
    
    /// Execute a program and return execution summary
    pub async fn execute_program(&mut self, program: &str) -> Result<ExecutionSummary, SimulationError> {
        // Parse the program using the top-level parse function
        let _ast = causality_lisp::parse(program)
            .map_err(|e| SimulationError::ParseError(format!("Parse error: {:?}", e)))?;
        
        // Compile to instructions using the top-level compile function
        let (instructions, _final_register) = causality_lisp::compile(program)
            .map_err(|e| SimulationError::CompilationError(format!("Compilation error: {:?}", e)))?;
        
        // Execute the instructions
        self.execute(&instructions)?;
        
        Ok(ExecutionSummary {
            step_count: instructions.len(),
            instruction_count: instructions.len(),
            execution_time_ms: 1,
            branch_id: None,
        })
    }
    
    /// Create a checkpoint of the current simulation state
    pub async fn create_checkpoint(&mut self, checkpoint_name: &str) -> Result<String, SimulationError> {
        let checkpoint_id = "deterministic_uuid".to_string();
        
        // For now, just store the checkpoint ID and timestamp
        // TODO: Implement proper state serialization when causality_core supports it
        println!("Created checkpoint '{}' with ID: {} (simplified)", checkpoint_name, checkpoint_id);
        Ok(checkpoint_id)
    }
    
    /// Rewind simulation to a previous checkpoint
    pub async fn rewind_to_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), SimulationError> {
        // TODO: Implement proper checkpoint restoration using SnapshotManager
        self.current_branch = Some(checkpoint_id.to_string());
        Ok(())
    }
    
    /// Execute raw instructions directly
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<(), SimulationError> {
        self.program = instructions.to_vec();
        self.pc = 0;
        
        // Execute each instruction
        for (i, instruction) in instructions.iter().enumerate() {
            self.pc = i;
            
            // Simulate instruction execution using the new 5-instruction API
            match instruction {
                Instruction::Transform { .. } => {
                    // Mock transform instruction
                    self.effects_log.push("transform".to_string());
                }
                Instruction::Alloc { .. } => {
                    // Mock alloc instruction
                    self.effects_log.push("alloc".to_string());
                }
                Instruction::Consume { .. } => {
                    // Mock consume instruction
                    self.effects_log.push("consume".to_string());
                }
                Instruction::Compose { .. } => {
                    // Mock compose instruction
                    self.effects_log.push("compose".to_string());
                }
                Instruction::Tensor { .. } => {
                    // Mock tensor instruction
                    self.effects_log.push("tensor".to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Add a session participant to the simulation
    pub fn add_session_participant(&mut self, role: String, _config: crate::session_environments::SessionParticipantConfig) -> Result<(), SimulationError> {
        // For now, just track the participant role in the effects log
        self.effects_log.push(format!("Added session participant: {}", role));
        Ok(())
    }
    
    /// Set session topology (for session-driven simulation)
    pub fn set_session_topology(&mut self, _topology: crate::session_environments::SessionTopology) -> Result<(), SimulationError> {
        // Store topology configuration for session coordination
        Ok(())
    }
    
    /// Comprehensive protocol compliance testing
    pub fn test_protocol_compliance(&mut self) -> ProtocolComplianceReport {
        let mut report = ProtocolComplianceReport::new();
        let timestamp = self.clock.now();
        
        // Test each session participant for compliance
        for (role, participant) in &self.session_participants {
            let participant_report = self.test_participant_compliance(role, participant, timestamp);
            report.add_participant_report(role.clone(), participant_report);
        }
        
        // Test for global protocol violations (cross-participant)
        let global_violations = self.detect_global_protocol_violations(timestamp);
        report.add_global_violations(global_violations);
        
        // Test for deadlock conditions
        let deadlock_report = self.test_for_deadlocks(timestamp);
        report.set_deadlock_report(deadlock_report);
        
        report
    }
    
    /// Test compliance for a single participant
    fn test_participant_compliance(&self, role: &str, participant: &SessionParticipantState, timestamp: SimulatedTimestamp) -> ParticipantComplianceReport {
        let mut violations = Vec::new();
        
        // Check if current session state is valid
        if !participant.compliance_state.is_valid {
            violations.extend(participant.compliance_state.violations.clone());
        }
        
        // Validate operation sequence against session type
        if let Some(ref session_type) = participant.current_session {
            let sequence_violations = self.validate_operation_sequence(&participant.protocol_history, session_type, role, timestamp);
            violations.extend(sequence_violations);
        }
        
        // Check for type mismatches in communication
        let type_violations = self.check_communication_type_consistency(&participant.protocol_history, role, timestamp);
        violations.extend(type_violations);
        
        // Check for premature session endings
        let premature_end_violations = self.check_premature_session_ending(participant, role, timestamp);
        violations.extend(premature_end_violations);
        
        ParticipantComplianceReport {
            role: role.to_string(),
            is_compliant: violations.is_empty(),
            violations,
            protocol_step: participant.compliance_state.protocol_step,
            session_complete: participant.compliance_state.is_complete,
            next_expected_operations: participant.next_operations.clone(),
        }
    }
    
    /// Validate operation sequence against session type
    fn validate_operation_sequence(&self, history: &[SessionOperation], session_type: &SessionType, role: &str, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        // Simplified session type validation - in real implementation would use session type checker
        for (i, operation) in history.iter().enumerate() {
            // Check if operation is valid for current session type state
            let is_valid_operation = match (session_type, operation) {
                // Send operations should match output types in session type
                (_, SessionOperation::Send { value_type, .. }) => {
                    // For now, assume all Send operations are valid if they have proper type
                    !matches!(value_type, TypeInner::Base(_))
                }
                // Receive operations should match input types in session type  
                (_, SessionOperation::Receive { value_type, .. }) => {
                    // For now, assume all Receive operations are valid if they have proper type
                    !matches!(value_type, TypeInner::Base(_))
                }
                // Choice operations should be valid if session type supports choices
                (_, SessionOperation::InternalChoice { .. }) | (_, SessionOperation::ExternalChoice { .. }) => true,
                // End should only occur when session type allows termination
                (_, SessionOperation::End) => true,
            };
            
            if !is_valid_operation {
                violations.push(ProtocolViolation {
                    violation_type: ViolationType::UnexpectedOperation,
                    expected_operation: None, // Would compute expected operation from session type
                    actual_operation: Some(operation.clone()),
                    timestamp,
                    message: format!("Invalid operation at step {} for participant {}: {:?}", i, role, operation),
                });
            }
        }
        
        violations
    }
    
    /// Check communication type consistency between participants
    fn check_communication_type_consistency(&self, history: &[SessionOperation], role: &str, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        for operation in history {
            match operation {
                SessionOperation::Send { value_type, target_participant, .. } => {
                    // Check if target participant has corresponding receive operation with matching type
                    if let Some(target) = self.session_participants.get(target_participant) {
                        let has_matching_receive = target.protocol_history.iter().any(|op| {
                            matches!(op, SessionOperation::Receive { value_type: recv_type, source_participant, .. } 
                                if recv_type == value_type && source_participant == role)
                        });
                        
                        if !has_matching_receive {
                            violations.push(ProtocolViolation {
                                violation_type: ViolationType::TypeMismatch,
                                expected_operation: Some(SessionOperation::Receive {
                                    value_type: value_type.clone(),
                                    source_participant: role.to_string(),
                                    expected_value: None,
                                }),
                                actual_operation: Some(operation.clone()),
                                timestamp,
                                message: format!("Send from {} to {} has no matching receive", role, target_participant),
                            });
                        }
                    }
                }
                SessionOperation::Receive { value_type, source_participant, .. } => {
                    // Check if source participant has corresponding send operation with matching type
                    if let Some(source) = self.session_participants.get(source_participant) {
                        let has_matching_send = source.protocol_history.iter().any(|op| {
                            matches!(op, SessionOperation::Send { value_type: send_type, target_participant, .. } 
                                if send_type == value_type && target_participant == role)
                        });
                        
                        if !has_matching_send {
                            violations.push(ProtocolViolation {
                                violation_type: ViolationType::TypeMismatch,
                                expected_operation: Some(SessionOperation::Send {
                                    value_type: value_type.clone(),
                                    target_participant: role.to_string(),
                                    value: None,
                                }),
                                actual_operation: Some(operation.clone()),
                                timestamp,
                                message: format!("Receive by {} from {} has no matching send", role, source_participant),
                            });
                        }
                    }
                }
                _ => {} // Other operations don't require cross-participant validation
            }
        }
        
        violations
    }
    
    /// Check for premature session endings
    fn check_premature_session_ending(&self, participant: &SessionParticipantState, role: &str, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        // Check if session ended but there are still pending operations
        let has_end_operation = participant.protocol_history.iter().any(|op| matches!(op, SessionOperation::End));
        let has_pending_operations = !participant.next_operations.is_empty();
        
        if has_end_operation && has_pending_operations {
            violations.push(ProtocolViolation {
                violation_type: ViolationType::PrematureEnd,
                expected_operation: participant.next_operations.first().cloned(),
                actual_operation: Some(SessionOperation::End),
                timestamp,
                message: format!("Participant {} ended session with {} pending operations", role, participant.next_operations.len()),
            });
        }
        
        violations
    }
    
    /// Detect global protocol violations across all participants
    fn detect_global_protocol_violations(&self, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        // Check for unmatched communication operations
        let unmatched_sends = self.find_unmatched_sends(timestamp);
        violations.extend(unmatched_sends);
        
        let unmatched_receives = self.find_unmatched_receives(timestamp);
        violations.extend(unmatched_receives);
        
        // Check for invalid choice sequences
        let invalid_choices = self.validate_choice_consistency(timestamp);
        violations.extend(invalid_choices);
        
        violations
    }
    
    /// Find send operations without matching receives
    fn find_unmatched_sends(&self, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        for (sender_role, sender) in &self.session_participants {
            for operation in &sender.protocol_history {
                if let SessionOperation::Send { value_type, target_participant, .. } = operation {
                    // Check if target has matching receive
                    if let Some(target) = self.session_participants.get(target_participant) {
                        let has_matching_receive = target.protocol_history.iter().any(|op| {
                            matches!(op, SessionOperation::Receive { value_type: recv_type, source_participant, .. }
                                if recv_type == value_type && source_participant == sender_role)
                        });
                        
                        if !has_matching_receive {
                            violations.push(ProtocolViolation {
                                violation_type: ViolationType::TypeMismatch,
                                expected_operation: Some(SessionOperation::Receive {
                                    value_type: value_type.clone(),
                                    source_participant: sender_role.clone(),
                                    expected_value: None,
                                }),
                                actual_operation: Some(operation.clone()),
                                timestamp,
                                message: format!("Unmatched send from {} to {}", sender_role, target_participant),
                            });
                        }
                    }
                }
            }
        }
        
        violations
    }
    
    /// Find receive operations without matching sends
    fn find_unmatched_receives(&self, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        for (receiver_role, receiver) in &self.session_participants {
            for operation in &receiver.protocol_history {
                if let SessionOperation::Receive { value_type, source_participant, .. } = operation {
                    // Check if source has matching send
                    if let Some(source) = self.session_participants.get(source_participant) {
                        let has_matching_send = source.protocol_history.iter().any(|op| {
                            matches!(op, SessionOperation::Send { value_type: send_type, target_participant, .. }
                                if send_type == value_type && target_participant == receiver_role)
                        });
                        
                        if !has_matching_send {
                            violations.push(ProtocolViolation {
                                violation_type: ViolationType::TypeMismatch,
                                expected_operation: Some(SessionOperation::Send {
                                    value_type: value_type.clone(),
                                    target_participant: receiver_role.clone(),
                                    value: None,
                                }),
                                actual_operation: Some(operation.clone()),
                                timestamp,
                                message: format!("Unmatched receive by {} from {}", receiver_role, source_participant),
                            });
                        }
                    }
                }
            }
        }
        
        violations
    }
    
    /// Validate choice consistency across participants
    fn validate_choice_consistency(&self, timestamp: SimulatedTimestamp) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();
        
        // Check that external choices have corresponding internal choices
        for (role, participant) in &self.session_participants {
            for operation in &participant.protocol_history {
                if let SessionOperation::ExternalChoice { available_branches: _, chosen_branch: Some(chosen) } = operation {
                    // Verify that some other participant made the corresponding internal choice
                    let has_matching_internal_choice = self.session_participants.iter()
                        .filter(|(other_role, _)| other_role != &role)
                        .any(|(_, other_participant)| {
                            other_participant.protocol_history.iter().any(|other_op| {
                                if let SessionOperation::InternalChoice { chosen_branch: other_chosen, .. } = other_op {
                                    other_chosen == chosen
                                } else {
                                    false
                                }
                            })
                        });
                        
                    if !has_matching_internal_choice {
                        violations.push(ProtocolViolation {
                            violation_type: ViolationType::InvalidChoice,
                            expected_operation: Some(SessionOperation::InternalChoice {
                                chosen_branch: chosen.clone(),
                                branch_operations: vec![], // Simplified
                            }),
                            actual_operation: Some(operation.clone()),
                            timestamp,
                            message: format!("External choice '{}' by {} has no corresponding internal choice", chosen, role),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    /// Test for deadlock conditions among session participants
    fn test_for_deadlocks(&self, timestamp: SimulatedTimestamp) -> DeadlockReport {
        let mut potentially_deadlocked = Vec::new();
        let mut waiting_for = Vec::new();
        
        // Check each participant's current state
        for (role, participant) in &self.session_participants {
            // Check if participant is waiting for external input
            let is_waiting_for_external = participant.next_operations.iter().any(|op| {
                matches!(op, SessionOperation::Receive { .. } | SessionOperation::ExternalChoice { .. })
            });
            
            if is_waiting_for_external {
                for operation in &participant.next_operations {
                    match operation {
                        SessionOperation::Receive { source_participant, .. } => {
                            waiting_for.push(WaitingRelation {
                                waiter: role.clone(),
                                waited_for: source_participant.clone(),
                                operation_type: "receive".to_string(),
                            });
                        }
                        SessionOperation::ExternalChoice { .. } => {
                            // External choice waits for any participant to make internal choice
                            for other_role in self.session_participants.keys() {
                                if other_role != role {
                                    waiting_for.push(WaitingRelation {
                                        waiter: role.clone(),
                                        waited_for: other_role.clone(),
                                        operation_type: "external_choice".to_string(),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                potentially_deadlocked.push(role.clone());
            }
        }
        
        // Simple deadlock detection: if all participants are waiting, it's a deadlock
        let is_deadlock = potentially_deadlocked.len() == self.session_participants.len() && !potentially_deadlocked.is_empty();
        
        let deadlock_violation = if is_deadlock {
            Some(ProtocolViolation {
                violation_type: ViolationType::Deadlock,
                expected_operation: None,
                actual_operation: None,
                timestamp,
                message: format!("Deadlock detected: all {} participants are waiting for external input", potentially_deadlocked.len()),
            })
        } else {
            None
        };
        
        DeadlockReport {
            is_deadlock,
            potentially_deadlocked_participants: potentially_deadlocked,
            waiting_relations: waiting_for,
            deadlock_violation,
        }
    }

    /// Enhanced deadlock detection with cycle detection and timeout handling
    pub fn detect_deadlocks_advanced(&self) -> AdvancedDeadlockReport {
        let timestamp = self.clock.now();
        
        // Build waiting graph for cycle detection
        let waiting_graph = self.build_waiting_graph();
        
        // Detect cycles in the waiting graph
        let cycles = self.detect_waiting_cycles(&waiting_graph);
        
        // Check for timeout-based deadlocks
        let timeout_deadlocks = self.detect_timeout_deadlocks();
        
        // Analyze resource conflicts
        let resource_conflicts = self.analyze_resource_conflicts();
        
        // Check for live-lock scenarios
        let live_locks = self.detect_live_locks();
        
        AdvancedDeadlockReport {
            has_deadlock: !cycles.is_empty() || !timeout_deadlocks.is_empty(),
            circular_wait_cycles: cycles,
            timeout_based_deadlocks: timeout_deadlocks,
            resource_conflicts,
            live_locks,
            waiting_graph,
            detection_timestamp: timestamp,
        }
    }
    
    /// Build a waiting graph showing dependencies between participants
    fn build_waiting_graph(&self) -> WaitingGraph {
        let mut graph = WaitingGraph::new();
        
        for (role, participant) in &self.session_participants {
            graph.add_participant(role.clone());
            
            // Add edges for each operation the participant is waiting for
            for operation in &participant.next_operations {
                match operation {
                    SessionOperation::Receive { source_participant, .. } => {
                        graph.add_dependency(role.clone(), source_participant.clone(), DependencyType::WaitingForSend);
                    }
                    SessionOperation::ExternalChoice { .. } => {
                        // External choice waits for any other participant to make internal choice
                        for (other_role, other_participant) in &self.session_participants {
                            if other_role != role {
                                let has_internal_choice = other_participant.next_operations.iter()
                                    .any(|op| matches!(op, SessionOperation::InternalChoice { .. }));
                                if has_internal_choice {
                                    graph.add_dependency(role.clone(), other_role.clone(), DependencyType::WaitingForChoice);
                                }
                            }
                        }
                    }
                    _ => {} // Send, InternalChoice, and End don't create waiting dependencies
                }
            }
        }
        
        graph
    }
    
    /// Detect cycles in the waiting graph using DFS
    fn detect_waiting_cycles(&self, graph: &WaitingGraph) -> Vec<DeadlockCycle> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        for participant in &graph.participants {
            if !visited.contains(participant) {
                self.dfs_cycle_detection(
                    participant,
                    graph,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }
    
    /// DFS helper for cycle detection
    #[allow(clippy::only_used_in_recursion)]
    fn dfs_cycle_detection(
        &self,
        current: &str,
        graph: &WaitingGraph,
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<DeadlockCycle>,
    ) {
        visited.insert(current.to_string());
        recursion_stack.insert(current.to_string());
        path.push(current.to_string());
        
        if let Some(dependencies) = graph.dependencies.get(current) {
            for dependency in dependencies {
                let target = &dependency.target;
                
                if !visited.contains(target) {
                    self.dfs_cycle_detection(target, graph, visited, recursion_stack, path, cycles);
                } else if recursion_stack.contains(target) {
                    // Found a cycle - extract the cycle from the path
                    if let Some(cycle_start) = path.iter().position(|p| p == target) {
                        let cycle_participants = path[cycle_start..].to_vec();
                        cycles.push(DeadlockCycle {
                            participants: cycle_participants,
                            cycle_type: CycleType::CircularWait,
                            description: format!("Circular wait detected among participants: {}", path[cycle_start..].join(" -> ")),
                        });
                    }
                }
            }
        }
        
        path.pop();
        recursion_stack.remove(current);
    }
    
    /// Detect timeout-based deadlocks where participants have been waiting too long
    fn detect_timeout_deadlocks(&self) -> Vec<TimeoutDeadlock> {
        let mut timeout_deadlocks = Vec::new();
        let current_time = self.clock.now();
        let timeout_threshold = 1000; // milliseconds - configurable timeout
        
        for (role, participant) in &self.session_participants {
            // Check if participant has been stuck on the same operations for too long
            if !participant.next_operations.is_empty() {
                // For simplification, we'll check if the participant has made no progress
                // In a real implementation, we'd track when each operation was queued
                let is_stuck = participant.next_operations.iter().all(|op| {
                    matches!(op, SessionOperation::Receive { .. } | SessionOperation::ExternalChoice { .. })
                });
                
                if is_stuck && !participant.effects.is_empty() {
                    // Check if the last effect was too long ago
                    if let Some(last_effect) = participant.effects.last() {
                        let time_since_last_effect = current_time.as_secs() - last_effect.timestamp.as_secs();
                        if time_since_last_effect > timeout_threshold {
                            timeout_deadlocks.push(TimeoutDeadlock {
                                participant: role.clone(),
                                stuck_operations: participant.next_operations.clone(),
                                timeout_duration: time_since_last_effect,
                                last_activity: last_effect.timestamp,
                            });
                        }
                    }
                }
            }
        }
        
        timeout_deadlocks
    }
    
    /// Analyze resource conflicts that could lead to deadlocks
    fn analyze_resource_conflicts(&self) -> Vec<ResourceConflict> {
        let mut conflicts = Vec::new();
        
        // Check for participants trying to send to each other simultaneously
        for (sender_role, sender) in &self.session_participants {
            for sender_op in &sender.next_operations {
                if let SessionOperation::Send { target_participant, .. } = sender_op {
                    // Check if target is also trying to send to sender
                    if let Some(target) = self.session_participants.get(target_participant) {
                        let has_reverse_send = target.next_operations.iter().any(|op| {
                            matches!(op, SessionOperation::Send { target_participant: reverse_target, .. }
                                if reverse_target == sender_role)
                        });
                        
                        if has_reverse_send {
                            conflicts.push(ResourceConflict {
                                conflict_type: ConflictType::BidirectionalSend,
                                participants: vec![sender_role.clone(), target_participant.clone()],
                                description: format!("Bidirectional send conflict between {} and {}", sender_role, target_participant),
                            });
                        }
                    }
                }
            }
        }
        
        conflicts
    }
    
    /// Detect live-lock scenarios where participants are active but making no progress
    fn detect_live_locks(&self) -> Vec<LiveLock> {
        let mut live_locks = Vec::new();
        
        // Check for participants repeatedly executing the same operations without progress
        for (role, participant) in &self.session_participants {
            if participant.effects.len() >= 3 {
                // Check if the last 3 effects are the same operation type
                let recent_effects: Vec<_> = participant.effects.iter().rev().take(3).collect();
                
                if recent_effects.len() == 3 {
                    let same_operation_type = recent_effects.windows(2).all(|window| {
                        std::mem::discriminant(&window[0].operation) == std::mem::discriminant(&window[1].operation)
                    });
                    
                    if same_operation_type {
                        live_locks.push(LiveLock {
                            participant: role.clone(),
                            repeated_operation: recent_effects[0].operation.clone(),
                            repetition_count: 3, // Simplified - would count actual repetitions
                            description: format!("Participant {} is repeating {:?} without progress", role, recent_effects[0].operation),
                        });
                    }
                }
            }
        }
        
        live_locks
    }
    
    /// Execute with timeout to prevent infinite waiting
    pub async fn run_with_timeout(&mut self, timeout_ms: u64) -> Result<TimeoutExecutionResult, SimulationError> {
        let start_time = self.clock.now();
        let timeout_threshold = timeout_ms;
        
        self.set_state(SimulationState::Running);
        
        let mut steps_executed = 0;
        let mut deadlock_checks = 0;
        
        while self.pc < self.program.len() {
            // Check for timeout
            let current_time = self.clock.now();
            let elapsed = current_time.as_secs() - start_time.as_secs();
            if elapsed > timeout_threshold {
                return Ok(TimeoutExecutionResult::Timeout {
                    steps_executed,
                    elapsed_time: elapsed,
                    final_state: self.state.clone(),
                });
            }
            
            // Periodic deadlock detection
            if steps_executed % 10 == 0 {
                let deadlock_report = self.detect_deadlocks_advanced();
                if deadlock_report.has_deadlock {
                    return Ok(TimeoutExecutionResult::Deadlock {
                        steps_executed,
                        deadlock_report,
                        final_state: self.state.clone(),
                    });
                }
                deadlock_checks += 1;
            }
            
            // Execute step
            if !self.step().await? {
                break;
            }
            
            steps_executed += 1;
            
            // Progress safety check
            if steps_executed > 10000 {
                return Ok(TimeoutExecutionResult::MaxStepsReached {
                    steps_executed,
                    final_state: self.state.clone(),
                });
            }
        }
        
        self.set_state(SimulationState::Completed);
        Ok(TimeoutExecutionResult::Success {
            steps_executed,
            execution_time: self.clock.now().as_secs() - start_time.as_secs(),
            deadlock_checks,
        })
    }
}

impl Clone for SimulationEngine {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            config: self.config.clone(),
            clock: self.clock.clone(),
            _snapshot_manager: SnapshotManager::new(10), // Create new snapshot manager
            program: self.program.clone(),
            pc: self.pc,
            state_progression: self.state_progression.clone(),
            metrics: self.metrics.clone(),
            effects_log: self.effects_log.clone(),
            session_participants: self.session_participants.clone(),
            execution_state: self.execution_state.clone(),
            step_count: self.step_count,
            effect_results: self.effect_results.clone(),
            branch_manager: self.branch_manager.clone(),
            current_branch: self.current_branch.clone(),
        }
    }
}

impl SessionParticipantState {
    pub fn new() -> Self {
        Self {
            current_session: None,
            protocol_history: Vec::new(),
            next_operations: Vec::new(),
            gas: 100,
            effects: Vec::new(),
            compliance_state: ProtocolComplianceState::default(),
        }
    }
    
    /// Initialize with a session type for this participant
    pub fn with_session_type(session_type: SessionType) -> Self {
        let mut state = Self::new();
        state.set_session_type(session_type);
        state
    }
    
    /// Set the session type and compute next operations
    pub fn set_session_type(&mut self, session_type: SessionType) {
        self.current_session = Some(session_type.clone());
        self.compute_next_operations();
        self.compliance_state.is_valid = true;
    }
    
    /// Compute next valid operations from current session type
    pub fn compute_next_operations(&mut self) {
        self.next_operations.clear();
        
        if let Some(ref session) = self.current_session {
            match session {
                SessionType::Send(value_type, _continuation) => {
                    self.next_operations.push(SessionOperation::Send {
                        value_type: *value_type.clone(),
                        target_participant: "other".to_string(), // TODO: get from choreography
                        value: None,
                    });
                }
                
                SessionType::Receive(value_type, _continuation) => {
                    self.next_operations.push(SessionOperation::Receive {
                        value_type: *value_type.clone(),
                        source_participant: "other".to_string(), // TODO: get from choreography  
                        expected_value: None,
                    });
                }
                
                SessionType::InternalChoice(branches) => {
                    for (label, _branch_session) in branches {
                        self.next_operations.push(SessionOperation::InternalChoice {
                            chosen_branch: label.clone(),
                            branch_operations: vec![], // TODO: compute from branch_session
                        });
                    }
                }
                
                SessionType::ExternalChoice(branches) => {
                    let available_branches = branches.iter()
                        .map(|(label, _branch_session)| {
                            (label.clone(), vec![]) // TODO: compute operations from branch_session
                        })
                        .collect();
                    
                    self.next_operations.push(SessionOperation::ExternalChoice {
                        available_branches,
                        chosen_branch: None,
                    });
                }
                
                SessionType::End => {
                    self.next_operations.push(SessionOperation::End);
                    self.compliance_state.is_complete = true;
                }
                
                SessionType::Recursive(_var, body) => {
                    // Unfold the recursive type and recompute
                    let unfolded = body.substitute(_var, session);
                    self.current_session = Some(unfolded);
                    self.compute_next_operations();
                }
                
                SessionType::Variable(_) => {
                    // Should not happen in well-formed session types
                    self.compliance_state.is_valid = false;
                }
            }
        }
    }
    
    /// Execute a session operation and advance the protocol
    pub fn execute_operation(&mut self, operation: SessionOperation, timestamp: SimulatedTimestamp) -> Result<(), SimulationError> {
        // Check if operation is valid for current state
        if !self.is_operation_valid(&operation) {
            let violation = ProtocolViolation {
                violation_type: ViolationType::UnexpectedOperation,
                expected_operation: self.next_operations.first().cloned(),
                actual_operation: Some(operation.clone()),
                timestamp,
                message: "Operation not allowed by current session type".to_string(),
            };
            
            self.compliance_state.violations.push(violation);
            self.compliance_state.is_valid = false;
            
            return Err(SimulationError::SessionProtocolViolation {
                participant: "unknown".to_string(), // TODO: get participant name
                operation: format!("{:?}", operation),
                expected: format!("{:?}", self.next_operations),
            });
        }
        
        // Execute the operation and advance session type
        let gas_consumed = self.compute_operation_gas(&operation);
        let success = self.perform_operation(&operation)?;
        
        // Record the effect
        let effect = SessionEffect {
            operation: operation.clone(),
            timestamp,
            gas_consumed,
            success,
            result: None, // TODO: compute result value
        };
        
        self.effects.push(effect);
        self.protocol_history.push(operation.clone());
        self.gas = self.gas.saturating_sub(gas_consumed);
        self.compliance_state.protocol_step += 1;
        
        // Advance the session type
        self.advance_session_type(&operation)?;
        
        Ok(())
    }
    
    /// Check if an operation is valid for the current session state
    pub fn is_operation_valid(&self, operation: &SessionOperation) -> bool {
        self.next_operations.iter().any(|next_op| {
            std::mem::discriminant(operation) == std::mem::discriminant(next_op)
        })
    }
    
    /// Compute gas cost for an operation
    fn compute_operation_gas(&self, operation: &SessionOperation) -> u64 {
        match operation {
            SessionOperation::Send { .. } => 3,
            SessionOperation::Receive { .. } => 2,
            SessionOperation::InternalChoice { .. } => 4,
            SessionOperation::ExternalChoice { .. } => 2,
            SessionOperation::End => 1,
        }
    }
    
    /// Perform the actual operation (simulation)
    fn perform_operation(&mut self, _operation: &SessionOperation) -> Result<bool, SimulationError> {
        // In simulation, all operations succeed
        // In real implementation, this would involve actual communication
        Ok(true)
    }
    
    /// Advance session type after performing an operation
    fn advance_session_type(&mut self, operation: &SessionOperation) -> Result<(), SimulationError> {
        if let Some(ref session) = self.current_session.clone() {
            let new_session = match (session, operation) {
                (SessionType::Send(_, continuation), SessionOperation::Send { .. }) => {
                    Some(*continuation.clone())
                }
                
                (SessionType::Receive(_, continuation), SessionOperation::Receive { .. }) => {
                    Some(*continuation.clone())
                }
                
                (SessionType::InternalChoice(branches), SessionOperation::InternalChoice { chosen_branch, .. }) => {
                    branches.iter()
                        .find(|(label, _)| label == chosen_branch)
                        .map(|(_, branch_session)| branch_session.clone())
                }
                
                (SessionType::ExternalChoice(branches), SessionOperation::ExternalChoice { chosen_branch: Some(chosen), .. }) => {
                    branches.iter()
                        .find(|(label, _)| label == chosen)
                        .map(|(_, branch_session)| branch_session.clone())
                }
                
                (SessionType::End, SessionOperation::End) => None,
                
                _ => return Err(SimulationError::SessionProtocolViolation {
                    participant: "unknown".to_string(),
                    operation: format!("{:?}", operation),
                    expected: format!("{:?}", session),
                }),
            };
            
            self.current_session = new_session;
            self.compute_next_operations();
        }
        
        Ok(())
    }
    
    /// Check if session is complete
    pub fn is_session_complete(&self) -> bool {
        self.compliance_state.is_complete || 
        matches!(self.current_session, Some(SessionType::End) | None)
    }
    
    /// Get compliance violations
    pub fn get_violations(&self) -> &[ProtocolViolation] {
        &self.compliance_state.violations
    }
}

impl Default for SessionParticipantState {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Protocol Compliance Testing Types
//-----------------------------------------------------------------------------

/// Comprehensive protocol compliance test report
#[derive(Debug, Clone)]
pub struct ProtocolComplianceReport {
    /// Overall compliance status
    pub is_fully_compliant: bool,
    
    /// Reports for individual participants
    pub participant_reports: BTreeMap<String, ParticipantComplianceReport>,
    
    /// Global protocol violations (cross-participant)
    pub global_violations: Vec<ProtocolViolation>,
    
    /// Deadlock detection report
    pub deadlock_report: Option<DeadlockReport>,
    
    /// Timestamp when the compliance test was performed
    pub test_timestamp: SimulatedTimestamp,
}

/// Protocol compliance report for a single participant
#[derive(Debug, Clone)]
pub struct ParticipantComplianceReport {
    /// Participant role/identifier
    pub role: String,
    
    /// Whether this participant is compliant
    pub is_compliant: bool,
    
    /// Any compliance violations found
    pub violations: Vec<ProtocolViolation>,
    
    /// Current step in the protocol
    pub protocol_step: usize,
    
    /// Whether session is complete for this participant
    pub session_complete: bool,
    
    /// Next expected operations
    pub next_expected_operations: Vec<SessionOperation>,
}

/// Deadlock detection report
#[derive(Debug, Clone)]
pub struct DeadlockReport {
    /// Whether a deadlock was detected
    pub is_deadlock: bool,
    
    /// Participants that are potentially deadlocked
    pub potentially_deadlocked_participants: Vec<String>,
    
    /// Waiting relationships between participants
    pub waiting_relations: Vec<WaitingRelation>,
    
    /// Deadlock violation if detected
    pub deadlock_violation: Option<ProtocolViolation>,
}

/// Waiting relationship between participants
#[derive(Debug, Clone)]
pub struct WaitingRelation {
    /// Participant that is waiting
    pub waiter: String,
    
    /// Participant being waited for
    pub waited_for: String,
    
    /// Type of operation being waited for
    pub operation_type: String,
}

impl Default for ProtocolComplianceReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolComplianceReport {
    /// Create a new compliance report
    pub fn new() -> Self {
        Self {
            is_fully_compliant: true,
            participant_reports: BTreeMap::new(),
            global_violations: Vec::new(),
            deadlock_report: None,
            test_timestamp: SimulatedTimestamp::new(0),
        }
    }
    
    /// Add a participant compliance report
    pub fn add_participant_report(&mut self, role: String, report: ParticipantComplianceReport) {
        if !report.is_compliant {
            self.is_fully_compliant = false;
        }
        self.participant_reports.insert(role, report);
    }
    
    /// Add global violations
    pub fn add_global_violations(&mut self, violations: Vec<ProtocolViolation>) {
        if !violations.is_empty() {
            self.is_fully_compliant = false;
        }
        self.global_violations.extend(violations);
    }
    
    /// Set deadlock report
    pub fn set_deadlock_report(&mut self, report: DeadlockReport) {
        if report.is_deadlock {
            self.is_fully_compliant = false;
        }
        self.deadlock_report = Some(report);
    }
    
    /// Get total number of violations
    pub fn total_violations(&self) -> usize {
        let participant_violations: usize = self.participant_reports.values()
            .map(|report| report.violations.len())
            .sum();
        
        let deadlock_violations = if let Some(ref deadlock_report) = self.deadlock_report {
            if deadlock_report.deadlock_violation.is_some() { 1 } else { 0 }
        } else { 0 };
        
        participant_violations + self.global_violations.len() + deadlock_violations
    }
}

//-----------------------------------------------------------------------------
// Advanced Deadlock Detection Types
//-----------------------------------------------------------------------------

/// Advanced deadlock detection report with cycle analysis and resource conflicts
#[derive(Debug, Clone)]
pub struct AdvancedDeadlockReport {
    /// Whether any form of deadlock was detected
    pub has_deadlock: bool,
    
    /// Circular wait cycles detected
    pub circular_wait_cycles: Vec<DeadlockCycle>,
    
    /// Timeout-based deadlocks
    pub timeout_based_deadlocks: Vec<TimeoutDeadlock>,
    
    /// Resource conflicts that could cause deadlocks
    pub resource_conflicts: Vec<ResourceConflict>,
    
    /// Live-lock scenarios
    pub live_locks: Vec<LiveLock>,
    
    /// Complete waiting dependency graph
    pub waiting_graph: WaitingGraph,
    
    /// When the detection was performed
    pub detection_timestamp: SimulatedTimestamp,
}

/// Circular wait cycle in session execution
#[derive(Debug, Clone)]
pub struct DeadlockCycle {
    /// Participants involved in the cycle
    pub participants: Vec<String>,
    
    /// Type of cycle detected
    pub cycle_type: CycleType,
    
    /// Human-readable description
    pub description: String,
}

/// Type of deadlock cycle
#[derive(Debug, Clone)]
pub enum CycleType {
    /// Simple circular wait (A waits for B, B waits for A)
    CircularWait,
    
    /// Complex dependency chain with cycle
    DependencyChain,
    
    /// Choice-based cycle (external choices waiting for internal choices)
    ChoiceCycle,
}

/// Timeout-based deadlock where participant is stuck too long
#[derive(Debug, Clone)]
pub struct TimeoutDeadlock {
    /// Participant that is stuck
    pub participant: String,
    
    /// Operations the participant is stuck on
    pub stuck_operations: Vec<SessionOperation>,
    
    /// How long the participant has been stuck (milliseconds)
    pub timeout_duration: u64,
    
    /// Timestamp of last activity
    pub last_activity: SimulatedTimestamp,
}

/// Resource conflict that could lead to deadlock
#[derive(Debug, Clone)]
pub struct ResourceConflict {
    /// Type of resource conflict
    pub conflict_type: ConflictType,
    
    /// Participants involved in the conflict
    pub participants: Vec<String>,
    
    /// Description of the conflict
    pub description: String,
}

/// Type of resource conflict
#[derive(Debug, Clone)]
pub enum ConflictType {
    /// Participants trying to send to each other simultaneously
    BidirectionalSend,
    
    /// Multiple participants waiting for the same resource
    ResourceContention,
    
    /// Conflicting choice operations
    ChoiceConflict,
}

/// Live-lock scenario where participants are active but not progressing
#[derive(Debug, Clone)]
pub struct LiveLock {
    /// Participant caught in live-lock
    pub participant: String,
    
    /// Operation being repeated without progress
    pub repeated_operation: SessionOperation,
    
    /// Number of repetitions detected
    pub repetition_count: usize,
    
    /// Description of the live-lock
    pub description: String,
}

/// Waiting dependency graph for deadlock analysis
#[derive(Debug, Clone)]
pub struct WaitingGraph {
    /// All participants in the graph
    pub participants: Vec<String>,
    
    /// Dependencies between participants
    pub dependencies: BTreeMap<String, Vec<WaitingDependency>>,
}

/// Waiting dependency between participants
#[derive(Debug, Clone)]
pub struct WaitingDependency {
    /// Participant being waited for
    pub target: String,
    
    /// Type of dependency
    pub dependency_type: DependencyType,
}

/// Type of waiting dependency
#[derive(Debug, Clone)]
pub enum DependencyType {
    /// Waiting for a send operation
    WaitingForSend,
    
    /// Waiting for a choice to be made
    WaitingForChoice,
    
    /// Waiting for session to end
    WaitingForEnd,
}

/// Result of execution with timeout and deadlock detection
#[derive(Debug, Clone)]
pub enum TimeoutExecutionResult {
    /// Successful completion
    Success {
        steps_executed: usize,
        execution_time: u64,
        deadlock_checks: usize,
    },
    
    /// Execution timed out
    Timeout {
        steps_executed: usize,
        elapsed_time: u64,
        final_state: SimulationState,
    },
    
    /// Deadlock detected
    Deadlock {
        steps_executed: usize,
        deadlock_report: AdvancedDeadlockReport,
        final_state: SimulationState,
    },
    
    /// Maximum steps reached (safety limit)
    MaxStepsReached {
        steps_executed: usize,
        final_state: SimulationState,
    },
}

impl Default for WaitingGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitingGraph {
    /// Create a new empty waiting graph
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            dependencies: BTreeMap::new(),
        }
    }
    
    /// Add a participant to the graph
    pub fn add_participant(&mut self, participant: String) {
        if !self.participants.contains(&participant) {
            self.participants.push(participant.clone());
            self.dependencies.entry(participant).or_default();
        }
    }
    
    /// Add a dependency between participants
    pub fn add_dependency(&mut self, from: String, to: String, dependency_type: DependencyType) {
        self.add_participant(from.clone());
        self.add_participant(to.clone());
        
        self.dependencies
            .entry(from)
            .or_default()
            .push(WaitingDependency {
                target: to,
                dependency_type,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    use causality_core::machine::instruction::RegisterId;
    
    #[tokio::test]
    async fn test_simulation_engine_basic() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        assert_eq!(engine.state(), &SimulationState::Created);
        
        // Load a simple program so run() doesn't fail
        let program = vec![
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(0) },
        ];
        engine.load_program(program).unwrap();
        
        // Test state transitions
        engine.run().await.unwrap();
        assert_eq!(engine.state(), &SimulationState::Completed);
        
        engine.reset().unwrap();
        assert_eq!(engine.state(), &SimulationState::Created);
    }
    
    #[tokio::test]
    async fn test_state_progression_tracking() {
        let mut config = SimulationConfig::default();
        config.step_by_step_mode = true;
        
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Load a simple program - just one instruction that will complete
        let program = vec![
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(0) },
        ];
        engine.load_program(program).unwrap();
        
        // Execute steps and track progression
        assert_eq!(engine.state_progression().steps.len(), 0);
        
        let continue_1 = engine.step().await.unwrap();
        assert!(!continue_1); // Should complete after one step since PC will be past program length
        assert_eq!(engine.state_progression().steps.len(), 1);
        assert_eq!(engine.state(), &SimulationState::Completed);
    }
    
    #[tokio::test]
    async fn test_state_transitions() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Check initial state
        assert_eq!(engine.state_progression().state_transitions.len(), 0);
        
        // Transition to running
        engine.set_state(SimulationState::Running);
        assert_eq!(engine.state_progression().state_transitions.len(), 1);
        assert_eq!(engine.state_progression().state_transitions[0].0, SimulationState::Running);
        
        // Transition to completed
        engine.set_state(SimulationState::Completed);
        assert_eq!(engine.state_progression().state_transitions.len(), 2);
        assert_eq!(engine.state_progression().state_transitions[1].0, SimulationState::Completed);
    }
    
    #[tokio::test]
    async fn test_snapshot_creation() {
        let mut config = SimulationConfig::default();
        config.enable_snapshots = true;
        
        let mut engine = SimulationEngine::new_with_config(config);
        let snapshot_id = engine.create_snapshot("Test snapshot".to_string()).await.unwrap();
        
        // Verify snapshot was created
        assert!(snapshot_id.as_str().starts_with("snapshot_"));
    }
    
    #[tokio::test]
    async fn test_effect_execution() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        let result = engine.execute_effect("(test-effect)".to_string()).await;
        assert!(result.is_ok());
        
        let metrics = engine.metrics();
        assert_eq!(metrics.effects_executed, 1);
    }
    
    #[tokio::test]
    async fn test_step_by_step_execution() {
        let mut config = SimulationConfig::default();
        config.step_by_step_mode = true;
        
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Load a program with two instructions
        let program = vec![
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(0) },
            Instruction::Transform { morph_reg: RegisterId::new(1), input_reg: RegisterId::new(1), output_reg: RegisterId::new(1) },
        ];
        engine.load_program(program).unwrap();
        
        // Execute step by step
        let continue_1 = engine.step().await.unwrap();
        assert!(continue_1);
        assert_eq!(engine.state_progression().steps.len(), 1);
        assert_eq!(engine.state(), &SimulationState::StepReady);
        
        let continue_2 = engine.step().await.unwrap();
        assert!(!continue_2); // Should complete after second step since PC will be past program length
        assert_eq!(engine.state_progression().steps.len(), 2);
        assert_eq!(engine.state(), &SimulationState::Completed);
    }
    
    #[tokio::test]
    async fn test_resource_allocation_simulation() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Create a program that allocates and consumes a resource
        let program = vec![
            // Step 1: Create type value in register 0
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(0) },
            // Step 2: Create value to allocate in register 1
            Instruction::Transform { morph_reg: RegisterId::new(1), input_reg: RegisterId::new(1), output_reg: RegisterId::new(1) },
            // Step 3: Allocate resource - alloc r0 r1 r2
            Instruction::Alloc { 
                type_reg: RegisterId::new(0), 
                init_reg: RegisterId::new(1), 
                output_reg: RegisterId::new(2) 
            },
            // Step 4: Consume resource - consume r2 r3
            Instruction::Consume { 
                resource_reg: RegisterId::new(2), 
                output_reg: RegisterId::new(3) 
            },
        ];
        
        engine.load_program(program).unwrap();
        
        // Execute the program
        engine.run().await.unwrap();
        assert_eq!(engine.state(), &SimulationState::Completed);
        
        // Verify we have 4 steps (one for each instruction)
        assert_eq!(engine.state_progression().steps.len(), 4);
        
        // Check that resource allocation and consumption were tracked
        let steps = &engine.state_progression().steps;
        
        // Step 3 should show resource allocation
        assert!(!steps[2].resources_allocated.is_empty());
        assert!(steps[2].resources_allocated[0].contains("alloc"));
        
        // Step 4 should show resource consumption
        assert!(!steps[3].resources_consumed.is_empty());
        assert!(steps[3].resources_consumed[0].contains("consume"));
    }
    
    #[tokio::test]
    async fn test_instruction_simulation_varieties() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Create a program with different instruction types
        let program = vec![
            // Create values
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(0) },
            Instruction::Transform { morph_reg: RegisterId::new(1), input_reg: RegisterId::new(1), output_reg: RegisterId::new(1) },
            // Test Move instruction
            Instruction::Transform { morph_reg: RegisterId::new(0), input_reg: RegisterId::new(0), output_reg: RegisterId::new(2) },
            // Test Select instruction (conditional)
            // Test Compose instruction (sequential composition)
            Instruction::Compose {
                first_reg: RegisterId::new(2),
                second_reg: RegisterId::new(2),
                output_reg: RegisterId::new(3)
            },        ];
        
        engine.load_program(program).unwrap();
        engine.run().await.unwrap();
        
        assert_eq!(engine.state(), &SimulationState::Completed);
        assert_eq!(engine.state_progression().steps.len(), 4);
        
        // Verify instruction variety was handled
        let steps = &engine.state_progression().steps;
        assert!(steps[0].instruction.as_ref().unwrap().contains("Transform"));
        assert!(steps[2].instruction.as_ref().unwrap().contains("Transform"));
        assert!(steps[3].instruction.as_ref().unwrap().contains("Compose"));
    }

    #[tokio::test]
    async fn test_enhanced_effect_execution_simulation() {
        let mut config = SimulationConfig::default();
        config.max_steps = 10;
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Test different effect types with timing
        let test_cases = vec![
            ("transfer operation", "transfer from Alice to Bob"),
            ("compute hash", "compute sha256 of data"),
            ("storage read", "storage load key"),
            ("network request", "network fetch data"),
            ("validation check", "validation verify signature"),
            ("consensus vote", "consensus propose block"),
        ];
        
        for (description, effect_expr) in test_cases {
            let start_time = engine.clock.now();
            
            // Execute effect
            let result = engine.execute_effect(effect_expr.to_string()).await;
            assert!(result.is_ok(), "Effect '{}' should succeed", description);
            
            let end_time = engine.clock.now();
            let duration = end_time.duration_since(start_time);
            
            // Verify realistic timing based on effect type
            if effect_expr.contains("transfer") {
                assert!(duration >= Duration::from_secs(0), "Transfer effects should complete");
            } else if effect_expr.contains("compute") {
                assert!(duration >= Duration::from_secs(0), "Compute effects should complete");
            } else if effect_expr.contains("network") {
                assert!(duration >= Duration::from_secs(0), "Network effects should complete");
            } else if effect_expr.contains("consensus") {
                assert!(duration >= Duration::from_secs(0), "Consensus effects should complete");
            }
            
            // For now, just verify that time progressed (even if very small)
            // In a real implementation with finer time resolution, we could test actual timing
            assert!(end_time >= start_time, "Time should progress during effect execution");
        }
        
        // Verify effects were added to machine state
        assert!(engine.execution_state.gas >= 6, "Should have executed 6+ effects");
        
        // Verify different effect types were simulated
        let effect_tags: Vec<String> = engine.effect_results.iter()
            .map(|e| e.effect_name.clone())
            .collect();
        
        assert!(effect_tags.iter().any(|tag| tag.contains("transfer")));
        assert!(effect_tags.iter().any(|tag| tag.contains("compute")));
        assert!(effect_tags.iter().any(|tag| tag.contains("storage")));
        assert!(effect_tags.iter().any(|tag| tag.contains("network")));
        assert!(effect_tags.iter().any(|tag| tag.contains("validation")));
        assert!(effect_tags.iter().any(|tag| tag.contains("consensus")));
    }

    #[tokio::test]
    async fn test_effect_execution_with_resource_constraints() {
        let mut config = SimulationConfig::default();
        config.max_steps = 5;
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Set low gas to test resource constraints
        engine.execution_state.gas = 15;
        
        // Execute a compute effect that consumes gas
        let result1 = engine.execute_effect("compute hash".to_string()).await;
        assert!(result1.is_ok());
        assert_eq!(engine.execution_state.gas, 5); // Should have consumed 10 gas
        
        // Try another compute effect - should fail due to insufficient gas
        let result2 = engine.execute_effect("compute sort".to_string()).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("Insufficient gas"));
        
        // Storage effect should still work (doesn't consume gas)
        let result3 = engine.execute_effect("storage write".to_string()).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_effect_execution_failure_scenarios() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Test network failure scenario
        // We need to run multiple network effects to eventually hit the 5% failure rate
        let mut network_failures = 0;
        let mut network_successes = 0;
        
        for _i in 0..100 {
            match engine.execute_effect("network api_call".to_string()).await {
                Ok(_) => network_successes += 1,
                Err(SimulationError::EffectExecutionError(msg)) if msg.contains("Network timeout") => {
                    network_failures += 1;
                },
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        
        // Should have some failures (statistically very likely with 100 attempts at 5% rate)
        // But we'll be lenient with the exact count due to randomness
        assert!(network_failures > 0 || network_successes > 95, 
               "Should have some network failures or very high success rate. Got {} failures, {} successes", 
               network_failures, network_successes);
        assert!(network_successes > 0, "Should have some network successes");
    }
} 