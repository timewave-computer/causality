// Resource register implementation
// Original file: src/resource/resource_register.rs

// ResourceRegister implementation for Causality
//
// This module implements the unified Resource-Register model as defined in ADR-021.
// It combines logical resource properties and physical register characteristics
// into a single abstraction, simplifying the mental model and implementation.
// This is the primary model used by the lifecycle_manager and relationship_tracker.

use std::fmt;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

use causality_types::{*};
use causality_crypto::ContentId;;
use causality_tel::Metadata;
use causality_types::{Error, Result};
use crate::time::TimeMapSnapshot;
use causality_crypto::Commitment;
use crate::resource::{StorageStrategy, StateVisibility};
use causality_resource_manager::ResourceRegisterLifecycleManager;
use causality_crypto::{ContentAddressed, HashOutput, ContentId, HashFactory};
use borsh::{BorshSerialize, BorshDeserialize};

/// The unified ResourceRegister abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRegister {
    // Identity
    pub id: ContentId,
    
    // Logical properties (previously in Resource)
    pub resource_logic: ResourceLogic,
    pub fungibility_domain: FungibilityDomain,
    pub quantity: Quantity,
    pub metadata: Metadata,
    
    // Physical properties (previously in Register)
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
    
    // Lifecycle manager for the resource
    #[serde(skip)]
    pub lifecycle_manager: ResourceRegisterLifecycleManager,
}

/// Resource logic determines how a resource behaves and transforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceLogic {
    Fungible,
    NonFungible,
    Capability,
    Data,
    Custom(String),
}

/// Fungibility domain specifies the asset/token type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FungibilityDomain(pub String);

/// Quantity represents an amount of a resource
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(pub u128);

/// State of a register
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RegisterState {
    Initial,
    Active,
    Consumed,
    Pending,
    Locked,
    Frozen,
    Archived,
}

/// Key used for nullifier generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NullifierKey(pub [u8; 32]);

/// Label for tracking controllers across domains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ControllerLabel(pub String);

/// Storage strategy for on-chain representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageStrategy {
    // Full on-chain storage - all fields available
    FullyOnChain {
        visibility: StateVisibility,
    },
    
    // Commitment-based with ZK proofs - minimal on-chain footprint
    CommitmentBased {
        commitment: Option<Commitment>,
        nullifier: Option<NullifierId>,
    },
    
    // Hybrid - critical fields on-chain, others as commitments
    Hybrid {
        on_chain_fields: HashSet<String>,
        remaining_commitment: Option<Commitment>,
    },
}

/// Visibility of state on the domain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateVisibility {
    Public,
    Private,
    Permissioned(HashSet<PermissionedEntity>),
}

/// Nullifier identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NullifierId(pub [u8; 32]);

/// Permissioned entity identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PermissionedEntity(pub String);

impl ResourceRegister {
    /// Create a new ResourceRegister
    pub fn new(
        id: ContentId,
        resource_logic: ResourceLogic,
        fungibility_domain: FungibilityDomain,
        quantity: Quantity,
        metadata: Metadata,
        storage_strategy: StorageStrategy,
    ) -> Self {
        Self {
            id,
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
            lifecycle_manager: ResourceRegisterLifecycleManager::new(),
        }
    }
    
    /// Create a new ResourceRegister from a ContentId (for backward compatibility)
    pub fn from_resource_id(
        resource_id: ContentId,
        resource_logic: ResourceLogic,
        fungibility_domain: FungibilityDomain,
        quantity: Quantity,
        metadata: Metadata,
        storage_strategy: StorageStrategy,
    ) -> Self {
        let id = ContentId::from(resource_id);
        Self::new(
            id, 
            resource_logic, 
            fungibility_domain,
            quantity,
            metadata,
            storage_strategy,
        )
    }
    
    /// Create a new ResourceRegister with active state
    pub fn new_active(
        id: ContentId,
        resource_logic: ResourceLogic,
        fungibility_domain: FungibilityDomain,
        quantity: Quantity,
        metadata: Metadata,
        storage_strategy: StorageStrategy,
    ) -> Self {
        Self {
            id,
            resource_logic,
            fungibility_domain,
            quantity,
            metadata,
            state: RegisterState::Active,
            nullifier_key: None,
            controller_label: None,
            observed_at: TimeMapSnapshot::default(),
            storage_strategy,
            contents: Vec::new(),
            version: "0".to_string(),
            controller: None,
            lifecycle_manager: ResourceRegisterLifecycleManager::new(),
        }
    }
    
    /// Create a new ResourceRegister with active state from a ContentId (for backward compatibility)
    pub fn from_resource_id_active(
        resource_id: ContentId,
        resource_logic: ResourceLogic,
        fungibility_domain: FungibilityDomain,
        quantity: Quantity,
        metadata: Metadata,
        storage_strategy: StorageStrategy,
    ) -> Self {
        let id = ContentId::from(resource_id);
        Self::new_active(
            id, 
            resource_logic, 
            fungibility_domain,
            quantity,
            metadata,
            storage_strategy,
        )
    }
    
