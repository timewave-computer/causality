//! Machine module for the causality core
//!
//! This module contains the minimal instruction set and execution engine
//! based on symmetric monoidal closed category theory.

pub mod instruction;
pub mod value;
pub mod reduction;
pub mod resource;
pub mod metering;
pub mod register_file;
pub mod bounded_execution;
pub mod channel_resource;
pub mod pattern;

// Re-export key types
pub use instruction::{Instruction, Label, RegisterId};
pub use reduction::MachineState;
pub use value::{MachineValue, SessionChannel, ChannelState};
pub use resource::Resource;
pub use register_file::{RegisterFile, RegisterFileError};
pub use bounded_execution::{BoundedExecutor, BoundedExecutionError, ExecutionResult};
pub use metering::{GasMeter, GasError, InstructionCosts};
pub use pattern::{Pattern, LiteralValue};

// Channel-resource integration
pub use channel_resource::{
    ChannelResourceManager, ChannelOperationResult, ChannelResourceError, ChannelResourceStats,
};

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::resource::ResourceManager;
    use crate::machine::metering::GasMeter;
    
    #[test]
    fn test_register_id_creation() {
        let reg1 = RegisterId::new(1);
        let reg2 = RegisterId::new(2);
        
        assert_eq!(reg1.id(), 1);
        assert_eq!(reg2.id(), 2);
        assert_ne!(reg1, reg2);
    }
    
    #[test]
    fn test_machine_state_creation() {
        let program = vec![
            Instruction::Transform {
                morph_reg: RegisterId::new(1),
                input_reg: RegisterId::new(2),
                output_reg: RegisterId::new(3),
            }
        ];
        let state = MachineState::new(program);
        assert_eq!(state.registers.len(), 0);
        assert_eq!(state.instruction_pointer, 0);
    }
    
    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        let resource_type = MachineValue::Type(crate::lambda::TypeInner::Base(crate::lambda::BaseType::Int));
        let value = MachineValue::Int(42);
        
        let resource_id = manager.allocate(resource_type, value.clone());
        
        // Check resource exists
        assert_eq!(manager.resource_count(), 1);
        
        // Peek at resource
        let peeked = manager.peek(&resource_id).unwrap();
        assert_eq!(peeked, &value);
        
        // Consume resource
        let consumed_value = manager.consume(resource_id.clone());
        assert!(consumed_value.is_ok());
        assert_eq!(consumed_value.unwrap().value, value);
        
        // Check resource is consumed
        assert_eq!(manager.resource_count(), 0);
        
        // Try to consume again - should fail
        assert!(manager.consume(resource_id).is_err());
    }
    
    #[test]
    fn test_gas_metering() {
        let mut meter = GasMeter::new(100);
        
        let instruction = Instruction::Transform {
            morph_reg: RegisterId::new(1),
            input_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        };
        
        assert!(meter.can_execute(&instruction));
        assert_eq!(meter.instruction_cost(&instruction), 3);
        
        meter.consume_gas(&instruction).unwrap();
        assert_eq!(meter.gas_used, 3);
        assert_eq!(meter.remaining_gas(), 97);
    }
}
