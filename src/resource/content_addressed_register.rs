// ContentAddressedRegister implementation for Causality
//
// This module implements a content-addressed version of the unified Resource-Register model 
// defined in ADR-021. It enables resources to be identified and retrieved by their content hash,
// which provides stronger verification guarantees and simplifies state synchronization.

use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::crypto::hash::{ContentAddressed, HashOutput, ContentId, HashFactory, HashError};
use crate::error::{Error, Result};
use crate::time::TimeMapSnapshot;
use crate::crypto::merkle::Commitment;
use crate::resource::resource_register::{
    ResourceRegister, ResourceLogic, FungibilityDomain, Quantity, 
    NullifierKey, ControllerLabel, StateVisibility, StorageStrategy
};

/// Content-addressed version of the ResourceRegister
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedRegister {
    // Identity
    pub id: ContentId,
    
    // Logical properties
    pub resource_logic: ResourceLogic,
    pub fungibility_domain: FungibilityDomain,
    pub quantity: Quantity,
    pub metadata: HashMap<String, String>,
    
    // Physical properties
    pub state: RegisterState,
    pub nullifier_key: Option<NullifierKey>,
    
    // Provenance tracking
    pub controller_label: Option<ControllerLabel>,
    
    // Temporal context
    pub observed_at: TimeMapSnapshot,
    
    // Storage strategy
    pub storage_strategy: StorageStrategy,
    
    // Contents of the resource
    pub contents: Vec<u8>,
    
    // Version of the resource
    pub version: String,
    
    // Current controller of the resource
    pub controller: Option<String>,
}

// Implement conversions between ContentAddressedRegister and ResourceRegister
impl From<ResourceRegister> for ContentAddressedRegister {
    fn from(register: ResourceRegister) -> Self {
        Self {
            id: register.id,
            resource_logic: register.resource_logic,
            fungibility_domain: register.fungibility_domain,
            quantity: register.quantity,
            metadata: register.metadata,
            state: register.state,
            nullifier_key: register.nullifier_key,
            controller_label: register.controller_label,
            observed_at: register.observed_at,
            storage_strategy: register.storage_strategy,
            contents: register.contents,
            version: register.version,
            controller: register.controller,
        }
    }
}

impl From<ContentAddressedRegister> for ResourceRegister {
    fn from(register: ContentAddressedRegister) -> Self {
        ResourceRegister::new(
            register.id.clone(),
            register.resource_logic.clone(),
            register.fungibility_domain.clone(),
            register.quantity,
            register.metadata.clone(),
            register.storage_strategy.clone(),
        )
    }
}

impl ContentAddressedRegister {
    /// Create a new ContentAddressedRegister
    pub fn new(
        resource_logic: ResourceLogic,
        fungibility_domain: FungibilityDomain,
        quantity: Quantity,
        metadata: HashMap<String, String>,
        storage_strategy: StorageStrategy,
    ) -> Self {
        // Create a temporary ResourceRegister with a placeholder ID
        let temp_register = ResourceRegister::new(
            ContentId::nil(),
            resource_logic.clone(),
            fungibility_domain.clone(),
            quantity,
            metadata.clone(),
            storage_strategy.clone(),
        );
        
        // Calculate the content hash
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = borsh::to_vec(&(
            &resource_logic,
            &fungibility_domain,
            &quantity,
            &metadata,
            &RegisterState::Initial,
            &storage_strategy,
        )).unwrap_or_default();
        
        let hash = hasher.hash(&data);
        let content_id = ContentId::from(hash);
        
        // Create the actual register with the content ID
        Self {
            id: content_id,
            resource_logic,
            fungibility_domain,
            quantity,
            metadata,
            state: RegisterState::Initial,
            nullifier_key: None,
            controller_label: None,
            observed_at: TimeMapSnapshot::default(),
            storage_strategy,
            contents: Vec::new(),
            version: "0".to_string(),
            controller: None,
        }
    }
    
