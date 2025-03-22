// Register Lifecycle and State Management for Unified ResourceRegister
//
// This module implements the register lifecycle stages, state transitions,
// and validation for the unified ResourceRegister model as defined in ADR-021.
// It builds upon the concepts from the original lifecycle.rs but adapts them
// for the ResourceRegister abstraction.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId, Timestamp, Metadata, RegisterState};
use crate::resource::lifecycle::StateTransition;
use crate::resource::lifecycle::TransitionReason;
use crate::resource::transition::TransitionSystem;

/// Type representing different operations that can be performed on a register
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterOperationType {
    Create,
    Read,
    Update,
    Delete,
    Lock,
    Unlock,
    Freeze,
    Unfreeze,
    Consume,
    Archive,
}

/// Record of a state transition for a resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionRecord {
    pub resource_id: ResourceId,
    pub from_state: RegisterState,
    pub to_state: RegisterState,
    pub timestamp: Timestamp,
    pub caused_by: Option<ResourceId>,
    pub metadata: Metadata,
}

/// The Resource Register Lifecycle Manager manages the lifecycle states
/// of resources and enforces valid state transitions.
pub struct ResourceRegisterLifecycleManager {
    /// Map of resource ID to current state
    states: HashMap<ResourceId, RegisterState>,
    /// Map of resource ID to resources it has locked
    locked_resources: HashMap<ResourceId, HashSet<ResourceId>>,
    /// History of state transitions for each resource
    transition_history: HashMap<ResourceId, Vec<StateTransitionRecord>>,
}

