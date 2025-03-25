// Resource capability integration
//
// This module integrates resource management with the capability system,
// allowing resources to be protected by capabilities and defining
// resource-specific capability types.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_domain::domain::DomainId;
use causality_types::{Error, Result, ContentId};
use crate::capability::{
    UnifiedCapability, EffectCapability, CrossDomainCapability,
    UnifiedCapabilityContext, UnifiedCapabilityManager
};
use crate::effect_id::EffectId;
use super::access::{ResourceAccessType, ResourceAccessManager};
use super::lifecycle::{ResourceLifecycleEvent, EffectResourceLifecycle};
use super::locking::{CrossDomainLockManager, CrossDomainLockType};
use super::dependency::{ResourceDependencyManager, DependencyType};

/// Resource-specific capability types that extend the unified capability system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCapability {
    // Access capabilities
    AccessResource(ResourceAccessType),
    
    // Lifecycle capabilities
    ManageLifecycle(ResourceLifecycleCapability),
    
    // Locking capabilities
    LockResource(CrossDomainLockType),
    
    // Dependency capabilities
    ManageDependencies(DependencyType),
    
    // Combined capability
    FullResourceControl,
}

/// Resource lifecycle-specific capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceLifecycleCapability {
    Create,
    Activate,
    Update,
    Lock,
    Unlock,
    Freeze,
    Unfreeze,
    Consume,
    Archive,
    All,
}

/// Extension of the UnifiedCapability enum to include resource capabilities
impl From<ResourceCapability> for UnifiedCapability {
    fn from(cap: ResourceCapability) -> Self {
        match cap {
            ResourceCapability::AccessResource(access_type) => {
                match access_type {
                    ResourceAccessType::Read => UnifiedCapability::Effect(EffectCapability::ReadResource),
                    ResourceAccessType::Write => UnifiedCapability::Effect(EffectCapability::UpdateResource),
                    ResourceAccessType::Execute => UnifiedCapability::Effect(EffectCapability::Custom("ExecuteResource".to_string())),
                    ResourceAccessType::Lock => UnifiedCapability::Effect(EffectCapability::Custom("LockResource".to_string())),
                    ResourceAccessType::Transfer => UnifiedCapability::Effect(EffectCapability::Custom("TransferResource".to_string())),
                }
            },
            ResourceCapability::ManageLifecycle(lifecycle_cap) => {
                match lifecycle_cap {
                    ResourceLifecycleCapability::Create => UnifiedCapability::Effect(EffectCapability::CreateResource),
                    ResourceLifecycleCapability::Update => UnifiedCapability::Effect(EffectCapability::UpdateResource),
                    ResourceLifecycleCapability::Consume => UnifiedCapability::Effect(EffectCapability::DeleteResource),
                    _ => UnifiedCapability::Effect(EffectCapability::Custom(format!("ResourceLifecycle_{:?}", lifecycle_cap))),
                }
            },
            ResourceCapability::LockResource(lock_type) => {
                UnifiedCapability::CrossDomain(CrossDomainCapability::ResourceLocking { 
                    lock_type: format!("{:?}", lock_type) 
                })
            },
            ResourceCapability::ManageDependencies(dep_type) => {
                UnifiedCapability::CrossDomain(CrossDomainCapability::ResourceDependency { 
                    dependency_type: format!("{:?}", dep_type) 
                })
            },
            ResourceCapability::FullResourceControl => {
                UnifiedCapability::CrossDomain(CrossDomainCapability::FullResourceControl)
            },
        }
    }
}

/// Resource capability manager that coordinates capabilities with resource operations
pub struct ResourceCapabilityManager {
    /// Reference to the unified capability manager
    capability_manager: Arc<UnifiedCapabilityManager>,
    
    /// Reference to the resource access manager
    access_manager: Arc<ResourceAccessManager>,
    
    /// Reference to the resource lifecycle manager
    lifecycle_manager: Arc<EffectResourceLifecycle>,
    
