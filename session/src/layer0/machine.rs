// Core machine state for Layer 0

use crate::layer0::{MessageId, Instruction};
use crate::layer0::instruction::{execute_instruction};
use std::collections::BTreeMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Values that can be stored in messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageValue {
    Unit,
    Bool(bool),
    Int(i64),
    Pair(Box<MessageValue>, Box<MessageValue>),
    Sum(SumVariant, Box<MessageValue>),
    Channel(ChannelId),
}

/// Sum type variants (Left or Right)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SumVariant {
    Left,
    Right,
}

/// Channel identifier for message passing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ChannelId(pub u64);

/// Register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Register(pub u32);

/// Machine execution errors
#[derive(Error, Debug)]
pub enum MachineError {
    #[error("Register {0:?} not bound")]
    UnboundRegister(Register),
    
    #[error("Message {0} not found")]
    MessageNotFound(MessageId),
    
    #[error("Message {0} already consumed")]
    MessageAlreadyConsumed(MessageId),
    
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Core machine state
#[derive(Debug, Clone)]
pub struct MachineState {
    /// Content-addressed message storage - deterministic ordering
    messages: BTreeMap<MessageId, MessageValue>,
    
    /// Register bindings (register -> message ID or value) - deterministic ordering
    bindings: BTreeMap<Register, Binding>,
    
    /// Program counter
    pc: usize,
    
    /// Channel states - deterministic ordering
    channels: BTreeMap<ChannelId, ChannelState>,
}

/// What a register can be bound to
#[derive(Debug, Clone)]
pub enum Binding {
    Message(MessageId),
    Value(MessageValue),
}

/// Channel state for message passing
#[derive(Debug, Clone)]
pub struct ChannelState {
    /// Messages waiting to be received
    pending: Vec<MessageId>,
}

impl MachineState {
    /// Create a new machine state
    pub fn new() -> Self {
        MachineState {
            messages: BTreeMap::new(),
            bindings: BTreeMap::new(),
            pc: 0,
            channels: BTreeMap::new(),
        }
    }
    
    /// Get the current program counter
    pub fn pc(&self) -> usize {
        self.pc
    }
    
    /// Set the program counter
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }
    
    /// Increment the program counter
    pub fn increment_pc(&mut self) {
        self.pc += 1;
    }
    
    /// Add a message to storage
    pub fn add_message(&mut self, id: MessageId, value: MessageValue) {
        self.messages.insert(id, value);
    }
    
    /// Remove a message from storage (for linear consumption)
    pub fn consume_message(&mut self, id: MessageId) -> Result<MessageValue, MachineError> {
        self.messages.remove(&id)
            .ok_or(MachineError::MessageNotFound(id))
    }
    
    /// Check if a message exists
    pub fn has_message(&self, id: &MessageId) -> bool {
        self.messages.contains_key(id)
    }
    
    /// Bind a register to a value or message
    pub fn bind_register(&mut self, reg: Register, binding: Binding) {
        self.bindings.insert(reg, binding);
    }
    
    /// Get register binding
    pub fn get_binding(&self, reg: Register) -> Result<&Binding, MachineError> {
        self.bindings.get(&reg)
            .ok_or(MachineError::UnboundRegister(reg))
    }
    
    /// Clear a register binding
    pub fn clear_register(&mut self, reg: Register) {
        self.bindings.remove(&reg);
    }
    
    /// Get or create a channel
    pub fn get_channel_mut(&mut self, id: ChannelId) -> &mut ChannelState {
        self.channels.entry(id).or_insert(ChannelState {
            pending: Vec::new(),
        })
    }
    
    /// Send a message through a channel
    pub fn send_message(&mut self, channel: ChannelId, msg_id: MessageId) {
        let chan = self.get_channel_mut(channel);
        chan.pending.push(msg_id);
    }
    
    /// Receive a message from a channel
    pub fn receive_message(&mut self, channel: ChannelId) -> Option<MessageId> {
        self.channels.get_mut(&channel)
            .and_then(|chan| chan.pending.pop())
    }
    
    /// Execute a program (list of instructions) on this machine
    pub fn execute_program(&mut self, program: &[Instruction]) -> Result<(), MachineError> {
        while self.pc < program.len() {
            let instruction = &program[self.pc];
            execute_instruction(self, instruction)?;
        }
        Ok(())
    }
    
    /// Execute a single step
    pub fn step(&mut self, program: &[Instruction]) -> Result<bool, MachineError> {
        if self.pc >= program.len() {
            return Ok(false); // Program complete
        }
        
        let instruction = &program[self.pc];
        execute_instruction(self, instruction)?;
        Ok(true) // More instructions remain
    }
    
    /// Reset the machine to initial state
    pub fn reset(&mut self) {
        self.messages.clear();
        self.bindings.clear();
        self.channels.clear();
        self.pc = 0;
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageValue {
    /// Serialize the value to bytes for content addressing
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MessageValue::Unit => vec![0],
            MessageValue::Bool(b) => vec![1, if *b { 1 } else { 0 }],
            MessageValue::Int(i) => {
                let mut bytes = vec![2];
                bytes.extend_from_slice(&i.to_le_bytes());
                bytes
            }
            MessageValue::Pair(left, right) => {
                let mut bytes = vec![3];
                let left_bytes = left.to_bytes();
                let right_bytes = right.to_bytes();
                bytes.extend_from_slice(&(left_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(&left_bytes);
                bytes.extend_from_slice(&right_bytes);
                bytes
            }
            MessageValue::Sum(variant, value) => {
                let mut bytes = vec![4, match variant {
                    SumVariant::Left => 0,
                    SumVariant::Right => 1,
                }];
                bytes.extend_from_slice(&value.to_bytes());
                bytes
            }
            MessageValue::Channel(id) => {
                let mut bytes = vec![5];
                bytes.extend_from_slice(&id.0.to_le_bytes());
                bytes
            }
        }
    }
    
    /// Create a content-addressed MessageId for this value
    pub fn to_message_id(&self) -> MessageId {
        MessageId::from_data(&self.to_bytes())
    }
}
