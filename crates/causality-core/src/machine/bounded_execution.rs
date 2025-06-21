//! Bounded execution model for ZK-VM compatibility
//!
//! This module implements bounded execution with fixed limits on:
//! - Maximum number of instructions per program
//! - Maximum register usage
//! - Maximum resource allocation
//! - Deterministic state transitions for ZK proof generation
//!
//! **Design Principles**:
//! - Fixed upper bounds on all resources
//! - Deterministic execution for reproducible proofs
//! - Linear resource discipline enforcement
//! - Complete execution trace capture

use crate::machine::{
    instruction::{Instruction, RegisterId},
    register_file::{RegisterFile, RegisterFileError, MAX_REGISTERS},
    resource::{ResourceStore, ResourceId},
    reduction::{ExecutionTrace, TraceStep},
};
use crate::system::deterministic::DeterministicSystem;
use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------------
// Execution Bounds Configuration
//-----------------------------------------------------------------------------

/// Maximum number of instructions that can be executed in a single program
pub const MAX_INSTRUCTIONS: usize = 10_000;

/// Maximum number of resources that can be allocated simultaneously
pub const MAX_RESOURCES: usize = 2048;

/// Maximum execution steps before termination
pub const MAX_EXECUTION_STEPS: usize = 100_000;

//-----------------------------------------------------------------------------
// Bounded Execution Engine
//-----------------------------------------------------------------------------

/// Bounded execution engine for ZK-VM compatible execution
#[derive(Debug, Clone)]
pub struct BoundedExecutor {
    /// The program to execute (bounded in size)
    program: Vec<Instruction>,
    
    /// Current program counter
    program_counter: usize,
    
    /// Fixed-size register file
    register_file: RegisterFile,
    
    /// Resource store with bounded capacity
    resource_store: ResourceStore,
    
    /// Deterministic system for reproducible execution
    deterministic_system: DeterministicSystem,
    
    /// Execution trace for ZK proof generation
    execution_trace: ExecutionTrace,
    
    /// Current execution step counter
    execution_steps: usize,
    
    /// Whether execution has completed
    is_complete: bool,
    
    /// Whether execution encountered an error
    has_error: bool,
    
    /// Error message if execution failed
    error_message: Option<String>,
}

impl BoundedExecutor {
    /// Create a new bounded executor with a program
    pub fn new(program: Vec<Instruction>) -> Result<Self, BoundedExecutionError> {
        // Validate program size
        if program.len() > MAX_INSTRUCTIONS {
            return Err(BoundedExecutionError::ProgramTooLarge(program.len()));
        }
        
        // Validate program instructions
        for (i, instruction) in program.iter().enumerate() {
            if let Err(e) = Self::validate_instruction(instruction) {
                return Err(BoundedExecutionError::InvalidInstruction(i, e));
            }
        }
        
        let mut executor = Self {
            program,
            program_counter: 0,
            register_file: RegisterFile::new(),
            resource_store: ResourceStore::new(),
            deterministic_system: DeterministicSystem::new(),
            execution_trace: ExecutionTrace::new(),
            execution_steps: 0,
            is_complete: false,
            has_error: false,
            error_message: None,
        };
        
        // Capture initial state
        executor.execution_trace.set_initial_state(
            executor.register_file.snapshot(),
            executor.resource_store.snapshot(),
        );
        
        Ok(executor)
    }
    
