// Resource VM integration module for TEL
//
// This module implements the integration between the resource
// management system and the VM system, allowing resources to be
// used in VM operations.

use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

// Import from our TEL module
use crate::tel::{
    error::{TelError, TelResult},
    types::{ResourceId, Domain, Address},
    resource::{
        ResourceManager, 
        Register, 
        RegisterId, 
        RegisterContents,
        RegisterState,
    },
};

/// A VM register ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VmRegId(pub Uuid);

impl VmRegId {
    /// Create a new VM register ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A VM register
#[derive(Debug, Clone)]
pub struct VmRegister {
    /// ID of the register
    pub id: VmRegId,
    /// Memory section the register belongs to
    pub section: String,
    /// Data contained in the register
    pub data: Vec<u8>,
}

/// An execution context for VM operations
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// ID of the context
    pub id: String,
    /// Initiator address
    pub initiator: Address,
    /// Domain of execution
    pub domain: Domain,
}

/// Result of a resource access operation
pub enum ResourceAccessResult {
    /// Resource access was successful
    Success,
    /// Resource access was denied
    AccessDenied(String),
    /// Resource does not exist
    NotFound,
    /// Resource is in an invalid state for the operation
    InvalidState(String),
    /// Internal error during resource access
    InternalError(String),
}

impl From<ResourceAccessResult> for Result<(), String> {
    fn from(result: ResourceAccessResult) -> Self {
        match result {
            ResourceAccessResult::Success => Ok(()),
            ResourceAccessResult::AccessDenied(reason) => Err(format!("Access denied: {}", reason)),
            ResourceAccessResult::NotFound => Err("Resource not found".to_string()),
            ResourceAccessResult::InvalidState(reason) => Err(format!("Invalid state: {}", reason)),
            ResourceAccessResult::InternalError(reason) => Err(format!("Internal error: {}", reason)),
        }
    }
}

/// Configuration for VM resource integration
#[derive(Debug, Clone)]
pub struct VmIntegrationConfig {
    /// Maximum registers per execution context
    pub max_registers_per_context: usize,
    /// Whether to auto-commit changes on context exit
    pub auto_commit_on_exit: bool,
    /// Whether to validate resource access against time system
    pub validate_time_access: bool,
    /// Memory section for resource data
    pub resource_memory_section: String,
}

impl Default for VmIntegrationConfig {
    fn default() -> Self {
        Self {
            max_registers_per_context: 1000,
            auto_commit_on_exit: true,
            validate_time_access: true,
            resource_memory_section: "resource_data".to_string(),
        }
    }
}

