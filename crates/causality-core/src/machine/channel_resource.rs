//! Channel operations as linear resource operations
//!
//! This module ensures that channel operations work seamlessly as linear resource
//! operations, integrating session channels with the resource management system.
//! Channels are treated as first-class linear resources with proper lifecycle management.

use crate::{
    lambda::base::{SessionType, Location, TypeInner},
    machine::{
        instruction::{Instruction, RegisterId},
        resource::{ResourceId, ResourceManager, ResourceError},
        value::{MachineValue, SessionChannel},
        register_file::{RegisterFile, RegisterFileError},
    },
    system::deterministic::DeterministicSystem,
};

/// Channel resource operations integrated with the machine layer
#[derive(Debug, Clone)]
pub struct ChannelResourceManager {
    /// Underlying resource manager
    pub resource_manager: ResourceManager,
    
    /// Register file for storing channel references
    pub register_file: RegisterFile,
}

/// Result of a channel operation at the machine level
#[derive(Debug, Clone)]
pub struct ChannelOperationResult {
    /// Register containing the result
    pub result_register: RegisterId,
    
    /// Any consumed resources
    pub consumed_resources: Vec<ResourceId>,
    
    /// Any allocated resources
    pub allocated_resources: Vec<ResourceId>,
    
    /// Instructions generated
    pub instructions: Vec<Instruction>,
}

/// Errors in channel-resource operations
#[derive(Debug, Clone)]
pub enum ChannelResourceError {
    /// Resource management error
    ResourceError(ResourceError),
    
    /// Register file error
    RegisterError(RegisterFileError),
    
    /// Channel not found
    ChannelNotFound(ResourceId),
    
    /// Session type mismatch
    SessionTypeMismatch(String),
    
    /// Linear discipline violation
    LinearViolation(String),
}

impl ChannelResourceManager {
    /// Create a new channel resource manager
    pub fn new() -> Self {
        Self {
            resource_manager: ResourceManager::new(),
            register_file: RegisterFile::new(),
        }
    }
    
    /// Create a new channel as a linear resource
    pub fn create_channel_resource(
        &mut self,
        session_type: SessionType,
        location: Location,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Create the session channel
        let channel = SessionChannel::new(session_type.clone(), location);
        let channel_value = MachineValue::Channel(channel);
        
        // Allocate as a resource
        let channel_type = TypeInner::Session(Box::new(session_type));
        let resource_id = self.resource_manager.allocate(
            MachineValue::Type(channel_type),
            channel_value
        );
        
        // Allocate a register to hold the channel reference
        let result_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Store the resource reference in the register
        self.register_file.write_register(result_register, Some(resource_id))
            .map_err(ChannelResourceError::RegisterError)?;
        
        // Generate the allocation instruction
        let instructions = vec![
            Instruction::Alloc {
                type_reg: result_register, // Simplified - in real implementation would be separate
                init_reg: result_register,  // Simplified - in real implementation would be separate
                output_reg: result_register,
            }
        ];
        
        Ok(ChannelOperationResult {
            result_register,
            consumed_resources: vec![],
            allocated_resources: vec![resource_id],
            instructions,
        })
    }
    
    /// Send a value through a channel (as resource operation)
    pub fn send_channel_resource(
        &mut self,
        channel_register: RegisterId,
        value_register: RegisterId,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Get the channel resource ID from the register
        let channel_resource_id = self.register_file.read_register(channel_register)
            .map_err(ChannelResourceError::RegisterError)?
            .ok_or_else(|| ChannelResourceError::ChannelNotFound(ResourceId::new(0)))?;
        
        // Get the value from the value register
        let _value_resource_id = self.register_file.read_register(value_register)
            .map_err(ChannelResourceError::RegisterError)?
            .ok_or_else(|| ChannelResourceError::LinearViolation(
                "Value register is empty".to_string()
            ))?;
        
        // Validate that the channel resource exists and is a channel
        let channel_resource = self.resource_manager.peek(&channel_resource_id)
            .map_err(ChannelResourceError::ResourceError)?;
        
        if !matches!(channel_resource, MachineValue::Channel(_)) {
            return Err(ChannelResourceError::SessionTypeMismatch(
                "Resource is not a channel".to_string()
            ));
        }
        
        // Allocate a register for the result (updated channel)
        let result_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Generate the transform instruction (send operation)
        let instructions = vec![
            Instruction::Transform {
                morph_reg: channel_register, // Channel acts as morphism
                input_reg: value_register,   // Value to send
                output_reg: result_register, // Updated channel
            }
        ];
        
        Ok(ChannelOperationResult {
            result_register,
            consumed_resources: vec![], // Channel is updated, not consumed (unless it reaches End)
            allocated_resources: vec![],
            instructions,
        })
    }
    