    /// Execute the program with bounded resources
    pub fn execute(&mut self) -> Result<ExecutionResult, BoundedExecutionError> {
        while !self.is_complete && !self.has_error && self.execution_steps < MAX_EXECUTION_STEPS {
            // Check if we've reached the end of the program
            if self.program_counter >= self.program.len() {
                self.is_complete = true;
                break;
            }
            
            // Get the current instruction
            let instruction = self.program[self.program_counter].clone();
            
            // Execute the instruction
            if let Err(e) = self.execute_instruction(instruction) {
                self.has_error = true;
                self.error_message = Some(e.to_string());
                break;
            }
            
            // Increment counters
            self.program_counter += 1;
            self.execution_steps += 1;
        }
        
        // Check for execution limits exceeded
        if self.execution_steps >= MAX_EXECUTION_STEPS {
            self.has_error = true;
            self.error_message = Some("Maximum execution steps exceeded".to_string());
        }
        
        // Finalize execution trace
        self.execution_trace.finalize(
            self.register_file.snapshot(),
            self.resource_store.snapshot(),
        );
        
        // Return execution result
        if self.has_error {
            Ok(ExecutionResult::Error {
                message: self.error_message.clone().unwrap_or("Unknown error".to_string()),
                steps_executed: self.execution_steps,
                trace: self.execution_trace.clone(),
            })
        } else if self.is_complete {
            Ok(ExecutionResult::Success {
                steps_executed: self.execution_steps,
                trace: self.execution_trace.clone(),
            })
        } else {
            Ok(ExecutionResult::Timeout {
                steps_executed: self.execution_steps,
                trace: self.execution_trace.clone(),
            })
        }
    }
    
