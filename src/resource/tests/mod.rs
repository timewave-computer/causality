// Resource system tests

// Import test modules
mod resource_register_tests;
mod effect_tests;
mod storage_tests;
mod capability_tests;
mod api_tests;
mod archival_test;
mod archival_integration_test;
mod summarization_test;
mod summarization_integration_test;
mod epoch_test;
mod versioning_test;
mod garbage_collection_test;
mod effect_template_integration_tests;
mod lifecycle_helper_tests;

// Re-export test helpers for use in other tests
pub use resource_register_tests::create_test_resource;
pub use resource_register_tests::create_test_resource_with_id;
pub use capability_tests::create_test_capability;
pub use capability_tests::create_test_capability_with_id;
pub use api_tests::create_test_api;
pub use api_tests::create_test_api_with_id;
pub use storage_tests::create_test_storage;
pub use storage_tests::create_test_storage_with_id;

// Test utilities for resource system
//
// This module provides test utilities for the resource system, including
// fixtures, helpers, and test data generators.

use crate::error::Result;
use crate::types::{ResourceId, RegisterState, DomainId, Metadata};
use crate::resource::{ResourceRegister, ResourceManager};
use crate::resource::lifecycle_manager::ResourceRegisterLifecycleManager;
use crate::resource::resource_register::ResourceRegister;
use crate::address::Address;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Helper for testing resource state transitions
///
/// This struct provides a consistent way to test resource lifecycle transitions
/// and validate state change sequences.
pub struct ResourceStateTransitionHelper {
    /// The resource manager being tested
    resource_manager: ResourceManager,
    /// Whether synchronous or asynchronous transitions should be used
    synchronous: bool,
}

impl ResourceStateTransitionHelper {
    /// Create a new test helper with a fresh resource manager
    pub fn new(synchronous: bool) -> Self {
        let lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
        
        Self {
            resource_manager,
            synchronous,
        }
    }
    
    /// Create a new test helper with an existing resource manager
    pub fn with_manager(resource_manager: ResourceManager, synchronous: bool) -> Self {
        Self {
            resource_manager,
            synchronous,
        }
    }
    
    /// Add a resource to the manager
    pub fn add_resource(&mut self, resource: ResourceRegister) -> Result<()> {
        self.resource_manager.add_resource(resource)
    }
    
    /// Get a resource from the manager
    pub fn get_resource(&self, resource_id: &ResourceId) -> Result<ResourceRegister> {
        self.resource_manager.get_resource(resource_id)
    }
    
    /// Transition a resource to the active state
    pub fn activate(&mut self, resource_id: &ResourceId) -> Result<()> {
        if self.synchronous {
            self.resource_manager.activate_resource(resource_id)
        } else {
            // For the async version, we'd use a runtime to run the async activate
            // Since this is a test helper, we'll just use the sync version for now
            self.resource_manager.activate_resource(resource_id)
        }
    }
    
    /// Transition a resource to the locked state
    pub fn lock(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.lock_resource(resource_id)
    }
    
    /// Transition a resource to the active state from locked
    pub fn unlock(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.unlock_resource(resource_id)
    }
    
    /// Transition a resource to the frozen state
    pub fn freeze(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.freeze_resource(resource_id)
    }
    
    /// Transition a resource to the active state from frozen
    pub fn unfreeze(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.unfreeze_resource(resource_id)
    }
    
    /// Transition a resource to the consumed state
    pub fn consume(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.consume_resource(resource_id)
    }
    
    /// Transition a resource to the archived state
    pub fn archive(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.archive_resource(resource_id)
    }
    
    /// Mark a resource as pending
    pub fn mark_pending(&mut self, resource_id: &ResourceId) -> Result<()> {
        self.resource_manager.mark_resource_pending(resource_id)
    }
    
    /// Execute a common transition sequence (Initial -> Active -> Locked -> Active -> Consumed)
    pub fn execute_common_sequence(&mut self, resource_id: &ResourceId) -> Result<Vec<RegisterState>> {
        let mut states = Vec::new();
        
        // Record initial state
        let initial_resource = self.get_resource(resource_id)?;
        states.push(initial_resource.state);
        
        // Activate
        self.activate(resource_id)?;
        let active_resource = self.get_resource(resource_id)?;
        states.push(active_resource.state);
        
        // Lock
        self.lock(resource_id)?;
        let locked_resource = self.get_resource(resource_id)?;
        states.push(locked_resource.state);
        
        // Unlock
        self.unlock(resource_id)?;
        let unlocked_resource = self.get_resource(resource_id)?;
        states.push(unlocked_resource.state);
        
        // Consume
        self.consume(resource_id)?;
        let consumed_resource = self.get_resource(resource_id)?;
        states.push(consumed_resource.state);
        
        Ok(states)
    }
    
    /// Execute a freezing sequence (Initial -> Active -> Frozen -> Active -> Archived)
    pub fn execute_freezing_sequence(&mut self, resource_id: &ResourceId) -> Result<Vec<RegisterState>> {
        let mut states = Vec::new();
        
        // Record initial state
        let initial_resource = self.get_resource(resource_id)?;
        states.push(initial_resource.state);
        
        // Activate
        self.activate(resource_id)?;
        let active_resource = self.get_resource(resource_id)?;
        states.push(active_resource.state);
        
        // Freeze
        self.freeze(resource_id)?;
        let frozen_resource = self.get_resource(resource_id)?;
        states.push(frozen_resource.state);
        
        // Unfreeze
        self.unfreeze(resource_id)?;
        let unfrozen_resource = self.get_resource(resource_id)?;
        states.push(unfrozen_resource.state);
        
        // Archive
        self.archive(resource_id)?;
        let archived_resource = self.get_resource(resource_id)?;
        states.push(archived_resource.state);
        
        Ok(states)
    }
    
    /// Validate a sequence of state transitions
    pub fn validate_transition_sequence(&self, actual_states: &[RegisterState], expected_states: &[RegisterState]) -> bool {
        if actual_states.len() != expected_states.len() {
            return false;
        }
        
        for (i, state) in actual_states.iter().enumerate() {
            if *state != expected_states[i] {
                return false;
            }
        }
        
        true
    }
    
    /// Get the underlying resource manager
    pub fn get_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }
    
    /// Get a mutable reference to the underlying resource manager
    pub fn get_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }
} 