    /// Generate a deterministic ID for this register based on its content
    pub fn deterministic_id(&self) -> ContentId {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = borsh::to_vec(&(
            &self.resource_logic,
            &self.fungibility_domain,
            &self.quantity,
            &self.metadata,
            &self.state,
            &self.storage_strategy,
        )).unwrap_or_default();
        
        ContentId::from(hasher.hash(&data))
    }
    
    /// Convert to a ResourceRegister
    pub fn to_resource_register(&self) -> ResourceRegister {
        ResourceRegister::new(
            self.id.clone(),
            self.resource_logic.clone(),
            self.fungibility_domain.clone(),
            self.quantity,
            self.metadata.clone(),
            self.storage_strategy.clone(),
        )
    }
    
    /// Check if the register has a valid content ID
    pub fn validate_content_id(&self) -> bool {
        self.id == self.deterministic_id()
    }
}

/// A register operation that can be content-addressed
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedRegisterOperation {
    /// Unique identifier for the operation
    pub operation_id: String,
    
    /// The type of register operation
    pub operation_type: RegisterOperationType,
    
    /// The primary resource register affected (by content ID)
    pub target_register: ContentId,
    
    /// Additional resource registers involved (by content ID)
    pub related_registers: Vec<ContentId>,
    
    /// The state before the operation
    pub pre_state: Option<RegisterState>,
    
    /// The state after the operation
    pub post_state: Option<RegisterState>,
    
    /// Parameters for the operation
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Authorization for this operation (e.g., signature, proof)
    pub authorization: Vec<u8>,
    
    /// Timestamp when this operation was created
    pub timestamp: u64,
    
    /// Domain where this operation originated
    pub domain_id: String,
}

/// Types of operations on the content-addressed register
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum RegisterOperationType {
    /// Create a new register entry
    CreateRegister,
    
    /// Update an existing register entry
    UpdateRegister,
    
    /// Freeze a register entry
    FreezeRegister,
    
    /// Unfreeze a register entry
    UnfreezeRegister,
    
    /// Lock a register entry
    LockRegister,
    
    /// Unlock a register entry
    UnlockRegister,
    
    /// Consume a register entry
    ConsumeRegister,
    
    /// Transfer ownership of a register entry
    TransferRegister,
    
    /// Archive a register entry
    ArchiveRegister,
    
    /// Create a relationship between registers
    CreateRelationship,
    
    /// Update a relationship between registers
    UpdateRelationship,
    
    /// Remove a relationship between registers
    RemoveRelationship,
    
    /// Execute custom register logic
    ExecuteLogic,
    
    /// Custom register operation
    Custom(String),
}

/// Error types for register operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterError {
    /// Register not found
    NotFound(String),
    
    /// Register already exists
    AlreadyExists(String),
    
    /// Invalid operation for current state
    InvalidState(String),
    
    /// Authorization failed
    AuthorizationFailed(String),
    
    /// Invalid parameters
    InvalidParameters(String),
    
    /// Storage error
    StorageError(String),
    
    /// Serialization error
    SerializationError(String),
    
    /// Hash error
    HashError(String),
}

// Implement ContentAddressed for ContentAddressedRegister
impl ContentAddressed for ContentAddressedRegister {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Use borsh serialization for stable hashing
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        // Verify that the content ID matches the content hash
        self.id == self.content_id()
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl ContentAddressedRegister {
    /// Get the content ID of this register
    pub fn content_id(&self) -> ContentId {
        ContentId::from(self.content_hash())
    }
}

impl ContentAddressedRegisterOperation {
    /// Create a new register operation
    pub fn new(
        operation_type: RegisterOperationType,
        target_register: ContentId,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Create an input for content-derived ID
        let operation_data = format!(
            "operation-{:?}-{}-{}",
            operation_type,
            target_register,
            timestamp
        );
        
        // Generate a content ID
        let hasher = HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(operation_data.as_bytes());
        let content_id = ContentId::from(hash);
            
        Self {
            operation_id: format!("op-{}", content_id),
            operation_type,
            target_register,
            related_registers: Vec::new(),
            pre_state: None,
            post_state: None,
            parameters: HashMap::new(),
            authorization: Vec::new(),
            timestamp,
            domain_id: "local".to_string(),
        }
    }
    