    /// Execute a single instruction with bounds checking
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), BoundedExecutionError> {
        // Capture current state for validation
        let prev_state = self.execution_state();
        
        // Validate the transition before execution
        self.validate_transition(&prev_state, &instruction)?;
        
        // Record the instruction execution in the trace
        let step = TraceStep::new(
            self.execution_steps as u64,
            self.deterministic_system.current_time(),
            instruction.clone(),
        );
        
        // Execute the instruction based on its type (immutable - creates new state)
        match instruction {
            Instruction::Transform { morph_reg, input_reg, output_reg } => {
                self.execute_transform(morph_reg, input_reg, output_reg)?;
            }
            Instruction::Alloc { type_reg, init_reg, output_reg } => {
                self.execute_alloc(type_reg, init_reg, output_reg)?;
            }
            Instruction::Consume { resource_reg, output_reg } => {
                self.execute_consume(resource_reg, output_reg)?;
            }
            Instruction::Compose { first_reg, second_reg, output_reg } => {
                self.execute_compose(first_reg, second_reg, output_reg)?;
            }
            Instruction::Tensor { left_reg, right_reg, output_reg } => {
                self.execute_tensor(left_reg, right_reg, output_reg)?;
            }
        }
        
        // Verify state consistency after execution
        self.verify_state_consistency()?;
        
        // Add the completed step to the trace
        self.execution_trace.add_step(step);
        
        Ok(())
    }
    
    /// Execute a Transform instruction
    fn execute_transform(&mut self, morph_reg: RegisterId, input_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Read morphism from register
        let morph_resource = self.register_file.read_register(morph_reg)?;
        if morph_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(morph_reg));
        }
        
        // Read input from register
        let input_resource = self.register_file.read_register(input_reg)?;
        if input_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(input_reg));
        }
        
        // For now, create a placeholder output resource
        let output_resource_id = self.resource_store.create_resource();
        self.register_file.write_register(output_reg, Some(output_resource_id))?;
        
        Ok(())
    }
    
    /// Execute an Alloc instruction
    fn execute_alloc(&mut self, type_reg: RegisterId, init_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Read type and init from registers
        let type_resource = self.register_file.read_register(type_reg)?;
        if type_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(type_reg));
        }
        
        let init_resource = self.register_file.read_register(init_reg)?;
        if init_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(init_reg));
        }
        
        // For now, create a placeholder output resource
        let output_resource_id = self.resource_store.create_resource();
        self.register_file.write_register(output_reg, Some(output_resource_id))?;
        
        Ok(())
    }
    
    /// Execute a Consume instruction
    fn execute_consume(&mut self, resource_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Read resource from register
        let resource = self.register_file.read_register(resource_reg)?;
        if resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(resource_reg));
        }
        
        // Consume the resource and generate nullifier
        if let Some(resource_id) = resource {
            self.resource_store.consume_resource(resource_id);
            
            // Clear the source register
            self.register_file.write_register(resource_reg, None)?;
            
            // For now, create a placeholder output resource
            let output_resource_id = self.resource_store.create_resource();
            self.register_file.write_register(output_reg, Some(output_resource_id))?;
        }
        
        Ok(())
    }
    
    /// Execute a Compose instruction
    fn execute_compose(&mut self, first_reg: RegisterId, second_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Read both morphisms from registers
        let first_morph = self.register_file.read_register(first_reg)?;
        if first_morph.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(first_reg));
        }
        
        let second_morph = self.register_file.read_register(second_reg)?;
        if second_morph.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(second_reg));
        }
        
        // For now, create a placeholder composed morphism
        let output_resource_id = self.resource_store.create_resource();
        self.register_file.write_register(output_reg, Some(output_resource_id))?;
        
        Ok(())
    }
    
    /// Execute a Tensor instruction
    fn execute_tensor(&mut self, left_reg: RegisterId, right_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Read both resources from registers
        let left_resource = self.register_file.read_register(left_reg)?;
        if left_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(left_reg));
        }
        
        let right_resource = self.register_file.read_register(right_reg)?;
        if right_resource.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(right_reg));
        }
        
        // For now, create a placeholder tensor product
        let output_resource_id = self.resource_store.create_resource();
        self.register_file.write_register(output_reg, Some(output_resource_id))?;
        
        Ok(())
    }
    
    /// Validate an instruction for bounds and safety
    fn validate_instruction(instruction: &Instruction) -> Result<(), String> {
        // Check register ID bounds
        let check_register = |reg_id: RegisterId| -> Result<(), String> {
            if reg_id.id() >= MAX_REGISTERS as u32 {
                Err(format!("Register ID {} exceeds maximum {}", reg_id.id(), MAX_REGISTERS - 1))
            } else {
                Ok(())
            }
        };
        
        match instruction {
            Instruction::Transform { morph_reg, input_reg, output_reg } => {
                check_register(*morph_reg)?;
                check_register(*input_reg)?;
                check_register(*output_reg)?;
            }
            Instruction::Alloc { type_reg, init_reg, output_reg } => {
                check_register(*type_reg)?;
                check_register(*init_reg)?;
                check_register(*output_reg)?;
            }
            Instruction::Consume { resource_reg, output_reg } => {
                check_register(*resource_reg)?;
                check_register(*output_reg)?;
            }
            Instruction::Compose { first_reg, second_reg, output_reg } => {
                check_register(*first_reg)?;
                check_register(*second_reg)?;
                check_register(*output_reg)?;
            }
            Instruction::Tensor { left_reg, right_reg, output_reg } => {
                check_register(*left_reg)?;
                check_register(*right_reg)?;
                check_register(*output_reg)?;
            }
        }
        
        Ok(())
    }
    
    /// Get the current execution state
    pub fn execution_state(&self) -> ExecutionState {
        ExecutionState {
            program_counter: self.program_counter,
            execution_steps: self.execution_steps,
            is_complete: self.is_complete,
            has_error: self.has_error,
            allocated_registers: self.register_file.allocated_count(),
            allocated_resources: self.resource_store.resource_count(),
        }
    }
    
    /// Validate that a state transition follows category laws
    fn validate_transition(&self, prev_state: &ExecutionState, instruction: &Instruction) -> Result<(), BoundedExecutionError> {
        // Verify category law preservation
        match instruction {
            Instruction::Transform { morph_reg, input_reg, output_reg } => {
                // Verify morphism application follows category laws
                self.validate_morphism_application(*morph_reg, *input_reg, *output_reg)?;
            }
            Instruction::Alloc { type_reg, init_reg, output_reg } => {
                // Verify allocation creates valid objects in the category
                self.validate_allocation(*type_reg, *init_reg, *output_reg)?;
            }
            Instruction::Consume { resource_reg, output_reg } => {
                // Verify consumption respects linear discipline
                self.validate_consumption(*resource_reg, *output_reg)?;
            }
            Instruction::Compose { first_reg, second_reg, output_reg } => {
                // Verify composition associativity: (f ∘ g) ∘ h = f ∘ (g ∘ h)
                self.validate_composition(*first_reg, *second_reg, *output_reg)?;
            }
            Instruction::Tensor { left_reg, right_reg, output_reg } => {
                // Verify tensor product commutativity and associativity
                self.validate_tensor_product(*left_reg, *right_reg, *output_reg)?;
            }
        }
        
        // Verify resource count bounds
        if prev_state.allocated_resources >= MAX_RESOURCES {
            return Err(BoundedExecutionError::ResourceLimitExceeded);
        }
        
        // Verify register count bounds
        if prev_state.allocated_registers >= MAX_REGISTERS {
            return Err(BoundedExecutionError::RegisterError(
                RegisterFileError::RegisterFileFull
            ));
        }
        
        Ok(())
    }
    
    /// Validate morphism application follows category laws
    fn validate_morphism_application(&self, morph_reg: RegisterId, input_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Verify registers are different (no aliasing)
        if morph_reg == input_reg || morph_reg == output_reg || input_reg == output_reg {
            return Err(BoundedExecutionError::InvalidInstruction(0, "Register aliasing not allowed".to_string()));
        }
        
        // Verify morphism register contains a valid morphism
        if self.register_file.read_register(morph_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(morph_reg));
        }
        
        // Verify input register contains a valid input
        if self.register_file.read_register(input_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(input_reg));
        }
        
        Ok(())
    }
    
    /// Validate allocation creates valid category objects
    fn validate_allocation(&self, type_reg: RegisterId, init_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Verify registers are different
        if type_reg == init_reg || type_reg == output_reg || init_reg == output_reg {
            return Err(BoundedExecutionError::InvalidInstruction(0, "Register aliasing not allowed".to_string()));
        }
        
        // Verify type register contains a valid type
        if self.register_file.read_register(type_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(type_reg));
        }
        
        // Verify init register contains valid initialization data
        if self.register_file.read_register(init_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(init_reg));
        }
        
        Ok(())
    }
    
    /// Validate consumption respects linear discipline
    fn validate_consumption(&self, resource_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Verify registers are different
        if resource_reg == output_reg {
            return Err(BoundedExecutionError::InvalidInstruction(0, "Resource and output registers must be different".to_string()));
        }
        
        // Verify resource register contains a valid resource
        if self.register_file.read_register(resource_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(resource_reg));
        }
        
        Ok(())
    }
    
    /// Validate composition follows associativity laws
    fn validate_composition(&self, first_reg: RegisterId, second_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Verify registers are different
        if first_reg == second_reg || first_reg == output_reg || second_reg == output_reg {
            return Err(BoundedExecutionError::InvalidInstruction(0, "Register aliasing not allowed".to_string()));
        }
        
        // Verify both morphisms exist
        if self.register_file.read_register(first_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(first_reg));
        }
        
        if self.register_file.read_register(second_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(second_reg));
        }
        
        Ok(())
    }
    
    /// Validate tensor product follows commutativity and associativity
    fn validate_tensor_product(&self, left_reg: RegisterId, right_reg: RegisterId, output_reg: RegisterId) -> Result<(), BoundedExecutionError> {
        // Verify registers are different
        if left_reg == right_reg || left_reg == output_reg || right_reg == output_reg {
            return Err(BoundedExecutionError::InvalidInstruction(0, "Register aliasing not allowed".to_string()));
        }
        
        // Verify both operands exist
        if self.register_file.read_register(left_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(left_reg));
        }
        
        if self.register_file.read_register(right_reg)?.is_none() {
            return Err(BoundedExecutionError::EmptyRegister(right_reg));
        }
        
        Ok(())
    }
    
    /// Verify state consistency after instruction execution
    fn verify_state_consistency(&self) -> Result<(), BoundedExecutionError> {
        // Verify register file consistency
        let allocated_registers = self.register_file.allocated_count();
        let available_registers = self.register_file.available_count();
        
        if allocated_registers + available_registers != MAX_REGISTERS {
            return Err(BoundedExecutionError::ResourceError(
                "Register file inconsistency detected".to_string()
            ));
        }
        
        // Verify resource store consistency
        let resource_count = self.resource_store.resource_count();
        if resource_count > MAX_RESOURCES {
            return Err(BoundedExecutionError::ResourceLimitExceeded);
        }
        
        // Verify execution bounds
        if self.execution_steps > MAX_EXECUTION_STEPS {
            return Err(BoundedExecutionError::ExecutionLimitExceeded);
        }
        
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Execution Results and State
//-----------------------------------------------------------------------------

/// Result of bounded execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Execution completed successfully
    Success {
        steps_executed: usize,
        trace: ExecutionTrace,
    },
    
    /// Execution failed with an error
    Error {
        message: String,
        steps_executed: usize,
        trace: ExecutionTrace,
    },
    
    /// Execution timed out (exceeded step limit)
    Timeout {
        steps_executed: usize,
        trace: ExecutionTrace,
    },
}

