//! Abstract machine state for the register machine
//!
//! This module defines the core state of the register machine, containing
//! registers, resource heap, effects list, and constraints as specified
//! in the three-layer architecture.

#![allow(clippy::result_large_err)]

use super::instruction::RegisterId;
use super::value::{MachineValue, RegisterValue};
use crate::system::content_addressing::ResourceId;
use super::resource::{ResourceHeap, ResourceManager};
use crate::lambda::TypeInner;
use crate::system::error::MachineError;
use std::collections::BTreeMap;
use super::effect::{Effect, Constraint};

/// Maximum call stack depth (bounded for ZK compatibility)
pub const MAX_CALL_STACK_DEPTH: usize = 256;

/// Abstract machine state
///
/// State = { 
///   registers: Map<RegisterId, Value>, 
///   heap: Map<ResourceId, Resource>, 
///   effects: [Effect], 
///   constraints: [Constraint] 
/// }
#[derive(Debug, Clone)]
pub struct MachineState {
    /// Register file - holds values for computation
    pub registers: BTreeMap<RegisterId, RegisterValue>,
    
    /// Resource heap - stores linear resources
    pub resources: ResourceHeap,
    
    /// Pending effects to be executed
    pub effects: Vec<Effect>,
    
    /// Active constraints to be checked
    pub constraints: Vec<Constraint>,
    
    /// Program counter for instruction execution
    pub pc: usize,

    /// Flag to indicate if the last instruction caused a jump
    pub jumped: bool,

    /// Execution terminated flag
    pub terminated: bool,
    
    /// Gas remaining (for resource accounting)
    pub gas: u64,
    
    /// Call stack for function returns (bounded for ZK compatibility)
    pub call_stack: Vec<usize>,
}

impl MachineState {
    /// Create a new empty machine state
    pub fn new() -> Self {
        Self {
            registers: BTreeMap::new(),
            resources: ResourceHeap::new(),
            effects: Vec::new(),
            constraints: Vec::new(),
            pc: 0,
            jumped: false,
            terminated: false,
            gas: 1000, // Default gas limit
            call_stack: Vec::new(),
        }
    }
    
    /// Allocate a new register and return its ID
    pub fn alloc_register(&mut self) -> RegisterId {
        let id = RegisterId::new(self.registers.len() as u32);
        self.registers.insert(id, RegisterValue {
            value: MachineValue::Unit,
            value_type: None,
            consumed: false,
        });
        id
    }
    
    /// Store a value in a register
    pub fn store_register(&mut self, reg: RegisterId, value: MachineValue, value_type: Option<TypeInner>) {
        self.registers.insert(reg, RegisterValue {
            value,
            value_type,
            consumed: false,
        });
    }
    
    /// Load a value from a register
    pub fn load_register(&self, reg: RegisterId) -> Result<&RegisterValue, MachineError> {
        self.registers.get(&reg)
            .ok_or(MachineError::InvalidRegister(reg))
    }
    
    /// Move value from one register to another (consuming source)
    pub fn move_register(&mut self, src: RegisterId, dst: RegisterId) -> Result<(), MachineError> {
        let src_value = self.registers.get(&src)
            .ok_or(MachineError::InvalidRegister(src))?;
        
        if src_value.consumed {
            return Err(MachineError::AlreadyConsumed(src));
        }
        
        let value = src_value.clone();
        
        // Mark source as consumed
        if let Some(src_reg) = self.registers.get_mut(&src) {
            src_reg.consumed = true;
        }
        
        // Store in destination
        self.registers.insert(dst, value);
        
        Ok(())
    }
    
    /// Consume a register (mark it as used for linearity)
    pub fn consume_register(&mut self, reg: RegisterId) -> Result<(), MachineError> {
        match self.registers.get_mut(&reg) {
            Some(register) if !register.consumed => {
                register.consumed = true;
                Ok(())
            }
            Some(_) => Err(MachineError::AlreadyConsumed(reg)),
            None => Err(MachineError::InvalidRegister(reg)),
        }
    }
    
    /// Add an effect to be performed
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
    
    /// Add a constraint to be checked
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    
    /// Push a return address onto the call stack
    pub fn push_call(&mut self, return_address: usize) -> Result<(), MachineError> {
        if self.call_stack.len() >= MAX_CALL_STACK_DEPTH {
            return Err(MachineError::CallStackOverflow);
        }
        self.call_stack.push(return_address);
        Ok(())
    }
    
    /// Pop a return address from the call stack
    pub fn pop_call(&mut self) -> Result<usize, MachineError> {
        self.call_stack.pop().ok_or(MachineError::CallStackUnderflow)
    }
    
    /// Get the current call stack depth
    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager for MachineState {
    fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId {
        self.resources.alloc_resource(value, resource_type)
    }
    
    fn consume_resource(&mut self, id: ResourceId) -> Result<MachineValue, MachineError> {
        self.resources.consume_resource(id)
    }
} 