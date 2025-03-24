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
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::resource::capability_system::AuthorizationService;
use crate::resource::relationship_tracker::RelationshipTracker;
use crate::relationship::cross_domain_query::{ResourceStateTransitionHelper, RelationshipQueryExecutor};
use crate::resource::CapabilityId;

/// Type of operation on a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegisterOperationType {
    /// Create a new register
    Create,
    
    /// Update an existing register
    Update,
    
    /// Lock a register
    Lock,
    
    /// Unlock a register
    Unlock,
    
    /// Freeze a register
    Freeze,
    
    /// Unfreeze a register
    Unfreeze,
    
    /// Consume a register
    Consume,
    
    /// Archive a register
    Archive,
}

/// Record of a state transition for a resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionRecord {
    pub resource_id: ContentId,
    pub from_state: RegisterState,
    pub to_state: RegisterState,
    pub timestamp: Timestamp,
    pub caused_by: Option<ContentId>,
    pub metadata: Metadata,
}

/// The Resource Register Lifecycle Manager manages the lifecycle states
/// of resources and enforces valid state transitions.
pub struct ResourceRegisterLifecycleManager {
    /// Map of resource ID to current state
    states: HashMap<ContentId, RegisterState>,
    /// Map of resource ID to resources it has locked
    locked_resources: HashMap<ContentId, HashSet<ContentId>>,
    /// History of state transitions for each resource
    transition_history: HashMap<ContentId, Vec<StateTransitionRecord>>,
    /// Valid state transitions
    valid_transitions: HashMap<RegisterState, HashSet<RegisterState>>,
    /// Relationship tracker for validating relationship constraints during transitions
    relationship_tracker: Option<Arc<RelationshipTracker>>,
    /// State transition helper for cross-domain relationship validation
    state_transition_helper: Option<Arc<ResourceStateTransitionHelper>>,
}

