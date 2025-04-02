// Resource Implementation for Domain Adapters
//
// This module implements the resource traits from causality-resource
// for domain adapters, making them compatible with the unified resource
// management system.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::any::Any;
use std::fmt::Debug;

use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use causality_types::ContentId;
use causality_core::resource::types::{Resource, ResourceType};
use causality_core::resource::interface::{
    ResourceState, ResourceAccessType, LockType, DependencyType, 
    ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency,
    ResourceResult, ResourceError,
    ResourceContext, ResourceAccessRecord, ResourceLockInfo, ResourceDependencyInfo,
    BasicResourceContext
};

use crate::resource::{
    CrossDomainResourceManager, CrossDomainResourceOperation, CrossDomainResourceResult, DomainResourceAdapter, ResourceRegister
};
use crate::domain::{DomainId, Transaction, DomainAdapter};

/// Implementation of resource traits for domain adapters
#[derive(Clone, Debug)]
pub struct DomainResourceImplementation {
    /// The domain adapter
    domain_adapter: Arc<dyn DomainAdapter>,
    /// The cross-domain resource manager
    resource_manager: Arc<CrossDomainResourceManager>,
    /// Access records storage
    access_records: Arc<RwLock<HashMap<ContentId, Vec<ResourceAccessRecord>>>>,
    /// Current resource states
    resource_states: Arc<RwLock<HashMap<ContentId, ResourceState>>>,
    /// Resource locks
    resource_locks: Arc<RwLock<HashMap<ContentId, ResourceLockInfo>>>,
    /// Resource dependencies (source -> targets)
    dependencies: Arc<RwLock<HashMap<ContentId, Vec<ResourceDependencyInfo>>>>,
    /// Resource dependents (target -> sources)
    dependents: Arc<RwLock<HashMap<ContentId, Vec<ResourceDependencyInfo>>>>,
}

