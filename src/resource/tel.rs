// TEL Resource Integration
//
// Direct integration between the one-time register system and TEL resources.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};
use crate::types::{Address, Domain};
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterState, RegisterOperation, OperationType,
    OneTimeRegisterSystem, RegisterResult, RegisterError, TimeMap, TimeMapEntry
};
use crate::domain::{DomainId, DomainRegistry};

// TEL resource types
use crate::tel::resource::model::{
    Register as TelRegister, 
    RegisterId as TelRegisterId,
    ResourceManager as TelResourceManager
};
use crate::tel::types::{ResourceId, Address as TelAddress, Domain as TelDomain};
use crate::tel::resource::operations::{ResourceOperation, ResourceOperationType};

/// Mapping between TEL resources and register system resources
#[derive(Default)]
pub struct TelResourceMapping {
    /// Mapping from TEL resource IDs to register IDs
    tel_to_register: RwLock<HashMap<ResourceId, RegisterId>>,
    
    /// Mapping from register IDs to TEL resource IDs
    register_to_tel: RwLock<HashMap<RegisterId, ResourceId>>,
}

impl TelResourceMapping {
    /// Create a new resource mapping
    pub fn new() -> Self {
        Self {
            tel_to_register: RwLock::new(HashMap::new()),
            register_to_tel: RwLock::new(HashMap::new()),
        }
    }
    
    /// Map a TEL resource ID to a register ID
    pub fn map_resource(&self, tel_id: ResourceId, register_id: RegisterId) -> Result<()> {
        let mut tel_to_register = self.tel_to_register.write()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        let mut register_to_tel = self.register_to_tel.write()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        tel_to_register.insert(tel_id.clone(), register_id.clone());
        register_to_tel.insert(register_id, tel_id);
        
        Ok(())
    }
    
    /// Get the register ID for a TEL resource ID
    pub fn get_register_id(&self, tel_id: &ResourceId) -> Result<Option<RegisterId>> {
        let tel_to_register = self.tel_to_register.read()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        Ok(tel_to_register.get(tel_id).cloned())
    }
    
    /// Get the TEL resource ID for a register ID
    pub fn get_tel_id(&self, register_id: &RegisterId) -> Result<Option<ResourceId>> {
        let register_to_tel = self.register_to_tel.read()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        Ok(register_to_tel.get(register_id).cloned())
    }
    
    /// Remove a mapping
    pub fn remove_mapping(&self, tel_id: &ResourceId, register_id: &RegisterId) -> Result<()> {
        let mut tel_to_register = self.tel_to_register.write()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        let mut register_to_tel = self.register_to_tel.write()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        tel_to_register.remove(tel_id);
        register_to_tel.remove(register_id);
        
        Ok(())
    }
    
    /// Get all TEL resource IDs
    pub fn get_all_tel_ids(&self) -> Result<HashSet<ResourceId>> {
        let tel_to_register = self.tel_to_register.read()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        Ok(tel_to_register.keys().cloned().collect())
    }
    
    /// Get all register IDs
    pub fn get_all_register_ids(&self) -> Result<HashSet<RegisterId>> {
        let register_to_tel = self.register_to_tel.read()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        Ok(register_to_tel.keys().cloned().collect())
    }
}

/// Adapter for working with TEL resources directly through the register system
pub struct TelResourceAdapter {
    /// The one-time register system
    register_system: Arc<OneTimeRegisterSystem>,
    
    /// TEL resource manager
    tel_resource_manager: Arc<TelResourceManager>,
    
    /// Resource mapping
    mapping: TelResourceMapping,
}

impl TelResourceAdapter {
    /// Create a new TEL resource adapter
    pub fn new(
        register_system: Arc<OneTimeRegisterSystem>,
        tel_resource_manager: Arc<TelResourceManager>
    ) -> Self {
        Self {
            register_system,
            tel_resource_manager,
            mapping: TelResourceMapping::new(),
        }
    }
    
