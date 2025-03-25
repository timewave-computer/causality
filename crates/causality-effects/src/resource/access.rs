// Resource access pattern module
// This file implements resource access patterns that work across domain and effect boundaries

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use async_trait::async_trait;

use causality_domain::domain::{DomainId, DomainAdapter};
use causality_types::{Error, Result, ContentId};
use crate::effect::{Effect, EffectContext, EffectId, EffectResult, EffectError, EffectOutcome};
use crate::domain_effect::{DomainAdapterEffect, DomainContext};

/// Resource access type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceAccessType {
    /// Read access to a resource
    Read,
    
    /// Write access to a resource
    Write,
    
    /// Execution access (for smart contracts)
    Execute,
    
    /// Lock acquisition
    Lock,
    
    /// Transfer ownership
    Transfer,
}

/// Resource access record
#[derive(Debug, Clone)]
pub struct ResourceAccess {
    /// Resource ID
    pub resource_id: ContentId,
    
    /// Access type
    pub access_type: ResourceAccessType,
    
    /// Domain ID where the resource is located
    pub domain_id: Option<DomainId>,
    
    /// Effect ID that requested access
    pub effect_id: EffectId,
    
    /// Whether the access was granted
    pub granted: bool,
    
    /// Timestamp when the access was requested
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ResourceAccess {
    /// Create a new resource access record
    pub fn new(
        resource_id: ContentId,
        access_type: ResourceAccessType,
        effect_id: EffectId,
    ) -> Self {
        Self {
            resource_id,
            access_type,
            domain_id: None,
            effect_id,
            granted: false,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create a new resource access with domain
    pub fn with_domain(
        resource_id: ContentId,
        access_type: ResourceAccessType,
        domain_id: DomainId,
        effect_id: EffectId,
    ) -> Self {
        Self {
            resource_id,
            access_type,
            domain_id: Some(domain_id),
            effect_id,
            granted: false,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Mark the access as granted
    pub fn grant(&mut self) {
        self.granted = true;
    }
}

/// Resource access tracker for effects
#[derive(Debug, Default)]
pub struct ResourceAccessTracker {
    /// Accesses by resource ID
    accesses_by_resource: RwLock<HashMap<ContentId, Vec<ResourceAccess>>>,
    
    /// Accesses by effect ID
    accesses_by_effect: RwLock<HashMap<EffectId, Vec<ResourceAccess>>>,
    
    /// Accesses by domain ID
    accesses_by_domain: RwLock<HashMap<DomainId, Vec<ResourceAccess>>>,
    
    /// Locks by resource ID
    locks: RwLock<HashMap<ContentId, ResourceAccess>>,
}

impl ResourceAccessTracker {
    /// Create a new resource access tracker
    pub fn new() -> Self {
        Self {
            accesses_by_resource: RwLock::new(HashMap::new()),
            accesses_by_effect: RwLock::new(HashMap::new()),
            accesses_by_domain: RwLock::new(HashMap::new()),
            locks: RwLock::new(HashMap::new()),
        }
    }
    
    /// Record a resource access
    pub fn record_access(&self, access: ResourceAccess) -> Result<()> {
        // Add to resource map
        {
            let mut map = self.accesses_by_resource.write().unwrap();
            map.entry(access.resource_id.clone())
                .or_insert_with(Vec::new)
                .push(access.clone());
        }
        
        // Add to effect map
        {
            let mut map = self.accesses_by_effect.write().unwrap();
            map.entry(access.effect_id.clone())
                .or_insert_with(Vec::new)
                .push(access.clone());
        }
        
        // Add to domain map if domain is present
        if let Some(domain_id) = &access.domain_id {
            let mut map = self.accesses_by_domain.write().unwrap();
            map.entry(domain_id.clone())
                .or_insert_with(Vec::new)
                .push(access.clone());
        }
        
        // Record lock if this is a lock access
        if access.access_type == ResourceAccessType::Lock && access.granted {
            let mut locks = self.locks.write().unwrap();
            locks.insert(access.resource_id.clone(), access);
        }
        
        Ok(())
    }
    
    /// Check if a resource is locked
    pub fn is_resource_locked(&self, resource_id: &ContentId) -> bool {
        let locks = self.locks.read().unwrap();
        locks.contains_key(resource_id)
    }
    
    /// Get the lock for a resource
    pub fn get_resource_lock(&self, resource_id: &ContentId) -> Option<ResourceAccess> {
        let locks = self.locks.read().unwrap();
        locks.get(resource_id).cloned()
    }
    
    /// Release a lock on a resource
    pub fn release_lock(&self, resource_id: &ContentId, effect_id: &EffectId) -> Result<()> {
        let mut locks = self.locks.write().unwrap();
        
        // Check if the lock exists and is owned by the effect
        if let Some(lock) = locks.get(resource_id) {
            if lock.effect_id == *effect_id {
                locks.remove(resource_id);
                return Ok(());
            } else {
                return Err(Error::AuthorizationError(
                    format!("Lock on resource {} is owned by a different effect", resource_id)
                ));
            }
        }
        
        Err(Error::NotFound(format!("No lock found for resource {}", resource_id)))
    }
    
    /// Get all accesses for a resource
    pub fn get_resource_accesses(&self, resource_id: &ContentId) -> Vec<ResourceAccess> {
        let map = self.accesses_by_resource.read().unwrap();
        map.get(resource_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all accesses for an effect
    pub fn get_effect_accesses(&self, effect_id: &EffectId) -> Vec<ResourceAccess> {
        let map = self.accesses_by_effect.read().unwrap();
        map.get(effect_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all accesses for a domain
    pub fn get_domain_accesses(&self, domain_id: &DomainId) -> Vec<ResourceAccess> {
        let map = self.accesses_by_domain.read().unwrap();
        map.get(domain_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Effect trait extension for resource access tracking
pub trait ResourceAccessTracking: Effect {
    /// Get resources that will be accessed by this effect
    fn resources_to_access(&self) -> Vec<(ContentId, ResourceAccessType)> {
        Vec::new()
    }
    
    /// Check if the effect can access resources
    fn can_access_resources(&self, tracker: &ResourceAccessTracker) -> EffectResult<()> {
        let resources = self.resources_to_access();
        
        for (resource_id, access_type) in resources {
            // Check if resource is locked
            if tracker.is_resource_locked(&resource_id) {
                // If this effect wants to obtain a lock, check that no lock exists
                if access_type == ResourceAccessType::Lock {
                    return Err(EffectError::ResourceError(
                        format!("Resource {} is already locked", resource_id)
                    ));
                }
                
                // For other access types, check the lock owner
                let lock = tracker.get_resource_lock(&resource_id).unwrap();
                if lock.effect_id != self.id() {
                    return Err(EffectError::ResourceError(
                        format!("Resource {} is locked by another effect", resource_id)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Track resource accesses
    fn track_resource_accesses(&self, tracker: &ResourceAccessTracker) -> EffectResult<()> {
        let resources = self.resources_to_access();
        
        for (resource_id, access_type) in resources {
            let mut access = ResourceAccess::new(
                resource_id,
                access_type,
                self.id().clone(),
            );
            
            // Get domain ID if this is a domain effect
            if let Some(domain_effect) = self.as_any().downcast_ref::<dyn DomainAdapterEffect>() {
                access.domain_id = Some(domain_effect.domain_id().clone());
            }
            
            // Mark the access as granted
            access.grant();
            
            // Record the access
            tracker.record_access(access)
                .map_err(|e| EffectError::ResourceError(format!("Failed to record resource access: {}", e)))?;
        }
        
        Ok(())
    }
}

/// Resource access manager for coordinating access across effects and domains
pub struct ResourceAccessManager {
    /// Access tracker
    tracker: Arc<ResourceAccessTracker>,
    
    /// Resources that are in use
    in_use_resources: Mutex<HashMap<ContentId, HashSet<EffectId>>>,
}

impl ResourceAccessManager {
    /// Create a new resource access manager
    pub fn new() -> Self {
        Self {
            tracker: Arc::new(ResourceAccessTracker::new()),
            in_use_resources: Mutex::new(HashMap::new()),
        }
    }
    
    /// Get the access tracker
    pub fn tracker(&self) -> Arc<ResourceAccessTracker> {
        Arc::clone(&self.tracker)
    }
    
    /// Request access to a resource
    pub async fn request_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
        effect: &dyn Effect,
    ) -> EffectResult<()> {
        // Check if resource is locked
        if self.tracker.is_resource_locked(resource_id) {
            // Only the lock owner can access the resource
            let lock = self.tracker.get_resource_lock(resource_id).unwrap();
            if lock.effect_id != effect.id() {
                return Err(EffectError::ResourceError(
                    format!("Resource {} is locked by another effect", resource_id)
                ));
            }
        }
        
        // Create and record the access
        let mut access = ResourceAccess::new(
            resource_id.clone(),
            access_type,
            effect.id().clone(),
        );
        
        // Get domain ID if this is a domain effect
        if let Some(domain_effect) = effect.as_any().downcast_ref::<dyn DomainAdapterEffect>() {
            access.domain_id = Some(domain_effect.domain_id().clone());
        }
        
        // Mark the access as granted
        access.grant();
        
        // Record the access
        self.tracker.record_access(access)
            .map_err(|e| EffectError::ResourceError(format!("Failed to record resource access: {}", e)))?;
        
        // Mark the resource as in use by this effect
        let mut in_use = self.in_use_resources.lock().unwrap();
        in_use.entry(resource_id.clone())
            .or_insert_with(HashSet::new)
            .insert(effect.id().clone());
        
        Ok(())
    }
    
    /// Release access to a resource
    pub fn release_access(
        &self,
        resource_id: &ContentId,
        effect_id: &EffectId,
    ) -> EffectResult<()> {
        // Remove from in-use resources
        let mut in_use = self.in_use_resources.lock().unwrap();
        if let Some(effects) = in_use.get_mut(resource_id) {
            effects.remove(effect_id);
            if effects.is_empty() {
                in_use.remove(resource_id);
            }
        }
        
        // Release any locks held by this effect
        if self.tracker.is_resource_locked(resource_id) {
            let lock = self.tracker.get_resource_lock(resource_id).unwrap();
            if lock.effect_id == *effect_id {
                self.tracker.release_lock(resource_id, effect_id)
                    .map_err(|e| EffectError::ResourceError(format!("Failed to release lock: {}", e)))?;
            }
        }
        
        Ok(())
    }
    
    /// Check if a resource is in use
    pub fn is_resource_in_use(&self, resource_id: &ContentId) -> bool {
        let in_use = self.in_use_resources.lock().unwrap();
        in_use.contains_key(resource_id)
    }
    
    /// Get effects using a resource
    pub fn get_effects_using_resource(&self, resource_id: &ContentId) -> HashSet<EffectId> {
        let in_use = self.in_use_resources.lock().unwrap();
        in_use.get(resource_id)
            .cloned()
            .unwrap_or_default()
    }
} 