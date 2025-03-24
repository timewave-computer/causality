// Resource Manager for Causality Resource System
//
// This module implements a unified resource manager for the ResourceRegister model
// as defined in ADR-021. It provides an interface for resource management operations
// including creation, updating, and lifecycle management of ResourceRegisters.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use crate::types::*;
use crate::types::trace::TraceId;
use crate::crypto::hash::ContentId;
use crate::error::{Error, Result};
use crate::resource::{
    allocator::ResourceAllocator, 
    request::{ResourceRequest, ResourceGrant, GrantId},
    usage::ResourceUsage,
    lifecycle_manager::ResourceRegisterLifecycleManager,
    relationship_tracker::RelationshipTracker,
    resource_register::{
        ResourceRegister, ResourceLogic, FungibilityDomain, 
        Quantity, RegisterState, StorageStrategy, StateVisibility
    }
};
use crate::resource::boundary_manager::BoundaryAwareResourceManager;
use crate::crypto::hash::ContentAddressed;
use crate::effect::Effect;
use crate::domain::DomainId;

/// Resource Guard that automatically releases resources when dropped
pub struct ResourceGuard {
    /// The resource grant
    grant: Option<ResourceGrant>,
    /// The resource manager
    manager: Arc<ResourceManager>,
    /// The resource content ID (for ResourceRegister)
    content_id: Option<ContentId>,
}

impl ResourceGuard {
    /// Create a new resource guard
    fn new(grant: ResourceGrant, manager: Arc<ResourceManager>) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            content_id: None,
        }
    }
    
    /// Create a new resource guard with a content ID
    fn with_content_id(grant: ResourceGrant, manager: Arc<ResourceManager>, content_id: ContentId) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            content_id: Some(content_id),
        }
    }
    
    /// Get a reference to the grant
    pub fn grant(&self) -> Option<&ResourceGrant> {
        self.grant.as_ref()
    }
    
    /// Get the grant ID
    pub fn grant_id(&self) -> Option<&GrantId> {
        self.grant.as_ref().map(|g| g.id())
    }
    
    /// Get the content ID
    pub fn content_id(&self) -> Option<&ContentId> {
        self.content_id.as_ref()
    }
    
    /// Release the resources manually
    pub fn release(&mut self) {
        if let Some(grant) = self.grant.take() {
            self.manager.release_resources(grant);
        }
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.release();
    }
}

