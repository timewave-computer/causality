//! Layer 0: Register Machine
//!
//! This module implements the minimal verifiable execution model with 9 core instructions.
//! It provides the computational substrate for the layers above, with deterministic
//! execution and linear resource tracking suitable for ZK verification.

pub mod state;
pub mod value;
pub mod resource;
pub mod effect;
pub mod instruction;
pub mod reduction;
pub mod metering;

// Re-export commonly used types
pub use instruction::{
    Instruction, RegisterId, Pattern, MatchArm, ConstraintExpr, EffectCall, LiteralValue,
};

pub use state::MachineState;

pub use value::{
    MachineValue, RegisterValue,
};

pub use resource::{
    ResourceId, Resource, ResourceHeap, ResourceManager,
};

pub use effect::{
    Effect, Constraint,
};

pub use reduction::{
    ReductionEngine,
};

pub use metering::{
    Metering, ComputeBudget, InstructionCosts,
};

// Re-export error types from system
pub use crate::system::error::{MachineError, ReductionError};

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
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
        let state = MachineState::new();
        assert_eq!(state.registers.len(), 0);
        assert_eq!(state.effects.len(), 0);
        assert_eq!(state.constraints.len(), 0);
        assert_eq!(state.pc, 0);
    }
    
    #[test]
    fn test_resource_allocation() {
        let mut state = MachineState::new();
        let value = MachineValue::Int(42);
        let resource_type = crate::lambda::TypeInner::Base(crate::lambda::BaseType::Int);
        
        let resource_id = state.alloc_resource(value.clone(), resource_type);
        
        // Check resource exists
        assert!(state.resources.is_available(resource_id));
        
        // Consume resource
        let consumed_value = state.consume_resource(resource_id);
        assert!(consumed_value.is_ok());
        assert_eq!(consumed_value.unwrap(), value);
        
        // Check resource is consumed
        assert!(!state.resources.is_available(resource_id));
        
        // Try to consume again - should fail
        assert!(state.consume_resource(resource_id).is_err());
    }
    
    #[test]
    fn test_linear_safety() {
        let mut state = MachineState::new();
        
        // Store value in register
        state.store_register(RegisterId::new(1), MachineValue::Int(42), None);
        
        // Consume register
        assert!(state.consume_register(RegisterId::new(1)).is_ok());
        
        // Try to consume again - should fail
        assert!(state.consume_register(RegisterId::new(1)).is_err());
    }
} 