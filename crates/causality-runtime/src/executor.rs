//! Basic execution engine for Causality instructions.
//!
//! This module provides core execution functionality for register machine
//! instructions, serving as the foundation for ZK-enabled execution.

use causality_core::machine::{Instruction, MachineState, MachineValue, RegisterId};
use crate::error::RuntimeResult;

/// Basic executor for instruction sequences
#[derive(Debug, Clone)]
pub struct Executor {
    /// Machine state for execution
    machine_state: MachineState,
    /// Instruction sequence
    instructions: Vec<Instruction>,
    /// Program counter
    pc: usize,
}

impl Executor {
    /// Create a new executor with default machine state
    pub fn new() -> Self {
        Self {
            machine_state: MachineState::new(Vec::new()),
            instructions: Vec::new(),
            pc: 0,
        }
    }

    /// Execute instructions sequentially and return the final result
    pub fn execute(&mut self, instructions: &[Instruction]) -> RuntimeResult<MachineValue> {
        // Reset machine state for fresh execution
        self.machine_state = MachineState::new(instructions.to_vec());
        self.instructions = instructions.to_vec();
        self.pc = 0;
        
        // Execute each instruction in sequence
        while self.pc < self.instructions.len() {
            if (self.step()?).is_some() {
                // Continue execution
            } else {
                break;
            }
        }
        
        // Return the final result from register 0 (convention)
        self.get_result()
    }

    /// Execute the current instruction and advance to the next
    pub fn step(&mut self) -> RuntimeResult<Option<MachineValue>> {
        if self.pc >= self.instructions.len() {
            return Ok(None);
        }

        let instruction = &self.instructions[self.pc].clone();
        self.pc += 1;

        match instruction {
            Instruction::Transform { morph_reg: _, input_reg, output_reg } => {
                if let Some(value) = self.machine_state.load_register(*input_reg) {
                    self.machine_state.store_register(*output_reg, value.clone());
                }
            }
            Instruction::Alloc { type_reg: _, init_reg, output_reg } => {
                // For now, just copy the init value to the output register  
                if let Some(value) = self.machine_state.load_register(*init_reg) {
                    self.machine_state.store_register(*output_reg, value.clone());
                }
            }
            Instruction::Consume { resource_reg, output_reg } => {
                if let Some(value) = self.machine_state.load_register(*resource_reg) {
                    self.machine_state.store_register(*output_reg, value.clone());
                    // Mark the resource as consumed
                    self.machine_state.store_register(*resource_reg, MachineValue::Unit);
                }
            }
            Instruction::Compose { first_reg: _, second_reg, output_reg } => {
                if let Some(second_value) = self.machine_state.load_register(*second_reg) {
                    // For now, just copy the second morphism to the output
                    self.machine_state.store_register(*output_reg, second_value.clone());
                }
            }
            Instruction::Tensor { left_reg, right_reg, output_reg } => {
                if let (Some(left_value), Some(right_value)) = (
                    self.machine_state.load_register(*left_reg),
                    self.machine_state.load_register(*right_reg)
                ) {
                    // Create a product value from the tensor operation
                    let tensor_value = MachineValue::Product(
                        Box::new(left_value.clone()), 
                        Box::new(right_value.clone())
                    );
                    self.machine_state.store_register(*output_reg, tensor_value);
                }
            }
        }

        // Return the current value in register 0, if any
        if let Some(value) = self.machine_state.load_register(RegisterId(0)) {
            Ok(Some(value.clone()))
        } else {
            Ok(Some(MachineValue::Unit))
        }
    }

    /// Get the final result from register 0
    pub fn get_result(&self) -> RuntimeResult<MachineValue> {
        if let Some(value) = self.machine_state.load_register(RegisterId(0)) {
            Ok(value.clone())
        } else {
            Ok(MachineValue::Unit)
        }
    }

    /// Get the current machine state
    pub fn machine_state(&self) -> &MachineState {
        &self.machine_state
    }

    /// Get a mutable reference to the machine state
    pub fn machine_state_mut(&mut self) -> &mut MachineState {
        &mut self.machine_state
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = Executor::new();
        // Test that we can access the machine state and it starts empty
        assert!(executor.machine_state.load_register(RegisterId(0)).is_none());
    }

    #[test]
    fn test_basic_execution() {
        let mut executor = Executor::new();
        
        // Test empty instruction sequence
        let instructions = vec![];
        let result = executor.execute(&instructions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MachineValue::Unit);
    }

    #[test]
    fn test_alloc_instruction() {
        let mut executor = Executor::new();
        
        // Test with an alloc instruction that should work
        let instructions = vec![
            Instruction::Alloc { 
                type_reg: RegisterId(1), 
                init_reg: RegisterId(2), 
                output_reg: RegisterId(0) 
            }
        ];
        
        let result = executor.execute(&instructions);
        assert!(result.is_ok());
        // Result should be whatever the alloc instruction produces
        println!("Result: {:?}", result.unwrap());
    }
} 