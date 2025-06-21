//! Machine state and execution semantics for the minimal instruction set
//!
//! This module implements the operational semantics for the register machine,
//! maintaining consistency with the symmetric monoidal closed category foundation.
//! The machine mediates between registers (machine-level storage) and resources (higher-level objects).

use crate::{
    lambda::{base::{TypeInner, BaseType, SessionType, Location, Value}, Symbol},
    machine::{
        instruction::{Instruction, RegisterId, Label},
        value::{MachineValue, SessionChannel, ChannelState},
        resource::{Resource, ResourceId, Nullifier},
    },
    system::{content_addressing::EntityId, Str},
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};
use sha2::{Sha256, Digest};

/// Execution trace step for ZK witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    /// Step number in execution
    pub step_number: u64,
    
    /// Lamport timestamp for this step
    pub lamport_time: u64,
    
    /// Instruction executed
    pub instruction: Instruction,
    
    /// Registers read during this step
    pub registers_read: Vec<(RegisterId, MachineValue)>,
    
    /// Registers written during this step
    pub registers_written: Vec<(RegisterId, MachineValue)>,
    
    /// Resources allocated during this step
    pub resources_allocated: Vec<(ResourceId, MachineValue)>,
    
    /// Resources consumed during this step
    pub resources_consumed: Vec<(ResourceId, MachineValue)>,
}

impl TraceStep {
    /// Create a new trace step
    pub fn new(step_number: u64, lamport_time: u64, instruction: Instruction) -> Self {
        Self {
            step_number,
            lamport_time,
            instruction,
            registers_read: Vec::new(),
            registers_written: Vec::new(),
            resources_allocated: Vec::new(),
            resources_consumed: Vec::new(),
        }
    }
}

/// Complete execution trace for ZK witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// All execution steps in order
    pub steps: Vec<TraceStep>,
    
    /// Initial machine state
    pub initial_state: MachineStateSnapshot,
    
    /// Final machine state
    pub final_state: MachineStateSnapshot,
}

impl ExecutionTrace {
    /// Create a new empty execution trace
    pub fn new() -> Self {
        let empty_snapshot = MachineStateSnapshot {
            registers: BTreeMap::new(),
            resources: BTreeMap::new(),
            instruction_pointer: 0,
            lamport_clock: 0,
        };
        
        Self {
            steps: Vec::new(),
            initial_state: empty_snapshot.clone(),
            final_state: empty_snapshot,
        }
    }
    
    /// Set the initial state snapshot
    pub fn set_initial_state(&mut self, register_snapshot: crate::machine::register_file::RegisterFileSnapshot, resource_snapshot: crate::machine::resource::ResourceStoreSnapshot) {
        // Convert snapshots to machine state snapshot
        let mut registers = BTreeMap::new();
        for (i, resource_id_opt) in register_snapshot.register_contents.iter().enumerate() {
            if let Some(resource_id) = resource_id_opt {
                registers.insert(RegisterId::new(i as u32), MachineValue::ResourceRef(*resource_id));
            }
        }
        
        self.initial_state = MachineStateSnapshot {
            registers,
            resources: BTreeMap::new(), // Simplified for now
            instruction_pointer: 0,
            lamport_clock: 0,
        };
    }
    
    /// Add a step to the execution trace
    pub fn add_step(&mut self, step: TraceStep) {
        self.steps.push(step);
    }
    
    /// Finalize the execution trace with final state
    pub fn finalize(&mut self, register_snapshot: crate::machine::register_file::RegisterFileSnapshot, resource_snapshot: crate::machine::resource::ResourceStoreSnapshot) {
        // Convert snapshots to machine state snapshot
        let mut registers = BTreeMap::new();
        for (i, resource_id_opt) in register_snapshot.register_contents.iter().enumerate() {
            if let Some(resource_id) = resource_id_opt {
                registers.insert(RegisterId::new(i as u32), MachineValue::ResourceRef(*resource_id));
            }
        }
        
        self.final_state = MachineStateSnapshot {
            registers,
            resources: BTreeMap::new(), // Simplified for now
            instruction_pointer: 0,
            lamport_clock: 0,
        };
    }
}

/// Snapshot of machine state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStateSnapshot {
    /// Register file contents
    pub registers: BTreeMap<RegisterId, MachineValue>,
    
    /// Resource store contents
    pub resources: BTreeMap<ResourceId, MachineValue>,
    
    /// Instruction pointer
    pub instruction_pointer: usize,
    
    /// Lamport clock value
    pub lamport_clock: u64,
}