/// A memory manager for managing VM registers
pub struct MemoryManager {
    /// Registers by ID
    registers: HashMap<VmRegId, VmRegister>,
    /// Sections in memory
    sections: HashMap<String, Vec<VmRegId>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            sections: HashMap::new(),
        }
    }
    
    /// Create a new register
    pub fn create_register(&mut self, id: &VmRegId, section: &str, data: Vec<u8>) -> TelResult<()> {
        let register = VmRegister {
            id: id.clone(),
            section: section.to_string(),
            data,
        };
        
        // Store the register
        self.registers.insert(id.clone(), register);
        
        // Add to section
        self.sections
            .entry(section.to_string())
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        Ok(())
    }
    
    /// Get a register by ID
    pub fn get_register(&self, id: &VmRegId) -> TelResult<Option<VmRegister>> {
        Ok(self.registers.get(id).cloned())
    }
    
    /// Get all registers in a section
    pub fn get_section_registers(&self, section: &str) -> TelResult<Vec<VmRegId>> {
        Ok(self.sections.get(section).cloned().unwrap_or_default())
    }
    
    /// Delete a register
    pub fn delete_register(&mut self, id: &VmRegId) -> TelResult<bool> {
        if let Some(register) = self.registers.remove(id) {
            // Remove from section
            if let Some(section_registers) = self.sections.get_mut(&register.section) {
                section_registers.retain(|reg_id| reg_id != id);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Integrates the resource system with the VM
pub struct ResourceVmIntegration {
    /// The resource manager
    resource_manager: Arc<ResourceManager>,
    /// The VM memory manager
    memory_manager: MemoryManager,
    /// Configuration
    config: VmIntegrationConfig,
    /// Register mappings for active execution contexts
    /// Maps (execution context ID, resource register ID) -> VM register ID
    register_mappings: HashMap<(String, RegisterId), VmRegId>,
}

impl ResourceVmIntegration {
    /// Create a new VM integration
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        memory_manager: MemoryManager,
        config: VmIntegrationConfig,
    ) -> Self {
        Self {
            resource_manager,
            memory_manager,
            config,
            register_mappings: HashMap::new(),
        }
    }
    
    /// Create a new VM integration with default configuration
    pub fn default(
        resource_manager: Arc<ResourceManager>,
    ) -> Self {
        Self::new(
            resource_manager,
            MemoryManager::new(),
            VmIntegrationConfig::default(),
        )
    }
    
    /// Load a resource into the VM
    pub fn load_resource(
        &mut self,
        resource_id: &ResourceId,
        ctx: &ExecutionContext,
        initiator: &Address,
    ) -> TelResult<VmRegId> {
        // Get the resource register from the resource manager
        let register = self.resource_manager.get_register(resource_id)?;
        
        // Check access control
        let access_result = self.check_resource_access(
            &register,
            ctx,
            initiator,
            AccessIntent::Read,
        );
        
        if let ResourceAccessResult::Success = access_result {
            // Map the resource contents to VM memory
            let vm_reg_id = self.map_resource_to_vm(&register, ctx)?;
            
            // Store the mapping
            self.register_mappings.insert((ctx.id.clone(), register.id.clone()), vm_reg_id.clone());
            
            Ok(vm_reg_id)
        } else {
            Err(TelError::AuthorizationError(format!(
                "Unable to load resource {}: {:?}", 
                resource_id, 
                access_result
            )))
        }
    }
    
    /// Store a VM register back to a resource
    pub fn store_resource(
        &mut self,
        vm_reg_id: &VmRegId,
        ctx: &ExecutionContext,
        initiator: &Address,
    ) -> TelResult<()> {
        // Find the register mapping
        let register_id = self.register_mappings.iter()
            .find_map(|((ctx_id, reg_id), vm_id)| {
                if ctx_id == &ctx.id && vm_id == vm_reg_id {
                    Some(reg_id.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| TelError::InternalError(
                format!("No resource mapping found for VM register {:?}", vm_reg_id)
            ))?;
        
        // Get the register
        let register = self.resource_manager.get_register_by_id(&register_id)?
            .ok_or_else(|| TelError::ResourceError(format!("Register not found: {:?}", register_id)))?;
        
        // Check access control
        let access_result = self.check_resource_access(
            &register,
            ctx,
            initiator,
            AccessIntent::Write,
        );
        
        if let ResourceAccessResult::Success = access_result {
            // Get the VM register data
            let vm_register = self.memory_manager.get_register(vm_reg_id)?
                .ok_or_else(|| TelError::InternalError(
                    format!("VM register {:?} not found", vm_reg_id)
                ))?;
            
            // Update the resource with the VM register data
            let new_contents = self.vm_register_to_resource_contents(&vm_register)?;
            
            // Update the register in the resource manager
            self.resource_manager.update_register(
                &register_id,
                new_contents,
            )?;
            
            Ok(())
        } else {
            Err(TelError::AuthorizationError(format!(
                "Unable to store resource for register {:?}: {:?}", 
                register_id, 
                access_result
            )))
        }
    }
    
    /// Commit all changes for a specific execution context
    pub fn commit_context(&mut self, ctx: &ExecutionContext) -> TelResult<()> {
        // Find all mappings for this context
        let ctx_mappings: Vec<_> = self.register_mappings.iter()
            .filter(|((ctx_id, _), _)| ctx_id == &ctx.id)
            .map(|((_, reg_id), vm_id)| (reg_id.clone(), vm_id.clone()))
            .collect();
        
        // Store all mapped resources
        for (reg_id, vm_id) in ctx_mappings {
            let register = self.resource_manager.get_register_by_id(&reg_id)?
                .ok_or_else(|| TelError::InternalError(
                    format!("Register {:?} not found during commit", reg_id)
                ))?;
                
            let vm_register = self.memory_manager.get_register(&vm_id)?
                .ok_or_else(|| TelError::InternalError(
                    format!("VM register {:?} not found during commit", vm_id)
                ))?;
                
            let new_contents = self.vm_register_to_resource_contents(&vm_register)?;
            
            // Update the register
            self.resource_manager.update_register(
                &reg_id,
                new_contents,
            )?;
        }
        
        // Clean up mappings for this context
        self.register_mappings.retain(|(ctx_id, _), _| ctx_id != &ctx.id);
        
        Ok(())
    }
    
    /// Clean up resources for a context without committing changes
    pub fn cleanup_context(&mut self, ctx: &ExecutionContext) -> TelResult<()> {
        // Clean up mappings for this context
        self.register_mappings.retain(|(ctx_id, _), _| ctx_id != &ctx.id);
        
        Ok(())
    }
    
    /// Map a resource register to a VM register
    fn map_resource_to_vm(
        &mut self,
        register: &Register,
        ctx: &ExecutionContext,
    ) -> TelResult<VmRegId> {
        // Create a VM register ID
        let vm_reg_id = VmRegId::new();
        
        // Convert resource contents to VM-compatible format
        let vm_data = self.resource_contents_to_vm_data(&register.contents)?;
        
        // Create a VM register
        self.memory_manager.create_register(
            &vm_reg_id,
            &self.config.resource_memory_section,
            vm_data,
        )?;
        
        Ok(vm_reg_id)
    }
    
    /// Convert resource contents to VM-compatible data
    fn resource_contents_to_vm_data(&self, contents: &RegisterContents) -> TelResult<Vec<u8>> {
        // In a real implementation, this would serialize the resource contents
        // in a format suitable for the VM system
        
        // For the purposes of this implementation, we'll use a simple conversion
        match contents {
            RegisterContents::Binary(data) => Ok(data.clone()),
            RegisterContents::Json(json) => Ok(json.to_string().into_bytes()),
            RegisterContents::Text(text) => Ok(text.clone().into_bytes()),
            RegisterContents::Empty => Ok(vec![]),
            _ => Err(TelError::InternalError(
                format!("Unsupported resource content type: {:?}", contents)
            )),
        }
    }
    
    /// Convert VM register data to resource contents
    fn vm_register_to_resource_contents(&self, vm_register: &VmRegister) -> TelResult<RegisterContents> {
        // In a real implementation, this would deserialize the VM data
        // into the appropriate resource content type
        
        // For the purposes of this implementation, use a simple conversion
        // Assume binary data
        Ok(RegisterContents::Binary(vm_register.data.clone()))
    }
    
    /// Check resource access against various security policies
    fn check_resource_access(
        &self,
        register: &Register,
        ctx: &ExecutionContext,
        initiator: &Address,
        intent: AccessIntent,
    ) -> ResourceAccessResult {
        // Check if register is owned by the initiator
        if register.owner != *initiator {
            // Not the owner, check ACL
            // For now, just deny
            return ResourceAccessResult::AccessDenied(
                "Initiator is not the resource owner".to_string()
            );
        }
        
        // Check register status
        match register.state {
            // Allow access to active registers
            RegisterState::Active => {},
            // Deny access to registers with other statuses
            RegisterState::Locked => {
                return ResourceAccessResult::InvalidState(
                    "Register is locked".to_string()
                );
            },
            RegisterState::Frozen => {
                return ResourceAccessResult::InvalidState(
                    "Register is frozen".to_string()
                );
            },
            RegisterState::PendingDeletion => {
                return ResourceAccessResult::InvalidState(
                    "Register is pending deletion".to_string()
                );
            },
            RegisterState::Tombstone => {
                return ResourceAccessResult::InvalidState(
                    "Register is tombstoned".to_string()
                );
            },
        }
        
        // Check time validity
        if self.config.validate_time_access {
            // Implement time-based access control here
            // This would check if the current time in the execution context
            // allows access to this resource
            // For now, just allow
        }
        
        // All checks passed
        ResourceAccessResult::Success
    }
}

/// Intent for resource access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessIntent {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
} 