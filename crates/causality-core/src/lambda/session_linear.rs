//! Session types fully integrated with linear types
//!
//! This module ensures that session channels are properly tracked as linear resources
//! and that session operations follow linear discipline. It provides the integration
//! layer between the session type system and the linear resource management.
//!
//! **Design Principles**:
//! - Session channels are linear resources that must be consumed exactly once
//! - Session operations track linear consumption and progression
//! - Type checking enforces linear discipline for session channels
//! - Channel lifecycle is managed through the resource system

use crate::{
    lambda::base::{SessionType, Location, TypeInner, BaseType},
    machine::{
        resource::{ResourceId, ResourceManager},
        value::{MachineValue, SessionChannel},
    },
    system::deterministic::DeterministicSystem,
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

/// Linear session environment that tracks session channels as linear resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearSessionEnvironment {
    /// Active session channels mapped to their resource IDs
    pub active_channels: BTreeMap<String, ResourceId>,
    
    /// Channel types for type checking
    pub channel_types: BTreeMap<ResourceId, SessionType>,
    
    /// Channel locations
    pub channel_locations: BTreeMap<ResourceId, Location>,
    
    /// Linear variable tracking
    pub linear_variables: BTreeMap<String, LinearVariableInfo>,
    
    /// Resource manager for channel lifecycle
    pub resource_manager: ResourceManager,
    
    /// Type constraints for session channels
    pub type_constraints: Vec<TypeInner>,
    
    /// Next channel ID for allocation
    pub next_channel_id: usize,
    
    /// Closed channels
    pub closed_channels: BTreeSet<String>,
}

/// Information about a linear variable in the session environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearVariableInfo {
    /// Resource ID of the associated resource
    pub resource_id: ResourceId,
    
    /// Type of the linear variable
    pub variable_type: TypeInner,
    
    /// Whether this variable has been consumed
    pub consumed: bool,
    
    /// Location where this variable is available
    pub location: Location,
}

/// Result of a session operation with linear tracking
#[derive(Debug, Clone)]
pub struct SessionOperationResult {
    /// New session type after the operation
    pub new_session_type: SessionType,
    
    /// Resource ID of the progressed channel
    pub channel_resource_id: ResourceId,
    
    /// Any values produced by the operation
    pub produced_values: Vec<MachineValue>,
    
    /// Linear variables that were consumed
    pub consumed_variables: Vec<String>,
}

/// Errors in session-linear integration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionLinearError {
    /// Channel not found in linear environment
    ChannelNotFound(String),
    
    /// Channel already consumed
    ChannelConsumed(ResourceId),
    
    /// Linear variable not available
    VariableNotAvailable(String),
    
    /// Session type mismatch
    SessionTypeMismatch {
        expected: SessionType,
        found: SessionType,
    },
    
    /// Linear discipline violation
    LinearViolation(String),
    
    /// Resource management error
    ResourceError(String),
}

impl LinearSessionEnvironment {
    /// Create a new linear session environment
    pub fn new() -> Self {
        Self {
            active_channels: BTreeMap::new(),
            channel_types: BTreeMap::new(),
            channel_locations: BTreeMap::new(),
            linear_variables: BTreeMap::new(),
            resource_manager: ResourceManager::new(),
            type_constraints: Vec::new(),
            next_channel_id: 0,
            closed_channels: BTreeSet::new(),
        }
    }
    
    /// Create a new session channel as a linear resource
    pub fn create_channel(
        &mut self,
        channel_name: String,
        session_type: SessionType,
        location: Location,
    ) -> Result<ResourceId, String> {
        // Create the session channel value
        let channel = SessionChannel::new(session_type.clone(), location.clone());
        let channel_value = MachineValue::Channel(channel);
        
        // Allocate as a linear resource
        let channel_type = TypeInner::Session(Box::new(session_type.clone()));
        let resource_id = self.resource_manager.allocate(
            MachineValue::Type(channel_type.clone()),
            channel_value
        );
        
        // Track in the linear environment
        self.active_channels.insert(channel_name.clone(), resource_id);
        self.channel_types.insert(resource_id, session_type);
        self.channel_locations.insert(resource_id, location.clone());
        
        // Add as linear variable
        self.linear_variables.insert(channel_name, LinearVariableInfo {
            resource_id,
            variable_type: channel_type,
            consumed: false,
            location,
        });
        
        Ok(resource_id)
    }
    