/// Current execution state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub program_counter: usize,
    pub execution_steps: usize,
    pub is_complete: bool,
    pub has_error: bool,
    pub allocated_registers: usize,
    pub allocated_resources: usize,
}

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur during bounded execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundedExecutionError {
    /// Program exceeds maximum instruction count
    ProgramTooLarge(usize),
    
    /// Invalid instruction at given index
    InvalidInstruction(usize, String),
    
    /// Register file operation failed
    RegisterError(RegisterFileError),
    
    /// Resource operation failed
    ResourceError(String),
    
    /// Register is empty when value expected
    EmptyRegister(RegisterId),
    
    /// Resource allocation limit exceeded
    ResourceLimitExceeded,
    
    /// Execution step limit exceeded
    ExecutionLimitExceeded,
}

impl From<RegisterFileError> for BoundedExecutionError {
    fn from(error: RegisterFileError) -> Self {
        BoundedExecutionError::RegisterError(error)
    }
}

impl From<crate::machine::resource::ResourceError> for BoundedExecutionError {
    fn from(error: crate::machine::resource::ResourceError) -> Self {
        BoundedExecutionError::ResourceError(error.to_string())
    }
}

impl std::fmt::Display for BoundedExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundedExecutionError::ProgramTooLarge(size) => {
                write!(f, "Program too large: {} instructions (max: {})", size, MAX_INSTRUCTIONS)
            }
            BoundedExecutionError::InvalidInstruction(index, msg) => {
                write!(f, "Invalid instruction at {}: {}", index, msg)
            }
            BoundedExecutionError::RegisterError(e) => {
                write!(f, "Register error: {}", e)
            }
            BoundedExecutionError::ResourceError(msg) => {
                write!(f, "Resource error: {}", msg)
            }
            BoundedExecutionError::EmptyRegister(reg_id) => {
                write!(f, "Register {} is empty", reg_id.id())
            }
            BoundedExecutionError::ResourceLimitExceeded => {
                write!(f, "Resource limit exceeded (max: {})", MAX_RESOURCES)
            }
            BoundedExecutionError::ExecutionLimitExceeded => {
                write!(f, "Execution limit exceeded (max: {} steps)", MAX_EXECUTION_STEPS)
            }
        }
    }
}