    /// Reference to the cross-domain lock manager
    lock_manager: Arc<CrossDomainLockManager>,
    
    /// Reference to the resource dependency manager
    dependency_manager: Arc<ResourceDependencyManager>,
    
    /// Cache of verified capabilities
    capability_cache: RwLock<HashMap<(ContentId, ResourceCapability), bool>>,
}

impl ResourceCapabilityManager {
    /// Create a new resource capability manager
    pub fn new(
        capability_manager: Arc<UnifiedCapabilityManager>,
        access_manager: Arc<ResourceAccessManager>,
        lifecycle_manager: Arc<EffectResourceLifecycle>,
        lock_manager: Arc<CrossDomainLockManager>,
        dependency_manager: Arc<ResourceDependencyManager>,
    ) -> Self {
        Self {
            capability_manager,
            access_manager,
            lifecycle_manager,
            lock_manager,
            dependency_manager,
            capability_cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Check if a capability for a resource operation is granted
    pub async fn check_resource_capability(
        &self,
        resource_id: &ContentId,
        capability: ResourceCapability,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        // Check cache first
        {
            let cache = self.capability_cache.read().unwrap();
            if let Some(result) = cache.get(&(resource_id.clone(), capability.clone())) {
                return Ok(*result);
            }
        }
        
        // Convert resource capability to unified capability
        let unified_cap = UnifiedCapability::from(capability.clone());
        
        // Check capability with the unified capability manager
        let result = self.capability_manager.check_capability(
            &unified_cap,
            resource_id,
            context,
        ).await?;
        
        // Cache result
        {
            let mut cache = self.capability_cache.write().unwrap();
            cache.insert((resource_id.clone(), capability), result);
        }
        
        Ok(result)
    }
    
    /// Check if an access operation is allowed for a resource
    pub async fn check_access_capability(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        self.check_resource_capability(
            resource_id,
            ResourceCapability::AccessResource(access_type),
            effect_id,
            domain_id,
            context,
        ).await
    }
    
    /// Check if a lifecycle operation is allowed for a resource
    pub async fn check_lifecycle_capability(
        &self,
        resource_id: &ContentId,
        lifecycle_cap: ResourceLifecycleCapability,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        self.check_resource_capability(
            resource_id,
            ResourceCapability::ManageLifecycle(lifecycle_cap),
            effect_id,
            domain_id,
            context,
        ).await
    }
    
    /// Check if a locking operation is allowed for a resource
    pub async fn check_lock_capability(
        &self,
        resource_id: &ContentId,
        lock_type: CrossDomainLockType,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        self.check_resource_capability(
            resource_id,
            ResourceCapability::LockResource(lock_type),
            effect_id,
            domain_id,
            context,
        ).await
    }
    
    /// Check if a dependency operation is allowed for a resource
    pub async fn check_dependency_capability(
        &self,
        resource_id: &ContentId,
        dependency_type: DependencyType,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        self.check_resource_capability(
            resource_id,
            ResourceCapability::ManageDependencies(dependency_type),
            effect_id,
            domain_id,
            context,
        ).await
    }
    
    /// Check if full resource control is granted
    pub async fn check_full_control(
        &self,
        resource_id: &ContentId,
        effect_id: Option<&EffectId>,
        domain_id: Option<&DomainId>,
        context: &UnifiedCapabilityContext,
    ) -> Result<bool> {
        self.check_resource_capability(
            resource_id,
            ResourceCapability::FullResourceControl,
            effect_id,
            domain_id,
            context,
        ).await
    }
    
    /// Clear cached capability checks
    pub fn clear_cache(&self) {
        let mut cache = self.capability_cache.write().unwrap();
        cache.clear();
    }
    
    /// Clear cached capability checks for a specific resource
    pub fn clear_cache_for_resource(&self, resource_id: &ContentId) {
        let mut cache = self.capability_cache.write().unwrap();
        cache.retain(|(id, _), _| id != resource_id);
    }
} 