impl ResourceRegisterLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            locked_resources: HashMap::new(),
            transition_history: HashMap::new(),
        }
    }

    /// Register a new resource in the initial state
    pub fn register_resource(&mut self, resource_id: ResourceId) -> Result<()> {
        if self.states.contains_key(&resource_id) {
            return Err(Error::InvalidOperation(format!(
                "Resource {} already registered",
                resource_id
            )));
        }

        // Insert into states with Initial state
        self.states.insert(resource_id, RegisterState::Initial);
        Ok(())
    }

    /// Get the current state of a resource
    pub fn get_state(&self, resource_id: &ResourceId) -> Result<RegisterState> {
        self.states
            .get(resource_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))
    }

    /// Record a state transition for a resource
    fn record_transition(
        &mut self,
        resource_id: &ResourceId,
        from_state: RegisterState,
        to_state: RegisterState,
        caused_by: Option<&ResourceId>,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::InternalError(format!("Failed to get timestamp: {}", e)))?
            .as_secs();

        let record = StateTransitionRecord {
            resource_id: resource_id.clone(),
            from_state,
            to_state,
            timestamp,
            caused_by: caused_by.cloned(),
            metadata: metadata.unwrap_or_default(),
        };

        // Add to transition history
        self.transition_history
            .entry(resource_id.clone())
            .or_insert_with(Vec::new)
            .push(record);

        Ok(())
    }

    /// Transition a resource to a new state if valid
    fn transition_state(
        &mut self,
        resource_id: &ResourceId,
        to_state: RegisterState,
        caused_by: Option<&ResourceId>,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        let from_state = self.get_state(resource_id)?;

        // Check if transition is valid
        if !self.is_valid_transition(&from_state, &to_state) {
            return Err(Error::InvalidOperation(format!(
                "Invalid state transition from {:?} to {:?} for resource {}",
                from_state, to_state, resource_id
            )));
        }

        // Update state
        self.states.insert(resource_id.clone(), to_state);

        // Record transition
        self.record_transition(resource_id, from_state, to_state, caused_by, metadata)?;

        Ok(())
    }

    /// Check if a state transition is valid
    fn is_valid_transition(&self, from_state: &RegisterState, to_state: &RegisterState) -> bool {
        match (from_state, to_state) {
            // Valid transitions from Initial
            (RegisterState::Initial, RegisterState::Active) => true,
            
            // Valid transitions from Active
            (RegisterState::Active, RegisterState::Locked) => true,
            (RegisterState::Active, RegisterState::Frozen) => true,
            (RegisterState::Active, RegisterState::Consumed) => true,
            (RegisterState::Active, RegisterState::Archived) => true,
            (RegisterState::Active, RegisterState::Pending) => true,
            
            // Valid transitions from Locked
            (RegisterState::Locked, RegisterState::Active) => true,
            (RegisterState::Locked, RegisterState::Consumed) => true,
            
            // Valid transitions from Frozen
            (RegisterState::Frozen, RegisterState::Active) => true,
            (RegisterState::Frozen, RegisterState::Consumed) => true,
            
            // No valid transitions from Consumed (terminal state)
            (RegisterState::Consumed, _) => false,
            
            // Valid transitions from Pending
            (RegisterState::Pending, RegisterState::Active) => true,
            (RegisterState::Pending, RegisterState::Consumed) => true,
            
            // Valid transitions from Archived
            (RegisterState::Archived, RegisterState::Active) => true,
            
            // Any other transition is invalid
            _ => false,
        }
    }

    /// Check if an operation is valid for a given resource state
    pub fn is_operation_valid(
        &self,
        resource_id: &ResourceId,
        operation: &RegisterOperationType,
    ) -> Result<bool> {
        let state = self.get_state(resource_id)?;
        
        Ok(match (&state, operation) {
            // Read is always valid
            (_, RegisterOperationType::Read) => true,
            
            // Initial state
            (RegisterState::Initial, RegisterOperationType::Create) => true,
            
            // Active state
            (RegisterState::Active, RegisterOperationType::Update) => true,
            (RegisterState::Active, RegisterOperationType::Delete) => true,
            (RegisterState::Active, RegisterOperationType::Lock) => true,
            (RegisterState::Active, RegisterOperationType::Freeze) => true,
            (RegisterState::Active, RegisterOperationType::Consume) => true,
            (RegisterState::Active, RegisterOperationType::Archive) => true,
            
            // Locked state
            (RegisterState::Locked, RegisterOperationType::Unlock) => true,
            (RegisterState::Locked, RegisterOperationType::Consume) => true,
            
            // Frozen state
            (RegisterState::Frozen, RegisterOperationType::Unfreeze) => true,
            (RegisterState::Frozen, RegisterOperationType::Consume) => true,
            
            // Pending state
            (RegisterState::Pending, RegisterOperationType::Update) => true,
            (RegisterState::Pending, RegisterOperationType::Consume) => true,
            
            // Archived state
            (RegisterState::Archived, RegisterOperationType::Update) => true,
            
            // Any other operation is invalid for the given state
            _ => false,
        })
    }

    /// Activate a resource (transition from Initial to Active)
    pub fn activate(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.transition_state(
            resource_id,
            RegisterState::Active,
            None,
            None,
        )
    }

    /// Lock a resource
    pub fn lock(&mut self, resource_id: &ResourceId, locker_id: Option<&ResourceId>) -> Result<()> {
        // Check if resource is in a state that can be locked
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Lock)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be locked in its current state",
                resource_id
            )));
        }

        // Transition to locked state
        self.transition_state(
            resource_id,
            RegisterState::Locked,
            locker_id,
            None,
        )?;

        // Record which resource locked this one
        if let Some(locker) = locker_id {
            self.locked_resources
                .entry(locker.clone())
                .or_insert_with(HashSet::new)
                .insert(resource_id.clone());
        }

        Ok(())
    }

    /// Unlock a resource
    pub fn unlock(&mut self, resource_id: &ResourceId, unlocker_id: Option<&ResourceId>) -> Result<()> {
        // Check if resource is in a state that can be unlocked
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Unlock)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be unlocked in its current state",
                resource_id
            )));
        }

        // If unlocker is provided, verify it's the same that locked the resource
        if let Some(unlocker) = unlocker_id {
            if let Some(locked_set) = self.locked_resources.get(unlocker) {
                if !locked_set.contains(resource_id) {
                    return Err(Error::InvalidOperation(format!(
                        "Resource {} was not locked by {}",
                        resource_id, unlocker
                    )));
                }
            } else {
                return Err(Error::InvalidOperation(format!(
                    "Resource {} has no locked resources",
                    unlocker
                )));
            }

            // Remove from locked resources
            if let Some(locked_set) = self.locked_resources.get_mut(unlocker) {
                locked_set.remove(resource_id);
                if locked_set.is_empty() {
                    self.locked_resources.remove(unlocker);
                }
            }
        }

        // Transition to active state
        self.transition_state(
            resource_id,
            RegisterState::Active,
            unlocker_id,
            None,
        )
    }

    /// Freeze a resource
    pub fn freeze(&mut self, resource_id: &ResourceId) -> Result<()> {
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Freeze)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be frozen in its current state",
                resource_id
            )));
        }

        self.transition_state(
            resource_id,
            RegisterState::Frozen,
            None,
            None,
        )
    }

    /// Unfreeze a resource
    pub fn unfreeze(&mut self, resource_id: &ResourceId) -> Result<()> {
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Unfreeze)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be unfrozen in its current state",
                resource_id
            )));
        }

        self.transition_state(
            resource_id,
            RegisterState::Active,
            None,
            None,
        )
    }

    /// Mark a resource as pending consumption
    pub fn mark_pending(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.transition_state(
            resource_id,
            RegisterState::Pending,
            None,
            None,
        )
    }

    /// Consume a resource (terminal state)
    pub fn consume(&mut self, resource_id: &ResourceId) -> Result<()> {
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Consume)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be consumed in its current state",
                resource_id
            )));
        }

        self.transition_state(
            resource_id,
            RegisterState::Consumed,
            None,
            None,
        )
    }

    /// Archive a resource
    pub fn archive(&mut self, resource_id: &ResourceId) -> Result<()> {
        if !self.is_operation_valid(resource_id, &RegisterOperationType::Archive)? {
            return Err(Error::InvalidOperation(format!(
                "Resource {} cannot be archived in its current state",
                resource_id
            )));
        }

        self.transition_state(
            resource_id,
            RegisterState::Archived,
            None,
            None,
        )
    }

    /// Get the transition history for a resource
    pub fn get_transition_history(&self, resource_id: &ResourceId) -> Result<Vec<StateTransitionRecord>> {
        self.transition_history
            .get(resource_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("No transition history for resource {}", resource_id)))
    }

    /// Get all resources locked by a specific resource
    pub fn get_locked_resources(&self, locker_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        Ok(self.locked_resources
            .get(locker_id)
            .cloned()
            .unwrap_or_default())
    }
}