    /// Get the resource ID corresponding to this ResourceRegister (for backward compatibility)
    pub fn resource_id(&self) -> ContentId {
        ContentId::from(self.id.clone())
    }
    
    /// Get all fields of the resource register
    pub fn all_fields(&self) -> HashSet<String> {
        let mut fields = HashSet::new();
        fields.insert("id".to_string());
        fields.insert("resource_logic".to_string());
        fields.insert("fungibility_domain".to_string());
        fields.insert("quantity".to_string());
        fields.insert("metadata".to_string());
        fields.insert("state".to_string());
        if self.nullifier_key.is_some() {
            fields.insert("nullifier_key".to_string());
        }
        if self.controller_label.is_some() {
            fields.insert("controller_label".to_string());
        }
        fields.insert("observed_at".to_string());
        fields.insert("storage_strategy".to_string());
        fields
    }
    
    /// Update the quantity of the resource
    pub fn update_quantity(&mut self, new_quantity: Quantity) -> Result<()> {
        // In a real implementation, this would validate the update
        self.quantity = new_quantity;
        Ok(())
    }
    
    /// Check if the resource is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, RegisterState::Active)
    }
    
    /// Check if the resource is initial (pending activation)
    pub fn is_initial(&self) -> bool {
        matches!(self.state, RegisterState::Initial)
    }
    
    /// Check if the resource is consumed
    pub fn is_consumed(&self) -> bool {
        matches!(self.state, RegisterState::Consumed)
    }
    
    /// Check if the resource is locked
    pub fn is_locked(&self) -> bool {
        matches!(self.state, RegisterState::Locked)
    }
    
    /// Check if the resource is frozen
    pub fn is_frozen(&self) -> bool {
        matches!(self.state, RegisterState::Frozen)
    }
    
    /// Check if the resource is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.state, RegisterState::Pending)
    }
    
    /// Check if the resource is archived
    pub fn is_archived(&self) -> bool {
        matches!(self.state, RegisterState::Archived)
    }
    
    /// Activate this resource (mark as active)
    pub fn activate(&mut self) -> Result<()> {
        if !self.is_initial() && !self.is_pending() && !self.is_locked() && !self.is_frozen() && !self.is_archived() {
            return Err(Error::InvalidOperation("Cannot activate a resource that is not in initial, pending, locked, frozen, or archived state".into()));
        }
        
        self.state = RegisterState::Active;
        self.lifecycle_manager.activate(&self.id)
    }
    
    /// Lock this resource
    pub fn lock(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot lock a resource that is not active".into()));
        }
        
        self.state = RegisterState::Locked;
        self.lifecycle_manager.lock(&self.id)
    }
    
    /// Unlock this resource
    pub fn unlock(&mut self) -> Result<()> {
        if !self.is_locked() {
            return Err(Error::InvalidOperation("Cannot unlock a resource that is not locked".into()));
        }
        
        self.state = RegisterState::Active;
        self.lifecycle_manager.unlock(&self.id)
    }
    
    /// Freeze this resource
    pub fn freeze(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot freeze a resource that is not active".into()));
        }
        
        self.state = RegisterState::Frozen;
        self.lifecycle_manager.freeze(&self.id)
    }
    
    /// Unfreeze this resource
    pub fn unfreeze(&mut self) -> Result<()> {
        if !self.is_frozen() {
            return Err(Error::InvalidOperation("Cannot unfreeze a resource that is not frozen".into()));
        }
        
        self.state = RegisterState::Active;
        self.lifecycle_manager.unfreeze(&self.id)
    }
    
    /// Mark as pending
    pub fn mark_pending(&mut self) -> Result<()> {
        if !self.is_active() && !self.is_initial() {
            return Err(Error::InvalidOperation("Cannot mark as pending a resource that is not active or initial".into()));
        }
        
        self.state = RegisterState::Pending;
        Ok(())
    }
    
    /// Consume this resource (mark as used)
    pub fn consume(&mut self) -> Result<()> {
        if !self.is_active() && !self.is_pending() && !self.is_locked() && !self.is_frozen() {
            return Err(Error::InvalidOperation("Cannot consume a resource that is not active, pending, locked, or frozen".into()));
        }
        
        self.state = RegisterState::Consumed;
        self.lifecycle_manager.consume(&self.id)
    }
    
    /// Archive this resource
    pub fn archive(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot archive a resource that is not active".into()));
        }
        
        self.state = RegisterState::Archived;
        self.lifecycle_manager.archive(&self.id)
    }
    
    /// Unarchive this resource
    pub fn unarchive(&mut self) -> Result<()> {
        if !self.is_archived() {
            return Err(Error::InvalidOperation("Cannot unarchive a resource that is not archived".into()));
        }
        
        self.state = RegisterState::Active;
        Ok(())
    }
    
    /// Create a nullifier for this resource
    pub fn create_nullifier(&self) -> Result<NullifierId> {
        let nullifier_key = self.nullifier_key.as_ref()
            .ok_or_else(|| Error::InvalidOperation("No nullifier key available".into()))?;
            
        // In a real implementation, this would use cryptographic primitives
        let mut nullifier = [0u8; 32];
        // Simple mock implementation just to demonstrate
        for i in 0..32 {
            nullifier[i] = nullifier_key.0[i] ^ (self.id.0.as_u128() as u8);
        }
        
        Ok(NullifierId(nullifier))
    }
    
    /// Set the controller label
    pub fn set_controller_label(&mut self, label: ControllerLabel) {
        self.controller_label = Some(label);
    }
    
    /// Add or update metadata
    pub fn update_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Set the nullifier key
    pub fn set_nullifier_key(&mut self, key: NullifierKey) {
        self.nullifier_key = Some(key);
    }
    
    /// Update the timeframe
    pub fn update_timeframe(&mut self, snapshot: TimeMapSnapshot) {
        self.observed_at = snapshot;
    }
    
    /// Update the contents of the resource
    pub fn update_contents(&mut self, contents: Vec<u8>) {
        self.contents = contents;
        self.increment_version();
    }
    
    /// Increment the version of the resource
    fn increment_version(&mut self) {
        let version = self.version.parse::<u64>().unwrap_or(0);
        self.version = (version + 1).to_string();
    }
    
    /// Set the controller of the resource
    pub fn set_controller(&mut self, controller: String) {
        self.controller = Some(controller);
    }
}