/// Manager for resource allocation and tracking, integrated with the unified architecture
pub struct ResourceManager {
    /// The underlying resource allocator
    allocator: Arc<dyn ResourceAllocator>,
    /// Active resource grants
    active_grants: RwLock<HashMap<GrantId, ResourceGrant>>,
    /// Domain ID for this manager
    domain_id: DomainId,
    /// Current trace ID
    current_trace: Mutex<Option<TraceId>>,
    /// Lifecycle manager for resource operations
    lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
    /// Relationship tracker for resource relationships
    relationship_tracker: Option<Arc<RelationshipTracker>>,
    /// Boundary manager for boundary-aware operations
    boundary_manager: Arc<BoundaryAwareResourceManager>,
    /// Registry of resource registers
    registers: RwLock<HashMap<ContentId, ResourceRegister>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(allocator: Arc<dyn ResourceAllocator>, domain_id: DomainId) -> Self {
        // Create the boundary manager
        let boundary_manager = Arc::new(BoundaryAwareResourceManager::new(
            None,
        ));
        
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            domain_id,
            current_trace: Mutex::new(None),
            lifecycle_manager: None,
            relationship_tracker: None,
            boundary_manager,
            registers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new resource manager with unified lifecycle and relationship management
    pub fn with_unified_system(
        allocator: Arc<dyn ResourceAllocator>,
        domain_id: DomainId,
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        // Create the boundary manager
        let boundary_manager = Arc::new(BoundaryAwareResourceManager::new(
            lifecycle_manager.clone(),
        ));
        
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            domain_id,
            current_trace: Mutex::new(None),
            lifecycle_manager: Some(lifecycle_manager),
            relationship_tracker: Some(relationship_tracker),
            boundary_manager,
            registers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Set the current trace ID
    pub fn set_trace(&self, trace_id: TraceId) {
        let mut current = self.current_trace.lock().unwrap();
        *current = Some(trace_id);
    }
    
    /// Get the current trace ID
    fn get_trace(&self) -> Option<TraceId> {
        let current = self.current_trace.lock().unwrap();
        current.clone()
    }
    
    /// Allocate resources
    pub fn allocate_resources(&self, request: ResourceRequest) -> Result<ResourceGuard> {
        let grant = self.allocator.allocate(&request)
            .map_err(|e| Error::ResourceError(format!("Failed to allocate resources: {}", e)))?;
        
        // Add to active grants
        let mut active_grants = self.active_grants.write().unwrap();
        active_grants.insert(grant.grant_id.clone(), grant.clone());
        
        Ok(ResourceGuard::new(grant, Arc::new(self.clone())))
    }
    
    /// Release resources
    pub fn release_resources(&self, grant: ResourceGrant) {
        // Remove from active grants
        let mut active_grants = self.active_grants.write().unwrap();
        active_grants.remove(&grant.grant_id);
        
        // Release through allocator
        self.allocator.release(&grant);
    }
    
    /// Create a new ResourceRegister
    pub fn create_resource_register(
        &self,
        register: ResourceRegister,
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        // Allocate resources for the register
        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
            Error::ResourceError("No grant returned from allocator".to_string())
        })?.clone();
        
        let content_id = register.id.clone();
        
        // If unified system is available, create through lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            lifecycle_manager.register_resource(register.id.clone())
                .map_err(|e| Error::ResourceError(format!("Failed to register resource: {}", e)))?;
                
            // Store the register in our registry
            let mut registers = self.registers.write().unwrap();
            registers.insert(content_id.clone(), register);
        } else {
            // Legacy mode: Just store in our registry
            let mut registers = self.registers.write().unwrap();
            registers.insert(content_id.clone(), register);
        }
        
