// ResourceRegister implementation for Causality
//
// This module implements the unified Resource-Register model as defined in ADR-021.
// It combines logical resource properties and physical register characteristics
// into a single abstraction, simplifying the mental model and implementation.
// This is the primary model used by the lifecycle_manager and relationship_tracker.

use std::fmt;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::types::{ResourceId, DomainId};
use crate::tel::types::Metadata;
use crate::error::{Error, Result};
use crate::time::TimeMapSnapshot;

/// The unified ResourceRegister abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRegister {
    // Identity
    pub id: ResourceId,
    
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

/// Visibility of state in on-chain storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateVisibility {
    Public,
    Private,
    Permissioned(HashSet<PermissionedEntity>),
}

/// Crypto commitment of register state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commitment(pub [u8; 32]);

/// Nullifier ID for preventing double-spending
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NullifierId(pub [u8; 32]);

/// Entity that has permission to view state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PermissionedEntity(pub String);

impl ResourceRegister {
    /// Create a new ResourceRegister
    pub fn new(
        id: ResourceId,
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
        }
    }
    
    /// Create a new ResourceRegister with active state
    pub fn new_active(
        id: ResourceId,
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
        }
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
        Ok(())
    }
    
    /// Lock this resource
    pub fn lock(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot lock a resource that is not active".into()));
        }
        
        self.state = RegisterState::Locked;
        Ok(())
    }
    
    /// Unlock this resource
    pub fn unlock(&mut self) -> Result<()> {
        if !self.is_locked() {
            return Err(Error::InvalidOperation("Cannot unlock a resource that is not locked".into()));
        }
        
        self.state = RegisterState::Active;
        Ok(())
    }
    
    /// Freeze this resource
    pub fn freeze(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot freeze a resource that is not active".into()));
        }
        
        self.state = RegisterState::Frozen;
        Ok(())
    }
    
    /// Unfreeze this resource
    pub fn unfreeze(&mut self) -> Result<()> {
        if !self.is_frozen() {
            return Err(Error::InvalidOperation("Cannot unfreeze a resource that is not frozen".into()));
        }
        
        self.state = RegisterState::Active;
        Ok(())
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
        Ok(())
    }
    
    /// Archive this resource
    pub fn archive(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::InvalidOperation("Cannot archive a resource that is not active".into()));
        }
        
        self.state = RegisterState::Archived;
        Ok(())
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
}

/// ResourceState is the simplified state object used by lifecycle_manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceState {
    pub id: ResourceId,
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
            id: register.id,
            contents,
            metadata: register.metadata,
            state: register.state,
        }
    }
}

// Default implementations for various types

impl Default for TimeMapSnapshot {
    fn default() -> Self {
        // In a real implementation, this would use the actual time system
        Self {}
    }
}

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
        let id = ResourceId("test-resource".to_string());
        let mut register = ResourceRegister::new(
            id.clone(),
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
        let id = ResourceId("test-resource".to_string());
        let register = ResourceRegister::new(
            id.clone(),
            ResourceLogic::Data,
            FungibilityDomain("test-domain".to_string()),
            Quantity(1),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        let state: ResourceState = register.clone().into();
        
        assert_eq!(state.id, register.id);
        assert_eq!(state.state, register.state);
        assert_eq!(state.metadata, register.metadata);
        assert!(!state.contents.is_empty()); // Content should be serialized properties
    }
} 