/// ResourceState is the simplified state object used by lifecycle_manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceState {
    pub id: ContentId,
    pub contents: Vec<u8>,
    pub metadata: Metadata,
    pub state: RegisterState,
}

impl From<ResourceRegister> for ResourceState {
    fn from(register: ResourceRegister) -> Self {
        // Serialize the register's logical properties to binary
        let contents = serde_json::to_vec(&(
            register.resource_logic,
            register.fungibility_domain,
            register.quantity
        )).unwrap_or_default();
        
        ResourceState {
            id: register.id.into(),
            contents,
            metadata: register.metadata,
            state: register.state,
        }
    }
}

// Default implementations for various types

impl FungibilityDomain {
    pub fn new(domain: impl Into<String>) -> Self {
        Self(domain.into())
    }
}

impl Quantity {
    pub fn new(amount: u128) -> Self {
        Self(amount)
    }
    
    pub fn amount(&self) -> u128 {
        self.0
    }
}

impl Default for RegisterState {
    fn default() -> Self {
        RegisterState::Initial
    }
}

impl fmt::Display for FungibilityDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_register_state_transitions() {
        let id = ContentId::new("test-resource".to_string());
        let mut register = ResourceRegister::new(
            id.clone().into(),
            ResourceLogic::Data,
            FungibilityDomain("test-domain".to_string()),
            Quantity(1),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        // Test initial state
        assert!(register.is_initial());
        
        // Test activation
        register.activate().unwrap();
        assert!(register.is_active());
        
        // Test locking
        register.lock().unwrap();
        assert!(register.is_locked());
        
        // Test unlocking
        register.unlock().unwrap();
        assert!(register.is_active());
        
        // Test freezing
        register.freeze().unwrap();
        assert!(register.is_frozen());
        
        // Test unfreezing
        register.unfreeze().unwrap();
        assert!(register.is_active());
        
        // Test archiving
        register.archive().unwrap();
        assert!(register.is_archived());
        
        // Test unarchiving
        register.unarchive().unwrap();
        assert!(register.is_active());
        
        // Test marking as pending
        register.mark_pending().unwrap();
        assert!(register.is_pending());
        
        // Test activation from pending
        register.activate().unwrap();
        assert!(register.is_active());
        
        // Test consumption
        register.consume().unwrap();
        assert!(register.is_consumed());
    }
    
    #[test]
    fn test_resource_state_conversion() {
        let id = ContentId::new("test-resource".to_string());
        let register = ResourceRegister::new(
            id.clone().into(),
            ResourceLogic::Data,
            FungibilityDomain("test-domain".to_string()),
            Quantity(1),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        let state: ResourceState = register.clone().into();
        
        assert_eq!(state.id, id.into());
        assert_eq!(state.state, register.state);
        assert_eq!(state.metadata, register.metadata);
        assert!(!state.contents.is_empty()); // Content should be serialized properties
    }
}

// Add implement ContentAddressed trait for ResourceRegister
impl ContentAddressed for ResourceRegister {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher from the registry
        let hasher = HashFactory::default().create_hasher().unwrap();
        
