// Resource Implementation for Effect Handlers
//
// This module implements the resource traits from causality-resource
// for effect handlers, making them compatible with the unified resource
// management system.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;

use causality_common::identity::ContentId;
use causality_resource::interface::{
    ResourceState, ResourceAccessType, LockType, DependencyType, LockStatus,
    ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency,
    ResourceContext, ResourceAccessRecord, ResourceLockInfo, ResourceDependencyInfo,
    BasicResourceContext
};

use crate::effect::{EffectId, EffectHandler, EffectRegistry, EffectContext};
use crate::resource::{
    access::{ResourceAccessManager},
    lifecycle::{EffectResourceLifecycle, ResourceLifecycleEvent, LifecycleEvent},
    locking::{CrossDomainLockManager, CrossDomainLockType},
    dependency::{ResourceDependencyManager, ResourceDependency as ResourceDependencyRecord}
};

/// Implementation of resource traits for effect handlers
pub struct EffectResourceImplementation {
    /// The effect registry
    effect_registry: Arc<EffectRegistry>,
    /// Resource access manager
    access_manager: Arc<ResourceAccessManager>,
    /// Resource lifecycle manager
    lifecycle_manager: Arc<EffectResourceLifecycle>,
    /// Resource lock manager
    lock_manager: Arc<CrossDomainLockManager>,
    /// Resource dependency manager
    dependency_manager: Arc<ResourceDependencyManager>,
}

impl EffectResourceImplementation {
    /// Create a new effect resource implementation
    pub fn new(
        effect_registry: Arc<EffectRegistry>,
        access_manager: Arc<ResourceAccessManager>,
        lifecycle_manager: Arc<EffectResourceLifecycle>,
        lock_manager: Arc<CrossDomainLockManager>,
        dependency_manager: Arc<ResourceDependencyManager>,
    ) -> Self {
        Self {
            effect_registry,
            access_manager,
            lifecycle_manager,
            lock_manager,
            dependency_manager,
        }
    }
    
    /// Create a new effect resource implementation with default components
    pub fn new_default(effect_registry: Arc<EffectRegistry>) -> Self {
        let access_manager = Arc::new(ResourceAccessManager::new());
        let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
        let lock_manager = Arc::new(CrossDomainLockManager::new());
        let dependency_manager = Arc::new(ResourceDependencyManager::new());
        
        Self::new(
            effect_registry,
            access_manager,
            lifecycle_manager,
            lock_manager,
            dependency_manager,
        )
    }
    
    /// Convert resource context to effect context
    fn convert_to_effect_context(&self, context: &dyn ResourceContext) -> Result<EffectContext> {
        let effect_id = match context.effect_id() {
            Some(id) => id.clone(),
            None => return Err(anyhow::anyhow!("Resource context missing effect ID")),
        };
        
        // Get the effect from the registry
        let _effect_handler = self.effect_registry.get_effect_handler(&effect_id)
            .map_err(|e| anyhow::anyhow!("Failed to get effect handler: {}", e))?;
        
        // Create a new effect context
        let mut effect_context = EffectContext::new(effect_id);
        
        // Copy metadata
        for (key, value) in context.metadata() {
            effect_context.add_metadata(key.clone(), value.clone());
        }
        
        // If there's a domain ID, add it to the context
        if let Some(domain_id) = context.domain_id() {
            effect_context.add_metadata("domain_id".to_string(), domain_id.to_string());
        }
        
        Ok(effect_context)
    }
}

#[async_trait]
impl ResourceAccess for EffectResourceImplementation {
    async fn is_access_allowed(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        // Check with the access manager
        match self.access_manager.can_access_resource(
            resource_id.clone(),
            access_type,
            context.effect_id().cloned(),
            context.domain_id().cloned(),
        ) {
            Ok(allowed) => Ok(allowed),
            Err(e) => Err(anyhow::anyhow!("Failed to check access: {}", e)),
        }
    }
    
