// TEL integration for resources
// Original file: src/resource/tel.rs

// TEL Resource Integration
//
// Direct integration between the resource register system and TEL resources.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use causality_types::{Error, Result};
use crate::Address;
use crate::Domain;
use crate::resource::{
    ResourceRegister, UnifiedRegistry
};
use causality_resource::RegisterState;
use crate::operation::{RegisterOperation, OperationType};
use crate::domain::{DomainId, DomainRegistry};
use causality_crypto::ContentId;

// Mock TEL types for simplified implementation
// In a real implementation, these would be imported from the TEL crate

// TEL register contents
#[derive(Clone, Debug)]
pub enum TelRegisterContents {
    Binary(Vec<u8>),
    String(String),
    Json(serde_json::Value),
    Resource(Box<serde_json::Value>),
    Empty,
}

// TEL register state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelRegisterState {
    Active,
    Inactive,
    Consumed,
}

// TEL address
#[derive(Clone, Debug)]
pub struct TelAddress(String);

impl TelAddress {
    pub fn from_string(s: &str) -> Self {
        Self(s.to_string())
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

// TEL domain
#[derive(Clone, Debug)]
pub struct TelDomain(String);

impl TelDomain {
    pub fn from_string(s: &str) -> Self {
        Self(s.to_string())
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

// TEL content ID
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TelContentId(pub ContentId);

// TEL register
#[derive(Clone, Debug)]
pub struct TelRegister {
    pub id: TelContentId,
    pub owner: TelAddress,
    pub domain: TelDomain,
    pub contents: TelRegisterContents,
    pub metadata: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub state: TelRegisterState,
}

impl TelRegister {
    pub fn new(
        owner: TelAddress,
        domain: TelDomain,
        contents: TelRegisterContents,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
            
        Self {
            id: TelContentId(ContentId::from(format!("tel-register-{}", now).as_bytes())),
            owner,
            domain,
            contents,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            state: TelRegisterState::Active,
        }
    }
}

// TEL resource manager
#[derive(Clone, Debug)]
pub struct TelResourceManager {
    registers: Arc<RwLock<HashMap<ContentId, TelRegister>>>,
}

impl TelResourceManager {
    pub fn new() -> Self {
        Self {
            registers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register_resource(&self, register: TelRegister) -> Result<ContentId> {
        let id = register.id.0.clone();
        let mut registers = self.registers.write()
            .map_err(|_| Error::LockError("Failed to acquire TEL register lock".to_string()))?;
            
        registers.insert(id.clone(), register);
        Ok(id)
    }
    
    pub fn get_register(&self, id: &ContentId) -> Result<Option<TelRegister>> {
        let registers = self.registers.read()
            .map_err(|_| Error::LockError("Failed to acquire TEL register lock".to_string()))?;
            
        Ok(registers.get(id).cloned())
    }
    
    pub fn update_register(&self, id: &ContentId, register: TelRegister) -> Result<()> {
        let mut registers = self.registers.write()
            .map_err(|_| Error::LockError("Failed to acquire TEL register lock".to_string()))?;
            
        registers.insert(id.clone(), register);
        Ok(())
    }
}

// TEL operation type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceOperationType {
    Create,
    Update,
    Delete,
    Consume,
}

// TEL resource operation
#[derive(Clone, Debug)]
pub struct ResourceOperation {
    pub resource_id: ContentId,
    pub op_type: ResourceOperationType,
    pub transaction_id: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

// Now define the TelResourceMapping using our mock types
/// Mapping between TEL resources and register system resources
#[derive(Default)]
pub struct TelResourceMapping {
    /// Mapping from TEL resource IDs to register IDs
    tel_to_register: RwLock<HashMap<ContentId, ContentId>>,
    
    /// Mapping from register IDs to TEL resource IDs
    register_to_tel: RwLock<HashMap<ContentId, ContentId>>,
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
    pub fn map_resource(&self, tel_id: ContentId, register_id: ContentId) -> Result<()> {
        let mut tel_to_register = self.tel_to_register.write()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        let mut register_to_tel = self.register_to_tel.write()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        tel_to_register.insert(tel_id.clone(), register_id.clone());
        register_to_tel.insert(register_id, tel_id);
        
        Ok(())
    }
    
    /// Get the register ID for a TEL resource ID
    pub fn get_register_id(&self, tel_id: &ContentId) -> Result<Option<ContentId>> {
        let tel_to_register = self.tel_to_register.read()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        Ok(tel_to_register.get(tel_id).cloned())
    }
    
    /// Get the TEL resource ID for a register ID
    pub fn get_tel_id(&self, register_id: &ContentId) -> Result<Option<ContentId>> {
        let register_to_tel = self.register_to_tel.read()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        Ok(register_to_tel.get(register_id).cloned())
    }
    
    /// Remove a mapping
    pub fn remove_mapping(&self, tel_id: &ContentId, register_id: &ContentId) -> Result<()> {
        let mut tel_to_register = self.tel_to_register.write()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        let mut register_to_tel = self.register_to_tel.write()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        tel_to_register.remove(tel_id);
        register_to_tel.remove(register_id);
        
        Ok(())
    }
    
    /// Get all TEL resource IDs
    pub fn get_all_tel_ids(&self) -> Result<HashSet<ContentId>> {
        let tel_to_register = self.tel_to_register.read()
            .map_err(|_| Error::LockError("Failed to acquire tel_to_register lock".to_string()))?;
        
        Ok(tel_to_register.keys().cloned().collect())
    }
    
    /// Get all register IDs
    pub fn get_all_register_ids(&self) -> Result<HashSet<ContentId>> {
        let register_to_tel = self.register_to_tel.read()
            .map_err(|_| Error::LockError("Failed to acquire register_to_tel lock".to_string()))?;
        
        Ok(register_to_tel.keys().cloned().collect())
    }
}

/// Adapter for working with TEL resources directly through the register system
pub struct TelResourceAdapter {
    /// The unified registry
    register_system: Arc<RwLock<UnifiedRegistry>>,
    
    /// TEL resource manager
    tel_resource_manager: Arc<TelResourceManager>,
    
    /// Resource mapping
    mapping: TelResourceMapping,
}

impl TelResourceAdapter {
    /// Create a new TEL resource adapter
    pub fn new(
        register_system: Arc<RwLock<UnifiedRegistry>>,
        tel_resource_manager: Arc<TelResourceManager>
    ) -> Self {
        Self {
            register_system,
            tel_resource_manager,
            mapping: TelResourceMapping::new(),
        }
    }
    
    /// Convert a TEL register to our ResourceRegister format
    pub fn convert_tel_register_to_resource_register(&self, tel_register: &TelRegister) -> Result<ResourceRegister> {
        // Convert owner and domain
        let owner = Address::new(&tel_register.owner.to_string());
        let domain = Domain::new(&tel_register.domain.to_string());
        
        // Convert contents with correct method names
        let contents = match &tel_register.contents {
            TelRegisterContents::Binary(data) => {
                // Use binary data
                let content_data = data.clone();
                ResourceRegister::with_binary_content(content_data)
            },
            TelRegisterContents::String(data) => {
                // Use string data
                ResourceRegister::with_string_content(data.to_string())
            },
            TelRegisterContents::Json(data) => {
                // Convert JSON to string
                let json_str = serde_json::to_string(data).unwrap_or_default();
                ResourceRegister::with_json_content(json_str)
            },
            TelRegisterContents::Resource(resource) => {
                // Serialize resource to JSON
                let resource_json = serde_json::to_value(resource)
                    .map_err(|e| Error::SerializationError(format!("Failed to serialize resource: {}", e)))?;
                
                ResourceRegister::with_json_content(serde_json::to_string(&resource_json).unwrap_or_default())
            },
            TelRegisterContents::Empty => {
                ResourceRegister::empty()
            },
        };
        
        // Create register ID based on TEL register ID
        let register_id = ContentId::from_uuid(tel_register.id.0);
        
        // Create metadata from TEL register metadata
        let mut metadata = HashMap::new();
        for (k, v) in &tel_register.metadata {
            metadata.insert(k.clone(), v.to_string());
        }
        
        // Add TEL-specific metadata
        metadata.insert("tel_origin".to_string(), "true".to_string());
        
        // Create the register
        let register = ResourceRegister::new(
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
        register: &ResourceRegister
    ) -> Result<TelRegisterContents> {
        if let Some(binary_data) = register.binary_content() {
            Ok(TelRegisterContents::Binary(binary_data.to_vec()))
        } else if let Some(str_data) = register.string_content() {
            Ok(TelRegisterContents::String(str_data.to_string()))
        } else if let Some(json_data) = register.json_content() {
            // Parse JSON
            let json: serde_json::Value = serde_json::from_str(json_data)
                .map_err(|e| Error::SerializationError(format!("Failed to parse JSON: {}", e)))?;
                
            Ok(TelRegisterContents::Json(json))
        } else {
            // Empty content
            Ok(TelRegisterContents::Empty)
        }
    }
    
    /// Import a TEL register into our register system
    pub fn import_tel_register(&self, tel_id: &ContentId) -> Result<ContentId> {
        // Check if already imported
        if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
            return Ok(register_id);
        }
        
        // Get the TEL register
        let tel_register = self.tel_resource_manager.get_register(tel_id)?
            .ok_or_else(|| Error::ResourceNotFound(tel_id.clone()))?;
            
        // Convert to our register format
        let resource_register = self.convert_tel_register_to_resource_register(&tel_register)?;
        
        // Register in our system
        let register_id = {
            let mut registry = self.register_system.write()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.register(resource_register)?
        };
        
        // Create mapping
        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        Ok(register_id)
    }
    
    /// Export a register to TEL
    pub fn export_register_to_tel(&self, register_id: &ContentId) -> Result<ContentId> {
        // Check if already exported
        if let Some(tel_id) = self.mapping.get_tel_id(register_id)? {
            return Ok(tel_id);
        }
        
        // Get the register
        let register = {
            let registry = self.register_system.read()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.get(register_id)?
                .ok_or_else(|| Error::ResourceNotFound(register_id.clone()))?
        };
        
        // Convert to TEL format
        let tel_contents = self.convert_register_to_tel_contents(&register)?;
        
        // Create in TEL
        let tel_register = TelRegister::new(
            TelAddress::from_string(&register.owner.to_string()),
            TelDomain::from_string(&register.domain.to_string()),
            tel_contents,
        );
        
        // Register in TEL
        let tel_id = self.tel_resource_manager.register_resource(tel_register)?;
        
        // Create mapping
        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        Ok(tel_id)
    }
    
    /// Synchronize changes from TEL to our system
    pub fn sync_from_tel(&self, tel_id: &ContentId) -> Result<()> {
        // Check if imported
        let register_id = match self.mapping.get_register_id(tel_id)? {
            Some(id) => id,
            None => return self.import_tel_register(tel_id).map(|_| ()),
        };
        
        // Get TEL register
        let tel_register = self.tel_resource_manager.get_register(tel_id)?
            .ok_or_else(|| Error::ResourceNotFound(tel_id.clone()))?;
            
        // Get our register
        let mut registry = self.register_system.write()
            .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
            
        // Convert TEL register
        let resource_register = self.convert_tel_register_to_resource_register(&tel_register)?;
        
        // Update our register
        registry.update(&register_id, |r| {
            // Update fields
            r.contents = resource_register.contents.clone();
            r.metadata = resource_register.metadata.clone();
            r.updated_at = resource_register.updated_at;
            
            Ok(())
        })?;
        
        Ok(())
    }
    
    /// Synchronize changes from our system to TEL
    pub fn sync_to_tel(&self, register_id: &ContentId) -> Result<()> {
        // Check if exported
        let tel_id = match self.mapping.get_tel_id(register_id)? {
            Some(id) => id,
            None => return self.export_register_to_tel(register_id).map(|_| ()),
        };
        
        // Get our register
        let register = {
            let registry = self.register_system.read()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.get(register_id)?
                .ok_or_else(|| Error::ResourceNotFound(register_id.clone()))?
        };
        
        // Check if TEL register exists
        if let Some(tel_register) = self.tel_resource_manager.get_register(&tel_id)? {
            // Convert to TEL contents
            let tel_contents = self.convert_register_to_tel_contents(&register)?;
            
            // Update TEL register
            let updated_tel_register = TelRegister {
                id: tel_register.id.clone(),
                owner: TelAddress::from_string(&register.owner.to_string()),
                domain: TelDomain::from_string(&register.domain.to_string()),
                contents: tel_contents,
                metadata: tel_register.metadata.clone(), // Preserve TEL metadata
                created_at: tel_register.created_at,
                updated_at: register.updated_at as i64,
                state: match register.state {
                    RegisterState::Active => TelRegisterState::Active,
                    RegisterState::Consumed => TelRegisterState::Consumed,
                    _ => TelRegisterState::Inactive,
                },
            };
            
            // Update in TEL
            self.tel_resource_manager.update_register(&tel_id, updated_tel_register)?;
        } else {
            // Register doesn't exist in TEL, create it
            self.export_register_to_tel(register_id)?;
        }
        
        Ok(())
    }
    
    /// Process a TEL operation
    pub fn process_tel_operation(
        &self,
        operation: &ResourceOperation
    ) -> Result<()> {
        match operation.op_type {
            ResourceOperationType::Create => {
                // Get resource ID
                let tel_id = &operation.resource_id;
                
                // Import to our system
                self.import_tel_register(tel_id)?;
            },
            ResourceOperationType::Update => {
                // Get resource ID
                let tel_id = &operation.resource_id;
                
                // Check if we have this resource
                if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
                    // Sync changes from TEL
                    self.sync_from_tel(tel_id)?;
                } else {
                    // Not yet imported, import it
                    self.import_tel_register(tel_id)?;
                }
            },
            ResourceOperationType::Delete => {
                // Get resource ID
                let tel_id = &operation.resource_id;
                
                // Check if we have this resource
                if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
                    // Delete from our system
                    let mut registry = self.register_system.write()
                        .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                        
                    // Remove from registry
                    registry.remove(&register_id)?;
                    
                    // Remove mapping
                    self.mapping.remove_mapping(tel_id, &register_id)?;
                }
            },
            ResourceOperationType::Consume => {
                // Get resource ID
                let tel_id = &operation.resource_id;
                
                // Check if we have this resource
                if let Some(register_id) = self.mapping.get_register_id(tel_id)? {
                    // Consume in our system
                    let mut registry = self.register_system.write()
                        .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                        
                    // Update state to consumed
                    registry.consume(&register_id)?;
                }
            },
        }
        
        Ok(())
    }
    
    /// Convert a TEL operation to our operation model
    pub fn convert_tel_to_register_operation(
        &self,
        tel_operation: &ResourceOperation
    ) -> Result<RegisterOperation> {
        // Map operation type
        let op_type = match tel_operation.op_type {
            ResourceOperationType::Create => OperationType::Create,
            ResourceOperationType::Update => OperationType::Update,
            ResourceOperationType::Delete => OperationType::Delete,
            ResourceOperationType::Consume => OperationType::Consume,
        };
        
        // Get our register ID
        let register_id = match self.mapping.get_register_id(&tel_operation.resource_id)? {
            Some(id) => id,
            None => return Err(Error::ResourceNotFound(tel_operation.resource_id.clone())),
        };
        
        // Create operation with the correct fields
        let operation = RegisterOperation::new(
            register_id,
            op_type,
            "tel".to_string(),  // Source field
            tel_operation.metadata.clone(),
        );
        
        Ok(operation)
    }
    
    /// Validate a TEL operation against our register
    pub fn validate_operation(
        &self,
        register_id: &ContentId,
        operation: &ResourceOperation
    ) -> Result<()> {
        let registry = self.register_system.read()
            .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
            
        let register = registry.get(register_id)?
            .ok_or_else(|| Error::ResourceNotFound(register_id.clone()))?;
            
        // Validate based on operation type
        match operation.op_type {
            ResourceOperationType::Create => {
                // Can't create an existing register
                return Err(Error::InvalidOperation(
                    "Cannot create an existing register".to_string()
                ));
            },
            ResourceOperationType::Update => {
                // Can only update active registers
                if register.state != RegisterState::Active {
                    return Err(Error::InvalidState(
                        "Cannot update a non-active register".to_string()
                    ));
                }
            },
            ResourceOperationType::Delete => {
                // Can only delete active registers
                if register.state != RegisterState::Active {
                    return Err(Error::InvalidState(
                        "Cannot delete a non-active register".to_string()
                    ));
                }
            },
            ResourceOperationType::Consume => {
                // Can only consume active registers
                if register.state != RegisterState::Active {
                    return Err(Error::InvalidState(
                        "Cannot consume a non-active register".to_string()
                    ));
                }
            },
        }
        
        Ok(())
    }
    
    /// Create a register in a specific domain
    pub fn create_register_in_domain(
        &self,
        tel_id: &ContentId,
        domain_id: &DomainId,
        transaction_id: &str
    ) -> Result<ContentId> {
        // Get TEL register
        let tel_register = self.tel_resource_manager.get_register(tel_id)?
            .ok_or_else(|| Error::ResourceNotFound(tel_id.clone()))?;
            
        // Convert to our register format, setting domain
        let mut resource_register = self.convert_tel_register_to_resource_register(&tel_register)?;
        resource_register.domain = Domain::new(&domain_id.to_string());
        
        // Add transaction metadata
        resource_register.metadata.insert("transaction_id".to_string(), transaction_id.to_string());
        
        // Register in our system
        let register_id = {
            let mut registry = self.register_system.write()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.register(resource_register)?
        };
        
        // Create mapping
        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        Ok(register_id)
    }
    
    /// Create a register with time information
    pub fn create_register_with_time_info(
        &self,
        tel_id: &ContentId,
        transaction_id: &str
    ) -> Result<ContentId> {
        // Get TEL register
        let tel_register = self.tel_resource_manager.get_register(tel_id)?
            .ok_or_else(|| Error::ResourceNotFound(tel_id.clone()))?;
            
        // Convert to our register format
        let mut resource_register = self.convert_tel_register_to_resource_register(&tel_register)?;
        
        // Add time metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::TimeError(format!("Failed to get current time: {}", e)))?
            .as_secs();
            
        resource_register.metadata.insert("created_time".to_string(), now.to_string());
        resource_register.metadata.insert("transaction_id".to_string(), transaction_id.to_string());
        
        // Register in our system
        let register_id = {
            let mut registry = self.register_system.write()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.register(resource_register)?
        };
        
        // Create mapping
        self.mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        Ok(register_id)
    }
    
    /// Update register with time and domain info
    pub fn update_register_with_time_and_domain(
        &self,
        tel_id: &ContentId
    ) -> Result<()> {
        // Get register ID
        let register_id = match self.mapping.get_register_id(tel_id)? {
            Some(id) => id,
            None => return Err(Error::ResourceNotFound(tel_id.clone())),
        };
        
        // Update register
        let mut registry = self.register_system.write()
            .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
            
        registry.update(&register_id, |register| {
            // Add time metadata
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Error::TimeError(format!("Failed to get current time: {}", e)))?
                .as_secs();
                
            register.metadata.insert("updated_time".to_string(), now.to_string());
            register.updated_at = now;
            
            Ok(())
        })?;
        
        Ok(())
    }
    
    /// Get the resource mapping
    pub fn mapping(&self) -> &TelResourceMapping {
        &self.mapping
    }
    
    /// Get the register system
    pub fn register_system(&self) -> &Arc<RwLock<UnifiedRegistry>> {
        &self.register_system
    }
    
    /// Get the TEL resource manager
    pub fn tel_resource_manager(&self) -> &Arc<TelResourceManager> {
        &self.tel_resource_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_tel_resource_manager() -> Arc<TelResourceManager> {
        Arc::new(TelResourceManager::new())
    }
    
    fn create_test_register_system() -> Result<Arc<RwLock<UnifiedRegistry>>> {
        let registry = UnifiedRegistry::new();
        Ok(Arc::new(RwLock::new(registry)))
    }
    
    #[test]
    fn test_resource_mapping() -> Result<()> {
        let mapping = TelResourceMapping::new();
        
        let tel_id = ContentId::random();
        let register_id = ContentId::random();
        
        // Test mapping
        mapping.map_resource(tel_id.clone(), register_id.clone())?;
        
        // Test retrieval
        assert_eq!(mapping.get_register_id(&tel_id)?, Some(register_id.clone()));
        assert_eq!(mapping.get_tel_id(&register_id)?, Some(tel_id.clone()));
        
        // Test removal
        mapping.remove_mapping(&tel_id, &register_id)?;
        
        assert_eq!(mapping.get_register_id(&tel_id)?, None);
        assert_eq!(mapping.get_tel_id(&register_id)?, None);
        
        Ok(())
    }
    
    #[test]
    fn test_import_export() -> Result<()> {
        let tel_manager = create_test_tel_resource_manager();
        let register_system = create_test_register_system()?;
        
        let adapter = TelResourceAdapter::new(register_system, tel_manager.clone());
        
        // Create a TEL register
        let tel_register = TelRegister::new(
            TelAddress::from_string("owner1"),
            TelDomain::from_string("domain1"),
            TelRegisterContents::String("test content".to_string()),
        );
        
        let tel_id = tel_manager.register_resource(tel_register)?;
        
        // Import to our system
        let register_id = adapter.import_tel_register(&tel_id)?;
        
        // Verify mapping
        assert_eq!(adapter.mapping().get_register_id(&tel_id)?, Some(register_id.clone()));
        assert_eq!(adapter.mapping().get_tel_id(&register_id)?, Some(tel_id.clone()));
        
        // Verify register was created
        let registry = adapter.register_system().read()
            .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
            
        let register = registry.get(&register_id)?
            .expect("Register should exist");
            
        assert_eq!(register.owner.to_string(), "owner1");
        assert_eq!(register.domain.to_string(), "domain1");
        assert_eq!(register.contents.as_string().unwrap(), "test content");
        
        // Now export a new register back to TEL
        let new_register = ResourceRegister::new(
            Address::new("owner2"),
            Domain::new("domain2"),
            RegisterContents::with_string("exported content"),
            HashMap::new(),
            0, 0,
        );
        
        let new_register_id = {
            let mut registry = adapter.register_system().write()
                .map_err(|_| Error::LockError("Failed to acquire registry lock".to_string()))?;
                
            registry.register(new_register)?
        };
        
        // Export to TEL
        let new_tel_id = adapter.export_register_to_tel(&new_register_id)?;
        
        // Verify mapping
        assert_eq!(adapter.mapping().get_register_id(&new_tel_id)?, Some(new_register_id.clone()));
        assert_eq!(adapter.mapping().get_tel_id(&new_register_id)?, Some(new_tel_id.clone()));
        
        // Verify TEL register was created
        let tel_register = tel_manager.get_register(&new_tel_id)?
            .expect("TEL register should exist");
            
        assert_eq!(tel_register.owner.to_string(), "owner2");
        assert_eq!(tel_register.domain.to_string(), "domain2");
        
        match tel_register.contents {
            TelRegisterContents::String(s) => {
                assert_eq!(s, "exported content");
            },
            _ => panic!("Expected string content"),
        }
        
        Ok(())
    }
    
    // Additional tests would follow the same pattern
} 