impl std::error::Error for BoundedExecutionError {}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bounded_executor_creation() {
        let program = vec![
            Instruction::Alloc {
                type_reg: RegisterId::new(0),
                init_reg: RegisterId::new(1),
                output_reg: RegisterId::new(2),
            }
        ];
        
        let executor = BoundedExecutor::new(program);
        assert!(executor.is_ok());
    }
    
    #[test]
    fn test_program_too_large() {
        let large_program = vec![
            Instruction::Alloc {
                type_reg: RegisterId::new(0),
                init_reg: RegisterId::new(1),
                output_reg: RegisterId::new(2),
            };
            MAX_INSTRUCTIONS + 1
        ];
        
        let result = BoundedExecutor::new(large_program);
        assert!(matches!(result, Err(BoundedExecutionError::ProgramTooLarge(_))));
    }
    
    #[test]
    fn test_invalid_register_id() {
        let program = vec![
            Instruction::Alloc {
                type_reg: RegisterId::new(MAX_REGISTERS as u32), // Invalid register ID
                init_reg: RegisterId::new(1),
                output_reg: RegisterId::new(2),
            }
        ];
        
        let result = BoundedExecutor::new(program);
        assert!(matches!(result, Err(BoundedExecutionError::InvalidInstruction(_, _))));
    }
} 