    /// Receive a value from a channel (as resource operation)
    pub fn receive_channel_resource(
        &mut self,
        channel_register: RegisterId,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Get the channel resource ID from the register
        let channel_resource_id = self.register_file.read_register(channel_register)
            .map_err(ChannelResourceError::RegisterError)?
            .ok_or_else(|| ChannelResourceError::ChannelNotFound(ResourceId::new(0)))?;
        
        // Validate that the channel resource exists and is a channel
        let channel_resource = self.resource_manager.peek(&channel_resource_id)
            .map_err(ChannelResourceError::ResourceError)?;
        
        if !matches!(channel_resource, MachineValue::Channel(_)) {
            return Err(ChannelResourceError::SessionTypeMismatch(
                "Resource is not a channel".to_string()
            ));
        }
        
        // Allocate registers for the received value and updated channel
        let value_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        let _updated_channel_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Generate the transform instruction (receive operation)
        // In this case, the channel transforms itself to produce a value and updated channel
        let instructions = vec![
            Instruction::Transform {
                morph_reg: channel_register,        // Channel acts as morphism
                input_reg: channel_register,        // Channel is also input (self-transformation)
                output_reg: value_register,         // Received value
            },
            // The updated channel would be handled by the transform operation
        ];
        
        Ok(ChannelOperationResult {
            result_register: value_register,
            consumed_resources: vec![],
            allocated_resources: vec![],
            instructions,
        })
    }
    
    /// Close a channel (consume as resource)
    pub fn close_channel_resource(
        &mut self,
        channel_register: RegisterId,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Get the channel resource ID from the register
        let channel_resource_id = self.register_file.read_register(channel_register)
            .map_err(ChannelResourceError::RegisterError)?
            .ok_or_else(|| ChannelResourceError::ChannelNotFound(ResourceId::new(0)))?;
        
        // Allocate a register for the final value
        let result_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Generate the consume instruction
        let instructions = vec![
            Instruction::Consume {
                resource_reg: channel_register,
                output_reg: result_register,
            }
        ];
        
        Ok(ChannelOperationResult {
            result_register,
            consumed_resources: vec![channel_resource_id],
            allocated_resources: vec![],
            instructions,
        })
    }
    
    /// Create a channel pair (dual channels for bidirectional communication)
    pub fn create_channel_pair(
        &mut self,
        session_type: SessionType,
        location: Location,
        det_sys: &mut DeterministicSystem,
    ) -> Result<(ChannelOperationResult, ChannelOperationResult), ChannelResourceError> {
        // Create the first channel with the given session type
        let channel1_result = self.create_channel_resource(
            session_type.clone(),
            location.clone(),
            det_sys,
        )?;
        
        // Create the second channel with the dual session type
        let dual_session_type = session_type.dual();
        let channel2_result = self.create_channel_resource(
            dual_session_type,
            location,
            det_sys,
        )?;
        
        Ok((channel1_result, channel2_result))
    }
    
    /// Compose two channel operations (sequential composition)
    pub fn compose_channel_operations(
        &mut self,
        first_channel_reg: RegisterId,
        second_channel_reg: RegisterId,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Allocate a register for the composed result
        let result_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Generate the compose instruction
        let instructions = vec![
            Instruction::Compose {
                first_reg: first_channel_reg,
                second_reg: second_channel_reg,
                output_reg: result_register,
            }
        ];
        
        Ok(ChannelOperationResult {
            result_register,
            consumed_resources: vec![],
            allocated_resources: vec![],
            instructions,
        })
    }
    
    /// Parallel composition of channels (tensor product)
    pub fn tensor_channels(
        &mut self,
        left_channel_reg: RegisterId,
        right_channel_reg: RegisterId,
        det_sys: &mut DeterministicSystem,
    ) -> Result<ChannelOperationResult, ChannelResourceError> {
        // Allocate a register for the tensor result
        let result_register = self.register_file.allocate_register(det_sys)
            .ok_or(ChannelResourceError::RegisterError(
                RegisterFileError::NoRegistersAvailable
            ))?;
        
        // Generate the tensor instruction
        let instructions = vec![
            Instruction::Tensor {
                left_reg: left_channel_reg,
                right_reg: right_channel_reg,
                output_reg: result_register,
            }
        ];
        
        Ok(ChannelOperationResult {
            result_register,
            consumed_resources: vec![],
            allocated_resources: vec![],
            instructions,
        })
    }
    