impl Default for ResourceRegisterLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lifecycle_basic_transitions() -> Result<()> {
        let mut manager = ResourceRegisterLifecycleManager::new();
        
        let resource_id = ResourceId("test1".to_string());
        
        // Register and check initial state
        manager.register_resource(resource_id.clone())?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Initial);
        
        // Activate and check state
        manager.activate(&resource_id)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Active);
        
        // Lock and check state
        manager.lock(&resource_id, None)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Locked);
        
        // Unlock and check state
        manager.unlock(&resource_id, None)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Active);
        
        // Freeze and check state
        manager.freeze(&resource_id)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Frozen);
        
        // Unfreeze and check state
        manager.unfreeze(&resource_id)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Active);
        
        // Consume and check state
        manager.consume(&resource_id)?;
        assert_eq!(manager.get_state(&resource_id)?, RegisterState::Consumed);
        
        // Attempt invalid transition from terminal state
        let result = manager.activate(&resource_id);
        assert!(result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_lock_relationships() -> Result<()> {
        let mut manager = ResourceRegisterLifecycleManager::new();
        
        let parent = ResourceId("parent".to_string());
        let child1 = ResourceId("child1".to_string());
        let child2 = ResourceId("child2".to_string());
        
        // Register and activate all resources
        for resource in [parent.clone(), child1.clone(), child2.clone()] {
            manager.register_resource(resource.clone())?;
            manager.activate(&resource)?;
        }
        
        // Parent locks both children
        manager.lock(&child1, Some(&parent))?;
        manager.lock(&child2, Some(&parent))?;
        
        // Check states
        assert_eq!(manager.get_state(&parent)?, RegisterState::Active);
        assert_eq!(manager.get_state(&child1)?, RegisterState::Locked);
        assert_eq!(manager.get_state(&child2)?, RegisterState::Locked);
        
        // Check locked resources
        let locked = manager.get_locked_resources(&parent)?;
        assert_eq!(locked.len(), 2);
        assert!(locked.contains(&child1));
        assert!(locked.contains(&child2));
        
        // Try to unlock with wrong unlocker
        let wrong_unlocker = ResourceId("wrong".to_string());
        manager.register_resource(wrong_unlocker.clone())?;
        manager.activate(&wrong_unlocker)?;
        
        let result = manager.unlock(&child1, Some(&wrong_unlocker));
        assert!(result.is_err());
        
        // Unlock with correct unlocker
        manager.unlock(&child1, Some(&parent))?;
        manager.unlock(&child2, Some(&parent))?;
        
        // Check states after unlock
        assert_eq!(manager.get_state(&child1)?, RegisterState::Active);
        assert_eq!(manager.get_state(&child2)?, RegisterState::Active);
        
        // Check that parent no longer has locked resources
        let locked = manager.get_locked_resources(&parent)?;
        assert_eq!(locked.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_transition_history() -> Result<()> {
        let mut manager = ResourceRegisterLifecycleManager::new();
        
        let resource_id = ResourceId("test-history".to_string());
        
        // Register and perform several transitions
        manager.register_resource(resource_id.clone())?;
        manager.activate(&resource_id)?;
        manager.lock(&resource_id, None)?;
        manager.unlock(&resource_id, None)?;
        manager.freeze(&resource_id)?;
        manager.unfreeze(&resource_id)?;
        manager.consume(&resource_id)?;
        
        // Get transition history
        let history = manager.get_transition_history(&resource_id)?;
        
        // Check history length
        assert_eq!(history.len(), 6);
        
        // Check first transition
        assert_eq!(history[0].from_state, RegisterState::Initial);
        assert_eq!(history[0].to_state, RegisterState::Active);
        
        // Check last transition
        assert_eq!(history[5].from_state, RegisterState::Active);
        assert_eq!(history[5].to_state, RegisterState::Consumed);
        
        Ok(())
    }
} 