// VM integration for TEL resources
// Original file: src/tel/resource/vm.rs

// Resource VM integration module for TEL
//
// This module implements the integration between the resource
// management system and the VM system, allowing resources to be
// used in VM operations.
// Migration note: This file has been migrated to use the unified ResourceRegister model.

use std::sync::Arc;
use std::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use crypto::{
    hash::{ContentId, HashError, HashFactory, HashOutput},
    ContentAddressed,
};

// Import from our TEL module
use crate::tel::{
    error::{TelError, TelResult},
    types::{Domain, Address},
    resource::{
        ResourceManager,
    },
};

// Import from resource module for unified model
use crate::resource::{
    ResourceRegister, 
    RegisterState,
    StorageStrategy,
};

/// A VM register ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct VmRegId(pub ContentId);

impl VmRegId {
    /// Create a new VM register ID
    pub fn new() -> Self {
        // Generate a unique string based on the current time to hash
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
            
        let reg_data = format!("vm-register-{}", now);
        
        // Generate a content ID
        let hasher = HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(reg_data.as_bytes());
        let content_id = ContentId::from(hash);
        
        // Create VM register ID from the content_id
        Self(content_id)
    }
    
    /// Create from a ContentId
    pub fn from_content_id(content_id: &ContentId) -> Self {
        Self(*content_id)
    }
}

impl ContentAddressed for VmRegId {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let bytes = self.0.hash().as_bytes();
        hasher.hash(bytes)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.0.hash().as_bytes().to_vec()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        if bytes.len() < 16 {
            return Err(HashError::InvalidLength);
        }
        
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&bytes[..16]);
        
        Ok(Self(ContentId::from(uuid_bytes)))
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
            resource_memory_section: "resource_data",
        }
    }
}

