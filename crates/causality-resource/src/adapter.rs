// Resource Adapter Patterns
//
// This module provides adapter patterns that facilitate the interaction
// between domain and effect resource management systems. These adapters
// ensure proper capability context propagation and resource lifecycle
// management across system boundaries.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;

use causality_common::identity::ContentId;
use causality_domain::domain::{DomainId, DomainRegistry};
use causality_effects::effect::{EffectId, EffectRegistry};

use crate::interface::{
    ResourceState, ResourceAccessType, LockType, DependencyType, LockStatus,
    ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency,
    ResourceContext, ResourceAccessRecord, ResourceLockInfo, ResourceDependencyInfo
};

/// Adapter for translating domain resource operations to effect resource operations
pub struct DomainToEffectResourceAdapter {
    /// Domain registry for context lookup
    domain_registry: Arc<DomainRegistry>,
    /// Effect registry for context lookup
    effect_registry: Arc<EffectRegistry>,
    /// Resource access implementation on the effect side
    effect_resource_access: Arc<dyn ResourceAccess + Send + Sync>,
    /// Resource lifecycle implementation on the effect side
    effect_resource_lifecycle: Arc<dyn ResourceLifecycle + Send + Sync>,
    /// Resource locking implementation on the effect side
    effect_resource_locking: Arc<dyn ResourceLocking + Send + Sync>,
    /// Resource dependency implementation on the effect side
    effect_resource_dependency: Arc<dyn ResourceDependency + Send + Sync>,
    /// Context conversion mappings (domain ID to effect ID)
    context_mappings: Arc<RwLock<HashMap<DomainId, EffectId>>>,
}

impl DomainToEffectResourceAdapter {
    /// Create a new domain-to-effect resource adapter
    pub fn new(
        domain_registry: Arc<DomainRegistry>,
        effect_registry: Arc<EffectRegistry>,
        effect_resource_access: Arc<dyn ResourceAccess + Send + Sync>,
        effect_resource_lifecycle: Arc<dyn ResourceLifecycle + Send + Sync>,
        effect_resource_locking: Arc<dyn ResourceLocking + Send + Sync>,
        effect_resource_dependency: Arc<dyn ResourceDependency + Send + Sync>,
    ) -> Self {
        Self {
            domain_registry,
            effect_registry,
            effect_resource_access,
            effect_resource_lifecycle,
            effect_resource_locking,
            effect_resource_dependency,
            context_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a domain-to-effect context mapping
    pub fn register_context_mapping(&self, domain_id: DomainId, effect_id: EffectId) -> Result<()> {
        let mut mappings = self.context_mappings.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on context mappings")
        })?;
        
        mappings.insert(domain_id, effect_id);
        Ok(())
    }
    
    /// Get the effect ID for a domain ID
    pub fn get_effect_for_domain(&self, domain_id: &DomainId) -> Result<Option<EffectId>> {
        let mappings = self.context_mappings.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on context mappings")
        })?;
        
        Ok(mappings.get(domain_id).cloned())
    }
    
    /// Convert a domain resource context to an effect resource context
    fn convert_context(&self, domain_context: &dyn ResourceContext) -> Result<EffectResourceContext> {
        let domain_id = match domain_context.domain_id() {
            Some(id) => id.clone(),
            None => return Err(anyhow::anyhow!("Domain context missing domain ID")),
        };
        
        // Look up the effect ID for this domain
        let effect_id = match self.get_effect_for_domain(&domain_id)? {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("No effect mapped for domain {}", domain_id)),
        };
        
        // Create a new effect context
        let mut context = EffectResourceContext::new(domain_context.context_id());
        context.domain_id = Some(domain_id);
        context.effect_id = Some(effect_id);
        context.timestamp = domain_context.timestamp();
        
        // Copy metadata
        for (key, value) in domain_context.metadata() {
            context.metadata.insert(key.clone(), value.clone());
        }
        
        Ok(context)
    }
}

