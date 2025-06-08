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
            machine_state: MachineState::new(),
            instructions: Vec::new(),
            pc: 0,
        }
    }

    /// Execute instructions sequentially and return the final result
    pub fn execute(&mut self, instructions: &[Instruction]) -> RuntimeResult<MachineValue> {
        // Reset machine state for fresh execution
        self.machine_state = MachineState::new();
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
            Instruction::Move { src, dst } => {
                if let Ok(value) = self.machine_state.load_register(*src) {
                    self.machine_state.store_register(*dst, value.value.clone(), None);
                }
            }
            Instruction::Witness { out_reg } => {
                // Witness instruction produces a default value for testing
                self.machine_state.store_register(*out_reg, MachineValue::Int(42), None);
            }
            Instruction::Alloc { type_reg: _, val_reg, out_reg } => {
                // For now, just copy the value to the output register  
                if let Ok(value) = self.machine_state.load_register(*val_reg) {
                    self.machine_state.store_register(*out_reg, value.value.clone(), None);
                }
            }
            Instruction::Consume { resource_reg, out_reg } => {
                if let Ok(value) = self.machine_state.load_register(*resource_reg) {
                    self.machine_state.store_register(*out_reg, value.value.clone(), None);
                    // Mark the resource as consumed
                    self.machine_state.store_register(*resource_reg, MachineValue::Unit, None);
                }
            }
            Instruction::Apply { fn_reg: _, arg_reg, out_reg } => {
                if let Ok(arg_value) = self.machine_state.load_register(*arg_reg) {
                    // For now, just copy the argument to the output
                    self.machine_state.store_register(*out_reg, arg_value.value.clone(), None);
                }
            }
            Instruction::Select { cond_reg, true_reg, false_reg, out_reg } => {
                if let Ok(cond_value) = self.machine_state.load_register(*cond_reg) {
                    let result_reg = if let MachineValue::Bool(true) = cond_value.value {
                        *true_reg
                    } else {
                        *false_reg
                    };
                    
                    if let Ok(result_value) = self.machine_state.load_register(result_reg) {
                        self.machine_state.store_register(*out_reg, result_value.value.clone(), None);
                    }
                }
            }
            _ => {
                // Handle other instruction variants as needed
            }
        }

        // Return the current value in register 0, if any
        if let Ok(value) = self.machine_state.load_register(RegisterId(0)) {
            Ok(Some(value.value.clone()))
        } else {
            Ok(Some(MachineValue::Unit))
        }
    }

    /// Get the final result from register 0
    pub fn get_result(&self) -> RuntimeResult<MachineValue> {
        if let Ok(value) = self.machine_state.load_register(RegisterId(0)) {
            Ok(value.value.clone())
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
        assert!(executor.machine_state.load_register(RegisterId(0)).is_err());
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
    fn test_witness_instruction() {
        let mut executor = Executor::new();
        
        // Test with a witness instruction that should work
        let instructions = vec![
            Instruction::Witness { out_reg: RegisterId(0) }
        ];
        
        let result = executor.execute(&instructions);
        assert!(result.is_ok());
        // Result should be whatever the witness instruction produces
        println!("Result: {:?}", result.unwrap());
    }
} 