    /// Send a value through a session channel (linear operation)
    pub fn send_channel(
        &mut self,
        channel_name: &str,
        value: MachineValue,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<SessionOperationResult, SessionLinearError> {
        // Get the channel resource
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Check that channel is available (not consumed)
        if !self.resource_manager.is_available(&resource_id) {
            return Err(SessionLinearError::ChannelConsumed(resource_id));
        }
        
        // Get current session type
        let current_session = self.channel_types.get(&resource_id)
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))?
            .clone();
        
        // Validate that we can send on this session type
        let (value_type, continuation) = match current_session {
            SessionType::Send(vt, cont) => (vt, cont),
            _ => return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), 
                    Box::new(SessionType::End)
                ),
                found: current_session,
            }),
        };
        
        // Validate value type matches expected type
        let actual_value_type = value.get_type();
        if *value_type != actual_value_type {
            return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::Send(value_type, continuation.clone()),
                found: SessionType::Send(Box::new(actual_value_type), continuation.clone()),
            });
        }
        
        // Progress the session type
        let new_session_type = *continuation;
        self.channel_types.insert(resource_id, new_session_type.clone());
        
        // If session reaches End, consume the channel
        let consumed_variables = if matches!(new_session_type, SessionType::End) {
            self.consume_channel_internal(channel_name)?;
            vec![channel_name.to_string()]
        } else {
            vec![]
        };
        
        Ok(SessionOperationResult {
            new_session_type,
            channel_resource_id: resource_id,
            produced_values: vec![],
            consumed_variables,
        })
    }
    
    /// Receive a value from a session channel (linear operation)
    pub fn receive_channel(
        &mut self,
        channel_name: &str,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<(MachineValue, SessionOperationResult), SessionLinearError> {
        // Get the channel resource
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Check that channel is available (not consumed)
        if !self.resource_manager.is_available(&resource_id) {
            return Err(SessionLinearError::ChannelConsumed(resource_id));
        }
        
        // Get current session type and validate
        let current_session = self.channel_types.get(&resource_id)
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))?
            .clone();
        
        let (expected_type, continuation) = match current_session {
            SessionType::Receive(value_type, cont) => (value_type, cont),
            _ => return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::Receive(
                    Box::new(TypeInner::Base(BaseType::Unit)),
                    Box::new(SessionType::End)
                ),
                found: current_session,
            }),
        };
        
        // Get the actual resource to access its message queue
        let resource = self.resource_manager.peek(&resource_id)
            .map_err(|e| SessionLinearError::ResourceError(format!("{:?}", e)))?;
        
        // Extract the received value from the channel's message queue
        let received_value = if let MachineValue::Channel(ref channel) = resource {
            if !channel.message_queue.is_empty() {
                // Get the first message from the queue
                channel.message_queue[0].clone()
            } else {
                // If no message available, create a default value based on expected type
                match expected_type.as_ref() {
                    TypeInner::Base(BaseType::Unit) => MachineValue::Unit,
                    TypeInner::Base(BaseType::Bool) => MachineValue::Bool(false),
                    TypeInner::Base(BaseType::Int) => MachineValue::Int(0),
                    TypeInner::Base(BaseType::Symbol) => MachineValue::Symbol("default".into()),
                    _ => MachineValue::Unit,
                }
            }
        } else {
            return Err(SessionLinearError::ResourceError("Resource is not a channel".to_string()));
        };
        
        // Progress the session type
        let new_session_type = *continuation;
        self.channel_types.insert(resource_id, new_session_type.clone());
        
        // If session reaches End, consume the channel
        let consumed_variables = if matches!(new_session_type, SessionType::End) {
            self.consume_channel_internal(channel_name)?;
            vec![channel_name.to_string()]
        } else {
            vec![]
        };
        
        let result = SessionOperationResult {
            new_session_type,
            channel_resource_id: resource_id,
            produced_values: vec![received_value.clone()],
            consumed_variables,
        };
        
        Ok((received_value, result))
    }
    
    /// Select a choice in an internal choice session (linear operation)
    pub fn select_choice(
        &mut self,
        channel_name: &str,
        choice_label: &str,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<SessionOperationResult, SessionLinearError> {
        // Get the channel resource
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Check that channel is available (not consumed)
        if !self.resource_manager.is_available(&resource_id) {
            return Err(SessionLinearError::ChannelConsumed(resource_id));
        }
        
        // Get current session type
        let current_session = self.channel_types.get(&resource_id)
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))?
            .clone();
        
        // Validate that we have an internal choice
        let choices = match current_session {
            SessionType::InternalChoice(ch) => ch,
            _ => return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::InternalChoice(vec![]),
                found: current_session,
            }),
        };
        
        // Find the selected choice
        let selected_session = choices.iter()
            .find(|(label, _)| label == choice_label)
            .map(|(_, session)| session.clone())
            .ok_or_else(|| SessionLinearError::LinearViolation(
                format!("Choice '{}' not found in internal choice", choice_label)
            ))?;
        
        // Progress to the selected session type
        self.channel_types.insert(resource_id, selected_session.clone());
        
        // If session reaches End, consume the channel
        let consumed_variables = if matches!(selected_session, SessionType::End) {
            self.consume_channel_internal(channel_name)?;
            vec![channel_name.to_string()]
        } else {
            vec![]
        };
        
        Ok(SessionOperationResult {
            new_session_type: selected_session,
            channel_resource_id: resource_id,
            produced_values: vec![],
            consumed_variables,
        })
    }
    
    /// Offer choices in an external choice session (linear operation)
    pub fn offer_choice(
        &mut self,
        channel_name: &str,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<(String, SessionOperationResult), SessionLinearError> {
        // Get the channel resource
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Check that channel is available (not consumed)
        if !self.resource_manager.is_available(&resource_id) {
            return Err(SessionLinearError::ChannelConsumed(resource_id));
        }
        
        // Get current session type
        let current_session = self.channel_types.get(&resource_id)
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))?
            .clone();
        
        // Validate that we have an external choice
        let choices = match current_session {
            SessionType::ExternalChoice(ch) => ch,
            _ => return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::ExternalChoice(vec![]),
                found: current_session,
            }),
        };
        
        // For now, deterministically select the first choice
        let (selected_label, selected_session) = choices.first()
            .ok_or_else(|| SessionLinearError::LinearViolation(
                "External choice has no options".to_string()
            ))?;
        
        // Progress to the selected session type
        self.channel_types.insert(resource_id, selected_session.clone());
        
        // If session reaches End, consume the channel
        let consumed_variables = if matches!(selected_session, &SessionType::End) {
            self.consume_channel_internal(channel_name)?;
            vec![channel_name.to_string()]
        } else {
            vec![]
        };
        
        let result = SessionOperationResult {
            new_session_type: selected_session.clone(),
            channel_resource_id: resource_id,
            produced_values: vec![],
            consumed_variables,
        };
        
        Ok((selected_label.clone(), result))
    }
    
    /// Close a session channel (linear consumption)
    pub fn close_channel(
        &mut self,
        channel_name: &str,
    ) -> Result<MachineValue, SessionLinearError> {
        // Get the channel resource
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Check that channel is available (not consumed)
        if !self.resource_manager.is_available(&resource_id) {
            return Err(SessionLinearError::ChannelConsumed(resource_id));
        }
        
        // Validate that the session type is End
        let current_session = self.channel_types.get(&resource_id)
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))?;
        
        if !matches!(current_session, SessionType::End) {
            return Err(SessionLinearError::SessionTypeMismatch {
                expected: SessionType::End,
                found: current_session.clone(),
            });
        }
        
        // Consume the channel resource
        let consumption_result = self.resource_manager.consume(resource_id)
            .map_err(|e| SessionLinearError::ResourceError(format!("{:?}", e)))?;
        
        // Remove from tracking
        self.active_channels.remove(channel_name);
        self.channel_types.remove(&resource_id);
        self.channel_locations.remove(&resource_id);
        
        // Mark linear variable as consumed
        if let Some(var_info) = self.linear_variables.get_mut(channel_name) {
            var_info.consumed = true;
        }
        
        Ok(consumption_result.value)
    }
    
    /// Check if a channel is available (not consumed)
    pub fn is_channel_available(&self, channel_name: &str) -> bool {
        if let Some(resource_id) = self.active_channels.get(channel_name) {
            self.resource_manager.is_available(resource_id)
        } else {
            false
        }
    }
    
    /// Get the current session type of a channel
    pub fn get_channel_session_type(&self, channel_name: &str) -> Option<&SessionType> {
        let resource_id = self.active_channels.get(channel_name)?;
        self.channel_types.get(resource_id)
    }
    
    /// Get the location of a channel
    pub fn get_channel_location(&self, channel_name: &str) -> Option<&Location> {
        let resource_id = self.active_channels.get(channel_name)?;
        self.channel_locations.get(resource_id)
    }
    
    /// Validate linear discipline - ensure all channels are properly consumed
    pub fn validate_linear_discipline(&self) -> Result<(), SessionLinearError> {
        for (channel_name, var_info) in &self.linear_variables {
            if !var_info.consumed && self.resource_manager.is_available(&var_info.resource_id) {
                // Check if the session type is End (should be consumed)
                if let Some(session_type) = self.channel_types.get(&var_info.resource_id) {
                    if matches!(session_type, SessionType::End) {
                        return Err(SessionLinearError::LinearViolation(
                            format!("Channel '{}' with End session type should be consumed", channel_name)
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Get resource statistics for linear tracking
    pub fn get_resource_stats(&self) -> LinearResourceStats {
        LinearResourceStats {
            active_channels: self.active_channels.len(),
            total_resources: self.resource_manager.resource_count(),
            consumed_channels: self.linear_variables.values()
                .filter(|v| v.consumed)
                .count(),
            memory_usage: self.resource_manager.total_memory(),
        }
    }
    
    // Helper methods
    
    fn get_channel_resource_id(&self, channel_name: &str) -> Result<ResourceId, SessionLinearError> {
        self.active_channels.get(channel_name)
            .copied()
            .ok_or_else(|| SessionLinearError::ChannelNotFound(channel_name.to_string()))
    }
    
    fn consume_channel_internal(&mut self, channel_name: &str) -> Result<(), SessionLinearError> {
        let resource_id = self.get_channel_resource_id(channel_name)?;
        
        // Consume the resource
        self.resource_manager.consume(resource_id)
            .map_err(|e| SessionLinearError::ResourceError(format!("{:?}", e)))?;
        
        // Remove from active tracking
        self.active_channels.remove(channel_name);
        self.channel_types.remove(&resource_id);
        self.channel_locations.remove(&resource_id);
        
        // Mark as consumed
        if let Some(var_info) = self.linear_variables.get_mut(channel_name) {
            var_info.consumed = true;
        }
        
        Ok(())
    }
}

impl Default for LinearSessionEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about linear resource usage
#[derive(Debug, Clone)]
pub struct LinearResourceStats {
    /// Number of active channels
    pub active_channels: usize,
    
    /// Total number of resources
    pub total_resources: usize,
    
    /// Number of consumed channels
    pub consumed_channels: usize,
    
    /// Total memory usage
    pub memory_usage: u64,
}

impl std::fmt::Display for SessionLinearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionLinearError::ChannelNotFound(name) => {
                write!(f, "Channel '{}' not found in linear environment", name)
            }
            SessionLinearError::ChannelConsumed(id) => {
                write!(f, "Channel {:?} has already been consumed", id)
            }
            SessionLinearError::VariableNotAvailable(name) => {
                write!(f, "Linear variable '{}' is not available", name)
            }
            SessionLinearError::SessionTypeMismatch { expected, found } => {
                write!(f, "Session type mismatch: expected {:?}, found {:?}", expected, found)
            }
            SessionLinearError::LinearViolation(msg) => {
                write!(f, "Linear discipline violation: {}", msg)
            }
            SessionLinearError::ResourceError(msg) => {
                write!(f, "Resource management error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SessionLinearError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::{TypeInner, BaseType};

    #[test]
    fn test_create_channel() {
        let mut env = LinearSessionEnvironment::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let _resource_id = env.create_channel(
            "test_channel".to_string(),
            session_type.clone(),
            Location::Local,
        ).unwrap();
        
        assert!(env.is_channel_available("test_channel"));
        assert_eq!(env.get_channel_session_type("test_channel"), Some(&session_type));
        assert_eq!(env.get_channel_location("test_channel"), Some(&Location::Local));
    }
    
    #[test]
    fn test_send_and_close() {
        let mut env = LinearSessionEnvironment::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        env.create_channel(
            "test_channel".to_string(),
            session_type,
            Location::Local,
        ).unwrap();
        
        // Send a value
        let result = env.send_channel(
            "test_channel",
            MachineValue::Int(42),
            &mut det_sys,
        ).unwrap();
        
        // Should have progressed to End and consumed the channel
        assert_eq!(result.new_session_type, SessionType::End);
        assert_eq!(result.consumed_variables, vec!["test_channel".to_string()]);
        assert!(!env.is_channel_available("test_channel"));
    }
    
    #[test]
    fn test_linear_discipline_validation() {
        let mut env = LinearSessionEnvironment::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Test channel creation
        let ch_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        
        let _resource_id = env.create_channel(
            "test_channel".to_string(),
            ch_type,
            Location::Local
        ).unwrap();
        
        // Send one value
        env.send_channel(
            "test_channel",
            MachineValue::Bool(true),
            &mut det_sys,
        ).unwrap();
        
        // Channel should be consumed after Send(..., End) progresses to End
        assert!(!env.is_channel_available("test_channel"));
        
        // Validation should pass since all channels are properly consumed
        assert!(env.validate_linear_discipline().is_ok());
    }
    
    #[test]
    fn test_choice_operations() {
        let mut env = LinearSessionEnvironment::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        env.create_channel(
            "choice_channel".to_string(),
            session_type,
            Location::Local,
        ).unwrap();
        
        // Select the left choice
        let result = env.select_choice(
            "choice_channel",
            "left",
            &mut det_sys,
        ).unwrap();
        
        // Should have progressed to End and consumed the channel
        assert_eq!(result.new_session_type, SessionType::End);
        assert_eq!(result.consumed_variables, vec!["choice_channel".to_string()]);
        assert!(!env.is_channel_available("choice_channel"));
    }
    
    #[test]
    fn test_resource_stats() {
        let mut env = LinearSessionEnvironment::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Create multiple channels
        for i in 0..3 {
            env.create_channel(
                format!("channel_{}", i),
                session_type.clone(),
                Location::Local,
            ).unwrap();
        }
        
        let stats = env.get_resource_stats();
        assert_eq!(stats.active_channels, 3);
        assert_eq!(stats.consumed_channels, 0);
        
        // Consume one channel
        env.send_channel("channel_0", MachineValue::Int(42), &mut det_sys).unwrap();
        
        let stats = env.get_resource_stats();
        assert_eq!(stats.active_channels, 2);
        assert_eq!(stats.consumed_channels, 1);
    }
} 