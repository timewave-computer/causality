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
    allocator::ResourceAllocator, 
    request::{ResourceRequest, ResourceGrant, GrantId},
    usage::ResourceUsage,
    lifecycle_manager::ResourceRegisterLifecycleManager,
    relationship_tracker::RelationshipTracker,
    ResourceRegister
};
use crate::log::FactLogger;
use crate::effect::{EffectContext, random::{RandomEffectFactory, RandomType}};
use crate::resource::boundary_manager::{BoundaryAwareResourceManager, ResourceCrossingStrategy, ResourceBoundaryCrossing};
use crate::boundary::BoundaryType;
use crate::effect::{
    Effect, EffectId, EffectResult, EffectOutcome, EffectError,
    ExecutionBoundary,
};
use crate::effect::templates::{
    create_resource_effect,
    update_resource_effect,
    lock_resource_effect,
    unlock_resource_effect,
    consume_resource_effect,
    transfer_resource_effect,
    freeze_resource_effect,
    unfreeze_resource_effect,
    archive_resource_effect,
    create_resource_with_boundary_effect,
    cross_domain_resource_effect,
    resource_operation_with_capability_effect,
    resource_operation_with_timemap_effect,
    resource_operation_with_commitment_effect,
};

// Determine where RegisterId is defined and update the import
// If not defined in a submodule, remove the 'register::' part
type RegisterId = String;

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
    /// Boundary manager for boundary-aware operations
    boundary_manager: Arc<BoundaryAwareResourceManager>,
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
    
    /// Check if a resource can cross a boundary
    pub fn can_cross_boundary(
        &self,
        resource_id: &ResourceId,
        source: BoundaryType,
        target: BoundaryType
    ) -> Result<bool> {
        self.boundary_manager.can_cross_boundary(resource_id, source, target)
    }
    
    /// Prepare a resource for crossing a boundary
    pub fn prepare_for_crossing(
        &self,
        resource_id: &ResourceId,
        source: BoundaryType,
        target: BoundaryType
    ) -> Result<ResourceBoundaryCrossing> {
        self.boundary_manager.prepare_for_crossing(resource_id, source, target)
    }
    
    /// Complete a resource crossing
    pub fn complete_crossing(&self, crossing: &ResourceBoundaryCrossing) -> Result<()> {
        self.boundary_manager.complete_crossing(crossing)
    }
    
    /// Set a default strategy for crossing boundaries
    pub fn set_boundary_crossing_strategy(
        &self,
        source: BoundaryType,
        target: BoundaryType,
        strategy: ResourceCrossingStrategy
    ) {
        self.boundary_manager.set_default_strategy(source, target, strategy)
    }
    
    /// Get all boundaries where a resource exists
    pub fn get_resource_boundaries(&self, resource_id: &ResourceId) -> HashSet<BoundaryType> {
        self.boundary_manager.resource_boundaries(resource_id)
    }

    /// Create a new resource with effect generation
    pub fn create_resource_with_effect(
        &mut self,
        resource: ResourceRegister,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<(ResourceId, Arc<dyn Effect>)> {
        // Register the resource first
        let resource_id = self.lifecycle_manager.register_resource(resource.id.clone())?;
        
        // Store in local registry
        self.resources.insert(resource_id.clone(), resource.clone());
        
        // Create the effect for this operation
        let effect = create_resource_effect(&resource, domain_id, invoker)?;
        
        Ok((resource_id, effect))
    }
    
    /// Create a new resource with boundary awareness and effect generation
    pub fn create_resource_with_boundary_effect(
        &mut self,
        resource: ResourceRegister,
        boundary: ExecutionBoundary,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<(ResourceId, Arc<dyn Effect>)> {
        // Register the resource first
        let resource_id = self.lifecycle_manager.register_resource(resource.id.clone())?;
        
        // Store in local registry
        self.resources.insert(resource_id.clone(), resource.clone());
        
        // Create the effect for this operation with boundary awareness
        let effect = create_resource_with_boundary_effect(&resource, boundary, domain_id, invoker)?;
        
        Ok((resource_id, effect))
    }
    
    /// Lock a resource with effect generation
    pub fn lock_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        locker_id: Option<&ResourceId>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Lock the resource in the lifecycle manager
        self.lifecycle_manager.lock(resource_id, locker_id)?;
        
        // Update resource state
        resource.state = RegisterState::Locked;
        
        // Create the effect for this operation
        let effect = lock_resource_effect(resource, domain_id, invoker)?;
        
        Ok(effect)
    }
    
    /// Unlock a resource with effect generation
    pub fn unlock_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        unlocker_id: Option<&ResourceId>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Unlock the resource in the lifecycle manager
        self.lifecycle_manager.unlock(resource_id, unlocker_id)?;
        
        // Update resource state
        resource.state = RegisterState::Active;
        
        // Create the effect for this operation
        let effect = unlock_resource_effect(resource, domain_id, invoker)?;
        
        Ok(effect)
    }
    
    /// Freeze a resource with effect generation
    pub fn freeze_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Freeze the resource in the lifecycle manager
        self.lifecycle_manager.freeze(resource_id)?;
        
        // Update resource state
        resource.state = RegisterState::Frozen;
        
        // Create the effect for this operation
        let effect = freeze_resource_effect(resource, domain_id, invoker)?;
        
        Ok(effect)
    }
    
    /// Unfreeze a resource with effect generation
    pub fn unfreeze_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Unfreeze the resource in the lifecycle manager
        self.lifecycle_manager.unfreeze(resource_id)?;
        
        // Update resource state
        resource.state = RegisterState::Active;
        
        // Create the effect for this operation
        let effect = unfreeze_resource_effect(resource, domain_id, invoker)?;
        
        Ok(effect)
    }
    
    /// Consume a resource with effect generation
    pub fn consume_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        consumer_id: Option<&ResourceId>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Consume the resource in the lifecycle manager
        self.lifecycle_manager.consume(resource_id)?;
        
        // Update resource state
        resource.state = RegisterState::Consumed;
        
        // Get the relationship tracker
        let mut relationship_tracker = self.relationship_tracker.clone();
        
        // If a consumer is provided, create a consumption relationship
        if let Some(consumer) = consumer_id {
            relationship_tracker.add_relationship(
                consumer.clone(),
                resource_id.clone(),
                RelationshipType::Consumption,
                None,
            )?;
        }
        
        // Create the effect for this operation
        let effect = consume_resource_effect(resource, domain_id, invoker, &mut relationship_tracker)?;
        
        // Update the relationship tracker
        self.relationship_tracker = relationship_tracker;
        
        Ok(effect)
    }
    
    /// Archive a resource with effect generation
    pub fn archive_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Archive the resource in the lifecycle manager
        self.lifecycle_manager.archive(resource_id)?;
        
        // Update resource state
        resource.state = RegisterState::Archived;
        
        // Create the effect for this operation
        let effect = archive_resource_effect(resource, domain_id, invoker)?;
        
        Ok(effect)
    }
    
    /// Transfer a resource with effect generation
    pub fn transfer_resource_with_effect(
        &mut self,
        resource_id: &ResourceId,
        from: Address,
        to: Address,
        domain_id: DomainId,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Check if resource is in a state that allows transfer
        if resource.state != RegisterState::Active {
            return Err(Error::InvalidOperation(
                format!("Resource {} cannot be transferred in state {:?}", 
                    resource_id, resource.state)
            ));
        }
        
        // Validate relationship constraints
        self.relationship_tracker.validate_transfer_relationships(resource_id, &from, &to)?;
        
        // Create the effect for this operation
        let effect = transfer_resource_effect(resource, from, to, domain_id)?;
        
        Ok(effect)
    }
    
    /// Cross-domain resource operation with effect generation
    pub fn cross_domain_resource_operation(
        &mut self,
        resource_id: &ResourceId,
        source_domain: DomainId,
        target_domain: DomainId,
        invoker: Address,
        operation_type: RegisterOperationType,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource(resource_id)?;
        
        // Check if the operation is valid for the resource's current state
        if !self.lifecycle_manager.is_operation_valid(resource_id, &operation_type)? {
            return Err(Error::InvalidOperation(
                format!("Operation {:?} not valid for resource {} in state {:?}", 
                    operation_type, resource_id, resource.state)
            ));
        }
        
        // Create the cross-domain effect
        let effect = cross_domain_resource_effect(
            &resource,
            source_domain,
            target_domain,
            invoker,
            operation_type,
        )?;
        
        Ok(effect)
    }
    
    /// Resource operation with capability validation and effect generation
    pub fn resource_operation_with_capability(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
        operation_type: RegisterOperationType,
        capability_ids: Vec<String>,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Check if the operation is valid for the resource's current state
        if !self.lifecycle_manager.is_operation_valid(resource_id, &operation_type)? {
            return Err(Error::InvalidOperation(
                format!("Operation {:?} not valid for resource {} in state {:?}", 
                    operation_type, resource_id, resource.state)
            ));
        }
        
        // Create the capability-validated effect
        let effect = resource_operation_with_capability_effect(
            resource,
            domain_id,
            invoker,
            operation_type,
            capability_ids,
        )?;
        
        Ok(effect)
    }
    
    /// Resource operation with time map validation and effect generation
    pub fn resource_operation_with_timemap(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
        operation_type: RegisterOperationType,
        time_map_snapshot: TimeMapSnapshot,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Check if the operation is valid for the resource's current state
        if !self.lifecycle_manager.is_operation_valid(resource_id, &operation_type)? {
            return Err(Error::InvalidOperation(
                format!("Operation {:?} not valid for resource {} in state {:?}", 
                    operation_type, resource_id, resource.state)
            ));
        }
        
        // Create the time-map validated effect
        let effect = resource_operation_with_timemap_effect(
            resource,
            domain_id,
            invoker,
            operation_type,
            time_map_snapshot,
        )?;
        
        Ok(effect)
    }
    
    /// Resource operation with on-chain commitment and effect generation
    pub fn resource_operation_with_commitment(
        &mut self,
        resource_id: &ResourceId,
        domain_id: DomainId,
        invoker: Address,
        operation_type: RegisterOperationType,
    ) -> Result<Arc<dyn Effect>> {
        // Get the resource
        let resource = self.get_resource_mut(resource_id)?;
        
        // Check if the operation is valid for the resource's current state
        if !self.lifecycle_manager.is_operation_valid(resource_id, &operation_type)? {
            return Err(Error::InvalidOperation(
                format!("Operation {:?} not valid for resource {} in state {:?}", 
                    operation_type, resource_id, resource.state)
            ));
        }
        
        // Create the commitment-backed effect
        let effect = resource_operation_with_commitment_effect(
            resource,
            domain_id,
            invoker,
            operation_type,
        )?;
        
        Ok(effect)
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
            boundary_manager: self.boundary_manager.clone(),
        }
    }
}