    async fn record_access(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Create a resource access record
        let access = crate::resource::access::ResourceAccess {
            resource_id: resource_id.clone(),
            access_type,
            domain_id: context.domain_id().cloned(),
            effect_id: context.effect_id().cloned(),
            granted: true,  // Assume granted, the manager will check
            timestamp: SystemTime::now(),
            metadata: context.metadata().clone(),
        };
        
        // Record with the access manager
        match self.access_manager.record_access(access) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Failed to record access: {}", e)),
        }
    }
    
    async fn get_resource_accesses(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceAccessRecord>> {
        // Get access records from access manager
        let accesses = self.access_manager.get_resource_accesses(resource_id)?;
        
        // Convert to ResourceAccessRecord
        let access_records = accesses.into_iter()
            .map(|access| ResourceAccessRecord {
                resource_id: access.resource_id,
                access_type: access.access_type,
                domain_id: access.domain_id,
                effect_id: access.effect_id,
                granted: access.granted,
                timestamp: access.timestamp,
                metadata: access.metadata,
            })
            .collect();
        
        Ok(access_records)
    }
}

#[async_trait]
impl ResourceLifecycle for EffectResourceImplementation {
    async fn register_resource(
        &self, 
        resource_id: ContentId, 
        initial_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Register the resource with the lifecycle manager
        let event = ResourceLifecycleEvent::Created;
        
        let lifecycle_event = LifecycleEvent {
            resource_id: resource_id.clone(),
            event_type: event,
            effect_id: context.effect_id().cloned(),
            domain_id: context.domain_id().cloned(),
            timestamp: SystemTime::now(),
            metadata: context.metadata().clone(),
        };
        
        let initial_event = match initial_state {
            ResourceState::Created => ResourceLifecycleEvent::Created,
            ResourceState::Active => ResourceLifecycleEvent::Activated,
            ResourceState::Locked => ResourceLifecycleEvent::Locked,
            ResourceState::Frozen => ResourceLifecycleEvent::Frozen,
            ResourceState::Consumed => ResourceLifecycleEvent::Consumed,
            ResourceState::Archived => ResourceLifecycleEvent::Archived,
        };
        
        self.lifecycle_manager.register_resource(
            resource_id,
            lifecycle_event,
            Some(initial_event),
        ).map_err(|e| anyhow::anyhow!("Failed to register resource: {}", e))
    }
    
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Determine the event type based on the new state
        let event = match new_state {
            ResourceState::Created => ResourceLifecycleEvent::Created,
            ResourceState::Active => ResourceLifecycleEvent::Activated,
            ResourceState::Locked => ResourceLifecycleEvent::Locked,
            ResourceState::Frozen => ResourceLifecycleEvent::Frozen,
            ResourceState::Consumed => ResourceLifecycleEvent::Consumed,
            ResourceState::Archived => ResourceLifecycleEvent::Archived,
        };
        
        // Create a lifecycle event
        let lifecycle_event = LifecycleEvent {
            resource_id: resource_id.clone(),
            event_type: event,
            effect_id: context.effect_id().cloned(),
            domain_id: context.domain_id().cloned(),
            timestamp: SystemTime::now(),
            metadata: context.metadata().clone(),
        };
        
        // Update the resource state
        self.lifecycle_manager.update_state(resource_id.clone(), lifecycle_event)
            .map_err(|e| anyhow::anyhow!("Failed to update resource state: {}", e))
    }
    
    async fn get_resource_state(
        &self, 
        resource_id: &ContentId
    ) -> Result<ResourceState> {
        // Get the state from the lifecycle manager
        let state = self.lifecycle_manager.get_resource_state(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to get resource state: {}", e))?;
        
        // Convert to ResourceState
        let resource_state = match state {
            ResourceLifecycleEvent::Created => ResourceState::Created,
            ResourceLifecycleEvent::Activated => ResourceState::Active,
            ResourceLifecycleEvent::Locked => ResourceState::Locked,
            ResourceLifecycleEvent::Unlocked => ResourceState::Active,  // Unlocked means active
            ResourceLifecycleEvent::Frozen => ResourceState::Frozen,
            ResourceLifecycleEvent::Unfrozen => ResourceState::Active,  // Unfrozen means active
            ResourceLifecycleEvent::Consumed => ResourceState::Consumed,
            ResourceLifecycleEvent::Archived => ResourceState::Archived,
            ResourceLifecycleEvent::Updated => ResourceState::Active,   // Updated means active
        };
        
        Ok(resource_state)
    }
    
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.lifecycle_manager.resource_exists(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to check if resource exists: {}", e))
    }
}