/// Resource context for effect operations
#[derive(Debug, Clone)]
pub struct EffectResourceContext {
    /// ID of the context
    pub context_id: ContentId,
    /// ID of the domain
    pub domain_id: Option<ContentId>,
    /// ID of the effect
    pub effect_id: Option<ContentId>,
    /// Timestamp of the operation
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl EffectResourceContext {
    /// Create a new effect resource context
    pub fn new(context_id: ContentId) -> Self {
        Self {
            context_id,
            domain_id: None,
            effect_id: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
}

impl ResourceContext for EffectResourceContext {
    fn context_id(&self) -> ContentId {
        self.context_id.clone()
    }
    
    fn domain_id(&self) -> Option<&ContentId> {
        self.domain_id.as_ref()
    }
    
    fn effect_id(&self) -> Option<&ContentId> {
        self.effect_id.as_ref()
    }
    
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Adapter for domain resource access using effect resource access
#[async_trait]
impl ResourceAccess for DomainToEffectResourceAdapter {
    async fn is_access_allowed(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_access.is_access_allowed(
            resource_id, 
            access_type, 
            &effect_context
        ).await
    }
    
    async fn record_access(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_access.record_access(
            resource_id, 
            access_type, 
            &effect_context
        ).await
    }
    
    async fn get_resource_accesses(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceAccessRecord>> {
        self.effect_resource_access.get_resource_accesses(resource_id).await
    }
}

/// Adapter for domain resource lifecycle using effect resource lifecycle
#[async_trait]
impl ResourceLifecycle for DomainToEffectResourceAdapter {
    async fn register_resource(
        &self, 
        resource_id: ContentId, 
        initial_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_lifecycle.register_resource(
            resource_id, 
            initial_state, 
            &effect_context
        ).await
    }
    
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_lifecycle.update_resource_state(
            resource_id, 
            new_state, 
            &effect_context
        ).await
    }
    
    async fn get_resource_state(
        &self, 
        resource_id: &ContentId
    ) -> Result<ResourceState> {
        self.effect_resource_lifecycle.get_resource_state(resource_id).await
    }
    
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.effect_resource_lifecycle.resource_exists(resource_id).await
    }
}

/// Adapter for domain resource locking using effect resource locking
#[async_trait]
impl ResourceLocking for DomainToEffectResourceAdapter {
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> Result<LockStatus> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_locking.acquire_lock(
            resource_id, 
            lock_type, 
            holder_id, 
            timeout, 
            &effect_context
        ).await
    }
    
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_locking.release_lock(
            resource_id, 
            holder_id, 
            &effect_context
        ).await
    }
    
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.effect_resource_locking.is_locked(resource_id).await
    }
    
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> Result<Option<ResourceLockInfo>> {
        self.effect_resource_locking.get_lock_info(resource_id).await
    }
}

/// Adapter for domain resource dependency using effect resource dependency
#[async_trait]
impl ResourceDependency for DomainToEffectResourceAdapter {
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_dependency.add_dependency(
            source_id, 
            target_id, 
            dependency_type, 
            &effect_context
        ).await
    }
    
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let effect_context = self.convert_context(context)?;
        self.effect_resource_dependency.remove_dependency(
            source_id, 
            target_id, 
            dependency_type, 
            &effect_context
        ).await
    }
    
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        self.effect_resource_dependency.get_dependencies(resource_id).await
    }
    
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        self.effect_resource_dependency.get_dependents(resource_id).await
    }
    
    async fn has_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.effect_resource_dependency.has_dependencies(resource_id).await
    }
    
    async fn has_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.effect_resource_dependency.has_dependents(resource_id).await
    }
}

/// Adapter for translating effect resource operations to domain resource operations
pub struct EffectToDomainResourceAdapter {
    /// Domain registry for context lookup
    domain_registry: Arc<DomainRegistry>,
    /// Effect registry for context lookup
    effect_registry: Arc<EffectRegistry>,
    /// Resource access implementation on the domain side
    domain_resource_access: Arc<dyn ResourceAccess + Send + Sync>,
    /// Resource lifecycle implementation on the domain side
    domain_resource_lifecycle: Arc<dyn ResourceLifecycle + Send + Sync>,
    /// Resource locking implementation on the domain side
    domain_resource_locking: Arc<dyn ResourceLocking + Send + Sync>,
    /// Resource dependency implementation on the domain side
    domain_resource_dependency: Arc<dyn ResourceDependency + Send + Sync>,
    /// Context conversion mappings (effect ID to domain ID)
    context_mappings: Arc<RwLock<HashMap<EffectId, DomainId>>>,
}

impl EffectToDomainResourceAdapter {
    /// Create a new effect-to-domain resource adapter
    pub fn new(
        domain_registry: Arc<DomainRegistry>,
        effect_registry: Arc<EffectRegistry>,
        domain_resource_access: Arc<dyn ResourceAccess + Send + Sync>,
        domain_resource_lifecycle: Arc<dyn ResourceLifecycle + Send + Sync>,
        domain_resource_locking: Arc<dyn ResourceLocking + Send + Sync>,
        domain_resource_dependency: Arc<dyn ResourceDependency + Send + Sync>,
    ) -> Self {
        Self {
            domain_registry,
            effect_registry,
            domain_resource_access,
            domain_resource_lifecycle,
            domain_resource_locking,
            domain_resource_dependency,
            context_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an effect-to-domain context mapping
    pub fn register_context_mapping(&self, effect_id: EffectId, domain_id: DomainId) -> Result<()> {
        let mut mappings = self.context_mappings.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on context mappings")
        })?;
        
        mappings.insert(effect_id, domain_id);
        Ok(())
    }
    