/// A shared resource manager
pub type SharedResourceManager = Arc<ResourceManager>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::future;
    use async_trait::async_trait;
    use crate::resource::{
        allocator::ResourceAllocator,
        request::{ResourceRequest, ResourceGrant, GrantId},
        usage::ResourceUsage
    };

    #[derive(Clone)]
    struct MockAllocator {}
    
    #[async_trait]
    impl ResourceAllocator for MockAllocator {
        async fn allocate(&self, request: &ResourceRequest) -> Result<ResourceGrant> {
            // In tests we use the deterministic random number generator
            // In production code, the RandomEffect would be passed in and used
            let context = EffectContext::default();
            let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
            
            // Run the effect to generate the random number
            let random_u64 = random_effect.gen_u64(&context)
                .await
                .map_err(|e| Error::ResourceError(format!("Failed to generate random number: {}", e)))?;
            
            Ok(ResourceGrant {
                grant_id: GrantId::from_string(format!("grant-{}", random_u64)),
                memory_bytes: request.memory_bytes,
                cpu_millis: request.cpu_millis,
                io_operations: request.io_operations,
                effect_count: request.effect_count,
            })
        }
        
        fn release(&self, _grant: &ResourceGrant) -> Result<()> {
            // No resources to release in mock
            Ok(())
        }
        
        fn check_usage(&self, _grant: &ResourceGrant) -> ResourceUsage {
            // Return dummy usage stats
            ResourceUsage {
                memory_bytes: 0,
                cpu_millis: 0,
                io_operations: 0,
                effect_count: 0,
            }
        }
        
        async fn subdivide(
            &self,
            grant: ResourceGrant,
            requests: Vec<ResourceRequest>,
        ) -> Result<Vec<ResourceGrant>> {
            // Subdivide the grant into smaller grants
            let mut remaining = grant.clone();
            let mut grants = Vec::new();
            
            for request in &requests {
                // Check if there are enough resources left
                if request.memory_bytes > remaining.memory_bytes ||
                   request.cpu_millis > remaining.cpu_millis ||
                   request.io_operations > remaining.io_operations ||
                   request.effect_count > remaining.effect_count {
                    return Err(Error::ResourceError("Insufficient resources for subdivision".into()));
                }
                
                // Create a new grant with the requested resources
                let child_grant = ResourceGrant {
                    grant_id: GrantId::from_string(format!("subdiv-{}", grants.len())),
                    memory_bytes: request.memory_bytes,
                    cpu_millis: request.cpu_millis,
                    io_operations: request.io_operations,
                    effect_count: request.effect_count,
                };
                
                // Subtract the resources from the remaining pool
                remaining.memory_bytes -= request.memory_bytes;
                remaining.cpu_millis -= request.cpu_millis;
                remaining.io_operations -= request.io_operations;
                remaining.effect_count -= request.effect_count;
                
                grants.push(child_grant);
            }
            
            Ok(grants)
        }
        
        fn validate_grant(&self, _grant: &ResourceGrant) -> std::result::Result<(), crate::resource::allocator::AllocationError> {
            // Always valid in mock
            Ok(())
        }
        
        fn name(&self) -> &str {
            "MockAllocator"
        }
    }
} 