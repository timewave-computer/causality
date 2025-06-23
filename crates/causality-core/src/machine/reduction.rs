//! Machine state and execution semantics for the minimal instruction set
//!
//! This module implements the operational semantics for the register machine,
//! maintaining consistency with the symmetric monoidal closed category foundation.
//! The machine mediates between registers (machine-level storage) and resources (higher-level objects).

use crate::{
    lambda::base::{TypeInner, Location},
    machine::{
        instruction::{Instruction, RegisterId, Label},
        value::{MachineValue, SessionChannel, ChannelState},
        resource::{ResourceId, Nullifier},
    },
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
    pub fn set_initial_state(&mut self, register_snapshot: crate::machine::register_file::RegisterFileSnapshot, _resource_snapshot: crate::machine::resource::ResourceStoreSnapshot) {
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
    pub fn finalize(&mut self, register_snapshot: crate::machine::register_file::RegisterFileSnapshot, _resource_snapshot: crate::machine::resource::ResourceStoreSnapshot) {
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
        
        let output = self.apply_morphism(morphism, input)?;
        
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
    fn execute_function(&mut self, params: Vec<RegisterId>, body: Vec<Instruction>, 
                       captured_env: BTreeMap<RegisterId, MachineValue>, input: MachineValue) -> Result<MachineValue, String> {
        // Save current state
        let saved_registers = self.registers.clone();
        let saved_ip = self.instruction_pointer;
        let saved_instructions = self.instructions.clone();
        
        // Set up function environment
        // Bind captured environment
        for (reg_id, value) in captured_env {
            self.registers.insert(reg_id, value);
        }
        
        // Bind parameters to input value
        if let Some(param_reg) = params.first() {
            self.registers.insert(*param_reg, input);
        }
        
        // Execute function body
        self.instructions = body;
        self.instruction_pointer = 0;
        self.finished = false;
        
        // Run until completion or error
        while !self.finished && self.error.is_none() {
            if let Err(e) = self.step() {
                self.error = Some(e);
                break;
            }
        }
        
        // Get result (assume it's in the last register used)
        let result = if let Some(last_reg) = self.registers.keys().last() {
            self.registers.get(last_reg).cloned().unwrap_or(MachineValue::Unit)
        } else {
            MachineValue::Unit
        };
        
        // Restore state
        self.registers = saved_registers;
        self.instruction_pointer = saved_ip;
        self.instructions = saved_instructions;
        self.finished = false;
        self.error = None;
        
        Ok(result)
    }
    
    /// Apply a morphism to an input value
    fn apply_morphism(&mut self, morphism: MachineValue, input: MachineValue) -> Result<MachineValue, String> {
        match morphism {
            MachineValue::Function { params, body, captured_env } => {
                // Execute function with input
                self.execute_function(params, body, captured_env, input)
            }
            MachineValue::MorphismRef(reg_id) => {
                // Dereference morphism and apply
                if let Some(actual_morphism) = self.registers.get(&reg_id).cloned() {
                    self.apply_morphism(actual_morphism, input)
                } else {
                    Err(format!("Morphism not found in register {:?}", reg_id))
                }
            }
            MachineValue::Symbol(name) => {
                // Built-in morphisms by name
                match name.as_str() {
                    "identity" => Ok(input),
                    "not" => match input {
                        MachineValue::Bool(b) => Ok(MachineValue::Bool(!b)),
                        _ => Err("Not morphism requires boolean input".to_string()),
                    },
                    "increment" => match input {
                        MachineValue::Int(i) => Ok(MachineValue::Int(i + 1)),
                        _ => Err("Increment morphism requires integer input".to_string()),
                    },
                    _ => Err(format!("Unknown built-in morphism: {}", name)),
                }
            }
            _ => {
                // For other values, treat as identity morphism
                Ok(input)
            }
        }
    }
    
    /// Compose two morphisms
    fn compose_morphisms(&mut self, first: MachineValue, second: MachineValue) -> Result<MachineValue, String> {
        match (&first, &second) {
            // Function composition: (g ∘ f)(x) = g(f(x))
            (MachineValue::Function { .. }, MachineValue::Function { .. }) => {
                // Create a new composite function
                // For now, return the second function (simplified)
                Ok(second)
            }
            // Transform composition
            (MachineValue::MorphismRef(_), MachineValue::MorphismRef(_)) => {
                // Compose transforms sequentially
                Ok(second)
            }
            // Symbol composition for built-ins
            (MachineValue::Symbol(f_name), MachineValue::Symbol(g_name)) => {
                // Create composed built-in morphism
                let composed_name = format!("{}∘{}", g_name, f_name);
                Ok(MachineValue::Symbol(composed_name.into()))
            }
            // Mixed composition - convert to common form
            _ => {
                // For mixed types, return the second morphism
                Ok(second)
            }
        }
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
    
    pub fn restore_snapshot(&mut self, register_snapshot: crate::machine::register_file::RegisterFileSnapshot, _resource_snapshot: crate::machine::resource::ResourceStoreSnapshot) {
        self.registers = BTreeMap::new();
        for (i, resource_id_opt) in register_snapshot.register_contents.iter().enumerate() {
            if let Some(resource_id) = resource_id_opt {
                self.registers.insert(RegisterId::new(i as u32), MachineValue::ResourceRef(*resource_id));
            }
        }
        // Restore resource store from snapshot
        // For now, we only restore register mappings to resource IDs
        // A full implementation would restore the actual resource values
    }
    
    /// Save the current machine state to snapshots
    pub fn save_snapshot(&self) -> (crate::machine::register_file::RegisterFileSnapshot, crate::machine::resource::ResourceStoreSnapshot) {
        let mut register_contents = [None; crate::machine::register_file::MAX_REGISTERS];
        let mut allocated_registers = std::collections::BTreeSet::new();
        
        // Fill in the register contents and allocated set
        for (&reg_id, value) in &self.registers {
            let index = reg_id.id() as usize;
            if index < crate::machine::register_file::MAX_REGISTERS {
                if let Some(resource_id) = value.get_resource_id() {
                    register_contents[index] = Some(resource_id);
                }
                allocated_registers.insert(reg_id.id());
            }
        }
        
        let register_snapshot = crate::machine::register_file::RegisterFileSnapshot {
            register_contents,
            allocated_registers,
            next_register_id: self.registers.len() as u32,
        };
        
        let resource_snapshot = crate::machine::resource::ResourceStoreSnapshot {
            resource_count: self.resources.len(),
            total_memory: 0, // Placeholder
            allocation_counter: 0, // Placeholder  
            nullifier_count: self.nullifiers.len(),
        };
        
        (register_snapshot, resource_snapshot)
    }
}
