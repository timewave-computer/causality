// Resource Manager for Causality Resource System
//
// This module provides a high-level interface for resource management,
// integrated with the unified lifecycle and relationship architecture.
// It maintains backward compatibility while leveraging the new systems.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use crate::types::{ResourceId, DomainId, TraceId};
use crate::error::{Error, Result};
use crate::resource::{
    ResourceAllocator, 
    ResourceRequest, 
    ResourceGrant, 
    ResourceUsage,
    GrantId,
    lifecycle_manager::ResourceRegisterLifecycleManager,
    relationship_tracker::RelationshipTracker,
    RegisterState
};
use crate::log::FactLogger;
use crate::resource::register::RegisterId;

/// Resource Guard that automatically releases resources when dropped
pub struct ResourceGuard {
    /// The resource grant
    grant: Option<ResourceGrant>,
    /// The resource manager
    manager: Arc<ResourceManager>,
    /// The resource ID (if associated with a register)
    register_id: Option<RegisterId>,
}

impl ResourceGuard {
    /// Create a new resource guard
    fn new(grant: ResourceGrant, manager: Arc<ResourceManager>) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            register_id: None,
        }
    }
    
    /// Create a new resource guard with a register ID
    fn with_register(grant: ResourceGrant, manager: Arc<ResourceManager>, register_id: RegisterId) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            register_id: Some(register_id),
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
    
    /// Get the register ID
    pub fn register_id(&self) -> Option<&RegisterId> {
        self.register_id.as_ref()
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
    /// Lifecycle manager for register operations
    lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
    /// Relationship tracker for register relationships
    relationship_tracker: Option<Arc<RelationshipTracker>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(allocator: Arc<dyn ResourceAllocator>, domain_id: DomainId) -> Self {
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            domain_id,
            current_trace: Mutex::new(None),
            lifecycle_manager: None,
            relationship_tracker: None,
        }
    }
    
    /// Create a new resource manager with unified lifecycle and relationship management
    pub fn with_unified_system(
        allocator: Arc<dyn ResourceAllocator>,
        domain_id: DomainId,
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            domain_id,
            current_trace: Mutex::new(None),
            lifecycle_manager: Some(lifecycle_manager),
            relationship_tracker: Some(relationship_tracker),
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
        let grant = self.allocator.allocate(request)
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
        self.allocator.release(grant);
    }
    
    /// Create a register with initial data
    pub fn create_register(
        &self,
        register_id: RegisterId,
        initial_data: &[u8],
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        // Allocate resources for the register
        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
            Error::ResourceError("No grant returned from allocator".to_string())
        })?.clone();
        
        // If unified system is available, create through lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            let state = RegisterState {
                id: register_id.clone(),
                contents: initial_data.to_vec(),
                metadata: Default::default(),
                // Other fields would be initialized with default values
                ..Default::default()
            };
            
            lifecycle_manager.create_resource(state)
                .map_err(|e| Error::ResourceError(format!("Failed to create register: {}", e)))?;
        }
        
        Ok(ResourceGuard::with_register(grant, Arc::new(self.clone()), register_id))
    }
    
    /// Update a register with new data
    pub fn update_register(
        &self,
        register_id: RegisterId,
        new_data: &[u8],
        previous_version: &str,
    ) -> Result<()> {
        // If unified system is available, update through lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            // Get current state
            let current_state = lifecycle_manager.get_resource_state(&register_id)
                .map_err(|e| Error::ResourceError(format!("Failed to get register state: {}", e)))?
                .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", register_id)))?;
            
            // Create updated state
            let mut updated_state = current_state.clone();
            updated_state.contents = new_data.to_vec();
            
            // Update the resource
            lifecycle_manager.update_resource_state(&register_id, updated_state)
                .map_err(|e| Error::ResourceError(format!("Failed to update register: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Transfer a register between domains
    pub fn transfer_register(
        &self,
        register_id: RegisterId,
        source_domain: &str,
        target_domain: &str,
    ) -> Result<()> {
        // If unified system is available, handle through relationship tracking
        if let (Some(lifecycle_manager), Some(relationship_tracker)) = (&self.lifecycle_manager, &self.relationship_tracker) {
            // Create relationship between register and domains
            relationship_tracker.create_relationship(
                &register_id.clone(),
                &ResourceId::from(target_domain.to_string()),
                "transferred_to"
            ).map_err(|e| Error::ResourceError(format!("Failed to create transfer relationship: {}", e)))?;
            
            // Get current state
            let current_state = lifecycle_manager.get_resource_state(&register_id)
                .map_err(|e| Error::ResourceError(format!("Failed to get register state: {}", e)))?
                .ok_or_else(|| Error::ResourceError(format!("Register not found: {:?}", register_id)))?;
            
            // Update domain information in state
            let mut updated_state = current_state.clone();
            // This assumes there's a domain field in the state metadata
            // Actual implementation would depend on how domains are tracked
            
            // Update the resource
            lifecycle_manager.update_resource_state(&register_id, updated_state)
                .map_err(|e| Error::ResourceError(format!("Failed to update register: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Merge multiple registers into a single one
    pub fn merge_registers(
        &self,
        source_registers: &[RegisterId],
        result_register: RegisterId,
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        // Allocate resources for the merged register
        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
            Error::ResourceError("No grant returned from allocator".to_string())
        })?.clone();
        
        // If unified system is available, handle through lifecycle and relationship managers
        if let (Some(lifecycle_manager), Some(relationship_tracker)) = (&self.lifecycle_manager, &self.relationship_tracker) {
            // Create the result register
            let state = RegisterState {
                id: result_register.clone(),
                contents: Vec::new(), // This would be populated with merged data
                metadata: Default::default(),
                // Other fields would be initialized with default values
                ..Default::default()
            };
            
            lifecycle_manager.create_resource(state)
                .map_err(|e| Error::ResourceError(format!("Failed to create merged register: {}", e)))?;
            
            // Create relationships between source and result registers
            for source_id in source_registers {
                relationship_tracker.create_relationship(
                    &result_register.clone(),
                    &source_id.clone(),
                    "merged_from"
                ).map_err(|e| Error::ResourceError(format!("Failed to create merge relationship: {}", e)))?;
            }
        }
        
        Ok(ResourceGuard::with_register(grant, Arc::new(self.clone()), result_register))
    }
    
    /// Split a register into multiple ones
    pub fn split_register(
        &self,
        source_register: RegisterId,
        result_registers: &[RegisterId],
        requests: Vec<ResourceRequest>,
    ) -> Result<Vec<ResourceGuard>> {
        // Ensure we have enough resource requests
        if requests.len() != result_registers.len() {
            return Err(Error::ResourceError(
                format!("Mismatch between result registers ({}) and resource requests ({})",
                    result_registers.len(), requests.len())
            ));
        }
        
        // Allocate resources for each result register
        let mut guards = Vec::with_capacity(result_registers.len());
        
        for (i, result_id) in result_registers.iter().enumerate() {
            let guard = self.allocate_resources(requests[i].clone())?;
            guards.push(ResourceGuard::with_register(
                guard.grant().ok_or_else(|| {
                    Error::ResourceError("No grant returned from allocator".to_string())
                })?.clone(),
                Arc::new(self.clone()),
                result_id.clone()
            ));
        }
        
        // If unified system is available, handle through lifecycle and relationship managers
        if let (Some(lifecycle_manager), Some(relationship_tracker)) = (&self.lifecycle_manager, &self.relationship_tracker) {
            // Get source register state
            let source_state = lifecycle_manager.get_resource_state(&source_register)
                .map_err(|e| Error::ResourceError(format!("Failed to get source register state: {}", e)))?
                .ok_or_else(|| Error::ResourceError(format!("Source register not found: {:?}", source_register)))?;
            
            // Create each result register
            for result_id in result_registers {
                // In a real implementation, each result register would contain a portion of the source data
                let state = RegisterState {
                    id: result_id.clone(),
                    contents: Vec::new(), // Would be populated with split data
                    metadata: source_state.metadata.clone(),
                    // Other fields would be initialized with default values
                    ..Default::default()
                };
                
                lifecycle_manager.create_resource(state)
                    .map_err(|e| Error::ResourceError(format!("Failed to create result register: {}", e)))?;
                
                // Create relationship between source and result register
                relationship_tracker.create_relationship(
                    &result_id.clone(),
                    &source_register.clone(),
                    "split_from"
                ).map_err(|e| Error::ResourceError(format!("Failed to create split relationship: {}", e)))?;
            }
        }
        
        Ok(guards)
    }
    
    /// Check resource usage
    pub fn check_usage(&self, guard: &ResourceGuard) -> Result<ResourceUsage> {
        guard.grant().ok_or_else(|| {
            Error::ResourceError("Guard has no grant".to_string())
        }).map(|grant| {
            self.allocator.check_usage(grant)
        })
    }
    
    /// Validate that a guard is still valid
    pub fn validate_guard(&self, guard: &ResourceGuard) -> Result<()> {
        // Check if the grant is in active grants
        let active_grants = self.active_grants.read().unwrap();
        if let Some(grant_id) = guard.grant_id() {
            if !active_grants.contains_key(grant_id) {
                return Err(Error::ResourceError("Guard is no longer valid".to_string()));
            }
        }
        Ok(())
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the lifecycle manager if available
    pub fn lifecycle_manager(&self) -> Option<&Arc<ResourceRegisterLifecycleManager>> {
        self.lifecycle_manager.as_ref()
    }
    
    /// Get the relationship tracker if available
    pub fn relationship_tracker(&self) -> Option<&Arc<RelationshipTracker>> {
        self.relationship_tracker.as_ref()
    }
}

impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        ResourceManager {
            allocator: self.allocator.clone(),
            active_grants: RwLock::new(HashMap::new()),
            domain_id: self.domain_id.clone(),
            current_trace: Mutex::new(self.get_trace()),
            lifecycle_manager: self.lifecycle_manager.clone(),
            relationship_tracker: self.relationship_tracker.clone(),
        }
    }
}

/// A shared resource manager
pub type SharedResourceManager = Arc<ResourceManager>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::{ResourceAllocator, ResourceRequest, ResourceGrant, ResourceUsage, GrantId};
    use std::sync::Arc;
    
    #[test]
    fn test_resource_allocation() {
        let manager = ResourceManager::new(
            Arc::new(MockAllocator {}),
            "test-domain".to_string(),
        );
        
        let request = ResourceRequest {
            memory: 1024,
            cpu: 1,
            storage: 2048,
            network: 100,
        };
        
        let guard = manager.allocate_resources(request).unwrap();
        assert!(guard.grant().is_some());
        assert_eq!(guard.grant().unwrap().memory(), 1024);
    }
    
    #[test]
    fn test_create_register() {
        let manager = ResourceManager::new(
            Arc::new(MockAllocator {}),
            "test-domain".to_string(),
        );
        
        let register_id = "test-register-1".to_string();
        let initial_data = b"test data";
        let request = ResourceRequest {
            memory: 1024,
            cpu: 1,
            storage: 2048,
            network: 100,
        };
        
        let guard = manager.create_register(register_id.clone(), initial_data, request).unwrap();
        assert!(guard.grant().is_some());
        assert_eq!(guard.register_id().unwrap(), &register_id);
    }
    
    // Additional tests would be updated to use the unified system
    
    #[derive(Clone)]
    struct MockAllocator {}
    
    impl ResourceAllocator for MockAllocator {
        fn allocate(&self, request: ResourceRequest) -> Result<ResourceGrant> {
            Ok(ResourceGrant {
                grant_id: GrantId(format!("grant-{}", rand::random::<u64>())),
                memory: request.memory,
                cpu: request.cpu,
                storage: request.storage,
                network: request.network,
            })
        }
        
        fn release(&self, _grant: ResourceGrant) {
            // Mock implementation
        }
        
        fn check_usage(&self, grant: &ResourceGrant) -> ResourceUsage {
            ResourceUsage {
                memory_used: grant.memory / 2,
                cpu_used: grant.cpu / 2,
                storage_used: grant.storage / 2,
                network_used: grant.network / 2,
            }
        }
    }
} 