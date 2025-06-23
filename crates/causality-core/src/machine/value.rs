//! Machine values for the minimal 5-operation instruction set
//!
//! This module defines the value types that can be stored in registers
//! and manipulated by the minimal register machine.

use super::instruction::RegisterId;
use crate::system::content_addressing::ResourceId;
use crate::lambda::{TypeInner, Symbol, BaseType};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

/// Values that can be stored in registers for the minimal instruction set
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineValue {
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(u32),
    
    /// Symbol value
    Symbol(Symbol),
    
    /// Product value (pair)
    Product(Box<MachineValue>, Box<MachineValue>),
    
    /// Sum value (tagged union)
    Sum {
        tag: Symbol,
        value: Box<MachineValue>,
    },
    
    /// Resource reference (points to higher-level resource)
    ResourceRef(crate::machine::resource::ResourceId),
    
    /// Morphism reference (points to morphism stored in register)
    MorphismRef(RegisterId),
    
    /// Tensor product of two values (parallel composition)
    Tensor(Box<MachineValue>, Box<MachineValue>),
    
    /// Type value (for alloc instruction)
    Type(TypeInner),
    
    /// Session channel for communication
    Channel(SessionChannel),
    
    /// Function closure
    Function {
        params: Vec<RegisterId>,
        body: Vec<super::instruction::Instruction>,
        captured_env: BTreeMap<RegisterId, MachineValue>,
    },
}

/// Session channel with linear resource tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionChannel {
    /// Unique identifier for this channel
    pub channel_id: ResourceId,
    
    /// Session type governing this channel
    pub session_type: crate::lambda::base::SessionType,
    
    /// Current state of the channel
    pub state: ChannelState,
    
    /// Message queue for asynchronous communication
    pub message_queue: Vec<MachineValue>,
    
    /// Location where this channel operates
    pub location: crate::lambda::base::Location,
}

/// Channel state for session-typed communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelState {
    /// Channel is open for communication
    Open,
    
    /// A choice has been selected with the given index
    ChoiceSelected(u32),
    
    /// Channel has been consumed (closed)
    Consumed,
}

impl MachineValue {
    /// Create a new session channel
    pub fn new_channel(
        session_type: crate::lambda::base::SessionType,
        location: crate::lambda::base::Location,
    ) -> Self {
        MachineValue::Channel(SessionChannel::new(session_type, location))
    }
    
    /// Check if this value is an available (unconsumed) channel
    pub fn is_available_channel(&self) -> bool {
        match self {
            MachineValue::Channel(channel) => channel.is_available(),
            _ => false,
        }
    }
    
    /// Get the session type if this is a channel
    pub fn get_session_type(&self) -> Option<&crate::lambda::base::SessionType> {
        match self {
            MachineValue::Channel(channel) => Some(&channel.session_type),
            _ => None,
        }
    }
    
    /// Consume a channel (mark as closed)
    pub fn consume_channel(&mut self) -> Result<(), String> {
        match self {
            MachineValue::Channel(channel) => {
                if channel.is_consumed() {
                    Err("Channel already consumed".to_string())
                } else {
                    channel.consume();
                    Ok(())
                }
            }
            _ => Err("Not a channel".to_string()),
        }
    }
    