    /// Convert a TEL register to our register format
    pub fn convert_tel_register_to_register(&self, tel_register: &TelRegister) -> Result<Register> {
        // Convert owner and domain
        let owner = Address::new(&tel_register.owner.to_string());
        let domain = Domain::new(&tel_register.domain.to_string());
        
        // Convert contents
        let contents = match &tel_register.contents {
            crate::tel::resource::model::RegisterContents::Binary(data) => {
                RegisterContents::with_binary(data.clone())
            },
            crate::tel::resource::model::RegisterContents::String(data) => {
                RegisterContents::with_string(data)
            },
            crate::tel::resource::model::RegisterContents::Json(data) => {
                RegisterContents::with_json(serde_json::to_string(data).unwrap_or_default())
            },
            crate::tel::resource::model::RegisterContents::Resource(resource) => {
                // Serialize resource to JSON
                let resource_json = serde_json::to_value(resource)
                    .map_err(|e| Error::SerializationError(format!("Failed to serialize resource: {}", e)))?;
                
                RegisterContents::with_json(serde_json::to_string(&resource_json).unwrap_or_default())
            },
            crate::tel::resource::model::RegisterContents::Empty => {
                RegisterContents::empty()
            },
        };
        
        // Create register ID based on TEL register ID
        let register_id = RegisterId::from_uuid(tel_register.id.0);
        
        // Create metadata from TEL register metadata
        let mut metadata = HashMap::new();
        for (k, v) in &tel_register.metadata {
            metadata.insert(k.clone(), v.to_string());
        }
        
        // Add TEL-specific metadata
        metadata.insert("tel_origin".to_string(), "true".to_string());
        
        // Create the register
        let register = Register::new(
            register_id,
            owner,
            domain,
            contents,
            metadata,
            tel_register.created_at as u64,
            tel_register.updated_at as u64,
        );
        
        Ok(register)
    }
    
