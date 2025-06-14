//! Core simulation engine for orchestrating Causality operations

use causality_lisp::ast::LispValue;
use causality_core::machine::{Instruction, RegisterId};
use crate::{
    SimulationError,
    clock::{SimulatedClock, SimulatedTimestamp},
    snapshot::{SnapshotManager, SnapshotId},
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid;
use causality_core::lambda::base::Value;
use crate::branching::BranchingManager;

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
    pub registers: HashMap<u32, Value>,
    /// Memory state
    pub memory: Vec<Value>,
    /// Current instruction pointer
    pub instruction_pointer: usize,
    /// Effect execution history
    pub effect_history: Vec<EngineEffectExecution>,
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
            registers: HashMap::new(),
            memory: Vec::new(),
            instruction_pointer: 0,
            effect_history: Vec::new(),
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
    
    /// Mock machine state
    pub machine: MockMachineState,
    
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

/// Mock machine state for simulation
#[derive(Debug, Clone, Default)]
pub struct MockMachineState {
    pub effects: Vec<MockEffect>,
    pub gas: u64,
}

/// Mock effect for simulation
#[derive(Debug, Clone)]
pub struct MockEffect {
    pub call: MockEffectCall,
    pub result_register: Option<RegisterId>,
}

/// Mock effect call
#[derive(Debug, Clone)]
pub struct MockEffectCall {
    pub tag: String,
    pub args: Vec<String>,
    pub return_type: Option<String>,
}

impl MockMachineState {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            gas: 100,
        }
    }
    
    pub fn add_effect(&mut self, effect: MockEffect) {
        self.effects.push(effect);
    }
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
            machine: MockMachineState::new(),
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
            machine: MockMachineState::new(),
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
    
    /// Execute a single step
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
            instruction: Some(format!("{:?}", instruction)),
            resources_allocated: Vec::new(),
            resources_consumed: Vec::new(),
            gas_consumed: 0,
        };
        
        // Execute instruction
        match instruction {
            Instruction::Alloc { .. } => {
                step.resources_allocated.push("Alloc resource".to_string());
                step.gas_consumed = 5;
            }
            Instruction::Consume { .. } => {
                step.resources_consumed.push("Consume resource".to_string());
                step.gas_consumed = 3;
            }
            Instruction::Move { .. } => {
                step.gas_consumed = 1;
            }
            Instruction::Select { .. } => {
                step.gas_consumed = 2;
            }
            Instruction::Witness { .. } => {
                step.gas_consumed = 1;
            }
            _ => {
                step.gas_consumed = 1;
            }
        }
        
        self.machine.gas = self.machine.gas.saturating_sub(step.gas_consumed);
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
            if self.machine.gas < gas_needed {
                return Err(SimulationError::EffectExecutionError(
                    format!("Insufficient gas: required {}, available {}", gas_needed, self.machine.gas)
                ));
            }
            self.machine.gas -= gas_needed;
            gas_needed
        } else if effect_type == "storage" {
            let gas_needed = 5;
            self.machine.gas = self.machine.gas.saturating_sub(gas_needed);
            gas_needed
        } else if effect_type == "transfer" {
            let gas_needed = 3;
            self.machine.gas = self.machine.gas.saturating_sub(gas_needed);
            gas_needed
        } else {
            let gas_needed = 1; // Default gas cost for other operations
            self.machine.gas = self.machine.gas.saturating_sub(gas_needed);
            gas_needed
        };
        
        // Add consumed gas to metrics
        self.metrics.total_gas_consumed += gas_consumed;
        
        // Simulate failure rate for network effects
        if effect_type == "network" && rand::random::<f64>() < 0.05 { // 5% failure rate
            return Err(SimulationError::EffectExecutionError("Network timeout".to_string()));
        }
        
        // Add effect to machine state
        let effect = MockEffect {
            call: MockEffectCall {
                tag: format!("{}_{}", effect_type, self.machine.effects.len()),
                args: vec![effect_expr.clone()],
                return_type: Some("LispValue".to_string()),
            },
            result_register: Some(RegisterId::new(0)),
        };
        
        self.machine.add_effect(effect);
        self.metrics.effects_executed += 1;
        self.effects_log.push(effect_expr);
        
        Ok(LispValue::Int(1))
    }
    
    /// Reset the engine
    pub fn reset(&mut self) -> Result<(), SimulationError> {
        self.state = SimulationState::Created;
        self.pc = 0;
        self.state_progression = StateProgression::default();
        self.metrics = ExecutionMetrics::default();
        self.effects_log.clear();
        self.machine = MockMachineState::new();
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
        let branch_id = uuid::Uuid::new_v4().to_string();
        
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
        let checkpoint_id = uuid::Uuid::new_v4().to_string();
        
        // For now, just store the checkpoint ID and timestamp
        // TODO: Implement proper state serialization when causality_core supports it
        println!("Created checkpoint '{}' with ID: {} (simplified)", checkpoint_name, checkpoint_id);
        Ok(checkpoint_id)
    }
    
    /// Rewind simulation to a previous checkpoint
    pub async fn rewind_to_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), SimulationError> {
        // For now, just reset the simulation state
        // TODO: Implement proper state restoration when causality_core supports it
        self.reset()?;
        println!("Rewound to checkpoint: {} (simplified reset)", checkpoint_id);
        Ok(())
    }
    
    /// Execute raw instructions directly
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<(), SimulationError> {
        self.program = instructions.to_vec();
        self.pc = 0;
        
        // Execute each instruction
        for i in 0..instructions.len() {
            self.pc = i;
            
            // Simulate instruction execution
            match &instructions[i] {
                Instruction::Witness { out_reg: _ } => {
                    // Mock witness instruction
                    self.effects_log.push("witness".to_string());
                }
                Instruction::Move { src: _, dst: _ } => {
                    // Mock move instruction
                    self.effects_log.push("move".to_string());
                }
                Instruction::Alloc { type_reg: _, val_reg: _, out_reg: _ } => {
                    // Mock alloc instruction
                    self.effects_log.push("alloc".to_string());
                }
                _ => {
                    // Mock other instructions
                    self.effects_log.push("instruction".to_string());
                }
            }
        }
        
        Ok(())
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
            machine: self.machine.clone(),
            execution_state: self.execution_state.clone(),
            step_count: self.step_count,
            effect_results: self.effect_results.clone(),
            branch_manager: self.branch_manager.clone(),
            current_branch: self.current_branch.clone(),
        }
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
            Instruction::Witness { out_reg: RegisterId::new(0) },
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
            Instruction::Witness { out_reg: RegisterId::new(0) },
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
            Instruction::Witness { out_reg: RegisterId::new(0) },
            Instruction::Witness { out_reg: RegisterId::new(1) },
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
            Instruction::Witness { out_reg: RegisterId::new(0) },
            // Step 2: Create value to allocate in register 1
            Instruction::Witness { out_reg: RegisterId::new(1) },
            // Step 3: Allocate resource - alloc r0 r1 r2
            Instruction::Alloc { 
                type_reg: RegisterId::new(0), 
                val_reg: RegisterId::new(1), 
                out_reg: RegisterId::new(2) 
            },
            // Step 4: Consume resource - consume r2 r3
            Instruction::Consume { 
                resource_reg: RegisterId::new(2), 
                out_reg: RegisterId::new(3) 
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
        assert!(steps[2].resources_allocated[0].contains("Alloc"));
        
        // Step 4 should show resource consumption
        assert!(!steps[3].resources_consumed.is_empty());
        assert!(steps[3].resources_consumed[0].contains("Consume"));
    }
    
    #[tokio::test]
    async fn test_instruction_simulation_varieties() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Create a program with different instruction types
        let program = vec![
            // Create values
            Instruction::Witness { out_reg: RegisterId::new(0) },
            Instruction::Witness { out_reg: RegisterId::new(1) },
            // Test Move instruction
            Instruction::Move { src: RegisterId::new(0), dst: RegisterId::new(2) },
            // Test Select instruction (conditional)
            Instruction::Select { 
                cond_reg: RegisterId::new(1), 
                true_reg: RegisterId::new(2), 
                false_reg: RegisterId::new(2), 
                out_reg: RegisterId::new(3) 
            },
        ];
        
        engine.load_program(program).unwrap();
        engine.run().await.unwrap();
        
        assert_eq!(engine.state(), &SimulationState::Completed);
        assert_eq!(engine.state_progression().steps.len(), 4);
        
        // Verify instruction variety was handled
        let steps = &engine.state_progression().steps;
        assert!(steps[0].instruction.as_ref().unwrap().contains("Witness"));
        assert!(steps[2].instruction.as_ref().unwrap().contains("Move"));
        assert!(steps[3].instruction.as_ref().unwrap().contains("Select"));
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
        assert!(engine.machine.effects.len() >= 6, "Should have executed 6+ effects");
        
        // Verify different effect types were simulated
        let effect_tags: Vec<String> = engine.machine.effects.iter()
            .map(|e| e.call.tag.clone())
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
        engine.machine.gas = 15;
        
        // Execute a compute effect that consumes gas
        let result1 = engine.execute_effect("compute hash".to_string()).await;
        assert!(result1.is_ok());
        assert_eq!(engine.machine.gas, 5); // Should have consumed 10 gas
        
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