    /// Set the pre-state for this operation
    pub fn with_pre_state(mut self, state: RegisterState) -> Self {
        self.pre_state = Some(state);
        self
    }
    
    /// Set the post-state for this operation
    pub fn with_post_state(mut self, state: RegisterState) -> Self {
        self.post_state = Some(state);
        self
    }
    
    /// Add related registers to this operation
    pub fn with_related_registers(mut self, registers: Vec<ContentId>) -> Self {
        self.related_registers = registers;
        self
    }
    
    /// Set a parameter for this operation
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
    
    /// Set the authorization for this operation
    pub fn with_authorization(mut self, auth: Vec<u8>) -> Self {
        self.authorization = auth;
        self
    }
    
    /// Set the domain ID for this operation
    pub fn with_domain(mut self, domain_id: String) -> Self {
        self.domain_id = domain_id;
        self
    }
    
    /// Get the content ID of this operation
    pub fn content_id(&self) -> ContentId {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        ContentId::from(hasher.hash(&data))
    }
}

/// A registry for content-addressed registers
pub struct ContentAddressedRegisterRegistry {
    /// Registers indexed by their content ID
    registers: HashMap<ContentId, ContentAddressedRegister>,
    
    /// Operations indexed by their content ID
    operations: HashMap<ContentId, ContentAddressedRegisterOperation>,
}

impl ContentAddressedRegisterRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            operations: HashMap::new(),
        }
    }
    
    /// Register a content-addressed register
    pub fn register(&mut self, register: ContentAddressedRegister) -> ContentId {
        let content_id = register.content_id();
        self.registers.insert(content_id.clone(), register);
        content_id
    }
    
    /// Get a register by its content ID
    pub fn get_register(&self, content_id: &ContentId) -> Option<&ContentAddressedRegister> {
        self.registers.get(content_id)
    }
    
    /// Record an operation
    pub fn record_operation(&mut self, operation: ContentAddressedRegisterOperation) -> ContentId {
        let content_id = operation.content_id();
        self.operations.insert(content_id.clone(), operation);
        content_id
    }
    
    /// Get an operation by its content ID
    pub fn get_operation(&self, content_id: &ContentId) -> Option<&ContentAddressedRegisterOperation> {
        self.operations.get(content_id)
    }
    
    /// Apply an operation to a register
    pub fn apply_operation(
        &mut self, 
        operation: ContentAddressedRegisterOperation
    ) -> std::result::Result<ContentId, RegisterError> {
        // Get the target register
        let target_register = self.get_register(&operation.target_register)
            .ok_or_else(|| RegisterError::NotFound(
                format!("Register not found: {:?}", operation.target_register)
            ))?
            .clone();
            
        // Process based on operation type
        let updated_register = match operation.operation_type {
            RegisterOperationType::CreateRegister => {
                // Can't create an existing register
                return Err(RegisterError::AlreadyExists(
                    format!("Register already exists: {:?}", operation.target_register)
                ));
            },
            RegisterOperationType::UpdateRegister => {
                let mut register = target_register.clone();
                
                // Apply parameter updates
                for (key, value) in &operation.parameters {
                    match key.as_str() {
                        "quantity" => {
                            if let Some(q) = value.as_u64() {
                                register.quantity = Quantity(q as u128);
                            }
                        },
                        "metadata" => {
                            if let Some(m) = value.as_object() {
                                // Update metadata fields
                                for (k, v) in m {
                                    if let Some(v_str) = v.as_str() {
                                        register.metadata.insert(k.clone(), v_str.to_string());
                                    }
                                }
                            }
                        },
                        "contents" => {
                            if let Some(c) = value.as_str() {
                                register.contents = c.as_bytes().to_vec();
                            }
                        },
                        "version" => {
                            if let Some(v) = value.as_str() {
                                register.version = v.to_string();
                            }
                        },
                        _ => {}
                    }
                }
                
                // Update state if provided
                if let Some(state) = operation.post_state {
                    register.state = state;
                }
                
                register
            },
            RegisterOperationType::FreezeRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Frozen;
                register
            },
            RegisterOperationType::UnfreezeRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Active;
                register
            },
            RegisterOperationType::LockRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Locked;
                register
            },
            RegisterOperationType::UnlockRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Active;
                register
            },
            RegisterOperationType::ConsumeRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Consumed;
                register
            },
            RegisterOperationType::TransferRegister => {
                let mut register = target_register.clone();
                
                // Update controller if provided
                if let Some(controller) = operation.parameters.get("controller") {
                    if let Some(c) = controller.as_str() {
                        register.controller = Some(c.to_string());
                    }
                }
                
                register
            },
            RegisterOperationType::ArchiveRegister => {
                let mut register = target_register.clone();
                register.state = RegisterState::Archived;
                register
            },
            // For more complex operations, additional implementation would be required
            RegisterOperationType::CreateRelationship |
            RegisterOperationType::UpdateRelationship |
            RegisterOperationType::RemoveRelationship |
            RegisterOperationType::ExecuteLogic |
            RegisterOperationType::Custom(_) => {
                // Return the original register unchanged for now
                // In a real implementation, these would be handled
                target_register
            }
        };
        
        // Register the updated register
        let new_content_id = self.register(updated_register);
        
        // Record the operation
        self.record_operation(operation);
        
        Ok(new_content_id)
    }
    
    /// Find registers by resource logic
    pub fn find_by_logic(&self, logic: &ResourceLogic) -> Vec<&ContentAddressedRegister> {
        self.registers.values()
            .filter(|r| &r.resource_logic == logic)
            .collect()
    }
    
    /// Find registers by state
    pub fn find_by_state(&self, state: &RegisterState) -> Vec<&ContentAddressedRegister> {
        self.registers.values()
            .filter(|r| &r.state == state)
            .collect()
    }
    
    /// Find operations by type
    pub fn find_operations_by_type(&self, op_type: &RegisterOperationType) -> Vec<&ContentAddressedRegisterOperation> {
        self.operations.values()
            .filter(|op| &op.operation_type == op_type)
            .collect()
    }
    
    /// Clear all registers and operations
    pub fn clear(&mut self) {
        self.registers.clear();
        self.operations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_addressing() {
        // Create a register with appropriate constructor
        let register = ContentAddressedRegister::new(
            ResourceLogic::Fungible,
            FungibilityDomain("token".to_string()),
            Quantity(100),
            HashMap::new(),
            StorageStrategy::FullyOnChain { 
                visibility: StateVisibility::Public 
            },
        );
        
        // Get its content ID
        let content_id = register.content_id();
        
        // Verify that the register matches its content hash
        assert!(register.verify());
        
        // Create a registry
        let mut registry = ContentAddressedRegisterRegistry::new();
        
        // Register the register
        let registered_id = registry.register(register.clone());
        
        // Verify that the registered ID matches the content ID
        assert_eq!(registered_id, content_id);
        
        // Retrieve the register
        let retrieved = registry.get_register(&content_id).unwrap();
        
        // Verify that the retrieved register has the expected values
        assert_eq!(retrieved.quantity, Quantity(100));
        
        // Create an operation
        let operation = ContentAddressedRegisterOperation::new(
            RegisterOperationType::UpdateRegister,
            content_id.clone(),
        )
        .with_pre_state(RegisterState::Initial)
        .with_post_state(RegisterState::Active)
        .with_parameter("quantity", serde_json::json!(200));
        
        // Apply the operation
        let updated_id = registry.apply_operation(operation).unwrap();
        
        // Retrieve the updated register
        let updated = registry.get_register(&updated_id).unwrap();
        
        // Verify that the update was applied
        assert_eq!(updated.quantity, Quantity(200));
        assert_eq!(updated.state, RegisterState::Active);
    }
} 