        Ok(ResourceGuard::with_content_id(grant, Arc::new(self.clone()), content_id))
    }
    
    /// Get a ResourceRegister by its content ID
    pub fn get_resource_register(&self, content_id: &ContentId) -> Result<Option<ResourceRegister>> {
        let registers = self.registers.read().unwrap();
        
        match registers.get(content_id) {
            Some(register) => Ok(Some(register.clone())),
            None => {
                // If not in our registry, try to get from lifecycle manager
                if let Some(lifecycle_manager) = &self.lifecycle_manager {
                    lifecycle_manager.get_state(content_id)
                        .map(|_| {
                            // State exists but we don't have the register in our cache
                            // This is a placeholder - in a real implementation we would
                            // retrieve the full register from storage
                            None
                        })
                        .map_err(|e| Error::ResourceError(format!("Failed to get register: {}", e)))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// Update a ResourceRegister
    pub fn update_resource_register(
        &self,
        content_id: &ContentId,
        update_fn: impl FnOnce(&mut ResourceRegister) -> Result<()>,
    ) -> Result<()> {
        // If unified system is available, update through lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            // Get current register
            let register = self.get_resource_register(content_id)?
                .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", content_id)))?;
            
            // Create updated register
            let mut updated_register = register.clone();
            update_fn(&mut updated_register)?;
            
            // Ensure we have valid state transition
            let current_state = lifecycle_manager.get_state(content_id)?;
            if current_state != updated_register.state {
                // Handle state transition through lifecycle manager explicitly
                match updated_register.state {
                    RegisterState::Active => lifecycle_manager.activate(content_id)?,
                    RegisterState::Locked => lifecycle_manager.lock(content_id, None)?,
                    RegisterState::Frozen => lifecycle_manager.freeze(content_id)?,
                    RegisterState::Consumed => lifecycle_manager.consume(content_id)?,
                    RegisterState::Archived => lifecycle_manager.archive(content_id)?,
                    _ => {} // Other states don't need special handling
                }
            }
                
            // Update our registry
            let mut registers = self.registers.write().unwrap();
            registers.insert(content_id.clone(), updated_register);
            
            Ok(())
        } else {
            // Legacy mode: Update in our registry
            let mut registers = self.registers.write().unwrap();
            
            match registers.get_mut(content_id) {
                Some(register) => {
                    update_fn(register)?;
        Ok(())
                },
                None => Err(Error::ResourceError(format!("Register not found: {:?}", content_id))),
            }
        }
    }
    
    /// Lock a ResourceRegister
    pub fn lock_resource_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        // If unified system is available, use lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            lifecycle_manager.lock(content_id, None)
                .map_err(|e| Error::ResourceError(format!("Failed to lock register: {}", e)))?;
                
            // Update our registry
            if let Some(mut register) = self.get_resource_register(content_id)? {
                register.state = RegisterState::Locked;
                
                let mut registers = self.registers.write().unwrap();
                registers.insert(content_id.clone(), register);
            }
            
            Ok(())
        } else {
            // Legacy mode: Update in our registry
            self.update_resource_register(content_id, |register| {
                register.state = RegisterState::Locked;
                Ok(())
            })
        }
    }
    
    /// Unlock a ResourceRegister
    pub fn unlock_resource_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        // If unified system is available, use lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            lifecycle_manager.unlock(content_id, None)
                .map_err(|e| Error::ResourceError(format!("Failed to unlock register: {}", e)))?;
                
            // Update our registry
            if let Some(mut register) = self.get_resource_register(content_id)? {
                register.state = RegisterState::Active;
                
                let mut registers = self.registers.write().unwrap();
                registers.insert(content_id.clone(), register);
            }
            
            Ok(())
        } else {
            // Legacy mode: Update in our registry
            self.update_resource_register(content_id, |register| {
                register.state = RegisterState::Active;
                Ok(())
            })
        }
    }
    
    /// Consume a ResourceRegister
    pub fn consume_resource_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        // If unified system is available, use lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            lifecycle_manager.consume(content_id)
                .map_err(|e| Error::ResourceError(format!("Failed to consume register: {}", e)))?;
                
            // Update our registry
            if let Some(mut register) = self.get_resource_register(content_id)? {
                register.state = RegisterState::Consumed;
                
                let mut registers = self.registers.write().unwrap();
                registers.insert(content_id.clone(), register);
            }
            
            Ok(())
        } else {
            // Legacy mode: Update in our registry
            self.update_resource_register(content_id, |register| {
                register.state = RegisterState::Consumed;
                Ok(())
            })
        }
    }
    
    // Legacy compatibility methods - these redirect to the unified methods
    
    /// Create a register (legacy API, redirects to create_resource_register)
    pub fn create_register(
        &self,
        register: ResourceRegister,
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        self.create_resource_register(register, request)
    }
    
    /// Get a register (legacy API, redirects to get_resource_register)
    pub fn get_register(&self, content_id: &ContentId) -> Result<Option<ResourceRegister>> {
        self.get_resource_register(content_id)
    }
    
    /// Update a register (legacy API, redirects to update_resource_register)
    pub fn update_register(
        &self,
        content_id: &ContentId,
        update_fn: impl FnOnce(&mut ResourceRegister) -> Result<()>,
    ) -> Result<()> {
        self.update_resource_register(content_id, update_fn)
    }
    
    /// Lock a register (legacy API, redirects to lock_resource_register)
    pub fn lock_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        self.lock_resource_register(content_id, reason)
    }
    
    /// Unlock a register (legacy API, redirects to unlock_resource_register)
    pub fn unlock_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        self.unlock_resource_register(content_id, reason)
    }
    
    /// Consume a register (legacy API, redirects to consume_resource_register)
    pub fn consume_register(&self, content_id: &ContentId, reason: &str) -> Result<()> {
        self.consume_resource_register(content_id, reason)
    }
}

impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        ResourceManager {
            allocator: Arc::clone(&self.allocator),
            active_grants: RwLock::new(HashMap::new()),
            domain_id: self.domain_id.clone(),
            current_trace: Mutex::new(self.get_trace()),
            lifecycle_manager: self.lifecycle_manager.clone(),
            relationship_tracker: self.relationship_tracker.clone(),
            boundary_manager: Arc::clone(&self.boundary_manager),
            registers: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_register_creation() {
        // Implement test cases
    }
    
    #[test]
    fn test_resource_register_update() {
        // Implement test cases
    }
    
    #[test]
    fn test_resource_register_relationships() {
        // Implement test cases
    }
} 
