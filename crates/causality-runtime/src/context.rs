//! Runtime context for effect execution

use std::collections::BTreeMap;
use causality_core::machine::{MachineState, value::MachineValue};
use causality_core::system::content_addressing::ResourceId;
use crate::error::{RuntimeError, RuntimeResult};

/// Runtime context for effect execution
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Current machine state
    pub machine_state: MachineState,
    
    /// Resource states for linearity tracking
    pub resource_states: BTreeMap<ResourceId, ResourceState>,
    
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// State of a linear resource during execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource is available for use
    Available,
    
    /// Resource has been consumed
    Consumed,
    
    /// Resource is temporarily borrowed
    Borrowed,
    
    /// Resource is in an error state
    Error { message: String },
}

/// Metadata about the current execution
#[derive(Debug, Clone)]
pub struct ExecutionMetadata {
    /// Current execution depth
    pub depth: usize,
    
    /// Maximum allowed depth
    pub max_depth: usize,
    
    /// Gas/computation units remaining
    pub gas_remaining: u64,
    
    /// Effects executed so far
    pub effects_executed: usize,
}

impl RuntimeContext {
    /// Create a new runtime context
    pub fn new() -> Self {
        Self {
            machine_state: MachineState::new(),
            resource_states: BTreeMap::new(),
            metadata: ExecutionMetadata::default(),
        }
    }
    
    /// Create a runtime context with initial machine state
    pub fn with_machine_state(machine_state: MachineState) -> Self {
        Self {
            machine_state,
            resource_states: BTreeMap::new(),
            metadata: ExecutionMetadata::default(),
        }
    }
    
    /// Check if a resource is available for consumption
    pub fn is_resource_available(&self, resource_id: &ResourceId) -> bool {
        match self.resource_states.get(resource_id) {
            Some(ResourceState::Available) => true,
            _ => false, // Return false for None (untracked) and other states
        }
    }
    
    /// Mark a resource as consumed
    pub fn consume_resource(&mut self, resource_id: ResourceId) -> RuntimeResult<()> {
        match self.resource_states.get(&resource_id) {
            Some(ResourceState::Consumed) => {
                Err(RuntimeError::linearity_violation(
                    format!("Resource {} already consumed", resource_id)
                ))
            }
            Some(ResourceState::Error { message }) => {
                Err(RuntimeError::resource_error(
                    format!("Resource {} in error state: {}", resource_id, message)
                ))
            }
            _ => {
                self.resource_states.insert(resource_id, ResourceState::Consumed);
                Ok(())
            }
        }
    }
    
    /// Add a new resource to tracking
    pub fn add_resource(&mut self, resource_id: ResourceId) {
        self.resource_states.insert(resource_id, ResourceState::Available);
    }
    
    /// Check gas and decrement
    pub fn consume_gas(&mut self, amount: u64) -> RuntimeResult<()> {
        if self.metadata.gas_remaining < amount {
            Err(RuntimeError::execution_failed("Insufficient gas"))
        } else {
            self.metadata.gas_remaining -= amount;
            Ok(())
        }
    }
    
    /// Increment execution depth
    pub fn enter_effect(&mut self) -> RuntimeResult<()> {
        if self.metadata.depth >= self.metadata.max_depth {
            Err(RuntimeError::execution_failed("Maximum execution depth exceeded"))
        } else {
            self.metadata.depth += 1;
            self.metadata.effects_executed += 1;
            Ok(())
        }
    }
    
    /// Decrement execution depth
    pub fn exit_effect(&mut self) {
        if self.metadata.depth > 0 {
            self.metadata.depth -= 1;
        }
    }
    
    /// Get a value from the machine state
    pub fn get_value(&self, register_id: causality_core::machine::RegisterId) -> Option<&MachineValue> {
        if let Ok(register) = self.machine_state.load_register(register_id) {
            Some(&register.value)
        } else {
            None
        }
    }
    
    /// Set a value in the machine state
    pub fn set_value(&mut self, register_id: causality_core::machine::RegisterId, value: MachineValue) {
        self.machine_state.store_register(register_id, value, None);
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionMetadata {
    /// Create new execution metadata
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 1000, // Reasonable default
            gas_remaining: 1_000_000, // 1M gas units
            effects_executed: 0,
        }
    }
    
    /// Create execution metadata with custom limits
    pub fn with_limits(max_depth: usize, gas_limit: u64) -> Self {
        Self {
            depth: 0,
            max_depth,
            gas_remaining: gas_limit,
            effects_executed: 0,
        }
    }
}

impl Default for ExecutionMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_context_creation() {
        let ctx = RuntimeContext::new();
        assert_eq!(ctx.metadata.depth, 0);
        assert_eq!(ctx.metadata.effects_executed, 0);
        assert!(ctx.resource_states.is_empty());
    }
    
    #[test]
    fn test_resource_lifecycle() {
        let mut ctx = RuntimeContext::new();
        let resource_id = ResourceId::from_bytes([0u8; 32]);
        
        // Initially not tracked
        assert!(!ctx.is_resource_available(&resource_id));
        
        // Add resource
        ctx.add_resource(resource_id.clone());
        assert!(ctx.is_resource_available(&resource_id));
        
        // Consume resource
        ctx.consume_resource(resource_id.clone()).unwrap();
        assert!(!ctx.is_resource_available(&resource_id));
        
        // Double consumption should fail
        let result = ctx.consume_resource(resource_id);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_gas_consumption() {
        let mut ctx = RuntimeContext::new();
        
        // Should succeed
        ctx.consume_gas(1000).unwrap();
        assert_eq!(ctx.metadata.gas_remaining, 999_000);
        
        // Consume all remaining gas
        ctx.consume_gas(999_000).unwrap();
        assert_eq!(ctx.metadata.gas_remaining, 0);
        
        // Should fail
        let result = ctx.consume_gas(1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_depth_tracking() {
        let mut ctx = RuntimeContext::new();
        
        // Enter effect
        ctx.enter_effect().unwrap();
        assert_eq!(ctx.metadata.depth, 1);
        assert_eq!(ctx.metadata.effects_executed, 1);
        
        // Exit effect
        ctx.exit_effect();
        assert_eq!(ctx.metadata.depth, 0);
    }
} 