    /// Convert our register to TEL register format
    pub fn convert_register_to_tel_contents(
        &self, 
        register: &Register
    ) -> Result<crate::tel::resource::model::RegisterContents> {
        match register.contents.as_binary() {
            Some(binary_data) => {
                Ok(crate::tel::resource::model::RegisterContents::Binary(binary_data.to_vec()))
            },
            None => {
                // Try as string
                if let Some(str_data) = register.contents.as_string() {
                    Ok(crate::tel::resource::model::RegisterContents::String(str_data.to_string()))
                } else {
                    // Try as JSON
                    if let Some(json_str) = register.contents.as_json() {
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                            Ok(crate::tel::resource::model::RegisterContents::Json(json_value))
                        } else {
                            // Default to string if JSON parsing fails
                            Ok(crate::tel::resource::model::RegisterContents::String(
                                register.contents.to_string()
                            ))
                        }
                    } else {
                        // Default to empty if we can't determine the type
                        Ok(crate::tel::resource::model::RegisterContents::Empty)
                    }
                }
            }
        }
    }
    
    /// Import a TEL register into the register system
    pub fn import_tel_register(&self, tel_id: &ResourceId) -> Result<RegisterId> {
        // Check if already imported
        if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
            return Ok(register_id);
        }
        
        // Get the TEL register
        let tel_register_id = TelRegisterId(uuid::Uuid::parse_str(&tel_id.to_string())
            .map_err(|e| Error::ParseError(format!("Invalid TEL register ID: {}", e)))?);
        
        let tel_register = self.tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        // Convert to our register format
        let register = self.convert_tel_register_to_register(&tel_register)?;
        
        // Import into register system
        let register_id = register.register_id.clone();
        self.register_system.import_register(register, "tel-import")
            .map_err(|e| Error::RegisterError(format!("Failed to import register: {}", e)))?;
        
        // Map the IDs
        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        Ok(register_id)
    }
    
    /// Export a register to TEL
    pub fn export_register_to_tel(&self, register_id: &RegisterId) -> Result<ResourceId> {
        // Check if already exported
        if let Some(tel_id) = self.mapping.get_tel_id(register_id)? {
            return Ok(tel_id);
        }
        
        // Get the register
        let register = self.register_system.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Convert register contents to TEL format
        let tel_contents = self.convert_register_to_tel_contents(&register)?;
        
        // Convert owner and domain to TEL format
        let tel_owner = TelAddress::from(register.owner.to_string());
        let tel_domain = TelDomain::from(register.domain.to_string());
        
        // Create the TEL register
        let tel_register_id = self.tel_resource_manager.create_register(
            tel_owner,
            tel_domain,
            tel_contents,
        ).map_err(|e| Error::ExternalError(format!("Failed to create TEL register: {}", e)))?;
        
        // Create resource ID from TEL register ID
        let tel_resource_id = ResourceId::from(tel_register_id.0.to_string());
        
        // Map the IDs
        self.mapping.map_resource(tel_resource_id.clone(), register_id.clone())?;
        
        Ok(tel_resource_id)
    }
    
    /// Sync a register with its TEL counterpart (from TEL to register)
    pub fn sync_from_tel(&self, tel_id: &ResourceId) -> Result<()> {
        // Get the register ID
        let register_id = self.mapping.get_register_id(tel_id)?
            .ok_or_else(|| Error::NotFound(format!("No mapping for TEL resource ID {}", tel_id)))?;
        
        // Get the TEL register
        let tel_register_id = TelRegisterId(uuid::Uuid::parse_str(&tel_id.to_string())
            .map_err(|e| Error::ParseError(format!("Invalid TEL register ID: {}", e)))?);
        
        let tel_register = self.tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        // Get the current register
        let current_register = self.register_system.get_register(&register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Only update if TEL register is newer
        if tel_register.updated_at > current_register.updated_at {
            // Convert to our register format
            let new_register = self.convert_tel_register_to_register(&tel_register)?;
            
            // Update register in system
            self.register_system.update_register(
                &register_id, 
                new_register, 
                "tel-sync"
            ).map_err(|e| Error::RegisterError(format!("Failed to update register: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Sync a TEL register with its register counterpart (from register to TEL)
    pub fn sync_to_tel(&self, register_id: &RegisterId) -> Result<()> {
        // Get the TEL resource ID
        let tel_id = self.mapping.get_tel_id(register_id)?
            .ok_or_else(|| Error::NotFound(format!("No mapping for register ID {}", register_id)))?;
        
        // Get the register
        let register = self.register_system.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Get the TEL register
        let tel_register_id = TelRegisterId(uuid::Uuid::parse_str(&tel_id.to_string())
            .map_err(|e| Error::ParseError(format!("Invalid TEL register ID: {}", e)))?);
        
        let tel_register = self.tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        // Only update if our register is newer
        if register.updated_at > tel_register.updated_at {
            // Convert register contents to TEL format
            let tel_contents = self.convert_register_to_tel_contents(&register)?;
            
            // Update TEL register
            self.tel_resource_manager.update_register(
                &tel_register_id,
                tel_contents,
            ).map_err(|e| Error::ExternalError(format!("Failed to update TEL register: {}", e)))?;
            
            // Update state if needed
            match register.state {
                RegisterState::Locked => {
                    if !tel_register.is_locked() {
                        self.tel_resource_manager.lock_register(&tel_register_id)
                            .map_err(|e| Error::ExternalError(format!("Failed to lock TEL register: {}", e)))?;
                    }
                },
                RegisterState::Active => {
                    if tel_register.is_locked() {
                        self.tel_resource_manager.unlock_register(&tel_register_id)
                            .map_err(|e| Error::ExternalError(format!("Failed to unlock TEL register: {}", e)))?;
                    }
                },
                RegisterState::Consumed | RegisterState::PendingGarbageCollection => {
                    self.tel_resource_manager.delete_register(&tel_register_id)
                        .map_err(|e| Error::ExternalError(format!("Failed to delete TEL register: {}", e)))?;
                },
                _ => {
                    // Other states don't have direct TEL equivalents
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a TEL resource operation through the register system
    pub fn process_tel_operation(
        &self,
        operation: &ResourceOperation
    ) -> Result<()> {
        // Get the register ID
        let tel_resource_id = operation.target.clone();
        
        let register_id = match self.mapping.get_register_id(&tel_resource_id)? {
            Some(id) => id,
            None => {
                // If register doesn't exist, import it
                self.import_tel_register(&tel_resource_id)?
            }
        };
        
        // Validate the operation
        self.validate_operation(&register_id, operation)?;
        
        // Process operation based on type
        match operation.operation_type {
            ResourceOperationType::Create => {
                // Already imported during register ID lookup
                Ok(())
            },
            ResourceOperationType::Update => {
                // Sync from TEL to get the latest changes
                self.sync_from_tel(&tel_resource_id)
            },
            ResourceOperationType::Delete => {
                // Consume the register
                self.register_system.consume_register_by_id(&register_id, "tel-operation", Vec::new())
                    .map_err(|e| Error::RegisterError(format!("Failed to consume register: {}", e)))?;
                
                // Remove the mapping
                self.mapping.remove_mapping(&tel_resource_id, &register_id)
            },
            ResourceOperationType::Transfer => {
                // Sync from TEL to get the owner change
                self.sync_from_tel(&tel_resource_id)
            },
            ResourceOperationType::Lock => {
                // Lock the register
                self.register_system.lock_register_by_id(&register_id, "tel-operation")
                    .map_err(|e| Error::RegisterError(format!("Failed to lock register: {}", e)))
            },
            ResourceOperationType::Unlock => {
                // Unlock the register
                self.register_system.unlock_register_by_id(&register_id, "tel-operation")
                    .map_err(|e| Error::RegisterError(format!("Failed to unlock register: {}", e)))
            },
        }
    }

    /// Convert TEL operation to Register operation
    pub fn convert_tel_to_register_operation(
        &self,
        tel_operation: &ResourceOperation
    ) -> Result<RegisterOperation> {
        let register_id = match self.mapping.get_register_id(&tel_operation.target)? {
            Some(id) => id,
            None => return Err(Error::NotFound(format!("Register not found for TEL resource {}", tel_operation.target)))
        };
        
        // Determine operation type
        let operation_type = match tel_operation.operation_type {
            ResourceOperationType::Create => OperationType::Create,
            ResourceOperationType::Update => OperationType::Update,
            ResourceOperationType::Delete => OperationType::Consume,
            ResourceOperationType::Transfer => OperationType::Update, // Transfer maps to update with new owner
            ResourceOperationType::Lock => OperationType::Lock,
            ResourceOperationType::Unlock => OperationType::Unlock,
        };
        
        // Create register operation
        let mut register_op = RegisterOperation {
            operation_type, 
            register_id,
            data: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Add operation-specific metadata from TEL operation
        for (k, v) in &tel_operation.metadata {
            register_op.metadata.insert(k.clone(), v.to_string());
        }
        
        // Add TEL-specific metadata
        register_op.metadata.insert("tel_origin".to_string(), "true".to_string());
        register_op.metadata.insert("tel_operation_id".to_string(), tel_operation.id.to_string());
        
        Ok(register_op)
    }
    
    /// Validate a TEL operation against domain and time constraints
    pub fn validate_operation(
        &self,
        register_id: &RegisterId,
        operation: &ResourceOperation
    ) -> Result<()> {
        // Convert TEL operation to register operation
        let register_op = self.convert_tel_to_register_operation(operation)?;
        
        // Get the register
        let register = self.register_system.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Validate operation against register state
        self.register_system.validate_operation(&register, &register_op.operation_type)
            .map_err(|e| Error::ValidationError(format!("Invalid operation for register state: {}", e)))?;
        
        // Check causal ordering if time map is available
        if let Some(time_map) = self.register_system.time_map() {
            let causal_valid = self.register_system.verify_causal_ordering(register_id, &register_op)
                .map_err(|e| Error::TimeError(format!("Failed to verify causal ordering: {}", e)))?;
                
            if !causal_valid {
                return Err(Error::TimeError(
                    "Operation violates causal ordering constraints".to_string()
                ));
            }
        }
        
        // Check domain constraints if domain registry is available
        if let Some(domain_registry) = self.register_system.domain_registry() {
            // Get domain for register if it exists
            if let Ok(Some(domain_id)) = self.register_system.get_domain_for_register(register_id) {
                // Get domain adapter
                let adapter = domain_registry.get_adapter(&domain_id)
                    .map_err(|e| Error::DomainError(format!("Failed to get domain adapter: {}", e)))?;
                
                // For domain-specific validation, we could call adapter methods here
                // This would depend on the specific validation needs
                
                // Check if the operation is supported by this domain
                let domain_support_metadata = format!("supports_{}", operation.operation_type.to_string());
                if let Ok(info) = adapter.domain_info() {
                    if let Some(supports) = info.metadata.get(&domain_support_metadata) {
                        if supports == "false" {
                            return Err(Error::DomainError(format!(
                                "Domain {} does not support operation {}", 
                                domain_id, operation.operation_type
                            )));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Create a register in a specific domain using TEL operation
    pub fn create_register_in_domain(
        &self,
        tel_id: &ResourceId,
        domain_id: &DomainId,
        transaction_id: &str
    ) -> Result<RegisterId> {
        // If already mapped, return existing register
        if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
            return Ok(register_id);
        }
        
        // Get the TEL register
        let tel_register_id = TelRegisterId(uuid::Uuid::parse_str(&tel_id.to_string())
            .map_err(|e| Error::ParseError(format!("Invalid TEL register ID: {}", e)))?);
        
        let tel_register = self.tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        // Convert to our register format
        let register = self.convert_tel_register_to_register(&tel_register)?;
        
        // Create register in domain
        let register = self.register_system.create_register_in_domain(
            domain_id,
            register.owner,
            register.domain,
            register.contents,
            transaction_id
        ).map_err(|e| Error::RegisterError(format!("Failed to create register in domain: {}", e)))?;
        
        // Map the IDs
        self.mapping.map_resource(tel_id.clone(), register.register_id.clone())?;
        
        Ok(register.register_id)
    }
    
    /// Create a register with time information
    pub fn create_register_with_time_info(
        &self,
        tel_id: &ResourceId,
        transaction_id: &str
    ) -> Result<RegisterId> {
        // If already mapped, return existing register
        if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
            return Ok(register_id);
        }
        
        // Get the TEL register
        let tel_register_id = TelRegisterId(uuid::Uuid::parse_str(&tel_id.to_string())
            .map_err(|e| Error::ParseError(format!("Invalid TEL register ID: {}", e)))?);
        
        let tel_register = self.tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        // Convert to our register format
        let register = self.convert_tel_register_to_register(&tel_register)?;
        
        // Create register with time info
        let register = self.register_system.create_register_with_time_info(
            register.owner,
            register.domain,
            register.contents,
            transaction_id
        ).map_err(|e| Error::RegisterError(format!("Failed to create register with time info: {}", e)))?;
        
        // Map the IDs
        self.mapping.map_resource(tel_id.clone(), register.register_id.clone())?;
        
        Ok(register.register_id)
    }
    
    /// Update a register with time and domain information
    pub fn update_register_with_time_and_domain(
        &self,
        tel_id: &ResourceId
    ) -> Result<()> {
        // Get register ID
        let register_id = self.mapping.get_register_id(tel_id)?
            .ok_or_else(|| Error::NotFound(format!("No mapping for TEL resource ID {}", tel_id)))?;
        
        // Get the register
        let mut register = self.register_system.get_register(&register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Update register with time info
        self.register_system.update_register_with_time_info(&mut register)
            .map_err(|e| Error::TimeError(format!("Failed to update register with time info: {}", e)))?;
        
        // Sync the register with its domain if it has one
        if let Ok(Some(_)) = self.register_system.get_domain_for_register(&register_id) {
            self.register_system.sync_register_with_domain(&register_id)
                .map_err(|e| Error::DomainError(format!("Failed to sync register with domain: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Get mapping
    pub fn mapping(&self) -> &TelResourceMapping {
        &self.mapping
    }
    
    /// Get register system
    pub fn register_system(&self) -> &Arc<OneTimeRegisterSystem> {
        &self.register_system
    }
    
    /// Get TEL resource manager
    pub fn tel_resource_manager(&self) -> &Arc<TelResourceManager> {
        &self.tel_resource_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test TEL resource manager
    fn create_test_tel_resource_manager() -> Arc<TelResourceManager> {
        Arc::new(TelResourceManager::new())
    }
    
    // Helper function to create a test register system
    fn create_test_register_system() -> Result<Arc<OneTimeRegisterSystem>> {
        let config = crate::resource::OneTimeRegisterConfig::default();
        let register_system = OneTimeRegisterSystem::new(config)?;
        Ok(Arc::new(register_system))
    }
    
    #[test]
    fn test_resource_mapping() -> Result<()> {
        let mapping = TelResourceMapping::new();
        
        // Create test IDs
        let tel_id = ResourceId::from("test_resource");
        let register_id = RegisterId::new_unique();
        
        // Map the IDs
        mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        // Check mapping
        assert_eq!(mapping.get_register_id(&tel_id)?, Some(register_id.clone()));
        assert_eq!(mapping.get_tel_id(&register_id)?, Some(tel_id.clone()));
        
        // Remove mapping
        mapping.remove_mapping(&tel_id, &register_id)?;
        
        // Verify removal
        assert_eq!(mapping.get_register_id(&tel_id)?, None);
        assert_eq!(mapping.get_tel_id(&register_id)?, None);
        
        Ok(())
    }
    
    #[test]
    fn test_import_export() -> Result<()> {
        let tel_resource_manager = create_test_tel_resource_manager();
        let register_system = create_test_register_system()?;
        
        let adapter = TelResourceAdapter::new(register_system, tel_resource_manager.clone());
        
        // Create a TEL register
        let tel_owner = TelAddress::from("user1");
        let tel_domain = TelDomain::from("domain1");
        let tel_contents = crate::tel::resource::model::RegisterContents::String("Test content".to_string());
        
        let tel_register_id = tel_resource_manager.create_register(
            tel_owner,
            tel_domain,
            tel_contents,
        ).map_err(|e| Error::ExternalError(format!("Failed to create TEL register: {}", e)))?;
        
        let tel_resource_id = ResourceId::from(tel_register_id.0.to_string());
        
        // Import the TEL register
        let register_id = adapter.import_tel_register(&tel_resource_id)?;
        
        // Verify mapping
        assert_eq!(adapter.mapping().get_tel_id(&register_id)?, Some(tel_resource_id.clone()));
        
        // Create a new register in the register system
        let owner = Address::new("user2");
        let domain = Domain::new("domain2");
        let contents = RegisterContents::with_string("New register");
        
        let register = adapter.register_system().create_register(
            owner,
            domain,
            contents,
            "test",
        ).map_err(|e| Error::RegisterError(format!("Failed to create register: {}", e)))?;
        
        // Export the register to TEL
        let exported_tel_id = adapter.export_register_to_tel(&register.register_id)?;
        
        // Verify mapping
        assert_eq!(adapter.mapping().get_register_id(&exported_tel_id)?, Some(register.register_id));
        
        Ok(())
    }
    
    #[test]
    fn test_sync() -> Result<()> {
        let tel_resource_manager = create_test_tel_resource_manager();
        let register_system = create_test_register_system()?;
        
        let adapter = TelResourceAdapter::new(register_system, tel_resource_manager.clone());
        
        // Create a TEL register
        let tel_owner = TelAddress::from("user1");
        let tel_domain = TelDomain::from("domain1");
        let tel_contents = crate::tel::resource::model::RegisterContents::String("Initial content".to_string());
        
        let tel_register_id = tel_resource_manager.create_register(
            tel_owner.clone(),
            tel_domain.clone(),
            tel_contents,
        ).map_err(|e| Error::ExternalError(format!("Failed to create TEL register: {}", e)))?;
        
        let tel_resource_id = ResourceId::from(tel_register_id.0.to_string());
        
        // Import the TEL register
        let register_id = adapter.import_tel_register(&tel_resource_id)?;
        
        // Update the TEL register
        let updated_contents = crate::tel::resource::model::RegisterContents::String("Updated content".to_string());
        tel_resource_manager.update_register(
            &tel_register_id, 
            updated_contents
        ).map_err(|e| Error::ExternalError(format!("Failed to update TEL register: {}", e)))?;
        
        // Sync from TEL to register
        adapter.sync_from_tel(&tel_resource_id)?;
        
        // Verify the register was updated
        let register = adapter.register_system().get_register(&register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        assert_eq!(register.contents.as_string(), Some("Updated content"));
        
        // Update the register
        let mut updated_register = register.clone();
        updated_register.contents = RegisterContents::with_string("Register updated");
        
        adapter.register_system().update_register(
            &register_id,
            updated_register,
            "test-update"
        ).map_err(|e| Error::RegisterError(format!("Failed to update register: {}", e)))?;
        
        // Sync from register to TEL
        adapter.sync_to_tel(&register_id)?;
        
        // Verify the TEL register was updated
        let tel_register = tel_resource_manager.get_register(&tel_register_id)
            .map_err(|e| Error::ExternalError(format!("Failed to get TEL register: {}", e)))?;
        
        match &tel_register.contents {
            crate::tel::resource::model::RegisterContents::String(s) => {
                assert_eq!(s, "Register updated");
            },
            _ => return Err(Error::TestError("Unexpected content type".to_string())),
        }
        
        Ok(())
    }
} 