    /// Get resource statistics
    pub fn get_resource_stats(&self) -> ChannelResourceStats {
        ChannelResourceStats {
            total_resources: self.resource_manager.resource_count(),
            allocated_registers: self.register_file.allocated_count(),
            available_registers: self.register_file.available_count(),
            memory_usage: self.resource_manager.total_memory(),
        }
    }
    
    /// Validate that all channels follow linear discipline
    pub fn validate_channel_linearity(&self) -> Result<(), ChannelResourceError> {
        // Check that no channels are leaked (all channels should be either:
        // 1. Still active with non-End session types
        // 2. Properly consumed when reaching End
        
        // For now, this is a placeholder validation
        // In a full implementation, this would check the session types
        // and ensure proper consumption
        Ok(())
    }
}

impl Default for ChannelResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about channel resource usage
#[derive(Debug, Clone)]
pub struct ChannelResourceStats {
    /// Total number of resources
    pub total_resources: usize,
    
    /// Number of allocated registers
    pub allocated_registers: usize,
    
    /// Number of available registers
    pub available_registers: usize,
    
    /// Total memory usage
    pub memory_usage: u64,
}

impl std::fmt::Display for ChannelResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelResourceError::ResourceError(e) => write!(f, "Resource error: {}", e),
            ChannelResourceError::RegisterError(e) => write!(f, "Register error: {}", e),
            ChannelResourceError::ChannelNotFound(id) => write!(f, "Channel not found: {:?}", id),
            ChannelResourceError::SessionTypeMismatch(msg) => write!(f, "Session type mismatch: {}", msg),
            ChannelResourceError::LinearViolation(msg) => write!(f, "Linear violation: {}", msg),
        }
    }
}

impl std::error::Error for ChannelResourceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::{TypeInner, BaseType};

    #[test]
    fn test_create_channel_resource() {
        let mut manager = ChannelResourceManager::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let result = manager.create_channel_resource(
            session_type,
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        assert_eq!(result.allocated_resources.len(), 1);
        assert_eq!(result.consumed_resources.len(), 0);
        assert_eq!(result.instructions.len(), 1);
        assert!(matches!(result.instructions[0], Instruction::Alloc { .. }));
    }
    
    #[test]
    fn test_channel_pair_creation() {
        let mut manager = ChannelResourceManager::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let (result1, result2) = manager.create_channel_pair(
            session_type,
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        assert_eq!(result1.allocated_resources.len(), 1);
        assert_eq!(result2.allocated_resources.len(), 1);
        assert_ne!(result1.result_register, result2.result_register);
    }
    
    #[test]
    fn test_channel_operations_as_instructions() {
        let mut manager = ChannelResourceManager::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Create a channel
        let channel_result = manager.create_channel_resource(
            session_type,
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        // Create a value resource to send
        let value_to_send = MachineValue::Int(42);
        let value_resource_id = manager.resource_manager.allocate(
            MachineValue::Type(TypeInner::Base(BaseType::Int)),
            value_to_send
        );
        
        // Create a value register and put the value resource ID in it
        let value_register = manager.register_file.allocate_register(&mut det_sys).unwrap();
        manager.register_file.write_register(value_register, Some(value_resource_id)).unwrap();
        
        // Send operation
        let send_result = manager.send_channel_resource(
            channel_result.result_register,
            value_register,
            &mut det_sys,
        ).unwrap();
        
        assert_eq!(send_result.instructions.len(), 1);
        assert!(matches!(send_result.instructions[0], Instruction::Transform { .. }));
    }
    
    #[test]
    fn test_channel_composition() {
        let mut manager = ChannelResourceManager::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Create two channels
        let channel1 = manager.create_channel_resource(
            session_type.clone(),
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        let channel2 = manager.create_channel_resource(
            session_type,
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        // Compose them
        let compose_result = manager.compose_channel_operations(
            channel1.result_register,
            channel2.result_register,
            &mut det_sys,
        ).unwrap();
        
        assert_eq!(compose_result.instructions.len(), 1);
        assert!(matches!(compose_result.instructions[0], Instruction::Compose { .. }));
    }
    
    #[test]
    fn test_resource_stats() {
        let mut manager = ChannelResourceManager::new();
        let mut det_sys = DeterministicSystem::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let initial_stats = manager.get_resource_stats();
        assert_eq!(initial_stats.total_resources, 0);
        
        // Create a channel
        manager.create_channel_resource(
            session_type,
            Location::Local,
            &mut det_sys,
        ).unwrap();
        
        let final_stats = manager.get_resource_stats();
        assert_eq!(final_stats.total_resources, 1);
        assert!(final_stats.allocated_registers > 0);
    }
} 