impl ResourceRegisterLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        let mut manager = Self {
            states: HashMap::new(),
            locked_resources: HashMap::new(),
            transition_history: HashMap::new(),
            valid_transitions: HashMap::new(),
            relationship_tracker: None,
            state_transition_helper: None,
        };
        
        // Initialize valid transitions
        manager.initialize_valid_transitions();
        
        manager
    }

    /// Create a new lifecycle manager with relationship tracking
    pub fn with_relationship_tracker(relationship_tracker: Arc<RelationshipTracker>) -> Self {
        let mut manager = Self::new();
        manager.relationship_tracker = Some(relationship_tracker);
        manager
    }
    
    /// Set the state transition helper for cross-domain relationship validation
    pub fn with_state_transition_helper(
        mut self,
        query_executor: Arc<RelationshipQueryExecutor>, 
        relationship_tracker: Arc<RelationshipTracker>
    ) -> Self {
        // Create the state transition helper
        let helper = ResourceStateTransitionHelper::new(query_executor, relationship_tracker.clone());
        self.state_transition_helper = Some(Arc::new(helper));
        
        // Make sure we also have the relationship tracker set
        if self.relationship_tracker.is_none() {
            self.relationship_tracker = Some(relationship_tracker);
        }
        
        self
    }

    /// Initialize the valid state transitions
    fn initialize_valid_transitions(&mut self) {
        // Initial state transitions
        let mut initial_transitions = HashSet::new();
        initial_transitions.insert(RegisterState::Active);
        self.valid_transitions.insert(RegisterState::Initial, initial_transitions);
        
        // Active state transitions
        let mut active_transitions = HashSet::new();
        active_transitions.insert(RegisterState::Locked);
        active_transitions.insert(RegisterState::Frozen);
        active_transitions.insert(RegisterState::Consumed);
        active_transitions.insert(RegisterState::Archived);
        self.valid_transitions.insert(RegisterState::Active, active_transitions);
        
        // Locked state transitions
        let mut locked_transitions = HashSet::new();
        locked_transitions.insert(RegisterState::Active);
        self.valid_transitions.insert(RegisterState::Locked, locked_transitions);
        
        // Frozen state transitions
        let mut frozen_transitions = HashSet::new();
        frozen_transitions.insert(RegisterState::Active);
        self.valid_transitions.insert(RegisterState::Frozen, frozen_transitions);
        
        // Consumed and Archived have no valid transitions (terminal states)
    }

    /// Register a new resource in the initial state
    pub fn register_resource(&mut self, resource_id: ContentId) -> Result<()> {
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
    pub fn get_state(&self, resource_id: &ContentId) -> Result<RegisterState> {
        self.states
            .get(resource_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))
    }

    /// Record a state transition for a resource
    fn record_transition(
        &mut self,
        resource_id: &ContentId,
        from_state: RegisterState,
        to_state: RegisterState,
        caused_by: Option<&ContentId>,
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

    /// Validate a state transition considering relationships
    async fn validate_state_transition_with_relationships(
        &self,
        resource_id: &ContentId,
        from_state: &RegisterState,
        to_state: &RegisterState,
    ) -> Result<bool> {
        // First check basic transition validity
        if !self.is_valid_transition(from_state, to_state) {
            return Ok(false);
        }
        
        // If we have a state transition helper, use it to validate relationships
        if let Some(helper) = &self.state_transition_helper {
            // Convert RegisterState to string for the helper
            let from_str = format!("{:?}", from_state);
            let to_str = format!("{:?}", to_state);
            
            // Validate relationships for this transition
            match helper.validate_relationships_for_transition(
                resource_id,
                &from_str,
                &to_str
            ).await {
                Ok(valid) => return Ok(valid),
                Err(e) => return Err(Error::InvalidOperation(format!(
                    "Failed to validate relationships for transition: {}", e
                ))),
            }
        }
        
        // If no helper or no relationships to check, default to base validation
        Ok(true)
    }
    
    /// Transition state with relationship validation
    pub async fn transition_state_async(
        &mut self,
        resource_id: &ContentId,
        to_state: RegisterState,
        caused_by: Option<&ContentId>,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        // Get current state
        let from_state = self.get_state(resource_id)?;
        
        // Validate the transition including relationships
        let is_valid = self.validate_state_transition_with_relationships(
            resource_id, 
            &from_state, 
            &to_state
        ).await?;
        
        if !is_valid {
            return Err(Error::InvalidOperation(format!(
                "Invalid state transition from {:?} to {:?} for resource {}",
                from_state, to_state, resource_id
            )));
        }
        
        // Update state
        self.states.insert(resource_id.clone(), to_state.clone());
        
        // Record the transition
        self.record_transition(resource_id, from_state, to_state, caused_by, metadata)?;
        
        // If we have a state transition helper, update relationships after transition
        if let Some(helper) = &self.state_transition_helper {
            // Convert RegisterState to string for the helper
            let from_str = format!("{:?}", from_state);
            let to_str = format!("{:?}", to_state);
            
            // Update relationships after this transition
            if let Err(e) = helper.update_relationships_after_transition(
                resource_id,
                &from_str,
                &to_str
            ).await {
                return Err(Error::InvalidOperation(format!(
                    "Failed to update relationships after transition: {}", e
                )));
            }
        }
        
        Ok(())
    }

    /// Check if a state transition is valid
    fn is_valid_transition(&self, from_state: &RegisterState, to_state: &RegisterState) -> bool {
        self.valid_transitions
            .get(from_state)
            .map_or(false, |transitions| transitions.contains(to_state))
    }

    /// Check if an operation is valid for a given resource state
    pub fn is_operation_valid(
        &self,
        resource_id: &ContentId,
        operation: &RegisterOperationType,
    ) -> Result<bool> {
        let state = self.get_state(resource_id)?;
        
        Ok(match (&state, operation) {
            // Read is always valid
            (_, RegisterOperationType::Read) => true,
            
            // Initial state
            (RegisterState::Initial, RegisterOperationType::Register) => true,
            
            // Active state
            (RegisterState::Active, RegisterOperationType::Update) => true,
            (RegisterState::Active, RegisterOperationType::Cancel) => true,
            (RegisterState::Active, RegisterOperationType::Lock) => true,
            (RegisterState::Active, RegisterOperationType::Freeze) => true,
            (RegisterState::Active, RegisterOperationType::Consume) => true,
            (RegisterState::Active, RegisterOperationType::Deactivate) => true,
            
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

    /// Validate an operation against provided capabilities
    pub fn validate_operation(
        &self,
        resource_id: &ContentId,
        operation_type: RegisterOperationType,
        capability_ids: &[CapabilityId],
    ) -> Result<bool> {
        // Create a temporary authorization service
        // In a real implementation, this would be provided via dependency injection
        let auth_service = AuthorizationService::new(Arc::new(self.clone()));
        
        // Check if the operation is allowed
        auth_service.check_operation_allowed(resource_id, operation_type, capability_ids)
    }
    
    /// Execute an operation only if it's allowed by the provided capabilities
    pub fn execute_with_capabilities(
        &self,
        resource_id: &ContentId,
        operation_type: RegisterOperationType,
        capability_ids: &[CapabilityId],
        // Using a function pointer for the operation to execute
        operation: fn(&Self, &ContentId) -> Result<()>,
    ) -> Result<()> {
        // Validate the operation first
        if !self.validate_operation(resource_id, operation_type, capability_ids)? {
            return Err(Error::PermissionDenied(format!(
                "Operation {:?} not allowed on resource {} with the provided capabilities",
                operation_type, resource_id
            )));
        }
        
        // Execute the operation
        operation(self, resource_id)
    }
    
    /// Activate a resource if allowed by capabilities
    pub fn activate_with_capabilities(
        &self, 
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Activate,
            capability_ids,
            Self::activate,
        )
    }
    
    /// Lock a resource if allowed by capabilities
    pub fn lock_with_capabilities(
        &self,
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Lock,
            capability_ids,
            Self::lock,
        )
    }
    
    /// Unlock a resource if allowed by capabilities
    pub fn unlock_with_capabilities(
        &self,
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Unlock,
            capability_ids,
            Self::unlock,
        )
    }
    
    /// Freeze a resource if allowed by capabilities
    pub fn freeze_with_capabilities(
        &self,
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Freeze,
            capability_ids,
            Self::freeze,
        )
    }
    
    /// Unfreeze a resource if allowed by capabilities
    pub fn unfreeze_with_capabilities(
        &self,
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Unfreeze,
            capability_ids,
            Self::unfreeze,
        )
    }
    
    /// Consume a resource if allowed by capabilities
    pub fn consume_with_capabilities(
        &self,
        resource_id: &ContentId,
        capability_ids: &[CapabilityId],
    ) -> Result<()> {
        self.execute_with_capabilities(
            resource_id,
            RegisterOperationType::Consume,
            capability_ids,
            Self::consume,
        )
    }

    /// Activate a resource (transition from Initial to Active)
    pub fn activate(&mut self, resource_id: &ContentId) -> Result<()> {
        self.transition_state(
            resource_id,
            RegisterState::Active,
            None,
            None,
        )
    }

    /// Lock a resource
    pub fn lock(&mut self, resource_id: &ContentId, locker_id: Option<&ContentId>) -> Result<()> {
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
    pub fn unlock(&mut self, resource_id: &ContentId, unlocker_id: Option<&ContentId>) -> Result<()> {
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
    pub fn freeze(&mut self, resource_id: &ContentId) -> Result<()> {
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
    pub fn unfreeze(&mut self, resource_id: &ContentId) -> Result<()> {
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
    pub fn mark_pending(&mut self, resource_id: &ContentId) -> Result<()> {
        self.transition_state(
            resource_id,
            RegisterState::Pending,
            None,
            None,
        )
    }

    /// Consume a resource (terminal state)
    pub fn consume(&mut self, resource_id: &ContentId) -> Result<()> {
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
    pub fn archive(&mut self, resource_id: &ContentId) -> Result<()> {
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
    pub fn get_transition_history(&self, resource_id: &ContentId) -> Result<Vec<StateTransitionRecord>> {
        self.transition_history
            .get(resource_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("No transition history for resource {}", resource_id)))
    }

    /// Get all resources locked by a specific resource
    pub fn get_locked_resources(&self, locker_id: &ContentId) -> Result<HashSet<ContentId>> {
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
        
        let resource_id = ContentId::new("test1".to_string());
        
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
        
        let parent = ContentId::new("parent".to_string());
        let child1 = ContentId::new("child1".to_string());
        let child2 = ContentId::new("child2".to_string());
        
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
        let wrong_unlocker = ContentId::new("wrong".to_string());
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
        
        let resource_id = ContentId::new("test-history".to_string());
        
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