        // Create a canonical serialization of the resource register
        let mut data = Vec::new();
        
        // Add all fields for hashing (except lifecycle_manager which is transient)
        data.extend_from_slice(self.id.as_bytes());
        
        // Add the resource logic type
        let logic_str = match &self.resource_logic {
            ResourceLogic::Fungible => "fungible",
            ResourceLogic::NonFungible => "non_fungible",
            ResourceLogic::Capability => "capability",
            ResourceLogic::Data => "data",
            ResourceLogic::Custom(s) => s.as_str(),
        };
        data.extend_from_slice(logic_str.as_bytes());
        
        // Add fungibility domain
        data.extend_from_slice(self.fungibility_domain.0.as_bytes());
        
        // Add quantity as bytes
        let quantity_bytes = self.quantity.0.to_le_bytes();
        data.extend_from_slice(&quantity_bytes);
        
        // Add metadata serialized
        if let Ok(metadata_json) = serde_json::to_vec(&self.metadata) {
            data.extend_from_slice(&metadata_json);
        }
        
        // Add state
        let state_str = match self.state {
            RegisterState::Initial => "initial",
            RegisterState::Active => "active",
            RegisterState::Consumed => "consumed",
            RegisterState::Pending => "pending",
            RegisterState::Locked => "locked",
            RegisterState::Frozen => "frozen",
            RegisterState::Archived => "archived",
        };
        data.extend_from_slice(state_str.as_bytes());
        
        // Add nullifier key if present
        if let Some(key) = &self.nullifier_key {
            data.extend_from_slice(&key.0);
        }
        
        // Add controller label if present
        if let Some(label) = &self.controller_label {
            data.extend_from_slice(label.0.as_bytes());
        }
        
        // Add observed timestamp
        if let Ok(time_bytes) = self.observed_at.to_bytes() {
            data.extend_from_slice(&time_bytes);
        }
        
        // Add storage strategy
        let strategy_str = match &self.storage_strategy {
            StorageStrategy::FullyOnChain { .. } => "fully_on_chain",
            StorageStrategy::CommitmentBased { .. } => "commitment_based",
            StorageStrategy::Hybrid { .. } => "hybrid",
        };
        data.extend_from_slice(strategy_str.as_bytes());
        
        // Add contents
        data.extend_from_slice(&self.contents);
        
        // Add version
        data.extend_from_slice(self.version.as_bytes());
        
        // Add controller if present
        if let Some(controller) = &self.controller {
            data.extend_from_slice(controller.as_bytes());
        }
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let id_hash = self.id.hash();
        hash == *id_hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Create a serializable version without the lifecycle manager
        let serializable = ResourceRegisterSerializable {
            id: self.id.clone(),
            resource_logic: self.resource_logic.clone(),
            fungibility_domain: self.fungibility_domain.clone(),
            quantity: self.quantity,
            metadata: self.metadata.clone(),
            state: self.state.clone(),
            nullifier_key: self.nullifier_key.clone(),
            controller_label: self.controller_label.clone(),
            observed_at: self.observed_at.clone(),
            storage_strategy: self.storage_strategy.clone(),
            contents: self.contents.clone(),
            version: self.version.clone(),
            controller: self.controller.clone(),
        };
        
        // Use Borsh serialization for consistency
        serializable.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, causality_crypto::HashError> {
        // Deserialize the serializable version
        let serializable = ResourceRegisterSerializable::try_from_slice(bytes)
            .map_err(|e| causality_crypto::HashError::SerializationError(e.to_string()))?;
        
        // Create a ResourceRegister with default lifecycle manager
        Ok(Self {
            id: serializable.id,
            resource_logic: serializable.resource_logic,
            fungibility_domain: serializable.fungibility_domain,
            quantity: serializable.quantity,
            metadata: serializable.metadata,
            state: serializable.state,
            nullifier_key: serializable.nullifier_key,
            controller_label: serializable.controller_label,
            observed_at: serializable.observed_at,
            storage_strategy: serializable.storage_strategy,
            contents: serializable.contents,
            version: serializable.version,
            controller: serializable.controller,
            lifecycle_manager: ResourceRegisterLifecycleManager::new(),
        })
    }
}

// Add a serializable version of ResourceRegister without the lifecycle manager
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct ResourceRegisterSerializable {
    id: ContentId,
    resource_logic: ResourceLogic,
    fungibility_domain: FungibilityDomain,
    quantity: Quantity,
    metadata: Metadata,
    state: RegisterState,
    nullifier_key: Option<NullifierKey>,
    controller_label: Option<ControllerLabel>,
    observed_at: TimeMapSnapshot,
    storage_strategy: StorageStrategy,
    contents: Vec<u8>,
    version: String,
    controller: Option<String>,
} 
