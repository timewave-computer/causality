// Resource Implementation for Domain Adapters
//
// This module implements the resource traits from causality-resource
// for domain adapters, making them compatible with the unified resource
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

use crate::resource::{
    CrossDomainResourceOperation, CrossDomainResourceResult,
    CrossDomainResourceManager, DomainResourceAdapter
};
use crate::domain::{DomainId, Transaction, DomainAdapter};

/// Implementation of resource traits for domain adapters
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
    ) -> Result<String> {
        let tx = Transaction {
            data: resource_id.to_string().as_bytes().to_vec(),
            transaction_type: operation_type.to_string(),
            metadata,
        };
        
        let tx_id = self.domain_adapter.submit_transaction(tx).await
            .map_err(|e| anyhow::anyhow!("Failed to submit transaction: {}", e))?;
        
        Ok(tx_id.to_string())
    }
}

#[async_trait]
impl ResourceAccess for DomainResourceImplementation {
    async fn is_access_allowed(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        // Check if the resource exists in this domain
        let result = self.resource_manager.verify_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to verify resource: {}", e))?;
        
        // If the resource doesn't exist, access is not allowed
        if !result {
            return Ok(false);
        }
        
        // Check if the resource is locked
        let locks = self.resource_locks.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource locks"))?;
        
        if let Some(lock_info) = locks.get(resource_id) {
            // For write/transfer/execute access, the resource must not be locked by another entity
            match access_type {
                ResourceAccessType::Write | ResourceAccessType::Transfer | ResourceAccessType::Execute => {
                    if let Some(holder_id) = context.effect_id() {
                        if &lock_info.holder_id != holder_id {
                            return Ok(false);
                        }
                    } else if let Some(holder_id) = context.domain_id() {
                        if &lock_info.holder_id != holder_id {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                },
                // Read access is allowed even if the resource is locked
                ResourceAccessType::Read => {},
                // Lock access requires the resource to not be locked at all
                ResourceAccessType::Lock => {
                    if lock_info.lock_type == LockType::Exclusive {
                        return Ok(false);
                    }
                }
            }
        }
        
        // Check resource state for certain access types
        let states = self.resource_states.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource states"))?;
        
        if let Some(state) = states.get(resource_id) {
            match state {
                ResourceState::Consumed | ResourceState::Archived => {
                    // Consumed or archived resources can only be read
                    match access_type {
                        ResourceAccessType::Read => Ok(true),
                        _ => Ok(false),
                    }
                },
                ResourceState::Frozen => {
                    // Frozen resources can be read but not modified
                    match access_type {
                        ResourceAccessType::Read => Ok(true),
                        _ => Ok(false),
                    }
                },
                ResourceState::Locked => {
                    // Locked resources require the holder to be the accessor
                    if let Some(lock_info) = locks.get(resource_id) {
                        if let Some(effect_id) = context.effect_id() {
                            if &lock_info.holder_id == effect_id {
                                Ok(true)
                            } else {
                                match access_type {
                                    ResourceAccessType::Read => Ok(true),
                                    _ => Ok(false),
                                }
                            }
                        } else if let Some(domain_id) = context.domain_id() {
                            if &lock_info.holder_id == domain_id {
                                Ok(true)
                            } else {
                                match access_type {
                                    ResourceAccessType::Read => Ok(true),
                                    _ => Ok(false),
                                }
                            }
                        } else {
                            match access_type {
                                ResourceAccessType::Read => Ok(true),
                                _ => Ok(false),
                            }
                        }
                    } else {
                        // No lock info found, but resource is in locked state
                        match access_type {
                            ResourceAccessType::Read => Ok(true),
                            _ => Ok(false),
                        }
                    }
                },
                // Created or Active resources allow all access types
                _ => Ok(true),
            }
        } else {
            // No state information found, default to allowing
            Ok(true)
        }
    }
    
    async fn record_access(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Check if access is allowed
        let allowed = self.is_access_allowed(resource_id, access_type, context).await?;
        
        // Create access record
        let record = ResourceAccessRecord {
            resource_id: resource_id.clone(),
            access_type,
            domain_id: context.domain_id().cloned(),
            effect_id: context.effect_id().cloned(),
            granted: allowed,
            timestamp: context.timestamp(),
            metadata: context.metadata().clone(),
        };
        
        // Record the access
        let mut access_records = self.access_records.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on access records"))?;
        
        access_records
            .entry(resource_id.clone())
            .or_insert_with(Vec::new)
            .push(record);
        
        // If access is allowed, record a transaction
        if allowed {
            let mut metadata = context.metadata().clone();
            metadata.insert("access_type".to_string(), format!("{:?}", access_type));
            
            let _ = self.record_transaction(
                resource_id,
                "resource_access",
                metadata,
            ).await?;
        }
        
        Ok(())
    }
    
    async fn get_resource_accesses(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceAccessRecord>> {
        let access_records = self.access_records.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on access records"))?;
        
        Ok(access_records
            .get(resource_id)
            .cloned()
            .unwrap_or_default())
    }
}

#[async_trait]
impl ResourceLifecycle for DomainResourceImplementation {
    async fn register_resource(
        &self, 
        resource_id: ContentId, 
        initial_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Create a register for the resource
        let mut register = causality_resource::ResourceRegister {
            id: resource_id.clone(),
            state: match initial_state {
                ResourceState::Created => causality_resource::RegisterState::Created,
                ResourceState::Active => causality_resource::RegisterState::Active,
                ResourceState::Locked => causality_resource::RegisterState::Locked,
                ResourceState::Frozen => causality_resource::RegisterState::Frozen,
                ResourceState::Consumed => causality_resource::RegisterState::Consumed,
                ResourceState::Archived => causality_resource::RegisterState::Archived,
            },
            metadata: context.metadata().clone(),
            ..Default::default()
        };
        
        // Store domain and effect information if available
        if let Some(domain_id) = context.domain_id() {
            register.metadata.insert("domain_id".to_string(), domain_id.to_string());
        }
        
        if let Some(effect_id) = context.effect_id() {
            register.metadata.insert("effect_id".to_string(), effect_id.to_string());
        }
        
        // Store in the domain
        let required_capabilities = std::collections::HashSet::new();
        let preferences = HashMap::new();
        
        self.resource_manager.store_resource_register_by_strategy(
            register,
            required_capabilities,
            preferences,
        ).await.map_err(|e| anyhow::anyhow!("Failed to register resource: {}", e))?;
        
        // Update local state
        let mut states = self.resource_states.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on resource states"))?;
        states.insert(resource_id, initial_state);
        
        Ok(())
    }
    
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        // Retrieve the current register
        let register = self.resource_manager.retrieve_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to retrieve resource: {}", e))?;
        
        // Update the state
        let mut updated_register = register.clone();
        updated_register.state = match new_state {
            ResourceState::Created => causality_resource::RegisterState::Created,
            ResourceState::Active => causality_resource::RegisterState::Active,
            ResourceState::Locked => causality_resource::RegisterState::Locked,
            ResourceState::Frozen => causality_resource::RegisterState::Frozen,
            ResourceState::Consumed => causality_resource::RegisterState::Consumed,
            ResourceState::Archived => causality_resource::RegisterState::Archived,
        };
        
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
    
    async fn get_resource_state(
        &self, 
        resource_id: &ContentId
    ) -> Result<ResourceState> {
        // First check our local cache
        let states = self.resource_states.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on resource states"))?;
        
        if let Some(state) = states.get(resource_id) {
            return Ok(*state);
        }
        
        // If not in cache, retrieve from domain
        let register = self.resource_manager.retrieve_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to retrieve resource: {}", e))?;
        
        // Convert the state
        let state = match register.state {
            causality_resource::RegisterState::Created => ResourceState::Created,
            causality_resource::RegisterState::Active => ResourceState::Active,
            causality_resource::RegisterState::Locked => ResourceState::Locked,
            causality_resource::RegisterState::Frozen => ResourceState::Frozen,
            causality_resource::RegisterState::Consumed => ResourceState::Consumed,
            causality_resource::RegisterState::Archived => ResourceState::Archived,
        };
        
        // Update our cache
        let mut states = self.resource_states.write().map_err(|_| 
            anyhow::anyhow!("Failed to acquire write lock on resource states"))?;
        states.insert(resource_id.clone(), state);
        
        Ok(state)
    }
    
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.resource_manager.verify_resource_register(
            resource_id.clone(),
            self.domain_adapter.domain_id().clone(),
        ).await.map_err(|e| anyhow::anyhow!("Failed to verify resource: {}", e))
    }
}

#[async_trait]
impl ResourceLocking for DomainResourceImplementation {
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> Result<LockStatus> {
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
    
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
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
    
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
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
    
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> Result<Option<ResourceLockInfo>> {
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
}

#[async_trait]
impl ResourceDependency for DomainResourceImplementation {
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
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
    
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
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
    
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        let dependencies = self.dependencies.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependencies"))?;
        
        Ok(dependencies
            .get(resource_id)
            .cloned()
            .unwrap_or_default())
    }
    
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        let dependents = self.dependents.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependents"))?;
        
        Ok(dependents
            .get(resource_id)
            .cloned()
            .unwrap_or_default())
    }
    
    async fn has_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        let dependencies = self.dependencies.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependencies"))?;
        
        Ok(dependencies.contains_key(resource_id))
    }
    
    async fn has_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        let dependents = self.dependents.read().map_err(|_| 
            anyhow::anyhow!("Failed to acquire read lock on dependents"))?;
        
        Ok(dependents.contains_key(resource_id))
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