/// Manages memory for VM operations
#[derive(Debug)]
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
    
    /// Create a register in memory
    pub fn create_register(&mut self, id: &VmRegId, section: &str, data: Vec<u8>) -> TelResult<()> {
        let register = VmRegister {
            id: id.clone(),
            section: section.to_string(),
            data,
        };
        
        self.registers.insert(id.clone(), register);
        
        self.sections
            .entry(section.to_string())
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        Ok(())
    }
    
    /// Get a register from memory
    pub fn get_register(&self, id: &VmRegId) -> TelResult<Option<VmRegister>> {
        Ok(self.registers.get(id).cloned())
    }
    
    /// Get all registers in a section
    pub fn get_section_registers(&self, section: &str) -> TelResult<Vec<VmRegId>> {
        Ok(self.sections.get(section).cloned().unwrap_or_default())
    }
    
    /// Delete a register from memory
    pub fn delete_register(&mut self, id: &VmRegId) -> TelResult<bool> {
        if let Some(register) = self.registers.remove(id) {
            // Remove from section as well
            if let Some(section_registers) = self.sections.get_mut(&register.section) {
                section_registers.retain(|reg_id| reg_id != id);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Integrates resource management with the VM
#[derive(Debug)]
pub struct ResourceVmIntegration {
    /// The resource manager
    resource_manager: Arc<ResourceManager>,
    /// The VM memory manager
    memory_manager: MemoryManager,
    /// Configuration
    config: VmIntegrationConfig,
    /// Register mappings for active execution contexts
    /// Maps (execution context ID, resource register ID) -> VM register ID
    register_mappings: HashMap<(String, ContentId), VmRegId>,
}

impl ResourceVmIntegration {
    /// Create a new resource VM integration
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
    
    /// Create with default configuration
    pub fn default(
        resource_manager: Arc<ResourceManager>,
    ) -> Self {
        Self::new(
            resource_manager,
            MemoryManager::new(),
            VmIntegrationConfig::default(),
        )
    }
    
    /// Load a resource into VM memory
    pub fn load_resource(
        &mut self,
        resource_id: &ContentId,
        ctx: &ExecutionContext,
        initiator: &Address,
    ) -> TelResult<VmRegId> {
        // Check if this resource is already loaded in this context
        let mapping_key = (ctx.id.clone(), *resource_id);
        if let Some(vm_reg_id) = self.register_mappings.get(&mapping_key) {
            return Ok(vm_reg_id.clone());
        }
        
        // Get the resource register from the resource manager
        let resource_register = self.resource_manager.get_resource_register(resource_id)?;
        
        // Check access permissions
        let access_result = self.check_resource_access(
            &resource_register,
            ctx,
            initiator,
            AccessIntent::Read,
        );
        
        if let ResourceAccessResult::Success = access_result {
            // Map the resource to a VM register
            let vm_reg_id = self.map_resource_to_vm(&resource_register, ctx)?;
            
            Ok(vm_reg_id)
        } else {
            // Convert access result to TelError
            let error_msg: Result<(), String> = access_result.into();
            Err(TelError::AccessDenied(error_msg.unwrap_err()))
        }
    }
    
    /// Store a VM register back to a resource
    pub fn store_resource(
        &mut self,
        vm_reg_id: &VmRegId,
        ctx: &ExecutionContext,
        initiator: &Address,
    ) -> TelResult<()> {
        // Get the VM register
        let vm_register = self.memory_manager.get_register(vm_reg_id)?
            .ok_or_else(|| TelError::ResourceError(
                format!("VM register {:?} not found", vm_reg_id)
            ))?;
        
        // Find the resource ID for this VM register
        let resource_id = self.register_mappings.iter()
            .find_map(|((ctx_id, res_id), vm_id)| {
                if ctx_id == &ctx.id && vm_id == vm_reg_id {
                    Some(res_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| TelError::ResourceError(
                format!("No resource mapped to VM register {:?}", vm_reg_id)
            ))?;
        
        // Get the current resource register
        let mut resource_register = self.resource_manager.get_resource_register(resource_id)?;
        
        // Check write access
        let access_result = self.check_resource_access(
            &resource_register,
            ctx,
            initiator,
            AccessIntent::Write,
        );
        
        if let ResourceAccessResult::Success = access_result {
            // Convert VM register data to resource contents
            let contents = self.vm_register_to_resource_contents(&vm_register)?;
            
            // Update the resource content
            resource_register.update_contents(contents);
            
            // Store the updated resource
            self.resource_manager.update_resource_register(resource_id, resource_register)?;
            
            Ok(())
        } else {
            // Convert access result to TelError
            let error_msg: Result<(), String> = access_result.into();
            Err(TelError::AccessDenied(error_msg.unwrap_err()))
        }
    }
    
    /// Commit all changes in an execution context
    pub fn commit_context(&mut self, ctx: &ExecutionContext) -> TelResult<()> {
        // Find all resources mapped in this context
        let resources: Vec<_> = self.register_mappings.iter()
            .filter_map(|((ctx_id, res_id), vm_id)| {
                if ctx_id == &ctx.id {
                    Some((res_id, vm_id))
                } else {
                    None
                }
            })
            .collect();
        
        // Store all VM registers back to resources
        for (resource_id, vm_reg_id) in resources {
            // Skip if we can't find the VM register
            if let Ok(Some(vm_register)) = self.memory_manager.get_register(vm_reg_id) {
                // Try to get the resource register
                if let Ok(mut resource_register) = self.resource_manager.get_resource_register(resource_id) {
                    // Only update if the resource is active
                    if resource_register.is_active() {
                        // Convert VM register data to resource contents
                        if let Ok(contents) = self.vm_register_to_resource_contents(&vm_register) {
                            // Update the resource content
                            resource_register.update_contents(contents);
                            
                            // Ignore errors during batch commit
                            let _ = self.resource_manager.update_resource_register(resource_id, resource_register);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Cleanup resources used in an execution context
    pub fn cleanup_context(&mut self, ctx: &ExecutionContext) -> TelResult<()> {
        // Optionally commit changes before cleanup
        if self.config.auto_commit_on_exit {
            self.commit_context(ctx)?;
        }
        
        // Remove all mappings for this context
        self.register_mappings.retain(|(ctx_id, _), _| ctx_id != &ctx.id);
        
        Ok(())
    }
    
    /// Map a resource register to a VM register
    fn map_resource_to_vm(
        &mut self,
        register: &ResourceRegister,
        ctx: &ExecutionContext,
    ) -> TelResult<VmRegId> {
        // Create a new VM register ID
        let vm_reg_id = VmRegId::new();
        
        // Convert resource contents to VM data
        let vm_data = self.resource_contents_to_vm_data(register)?;
        
        // Create a VM register in memory
        self.memory_manager.create_register(
            &vm_reg_id,
            &self.config.resource_memory_section,
            vm_data,
        )?;
        
        // Store the mapping
        self.register_mappings.insert(
            (ctx.id.clone(), register.id),
            vm_reg_id.clone(),
        );
        
        Ok(vm_reg_id)
    }
    
    /// Convert resource contents to VM data
    fn resource_contents_to_vm_data(&self, register: &ResourceRegister) -> TelResult<Vec<u8>> {
        // For now, just use the resource's contents directly
        Ok(register.contents.clone())
    }
    
    /// Convert VM register data back to resource contents
    fn vm_register_to_resource_contents(&self, vm_register: &VmRegister) -> TelResult<Vec<u8>> {
        // For now, just use the VM register's data directly
        Ok(vm_register.data.clone())
    }
    
    /// Check access permissions for a resource
    fn check_resource_access(
        &self,
        register: &ResourceRegister,
        ctx: &ExecutionContext,
        initiator: &Address,
        intent: AccessIntent,
    ) -> ResourceAccessResult {
        // Check if the resource exists and is active
        if !register.is_active() {
            // Check the state and return appropriate error
            if register.is_consumed() {
                return ResourceAccessResult::InvalidState(
                    "Resource has been consumed".to_string()
                );
            } else if register.is_locked() {
                return ResourceAccessResult::InvalidState(
                    "Resource is locked".to_string()
                );
            } else if register.is_frozen() {
                return ResourceAccessResult::InvalidState(
                    "Resource is frozen".to_string()
                );
            } else {
                return ResourceAccessResult::InvalidState(
                    format!("Resource is in invalid state: {:?}", register.state)
                );
            }
        }
        
        // Check time access if configured
        if self.config.validate_time_access {
            // Note: Time validation would go here
            // For now, assume all time accesses are valid
        }
        
        // Check domain access
        let metadata_val = register.metadata.get("tel_domain");
        if let Some(domain_value) = metadata_val {
            if let Some(domain_str) = domain_value.as_str() {
                if domain_str != ctx.domain {
                    return ResourceAccessResult::AccessDenied(
                        format!("Domain mismatch: {} vs {}", domain_str, ctx.domain)
                    );
                }
            }
        }
        
        // Check owner access for write operations
        if matches!(intent, AccessIntent::Write | AccessIntent::Execute) {
            let owner_val = register.metadata.get("tel_owner");
            if let Some(owner_value) = owner_val {
                if let Some(owner_str) = owner_value.as_str() {
                    if owner_str != initiator {
                        return ResourceAccessResult::AccessDenied(
                            format!("Not the owner: {} vs {}", owner_str, initiator)
                        );
                    }
                }
            }
        }
        
        // If we reach here, access is granted
        ResourceAccessResult::Success
    }
}

/// Intent for accessing a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessIntent {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
} 