/// Machine state for executing the minimal instruction set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineState {
    /// Register file - machine-level storage locations
    pub registers: BTreeMap<RegisterId, MachineValue>,
    
    /// Resource store - higher-level objects referenced by ResourceRef values
    pub resources: BTreeMap<ResourceId, MachineValue>,
    
    /// Current instruction pointer
    pub instruction_pointer: usize,
    
    /// Instruction sequence to execute
    pub instructions: Vec<Instruction>,
    
    /// Labels for jump targets
    pub labels: BTreeMap<Label, usize>,
    
    /// Execution finished flag
    pub finished: bool,
    
    /// Error state
    pub error: Option<String>,
    
    /// Nullifier set for consumed resources (ZK integration)
    pub nullifiers: BTreeSet<Nullifier>,
    
    /// Lamport clock for deterministic ordering
    pub lamport_clock: u64,
    
    /// Execution trace for ZK witness generation
    pub execution_trace: ExecutionTrace,
}

impl MachineState {
    /// Create a new machine state
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let initial_snapshot = MachineStateSnapshot {
            registers: BTreeMap::new(),
            resources: BTreeMap::new(),
            instruction_pointer: 0,
            lamport_clock: 0,
        };
        
        Self {
            registers: BTreeMap::new(),
            resources: BTreeMap::new(),
            instruction_pointer: 0,
            instructions,
            labels: BTreeMap::new(),
            finished: false,
            error: None,
            nullifiers: BTreeSet::new(),
            lamport_clock: 0,
            execution_trace: ExecutionTrace {
                steps: Vec::new(),
                initial_state: initial_snapshot.clone(),
                final_state: initial_snapshot,
            },
        }
    }
    
    /// Create a snapshot of current machine state
    pub fn create_snapshot(&self) -> MachineStateSnapshot {
        MachineStateSnapshot {
            registers: self.registers.clone(),
            resources: self.resources.clone(),
            instruction_pointer: self.instruction_pointer,
            lamport_clock: self.lamport_clock,
        }
    }
    
    /// Store a value in a register
    pub fn store_register(&mut self, register_id: RegisterId, value: MachineValue) {
        self.registers.insert(register_id, value);
        self.lamport_clock += 1;
    }
    
    /// Load a value from a register
    pub fn load_register(&self, register_id: RegisterId) -> Option<&MachineValue> {
        self.registers.get(&register_id)
    }
    
    /// Move a value from a register (consuming it)
    pub fn take_register(&mut self, register_id: RegisterId) -> Option<MachineValue> {
        let value = self.registers.remove(&register_id);
        if value.is_some() {
            self.lamport_clock += 1;
        }
        value
    }
    
    /// Store a resource (higher-level object)
    pub fn store_resource(&mut self, resource_id: ResourceId, value: MachineValue) {
        self.resources.insert(resource_id, value);
        self.lamport_clock += 1;
    }
    
    /// Load a resource
    pub fn load_resource(&self, resource_id: ResourceId) -> Option<&MachineValue> {
        self.resources.get(&resource_id)
    }
    
    /// Move a resource (consuming it)
    pub fn take_resource(&mut self, resource_id: ResourceId) -> Option<MachineValue> {
        let value = self.resources.remove(&resource_id);
        if value.is_some() {
            // Generate nullifier when consuming a resource
            let nullifier = self.generate_nullifier(resource_id);
            self.nullifiers.insert(nullifier);
            self.lamport_clock += 1;
        }
        value
    }
    
    /// Generate a nullifier for a consumed resource
    fn generate_nullifier(&self, resource_id: ResourceId) -> Nullifier {
        // Create deterministic nullifier using resource ID and current Lamport clock
        let input = format!("{}:{}", resource_id, self.lamport_clock);
        let hash = Sha256::digest(input.as_bytes());
        Nullifier::from_hash(hash.into())
    }
    
    /// Check if a resource has been consumed (nullifier exists)
    pub fn is_consumed(&self, resource_id: ResourceId) -> bool {
        let nullifier = self.generate_nullifier(resource_id);
        self.nullifiers.contains(&nullifier)
    }
    
    /// Execute a single step
    pub fn step(&mut self) -> Result<(), String> {
        if self.finished || self.instruction_pointer >= self.instructions.len() {
            if !self.finished {
                // Finalize execution trace
                self.execution_trace.final_state = self.create_snapshot();
                self.finished = true;
            }
            return Ok(());
        }
        
        let instruction = self.instructions[self.instruction_pointer].clone();
        self.execute_instruction(instruction)?;
        
        self.instruction_pointer += 1;
        Ok(())
    }
    
    /// Execute an instruction
    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), String> {
        // Start recording trace step
        let step_number = self.execution_trace.steps.len() as u64;
        let mut trace_step = TraceStep {
            step_number,
            lamport_time: self.lamport_clock,
            instruction: instruction.clone(),
            registers_read: Vec::new(),
            registers_written: Vec::new(),
            resources_allocated: Vec::new(),
            resources_consumed: Vec::new(),
        };
        
        // Execute the instruction and record operations
        let result = match instruction {
            Instruction::Transform { morph_reg, input_reg, output_reg } => {
                self.execute_transform_traced(morph_reg, input_reg, output_reg, &mut trace_step)
            }
            Instruction::Alloc { type_reg, init_reg, output_reg } => {
                self.execute_alloc_traced(type_reg, init_reg, output_reg, &mut trace_step)
            }
            Instruction::Consume { resource_reg, output_reg } => {
                self.execute_consume_traced(resource_reg, output_reg, &mut trace_step)
            }
            Instruction::Compose { first_reg, second_reg, output_reg } => {
                self.execute_compose_traced(first_reg, second_reg, output_reg, &mut trace_step)
            }
            Instruction::Tensor { left_reg, right_reg, output_reg } => {
                self.execute_tensor_traced(left_reg, right_reg, output_reg, &mut trace_step)
            }
        };
        
        // Add completed trace step
        self.execution_trace.steps.push(trace_step);
        
        result
    }
    
    /// Execute transform instruction with trace recording
    fn execute_transform_traced(&mut self, morph_reg: RegisterId, input_reg: RegisterId, output_reg: RegisterId, trace: &mut TraceStep) -> Result<(), String> {
        let morphism = self.load_register_traced(morph_reg, trace)
            .ok_or("Morphism not found in register")?
            .clone();
        let input = self.take_register_traced(input_reg, trace)
            .ok_or("Input not found in register")?;
        
        let output = match morphism {
            MachineValue::Function { params, body, captured_env } => {
                // Execute function by creating new machine state
                self.execute_function(params, body, captured_env, input)?
            }
            MachineValue::MorphismRef(morph_reg) => {
                // Resolve morphism reference and apply
                let resolved_morph = self.load_register_traced(morph_reg, trace)
                    .ok_or("Referenced morphism not found in register")?
                    .clone();
                self.apply_morphism(resolved_morph, input)?
            }
            _ => return Err("Invalid morphism type in register".to_string()),
        };
        
        self.store_register_traced(output_reg, output, trace);
        Ok(())
    }
    
    /// Load a register value with trace recording
    fn load_register_traced(&self, register_id: RegisterId, trace: &mut TraceStep) -> Option<&MachineValue> {
        if let Some(value) = self.registers.get(&register_id) {
            trace.registers_read.push((register_id, value.clone()));
            Some(value)
        } else {
            None
        }
    }
    
    /// Take a register value with trace recording
    fn take_register_traced(&mut self, register_id: RegisterId, trace: &mut TraceStep) -> Option<MachineValue> {
        if let Some(value) = self.registers.remove(&register_id) {
            trace.registers_read.push((register_id, value.clone()));
            self.lamport_clock += 1;
            Some(value)
        } else {
            None
        }
    }
    
    /// Store a register value with trace recording
    fn store_register_traced(&mut self, register_id: RegisterId, value: MachineValue, trace: &mut TraceStep) {
        trace.registers_written.push((register_id, value.clone()));
        self.registers.insert(register_id, value);
        self.lamport_clock += 1;
    }
    
    /// Execute alloc instruction with trace recording
    fn execute_alloc_traced(&mut self, type_reg: RegisterId, init_reg: RegisterId, output_reg: RegisterId, trace: &mut TraceStep) -> Result<(), String> {
        let resource_type = self.load_register_traced(type_reg, trace)
            .ok_or("Type not found in register")?
            .clone();
        let init_value = self.take_register_traced(init_reg, trace)
            .ok_or("Init value not found in register")?;
        
        let allocated = match resource_type {
            MachineValue::Type(TypeInner::Session(session_type)) => {
                // Allocate session channel
                let location = Location::Local; // Default to local
                let channel = SessionChannel::new(*session_type, location);
                MachineValue::Channel(channel)
            }
            MachineValue::Type(_) => {
                // Allocate data resource
                init_value
            }
            _ => return Err("Invalid type for allocation".to_string()),
        };
        
        self.store_register_traced(output_reg, allocated, trace);
        Ok(())
    }
    
    /// Execute consume instruction with trace recording
    fn execute_consume_traced(&mut self, resource_reg: RegisterId, output_reg: RegisterId, trace: &mut TraceStep) -> Result<(), String> {
        let resource_value = self.take_register_traced(resource_reg, trace)
            .ok_or("Resource not found in register")?;
        
        let final_value = match resource_value {
            MachineValue::ResourceRef(resource_id) => {
                // Consume the actual resource
                let resource = self.take_resource_traced(resource_id, trace)
                    .ok_or("Referenced resource not found")?;
                self.consume_resource_value(resource)?
            }
            other => {
                // Consume the value directly
                self.consume_resource_value(other)?
            }
        };
        
        self.store_register_traced(output_reg, final_value, trace);
        Ok(())
    }
    
    /// Execute compose instruction with trace recording
    fn execute_compose_traced(&mut self, first_reg: RegisterId, second_reg: RegisterId, output_reg: RegisterId, trace: &mut TraceStep) -> Result<(), String> {
        let first = self.load_register_traced(first_reg, trace)
            .ok_or("First morphism not found in register")?
            .clone();
        let second = self.load_register_traced(second_reg, trace)
            .ok_or("Second morphism not found in register")?
            .clone();
        
        // Create composed morphism
        let composed = self.compose_morphisms(first, second)?;
        self.store_register_traced(output_reg, composed, trace);
        Ok(())
    }
    
    /// Execute tensor instruction with trace recording
    fn execute_tensor_traced(&mut self, left_reg: RegisterId, right_reg: RegisterId, output_reg: RegisterId, trace: &mut TraceStep) -> Result<(), String> {
        let left = self.take_register_traced(left_reg, trace)
            .ok_or("Left value not found in register")?;
        let right = self.take_register_traced(right_reg, trace)
            .ok_or("Right value not found in register")?;
        
        let tensor_product = MachineValue::Tensor(Box::new(left), Box::new(right));
        self.store_register_traced(output_reg, tensor_product, trace);
        Ok(())
    }
    
    /// Take a resource with trace recording
    fn take_resource_traced(&mut self, resource_id: ResourceId, trace: &mut TraceStep) -> Option<MachineValue> {
        if let Some(value) = self.resources.remove(&resource_id) {
            trace.resources_consumed.push((resource_id, value.clone()));
            // Generate nullifier when consuming a resource
            let nullifier = self.generate_nullifier(resource_id);
            self.nullifiers.insert(nullifier);
            self.lamport_clock += 1;
            Some(value)
        } else {
            None
        }
    }
    
    /// Execute function with given parameters and body
    fn execute_function(&mut self, _params: Vec<RegisterId>, _body: Vec<Instruction>, 
                       _captured_env: BTreeMap<RegisterId, MachineValue>, input: MachineValue) -> Result<MachineValue, String> {
        // For now, return the input unchanged
        // TODO: Implement full function execution
        Ok(input)
    }
    
    /// Apply a morphism to an input value
    fn apply_morphism(&mut self, morphism: MachineValue, input: MachineValue) -> Result<MachineValue, String> {
        // For now, return the input unchanged
        // TODO: Implement morphism application
        Ok(input)
    }
    
    /// Compose two morphisms
    fn compose_morphisms(&mut self, first: MachineValue, second: MachineValue) -> Result<MachineValue, String> {
        // For now, return the second morphism
        // TODO: Implement proper composition
        Ok(second)
    }
    
    /// Consume a resource value and return its final form
    fn consume_resource_value(&mut self, resource: MachineValue) -> Result<MachineValue, String> {
        match resource {
            MachineValue::Channel(mut channel) => {
                // Close channel and return any remaining messages
                channel.state = ChannelState::Open; // Keep as Open since we don't have Closed variant
                Ok(MachineValue::Product(
                    Box::new(MachineValue::Unit),
                    Box::new(MachineValue::Int(channel.message_queue.len() as u32))
                ))
            }
            MachineValue::Function { .. } => {
                // Functions consumed to unit
                Ok(MachineValue::Unit)
            }
            other => {
                // Most resources consumed to their final value
                Ok(other)
            }
        }
    }
} 