#[async_trait]
impl ResourceLocking for EffectResourceImplementation {
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> Result<LockStatus> {
        // Convert lock type
        let cd_lock_type = match lock_type {
            LockType::Exclusive => CrossDomainLockType::Exclusive,
            LockType::Shared => CrossDomainLockType::Shared,
            LockType::Intent => CrossDomainLockType::Intent,
        };
        
        // Acquire the lock
        let result = self.lock_manager.acquire_lock(
            resource_id.clone(),
            cd_lock_type,
            holder_id.clone(),
            context.domain_id().cloned(),
            timeout,
            None,  // No specific transaction ID
        ).map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
        
        // Convert to LockStatus
        let lock_status = match result {
            crate::resource::locking::LockStatus::Acquired => LockStatus::Acquired,
            crate::resource::locking::LockStatus::AlreadyHeld => LockStatus::AlreadyHeld,
            crate::resource::locking::LockStatus::Unavailable => LockStatus::Unavailable,
            crate::resource::locking::LockStatus::TimedOut => LockStatus::TimedOut,
        };
        
        // If successfully acquired, update resource state
        if lock_status == LockStatus::Acquired {
            // Create a lifecycle event
            let lifecycle_event = LifecycleEvent {
                resource_id: resource_id.clone(),
                event_type: ResourceLifecycleEvent::Locked,
                effect_id: context.effect_id().cloned(),
                domain_id: context.domain_id().cloned(),
                timestamp: SystemTime::now(),
                metadata: context.metadata().clone(),
            };
            
            // Update the resource state
            self.lifecycle_manager.update_state(resource_id.clone(), lifecycle_event)
                .map_err(|e| anyhow::anyhow!("Failed to update resource state after locking: {}", e))?;
        }
        
        Ok(lock_status)
    }
    
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        // Release the lock
        let result = self.lock_manager.release_lock(
            resource_id.clone(),
            holder_id.clone(),
        ).map_err(|e| anyhow::anyhow!("Failed to release lock: {}", e))?;
        
        // If successfully released, update resource state
        if result {
            // Create a lifecycle event
            let lifecycle_event = LifecycleEvent {
                resource_id: resource_id.clone(),
                event_type: ResourceLifecycleEvent::Unlocked,
                effect_id: context.effect_id().cloned(),
                domain_id: context.domain_id().cloned(),
                timestamp: SystemTime::now(),
                metadata: context.metadata().clone(),
            };
            
            // Update the resource state
            self.lifecycle_manager.update_state(resource_id.clone(), lifecycle_event)
                .map_err(|e| anyhow::anyhow!("Failed to update resource state after unlocking: {}", e))?;
        }
        