    /// Get the type of this value
    pub fn get_type(&self) -> TypeInner {
        match self {
            MachineValue::Unit => TypeInner::Base(BaseType::Unit),
            MachineValue::Bool(_) => TypeInner::Base(BaseType::Bool),
            MachineValue::Int(_) => TypeInner::Base(BaseType::Int),
            MachineValue::Symbol(_) => TypeInner::Base(BaseType::Symbol),
            
            MachineValue::Product(l, r) => {
                TypeInner::Product(
                    Box::new(l.get_type()),
                    Box::new(r.get_type())
                )
            }
            
            MachineValue::Sum { .. } => {
                // For sum types, we'd need more context
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::ResourceRef(_) => {
                // Resource references would need type lookup
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::MorphismRef(_) => {
                // Morphism references represent function types
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::Tensor(l, r) => {
                // Tensor product type
                TypeInner::Product(
                    Box::new(l.get_type()),
                    Box::new(r.get_type())
                )
            }
            
            MachineValue::Type(_) => {
                // Type values represent types
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::Channel(session_channel) => {
                TypeInner::Session(Box::new(session_channel.session_type.clone()))
            }
            
            MachineValue::Function { .. } => {
                // Function types would need parameter and return type info
                TypeInner::Base(BaseType::Symbol)
            }
        }
    }
    
    /// Check if this value is a tensor product
    pub fn is_tensor(&self) -> bool {
        matches!(self, MachineValue::Tensor(_, _))
    }
    
    /// Extract tensor components if this is a tensor
    pub fn extract_tensor(&self) -> Option<(&MachineValue, &MachineValue)> {
        match self {
            MachineValue::Tensor(l, r) => Some((l.as_ref(), r.as_ref())),
            _ => None,
        }
    }
    
    /// Check if this value is a morphism reference
    pub fn is_morphism_ref(&self) -> bool {
        matches!(self, MachineValue::MorphismRef(_))
    }
    
    /// Get the resource ID if this is a resource reference
    pub fn get_resource_id(&self) -> Option<crate::machine::resource::ResourceId> {
        match self {
            MachineValue::ResourceRef(id) => Some(*id),
            _ => None,
        }
    }
    
    /// Get the morphism register ID if this is a morphism reference
    pub fn get_morphism_register(&self) -> Option<RegisterId> {
        match self {
            MachineValue::MorphismRef(reg) => Some(*reg),
            _ => None,
        }
    }
}

impl SessionChannel {
    /// Create a new session channel
    pub fn new(
        session_type: crate::lambda::base::SessionType,
        location: crate::lambda::base::Location,
    ) -> Self {
        // Simple counter for unique channel IDs
        static CHANNEL_COUNTER: AtomicU64 = AtomicU64::new(1);
        let counter = CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Generate a unique channel ID using the counter as a 4-byte array
        let counter_bytes = (counter as u32).to_le_bytes();
        let channel_id = ResourceId::from_content(&counter_bytes);
        
        Self {
            channel_id,
            session_type,
            state: ChannelState::Open,
            message_queue: Vec::new(),
            location,
        }
    }
    
    /// Check if the channel is available for use (not consumed)
    pub fn is_available(&self) -> bool {
        !matches!(self.state, ChannelState::Consumed)
    }
    
    /// Check if the channel has been consumed (closed)
    pub fn is_consumed(&self) -> bool {
        matches!(self.state, ChannelState::Consumed)
    }
    
    /// Consume the channel (mark as closed/finished)
    pub fn consume(&mut self) {
        self.state = ChannelState::Consumed;
        self.message_queue.clear(); // Clear any pending messages
    }
    
    /// Progress the session type (for session protocol advancement)
    pub fn progress_session(&mut self, new_session_type: crate::lambda::base::SessionType) {
        if self.is_available() {
            self.session_type = new_session_type;
            
            // If session reaches End, consume the channel
            if matches!(self.session_type, crate::lambda::base::SessionType::End) {
                self.consume();
            }
        }
    }
    
    /// Send a message through the channel (for async communication)
    pub fn send_message(&mut self, message: MachineValue) -> Result<(), String> {
        if !self.is_available() {
            return Err("Cannot send on consumed channel".to_string());
        }
        
        self.message_queue.push(message);
        Ok(())
    }
    
    /// Receive a message from the channel
    pub fn receive_message(&mut self) -> Option<MachineValue> {
        if self.is_available() {
            self.message_queue.pop()
        } else {
            None
        }
    }
    
    /// Get the dual session type (for creating channel pairs)
    pub fn dual_session_type(&self) -> crate::lambda::base::SessionType {
        self.session_type.dual()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::{SessionType, TypeInner, BaseType, Location};

    #[test]
    fn test_session_channel_creation() {
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        
        let channel = SessionChannel::new(session_type.clone(), location.clone());
        
        assert_eq!(channel.session_type, session_type);
        assert_eq!(channel.location, location);
        assert_eq!(channel.state, ChannelState::Open);
        assert!(channel.message_queue.is_empty());
        assert!(channel.is_available());
        assert!(!channel.is_consumed());
    }
    
    #[test]
    fn test_channel_consumption() {
        let session_type = SessionType::End;
        let location = Location::Local;
        
        let mut channel = SessionChannel::new(session_type, location);
        
        // Initially available
        assert!(channel.is_available());
        assert!(!channel.is_consumed());
        
        // Consume the channel
        channel.consume();
        
        // Now consumed
        assert!(!channel.is_available());
        assert!(channel.is_consumed());
        assert_eq!(channel.state, ChannelState::Consumed);
        assert!(channel.message_queue.is_empty());
    }
    
    #[test]
    fn test_session_progression() {
        let initial_session = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        
        let mut channel = SessionChannel::new(initial_session, location);
        
        // Progress to End
        channel.progress_session(SessionType::End);
        
        // Should be consumed when reaching End
        assert_eq!(channel.session_type, SessionType::End);
        assert!(channel.is_consumed());
    }
    
    #[test]
    fn test_message_sending_and_receiving() {
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        
        let mut channel = SessionChannel::new(session_type, location);
        
        // Send a message
        let message = MachineValue::Int(42);
        let result = channel.send_message(message.clone());
        assert!(result.is_ok());
        assert_eq!(channel.message_queue.len(), 1);
        
        // Receive the message
        let received = channel.receive_message();
        assert!(received.is_some());
        assert_eq!(received.unwrap(), message);
        assert!(channel.message_queue.is_empty());
    }
    
    #[test]
    fn test_consumed_channel_operations() {
        let session_type = SessionType::End;
        let location = Location::Local;
        
        let mut channel = SessionChannel::new(session_type, location);
        channel.consume();
        
        // Cannot send on consumed channel
        let message = MachineValue::Int(42);
        let result = channel.send_message(message);
        assert!(result.is_err());
        
        // Cannot receive from consumed channel
        let received = channel.receive_message();
        assert!(received.is_none());
    }
    
    #[test]
    fn test_machine_value_channel_helpers() {
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        
        // Create channel value
        let channel_value = MachineValue::new_channel(session_type.clone(), location);
        
        // Test helper methods
        assert!(channel_value.is_available_channel());
        assert_eq!(channel_value.get_session_type(), Some(&session_type));
        
        // Test consumption
        let mut channel_value = channel_value;
        let result = channel_value.consume_channel();
        assert!(result.is_ok());
        assert!(!channel_value.is_available_channel());
        
        // Cannot consume again
        let result = channel_value.consume_channel();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_channel_choice_selection() {
        let session_type = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        let location = Location::Local;
        
        let mut channel = SessionChannel::new(session_type, location);
        
        // Select a choice
        channel.state = ChannelState::ChoiceSelected(1);
        
        assert_eq!(channel.state, ChannelState::ChoiceSelected(1));
        assert!(channel.is_available());
    }
    
    #[test]
    fn test_dual_session_type() {
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        
        let channel = SessionChannel::new(session_type.clone(), location);
        let dual = channel.dual_session_type();
        
        // Dual of Send should be Receive
        match dual {
            SessionType::Receive(value_type, continuation) => {
                assert_eq!(*value_type, TypeInner::Base(BaseType::Int));
                assert_eq!(*continuation, SessionType::End);
            }
            _ => panic!("Expected Receive session type"),
        }
    }
    
    #[test]
    fn test_unique_channel_ids() {
        let session_type = SessionType::End;
        let location = Location::Local;
        
        let channel1 = SessionChannel::new(session_type.clone(), location.clone());
        let channel2 = SessionChannel::new(session_type, location);
        
        // Each channel should have a unique ID
        assert_ne!(channel1.channel_id, channel2.channel_id);
    }
    
    #[test]
    fn test_channel_as_linear_resource_lifecycle() {
        use crate::machine::resource::ResourceStore;
        
        let mut heap = ResourceStore::new();
        
        // Create a session channel
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        let channel = SessionChannel::new(session_type.clone(), location.clone());
        
        // Allocate the channel as a resource
        let channel_value = MachineValue::Channel(channel.clone());
        let resource_id = heap.allocate(
            MachineValue::Type(TypeInner::Session(Box::new(session_type))),
            channel_value.clone()
        );
        
        // Verify the resource exists and is available
        assert!(heap.is_available(&resource_id));
        assert!(!heap.is_consumed(&resource_id));
        
        // Peek at the resource to verify it's a channel
        let peeked_value = heap.peek(&resource_id).unwrap();
        assert!(matches!(peeked_value, MachineValue::Channel(_)));
        
        // Verify the channel is available
        if let MachineValue::Channel(peeked_channel) = peeked_value {
            assert!(peeked_channel.is_available());
            assert!(!peeked_channel.is_consumed());
        }
        
        // Consume the channel resource
        let consumed_result = heap.consume(resource_id).unwrap();
        let consumed_value = consumed_result.value;
        
        // Verify the consumed value is the channel
        assert!(matches!(consumed_value, MachineValue::Channel(_)));
        
        // Verify the resource is now consumed
        assert!(!heap.is_available(&resource_id));
        assert!(heap.is_consumed(&resource_id));
        
        // Verify we cannot consume it again
        let second_consume = heap.consume(resource_id);
        assert!(second_consume.is_err());
    }
    
    #[test]
    fn test_channel_resource_reference_pattern() {
        use crate::machine::resource::ResourceStore;
        
        let mut heap = ResourceStore::new();
        
        // Create a session channel
        let session_type = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        let location = Location::Local;
        let channel = SessionChannel::new(session_type.clone(), location);
        
        // Put a message in the channel for testing
        let mut channel_with_message = channel;
        channel_with_message.send_message(MachineValue::Bool(true)).unwrap();
        
        // Allocate the channel as a resource
        let channel_value = MachineValue::Channel(channel_with_message);
        let resource_id = heap.allocate(MachineValue::Type(TypeInner::Session(Box::new(session_type))), channel_value);
        
        // Create a resource reference (this is what would be stored in registers)
        let resource_ref = MachineValue::ResourceRef(resource_id);
        
        // Verify we can peek at the channel through the resource reference
        if let MachineValue::ResourceRef(ref_id) = resource_ref {
            let peeked_value = heap.peek(&ref_id).unwrap();
            
            if let MachineValue::Channel(channel) = peeked_value {
                assert!(channel.is_available());
                assert_eq!(channel.message_queue.len(), 1);
                assert_eq!(channel.message_queue[0], MachineValue::Bool(true));
            } else {
                panic!("Expected channel value");
            }
        }
        
        // Verify linear consumption works
        let consumed_result = heap.consume(resource_id).unwrap();
        assert!(matches!(consumed_result.value, MachineValue::Channel(_)));
        assert!(heap.is_consumed(&resource_id));
    }
} 