impl DomainResourceImplementation {
    /// Create a new domain resource implementation
    pub fn new(
        domain_adapter: Arc<dyn DomainAdapter>,
        resource_manager: Arc<CrossDomainResourceManager>,
    ) -> Self {
        Self {
            domain_adapter,
            resource_manager,
            access_records: Arc::new(RwLock::new(HashMap::new())),
            resource_states: Arc::new(RwLock::new(HashMap::new())),
            resource_locks: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            dependents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        self.domain_adapter.domain_id()
    }
    
    /// Record a transaction for a resource operation
    async fn record_transaction(
        &self,
        resource_id: &ContentId,
        operation_type: &str,
        metadata: HashMap<String, String>,
    ) -> AnyhowResult<String> {
        let tx = Transaction {
            data: resource_id.to_string().as_bytes().to_vec(),
            transaction_type: operation_type.to_string(),
            metadata,
        };
        
        let tx_id = self.domain_adapter.submit_transaction(tx).await
            .map_err(|e| anyhow::anyhow!("Failed to submit transaction: {}", e))?;
        
        Ok(tx_id.to_string())
    }
    
    /// Check if resource exists
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> AnyhowResult<bool> {
        self.resource_manager.verify_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to verify resource: {}", e))
    }
}

#[async_trait]
impl ResourceAccess for DomainResourceImplementation {
    async fn check_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<bool> {
        // Implementation based on is_access_allowed
        self.resource_exists(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        
        // Add implementation here
        Ok(true) // Simplified for now
    }
    
    async fn grant_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<()> {
        // Implementation
        Ok(())
    }
    
    async fn revoke_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<()> {
        // Implementation
        Ok(())
    }
    
    async fn get_access_types(&self, resource_id: &ContentId) -> ResourceResult<Vec<ResourceAccessType>> {
        // Implementation
        Ok(vec![ResourceAccessType::Read])
    }
}

#[async_trait]
impl ResourceLifecycle for DomainResourceImplementation {
    async fn get_state(&self, resource_id: &ContentId) -> ResourceResult<ResourceState> {
        let state = self.get_resource_state(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        Ok(state)
    }
    
    async fn set_state(
        &self,
        resource_id: &ContentId,
        state: ResourceState,
    ) -> ResourceResult<()> {
        // No context available, use a basic one
        let context = BasicResourceContext::new(ContentId::from_string("system").unwrap_or_default());
        
        self.update_resource_state(resource_id, state, &context).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        Ok(())
    }
    
    async fn get_state_history(
        &self,
        resource_id: &ContentId,
        limit: Option<usize>,
    ) -> ResourceResult<Vec<(ResourceState, chrono::DateTime<chrono::Utc>)>> {
        // Implementation - we don't track history yet
        let state = self.get_resource_state(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        Ok(vec![(state, chrono::Utc::now())])
    }
}

#[async_trait]
impl ResourceLocking for DomainResourceImplementation {
    async fn acquire_lock(
        &self,
        resource_id: &ContentId,
        lock_type: LockType,
    ) -> ResourceResult<()> {
        // No context available, use a basic one
        let context = BasicResourceContext::new(ContentId::from_string("system").unwrap_or_default());
        let holder_id = ContentId::from_string("system-lock").unwrap_or_default();
        
        self.acquire_lock(resource_id, lock_type, &holder_id, None, &context).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        Ok(())
    }
    
    async fn release_lock(
        &self,
        resource_id: &ContentId,
        lock_type: LockType,
    ) -> ResourceResult<()> {
        // No context available, use a basic one
        let context = BasicResourceContext::new(ContentId::from_string("system").unwrap_or_default());
        let holder_id = ContentId::from_string("system-lock").unwrap_or_default();
        
        self.release_lock(resource_id, &holder_id, &context).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        Ok(())
    }
    
    async fn get_lock_type(&self, resource_id: &ContentId) -> ResourceResult<Option<LockType>> {
        let lock_info = self.get_lock_info(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        
        Ok(lock_info.map(|info| info.lock_type))
    }
    
    async fn is_locked(&self, resource_id: &ContentId) -> ResourceResult<bool> {
        self.is_locked(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))
    }
}

#[async_trait]
impl ResourceDependency for DomainResourceImplementation {
    async fn add_dependency(
        &self,
        resource_id: &ContentId,
        dependency_id: &ContentId,
        dependency_type: DependencyType,
    ) -> ResourceResult<()> {
        // No context available, use a basic one
        let context = BasicResourceContext::new(ContentId::from_string("system").unwrap_or_default());
        
        self.add_dependency(resource_id, dependency_id, dependency_type, &context).await
            .map_err(|e| ResourceError::Internal(e.to_string()))
    }
    
    async fn remove_dependency(
        &self,
        resource_id: &ContentId,
        dependency_id: &ContentId,
    ) -> ResourceResult<()> {
        // No context available, use a basic one
        let context = BasicResourceContext::new(ContentId::from_string("system").unwrap_or_default());
        
        // Assuming any dependency type for removal
        let result = self.remove_dependency(resource_id, dependency_id, DependencyType::Strong, &context).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        
        if result {
            Ok(())
        } else {
            Err(ResourceError::DependencyError("Dependency not found".to_string()))
        }
    }
    
    async fn get_dependencies(
        &self,
        resource_id: &ContentId,
    ) -> ResourceResult<Vec<(ContentId, DependencyType)>> {
        let deps = self.get_dependencies(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        
        Ok(deps.into_iter()
            .map(|info| (info.target_id, info.dependency_type))
            .collect())
    }
    
    async fn get_dependents(
        &self,
        resource_id: &ContentId,
    ) -> ResourceResult<Vec<(ContentId, DependencyType)>> {
        let deps = self.get_dependents(resource_id).await
            .map_err(|e| ResourceError::Internal(e.to_string()))?;
        
        Ok(deps.into_iter()
            .map(|info| (info.source_id, info.dependency_type))
            .collect())
    }
}

// Additional implementation for internal methods that aren't part of the standard interfaces

impl DomainResourceImplementation {
    // Method used by ResourceLifecycle implementation
    async fn get_resource_state(&self, resource_id: &ContentId) -> AnyhowResult<ResourceState> {
        // First check our local cache
        {
            let states = self.resource_states.read().unwrap();
            if let Some(state) = states.get(resource_id) {
                return Ok(state.clone());
            }
        }
        
        // If not in cache, query the domain adapter
        let adapter_result = self.resource_manager.get_register_state(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await;
        
        match adapter_result {
            Ok(register_state) => {
                // Convert from domain register state to resource state
                let resource_state = match register_state {
                    causality_core::resource::types::RegisterState::Created => ResourceState::Created,
                    causality_core::resource::types::RegisterState::Active => ResourceState::Active,
                    causality_core::resource::types::RegisterState::Locked => ResourceState::Locked,
                    causality_core::resource::types::RegisterState::Frozen => ResourceState::Frozen,
                    causality_core::resource::types::RegisterState::Consumed => ResourceState::Consumed,
                    causality_core::resource::types::RegisterState::Archived => ResourceState::Archived,
                };
                
                // Update cache
                {
                    let mut states = self.resource_states.write().unwrap();
                    states.insert(resource_id.clone(), resource_state.clone());
                }
                
                Ok(resource_state)
            },
            Err(e) => Err(anyhow::anyhow!("Failed to get register state: {}", e)),
        }
    }
    
    // Method used by ResourceLifecycle implementation
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> AnyhowResult<()> {
        // Convert resource state to register state
        let register_state = match new_state {
            ResourceState::Created => causality_core::resource::types::RegisterState::Created,
            ResourceState::Active => causality_core::resource::types::RegisterState::Active,
            ResourceState::Locked => causality_core::resource::types::RegisterState::Locked,
            ResourceState::Frozen => causality_core::resource::types::RegisterState::Frozen,
            ResourceState::Consumed => causality_core::resource::types::RegisterState::Consumed,
            ResourceState::Archived => causality_core::resource::types::RegisterState::Archived,
        };
        
        // Retrieve the current register
        let register = self.resource_manager.retrieve_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to retrieve resource: {}", e))?;
        
        // Update the state
        let mut updated_register = register.clone();
        updated_register.state = register_state;
        
        // Update metadata
        for (key, value) in context.metadata() {
            updated_register.metadata.insert(key.clone(), value.clone());
        }
        
        // Store back in the domain
        let required_capabilities = std::collections::HashSet::new();
        let preferences = HashMap::new();
        
        self.resource_manager.store_resource_register_by_strategy(
            updated_register,
            required_capabilities,
            preferences,
        ).await.map_err(|e| anyhow::anyhow!("Failed to update resource state: {}", e))?;
        
        // Update local state
        let mut states = self.resource_states.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on resource states"))?;
        states.insert(resource_id.clone(), new_state);
        
        // Record transaction for state change
        let mut metadata = context.metadata().clone();
        metadata.insert("new_state".to_string(), format!("{:?}", new_state));
        
        let _ = self.record_transaction(
            resource_id,
            "resource_state_change",
            metadata,
        ).await?;
        
        Ok(())
    }
    
    // Method used by ResourceLocking implementation
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> AnyhowResult<LockStatus> {
        // First check if the resource exists
        let exists = self.resource_exists(resource_id).await?;
        if !exists {
            return Err(anyhow::anyhow!("Resource does not exist"));
        }
        
        // Check if the resource is already locked
        let locks = self.resource_locks.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource locks"))?;
        
        if let Some(lock_info) = locks.get(resource_id) {
            // Check if the lock is held by the same entity
            if lock_info.holder_id == *holder_id {
                return Ok(LockStatus::AlreadyHeld);
            }
            
            // Check if the lock has expired
            if let Some(expires_at) = lock_info.expires_at {
                if expires_at <= SystemTime::now() {
                    // Lock has expired, we can acquire it
                    drop(locks);
                } else {
                    // Lock is still valid and held by someone else
                    return Ok(LockStatus::Unavailable);
                }
            } else {
                // No expiration, lock is held indefinitely
                return Ok(LockStatus::Unavailable);
            }
        }
        
        // Calculate expiration time
        let expires_at = timeout.map(|t| SystemTime::now() + t);
        
        // Create lock info
        let lock_info = ResourceLockInfo {
            resource_id: resource_id.clone(),
            lock_type,
            holder_id: holder_id.clone(),
            acquired_at: SystemTime::now(),
            expires_at,
            transaction_id: None,
        };
        
        // Update the resource state to Locked
        self.update_resource_state(
            resource_id,
            ResourceState::Locked,
            context,
        ).await?;
        
        // Store the lock
        let mut locks = self.resource_locks.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on resource locks"))?;
        locks.insert(resource_id.clone(), lock_info);
        
        // Record lock transaction
        let mut metadata = context.metadata().clone();
        metadata.insert("lock_type".to_string(), format!("{:?}", lock_type));
        metadata.insert("holder_id".to_string(), holder_id.to_string());
        
        let tx_id = self.record_transaction(
            resource_id,
            "resource_lock",
            metadata,
        ).await?;
        
        // Update transaction ID in lock info
        let mut locks = self.resource_locks.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on resource locks"))?;
        if let Some(lock_info) = locks.get_mut(resource_id) {
            lock_info.transaction_id = Some(ContentId::from_string(&tx_id).unwrap_or_default());
        }
        
        Ok(LockStatus::Acquired)
    }
    
    // Method used by ResourceLocking implementation
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> AnyhowResult<bool> {
        // Check if the resource is locked
        let locks = self.resource_locks.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource locks"))?;
        
        let lock_held = if let Some(lock_info) = locks.get(resource_id) {
            // Check if the lock is held by the specified entity
            if lock_info.holder_id != *holder_id {
                return Ok(false);
            }
            true
        } else {
            // No lock found
            return Ok(false);
        };
        
        drop(locks);
        
        if lock_held {
            // Remove the lock
            let mut locks = self.resource_locks.write().map_err(|_| 
                anyhow::anyhow!("Failed to acquire write lock on resource locks"))?;
            locks.remove(resource_id);
            
            // Update the resource state to Active
            self.update_resource_state(
                resource_id,
                ResourceState::Active,
                context,
            ).await?;
            
            // Record unlock transaction
            let mut metadata = context.metadata().clone();
            metadata.insert("holder_id".to_string(), holder_id.to_string());
            
            let _ = self.record_transaction(
                resource_id,
                "resource_unlock",
                metadata,
            ).await?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    // Method used by ResourceLocking implementation
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> AnyhowResult<bool> {
        let locks = self.resource_locks.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource locks"))?;
        
        if let Some(lock_info) = locks.get(resource_id) {
            // Check if the lock has expired
            if let Some(expires_at) = lock_info.expires_at {
                if expires_at <= SystemTime::now() {
                    // Lock has expired
                    Ok(false)
                } else {
                    // Lock is still valid
                    Ok(true)
                }
            } else {
                // No expiration, lock is held indefinitely
                Ok(true)
            }
        } else {
            // No lock found
            Ok(false)
        }
    }
    
    // Method used by ResourceLocking implementation
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> AnyhowResult<Option<ResourceLockInfo>> {
        let locks = self.resource_locks.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource locks"))?;
        
        if let Some(lock_info) = locks.get(resource_id) {
            // Check if the lock has expired
            if let Some(expires_at) = lock_info.expires_at {
                if expires_at <= SystemTime::now() {
                    // Lock has expired
                    Ok(None)
                } else {
                    // Lock is still valid
                    Ok(Some(lock_info.clone()))
                }
            } else {
                // No expiration, lock is held indefinitely
                Ok(Some(lock_info.clone()))
            }
        } else {
            // No lock found
            Ok(None)
        }
    }
    
    // Method used by ResourceDependency implementation
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> AnyhowResult<()> {
        // Check if both resources exist
        let source_exists = self.resource_exists(source_id).await?;
        let target_exists = self.resource_exists(target_id).await?;
        
        if !source_exists || !target_exists {
            return Err(anyhow::anyhow!("One or both resources do not exist"));
        }
        
        // Create dependency info
        let dependency_info = ResourceDependencyInfo {
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            dependency_type,
            source_domain_id: context.domain_id().cloned(),
            target_domain_id: Some(self.domain_adapter.domain_id().clone()),
            creator_effect_id: context.effect_id().cloned(),
            created_at: context.timestamp(),
            metadata: context.metadata().clone(),
        };
        
        // Add to dependencies
        let mut dependencies = self.dependencies.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on dependencies"))?;
        
        dependencies
            .entry(source_id.clone())
            .or_insert_with(Vec::new)
            .push(dependency_info.clone());
        
        // Add to dependents
        let mut dependents = self.dependents.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on dependents"))?;
        
        dependents
            .entry(target_id.clone())
            .or_insert_with(Vec::new)
            .push(dependency_info);
        
        // Record dependency transaction
        let mut metadata = context.metadata().clone();
        metadata.insert("dependency_type".to_string(), format!("{:?}", dependency_type));
        metadata.insert("source_id".to_string(), source_id.to_string());
        metadata.insert("target_id".to_string(), target_id.to_string());
        
        let _ = self.record_transaction(
            source_id,
            "resource_dependency_created",
            metadata,
        ).await?;
        
        Ok(())
    }
    
    // Method used by ResourceDependency implementation
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> AnyhowResult<bool> {
        // Find dependency in dependencies
        let mut dependencies = self.dependencies.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on dependencies"))?;
        
        let mut found = false;
        if let Some(deps) = dependencies.get_mut(source_id) {
            let len_before = deps.len();
            deps.retain(|d| !(d.target_id == *target_id && d.dependency_type == dependency_type));
            found = len_before > deps.len();
            
            // Remove the entry if empty
            if deps.is_empty() {
                dependencies.remove(source_id);
            }
        }
        
        // Early return if not found
        if !found {
            return Ok(false);
        }
        
        // Remove from dependents
        let mut dependents = self.dependents.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on dependents"))?;
        
        if let Some(deps) = dependents.get_mut(target_id) {
            deps.retain(|d| !(d.source_id == *source_id && d.dependency_type == dependency_type));
            
            // Remove the entry if empty
            if deps.is_empty() {
                dependents.remove(target_id);
            }
        }
        
        // Record dependency removal transaction
        let mut metadata = context.metadata().clone();
        metadata.insert("dependency_type".to_string(), format!("{:?}", dependency_type));
        metadata.insert("source_id".to_string(), source_id.to_string());
        metadata.insert("target_id".to_string(), target_id.to_string());
        
        let _ = self.record_transaction(
            source_id,
            "resource_dependency_removed",
            metadata,
        ).await?;
        
        Ok(true)
    }
    
    // Method used by ResourceDependency implementation
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> AnyhowResult<Vec<ResourceDependencyInfo>> {
        let dependencies = self.dependencies.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependencies"))?;
        
        Ok(dependencies
            .get(resource_id)
            .cloned()
            .unwrap_or_default())
    }
    
    // Method used by ResourceDependency implementation
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> AnyhowResult<Vec<ResourceDependencyInfo>> {
        let dependents = self.dependents.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependents"))?;
        
        Ok(dependents
            .get(resource_id)
            .cloned()
            .unwrap_or_default())
    }
}

/// Helper function to create domain resource context from basic context
pub fn create_domain_context(
    domain_id: DomainId,
    context_id: Option<ContentId>,
) -> BasicResourceContext {
    let context_id = context_id.unwrap_or_else(|| ContentId::from_string("domain-context").unwrap_or_default());
    
    BasicResourceContext::new(context_id)
        .with_domain(domain_id.clone())
} 