        Ok(result)
    }
    
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.lock_manager.is_resource_locked(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to check if resource is locked: {}", e))
    }
    
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> Result<Option<ResourceLockInfo>> {
        // Get the lock from the lock manager
        let lock_opt = self.lock_manager.get_lock(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to get lock info: {}", e))?;
        
        // Convert to ResourceLockInfo
        if let Some(lock) = lock_opt {
            let lock_type = match lock.lock_type {
                CrossDomainLockType::Exclusive => LockType::Exclusive,
                CrossDomainLockType::Shared => LockType::Shared,
                CrossDomainLockType::Intent => LockType::Intent,
            };
            
            Ok(Some(ResourceLockInfo {
                resource_id: lock.resource_id,
                lock_type,
                holder_id: lock.holder_id,
                acquired_at: lock.acquired_at,
                expires_at: lock.timeout.map(|t| lock.acquired_at + t),
                transaction_id: lock.transaction_id,
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl ResourceDependency for EffectResourceImplementation {
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Create a dependency record
        let dependency = ResourceDependencyRecord {
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            dependency_type: match dependency_type {
                DependencyType::Strong => crate::resource::dependency::DependencyType::Strong,
                DependencyType::Weak => crate::resource::dependency::DependencyType::Weak,
                DependencyType::Temporal => crate::resource::dependency::DependencyType::Temporal,
                DependencyType::Data => crate::resource::dependency::DependencyType::Data,
                DependencyType::Identity => crate::resource::dependency::DependencyType::Identity,
            },
            source_domain_id: context.domain_id().cloned(),
            target_domain_id: None,  // We don't know the target domain ID here
            creator_effect_id: context.effect_id().cloned(),
            created_at: SystemTime::now(),
            metadata: context.metadata().clone(),
        };
        
        // Add the dependency
        self.dependency_manager.add_dependency(dependency)
            .map_err(|e| anyhow::anyhow!("Failed to add dependency: {}", e))
    }
    
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        // Convert the dependency type
        let dep_type = match dependency_type {
            DependencyType::Strong => crate::resource::dependency::DependencyType::Strong,
            DependencyType::Weak => crate::resource::dependency::DependencyType::Weak,
            DependencyType::Temporal => crate::resource::dependency::DependencyType::Temporal,
            DependencyType::Data => crate::resource::dependency::DependencyType::Data,
            DependencyType::Identity => crate::resource::dependency::DependencyType::Identity,
        };
        
        // Remove the dependency
        self.dependency_manager.remove_dependency(source_id, target_id, dep_type)
            .map_err(|e| anyhow::anyhow!("Failed to remove dependency: {}", e))
    }
    
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        // Get dependencies from the dependency manager
        let deps = self.dependency_manager.get_dependencies_for(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to get dependencies: {}", e))?;
        
        // Convert to ResourceDependencyInfo
        let dependency_infos = deps.into_iter()
            .map(|dep| ResourceDependencyInfo {
                source_id: dep.source_id,
                target_id: dep.target_id,
                dependency_type: match dep.dependency_type {
                    crate::resource::dependency::DependencyType::Strong => DependencyType::Strong,
                    crate::resource::dependency::DependencyType::Weak => DependencyType::Weak,
                    crate::resource::dependency::DependencyType::Temporal => DependencyType::Temporal,
                    crate::resource::dependency::DependencyType::Data => DependencyType::Data,
                    crate::resource::dependency::DependencyType::Identity => DependencyType::Identity,
                },
                source_domain_id: dep.source_domain_id,
                target_domain_id: dep.target_domain_id,
                creator_effect_id: dep.creator_effect_id,
                created_at: dep.created_at,
                metadata: dep.metadata,
            })
            .collect();
        
        Ok(dependency_infos)
    }
    
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        // Get dependents from the dependency manager
        let deps = self.dependency_manager.get_dependents_for(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to get dependents: {}", e))?;
        
        // Convert to ResourceDependencyInfo
        let dependency_infos = deps.into_iter()
            .map(|dep| ResourceDependencyInfo {
                source_id: dep.source_id,
                target_id: dep.target_id,
                dependency_type: match dep.dependency_type {
                    crate::resource::dependency::DependencyType::Strong => DependencyType::Strong,
                    crate::resource::dependency::DependencyType::Weak => DependencyType::Weak,
                    crate::resource::dependency::DependencyType::Temporal => DependencyType::Temporal,
                    crate::resource::dependency::DependencyType::Data => DependencyType::Data,
                    crate::resource::dependency::DependencyType::Identity => DependencyType::Identity,
                },
                source_domain_id: dep.source_domain_id,
                target_domain_id: dep.target_domain_id,
                creator_effect_id: dep.creator_effect_id,
                created_at: dep.created_at,
                metadata: dep.metadata,
            })
            .collect();
        
        Ok(dependency_infos)
    }
    
    async fn has_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.dependency_manager.has_dependencies(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to check if resource has dependencies: {}", e))
    }
    
    async fn has_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.dependency_manager.has_dependents(resource_id)
            .map_err(|e| anyhow::anyhow!("Failed to check if resource has dependents: {}", e))
    }
}

/// Helper function to create effect resource context from basic context
pub fn create_effect_context(
    effect_id: EffectId,
    context_id: Option<ContentId>,
) -> BasicResourceContext {
    let context_id = context_id.unwrap_or_else(|| ContentId::from_string("effect-context").unwrap_or_default());
    
    BasicResourceContext::new(context_id)
        .with_effect(effect_id.clone())
} 