    /// Get the domain ID for an effect ID
    pub fn get_domain_for_effect(&self, effect_id: &EffectId) -> Result<Option<DomainId>> {
        let mappings = self.context_mappings.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on context mappings")
        })?;
        
        Ok(mappings.get(effect_id).cloned())
    }
    
    /// Convert an effect resource context to a domain resource context
    fn convert_context(&self, effect_context: &dyn ResourceContext) -> Result<DomainResourceContext> {
        let effect_id = match effect_context.effect_id() {
            Some(id) => id.clone(),
            None => return Err(anyhow::anyhow!("Effect context missing effect ID")),
        };
        
        // Look up the domain ID for this effect
        let domain_id = match self.get_domain_for_effect(&effect_id)? {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("No domain mapped for effect {}", effect_id)),
        };
        
        // Create a new domain context
        let mut context = DomainResourceContext::new(effect_context.context_id());
        context.domain_id = Some(domain_id);
        context.effect_id = Some(effect_id);
        context.timestamp = effect_context.timestamp();
        
        // Copy metadata
        for (key, value) in effect_context.metadata() {
            context.metadata.insert(key.clone(), value.clone());
        }
        
        Ok(context)
    }
}

/// Resource context for domain operations
#[derive(Debug, Clone)]
pub struct DomainResourceContext {
    /// ID of the context
    pub context_id: ContentId,
    /// ID of the domain
    pub domain_id: Option<ContentId>,
    /// ID of the effect
    pub effect_id: Option<ContentId>,
    /// Timestamp of the operation
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl DomainResourceContext {
    /// Create a new domain resource context
    pub fn new(context_id: ContentId) -> Self {
        Self {
            context_id,
            domain_id: None,
            effect_id: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
}

impl ResourceContext for DomainResourceContext {
    fn context_id(&self) -> ContentId {
        self.context_id.clone()
    }
    
    fn domain_id(&self) -> Option<&ContentId> {
        self.domain_id.as_ref()
    }
    
    fn effect_id(&self) -> Option<&ContentId> {
        self.effect_id.as_ref()
    }
    
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Adapter for effect resource access using domain resource access
#[async_trait]
impl ResourceAccess for EffectToDomainResourceAdapter {
    async fn is_access_allowed(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_access.is_access_allowed(
            resource_id, 
            access_type, 
            &domain_context
        ).await
    }
    
    async fn record_access(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_access.record_access(
            resource_id, 
            access_type, 
            &domain_context
        ).await
    }
    
    async fn get_resource_accesses(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceAccessRecord>> {
        self.domain_resource_access.get_resource_accesses(resource_id).await
    }
}

/// Adapter for effect resource lifecycle using domain resource lifecycle
#[async_trait]
impl ResourceLifecycle for EffectToDomainResourceAdapter {
    async fn register_resource(
        &self, 
        resource_id: ContentId, 
        initial_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_lifecycle.register_resource(
            resource_id, 
            initial_state, 
            &domain_context
        ).await
    }
    
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_lifecycle.update_resource_state(
            resource_id, 
            new_state, 
            &domain_context
        ).await
    }
    
    async fn get_resource_state(
        &self, 
        resource_id: &ContentId
    ) -> Result<ResourceState> {
        self.domain_resource_lifecycle.get_resource_state(resource_id).await
    }
    
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.domain_resource_lifecycle.resource_exists(resource_id).await
    }
}

/// Adapter for effect resource locking using domain resource locking
#[async_trait]
impl ResourceLocking for EffectToDomainResourceAdapter {
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> Result<LockStatus> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_locking.acquire_lock(
            resource_id, 
            lock_type, 
            holder_id, 
            timeout, 
            &domain_context
        ).await
    }
    
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_locking.release_lock(
            resource_id, 
            holder_id, 
            &domain_context
        ).await
    }
    
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.domain_resource_locking.is_locked(resource_id).await
    }
    
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> Result<Option<ResourceLockInfo>> {
        self.domain_resource_locking.get_lock_info(resource_id).await
    }
}

/// Adapter for effect resource dependency using domain resource dependency
#[async_trait]
impl ResourceDependency for EffectToDomainResourceAdapter {
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_dependency.add_dependency(
            source_id, 
            target_id, 
            dependency_type, 
            &domain_context
        ).await
    }
    
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<bool> {
        let domain_context = self.convert_context(context)?;
        self.domain_resource_dependency.remove_dependency(
            source_id, 
            target_id, 
            dependency_type, 
            &domain_context
        ).await
    }
    
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        self.domain_resource_dependency.get_dependencies(resource_id).await
    }
    
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>> {
        self.domain_resource_dependency.get_dependents(resource_id).await
    }
    
    async fn has_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.domain_resource_dependency.has_dependencies(resource_id).await
    }
    
    async fn has_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool> {
        self.domain_resource_dependency.has_dependents(resource_id).await
    }
} 