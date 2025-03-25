// Resource lifecycle management integration
// This file implements resource lifecycle management for effects and domains

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;
use causality_domain::domain::{DomainId, DomainAdapter};
use causality_resource::lifecycle::{ResourceLifecycle, ResourceOperation, TransitionReason};
use causality_types::{Error, Result, ContentId};
use causality_resource::{ResourceRegister, RegisterState};
use crate::effect::{Effect, EffectContext, EffectId, EffectResult, EffectError, EffectOutcome};
use crate::domain_effect::{DomainAdapterEffect, DomainContext};

/// Resource lifecycle event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifecycleEvent {
    /// Resource created
    Created,
    
    /// Resource activated
    Activated,
    
    /// Resource locked
    Locked,
    
    /// Resource unlocked
    Unlocked,
    
    /// Resource frozen
    Frozen,
    
    /// Resource unfrozen
    Unfrozen,
    
    /// Resource consumed
    Consumed,
    
    /// Resource archived
    Archived,
    
    /// Resource updated
    Updated,
}

/// Resource lifecycle event
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    /// Resource ID
    pub resource_id: ContentId,
    
    /// Event type
    pub event_type: ResourceLifecycleEvent,
    
    /// Effect ID that triggered the event
    pub effect_id: EffectId,
    
    /// Domain ID where the resource is located
    pub domain_id: Option<DomainId>,
    
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event
    pub fn new(
        resource_id: ContentId,
        event_type: ResourceLifecycleEvent,
        effect_id: EffectId,
    ) -> Self {
        Self {
            resource_id,
            event_type,
            effect_id,
            domain_id: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a lifecycle event with domain
    pub fn with_domain(
        resource_id: ContentId,
        event_type: ResourceLifecycleEvent,
        effect_id: EffectId,
        domain_id: DomainId,
    ) -> Self {
        Self {
            resource_id,
            event_type,
            effect_id,
            domain_id: Some(domain_id),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Resource lifecycle manager for effects
pub struct EffectResourceLifecycle {
    /// Underlying resource lifecycle
    lifecycle: Arc<ResourceLifecycle>,
    
    /// Pending lifecycle events
    pending_events: RwLock<Vec<LifecycleEvent>>,
    
    /// Resource state cache
    resource_states: RwLock<HashMap<ContentId, RegisterState>>,
    
    /// Resources by domain
    resources_by_domain: RwLock<HashMap<DomainId, HashSet<ContentId>>>,
    
    /// Resources by effect
    resources_by_effect: RwLock<HashMap<EffectId, HashSet<ContentId>>>,
    
    /// Lifecycle event handlers
    event_handlers: Mutex<Vec<Box<dyn Fn(&LifecycleEvent) + Send + Sync>>>,
}

impl EffectResourceLifecycle {
    /// Create a new effect resource lifecycle
    pub fn new(lifecycle: Arc<ResourceLifecycle>) -> Self {
        Self {
            lifecycle,
            pending_events: RwLock::new(Vec::new()),
            resource_states: RwLock::new(HashMap::new()),
            resources_by_domain: RwLock::new(HashMap::new()),
            resources_by_effect: RwLock::new(HashMap::new()),
            event_handlers: Mutex::new(Vec::new()),
        }
    }
    
    /// Get the underlying resource lifecycle
    pub fn resource_lifecycle(&self) -> Arc<ResourceLifecycle> {
        Arc::clone(&self.lifecycle)
    }
    
    /// Register a resource
    pub fn register_resource(
        &self,
        resource: &ResourceRegister,
        effect_id: &EffectId,
        domain_id: Option<&DomainId>,
    ) -> Result<()> {
        // Register with the underlying lifecycle
        self.lifecycle.register(resource)?;
        
        // Update our state cache
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource.id.clone(), resource.state.clone());
        }
        
        // Update domain index if domain is provided
        if let Some(domain_id) = domain_id {
            let mut domain_resources = self.resources_by_domain.write().unwrap();
            domain_resources.entry(domain_id.clone())
                .or_insert_with(HashSet::new)
                .insert(resource.id.clone());
        }
        
        // Update effect index
        {
            let mut effect_resources = self.resources_by_effect.write().unwrap();
            effect_resources.entry(effect_id.clone())
                .or_insert_with(HashSet::new)
                .insert(resource.id.clone());
        }
        
        // Record the event
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource.id.clone(),
                ResourceLifecycleEvent::Created,
                effect_id.clone(),
                domain_id.clone(),
            )
        } else {
            LifecycleEvent::new(
                resource.id.clone(),
                ResourceLifecycleEvent::Created,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Update resource state
    pub fn update_resource_state(
        &self,
        resource_id: &ContentId,
        new_state: RegisterState,
        effect_id: &EffectId,
    ) -> Result<()> {
        // Update our state cache
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource_id.clone(), new_state.clone());
        }
        
        // Record the event
        let event_type = match new_state {
            RegisterState::Active => ResourceLifecycleEvent::Activated,
            RegisterState::Locked => ResourceLifecycleEvent::Locked,
            RegisterState::Frozen => ResourceLifecycleEvent::Frozen,
            RegisterState::Consumed => ResourceLifecycleEvent::Consumed,
            RegisterState::Archived => ResourceLifecycleEvent::Archived,
            RegisterState::Pending => ResourceLifecycleEvent::Updated,
            _ => ResourceLifecycleEvent::Updated,
        };
        
        // Get domain from our indices
        let domain_id = {
            let mut domain_id = None;
            let domain_resources = self.resources_by_domain.read().unwrap();
            
            for (did, resources) in domain_resources.iter() {
                if resources.contains(resource_id) {
                    domain_id = Some(did.clone());
                    break;
                }
            }
            
            domain_id
        };
        
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource_id.clone(),
                event_type,
                effect_id.clone(),
                domain_id,
            )
        } else {
            LifecycleEvent::new(
                resource_id.clone(),
                event_type,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Activate a resource
    pub fn activate_resource(
        &self,
        resource: &mut ResourceRegister,
        effect_id: &EffectId,
    ) -> Result<()> {
        // Activate with the underlying lifecycle
        self.lifecycle.activate(resource)?;
        
        // Update our state cache
        let resource_id = resource.id.clone();
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource_id.clone(), RegisterState::Active);
        }
        
        // Get domain from our indices
        let domain_id = {
            let mut domain_id = None;
            let domain_resources = self.resources_by_domain.read().unwrap();
            
            for (did, resources) in domain_resources.iter() {
                if resources.contains(&resource_id) {
                    domain_id = Some(did.clone());
                    break;
                }
            }
            
            domain_id
        };
        
        // Record the event
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource_id,
                ResourceLifecycleEvent::Activated,
                effect_id.clone(),
                domain_id,
            )
        } else {
            LifecycleEvent::new(
                resource_id,
                ResourceLifecycleEvent::Activated,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Lock a resource
    pub fn lock_resource(
        &self,
        resource: &mut ResourceRegister,
        effect_id: &EffectId,
        locker_id: Option<&ContentId>,
    ) -> Result<()> {
        // Lock with the underlying lifecycle
        self.lifecycle.lock(resource, locker_id)?;
        
        // Update our state cache
        let resource_id = resource.id.clone();
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource_id.clone(), RegisterState::Locked);
        }
        
        // Get domain from our indices
        let domain_id = {
            let mut domain_id = None;
            let domain_resources = self.resources_by_domain.read().unwrap();
            
            for (did, resources) in domain_resources.iter() {
                if resources.contains(&resource_id) {
                    domain_id = Some(did.clone());
                    break;
                }
            }
            
            domain_id
        };
        
        // Record the event
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource_id,
                ResourceLifecycleEvent::Locked,
                effect_id.clone(),
                domain_id,
            )
        } else {
            LifecycleEvent::new(
                resource_id,
                ResourceLifecycleEvent::Locked,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Unlock a resource
    pub fn unlock_resource(
        &self,
        resource: &mut ResourceRegister,
        effect_id: &EffectId,
        unlocker_id: Option<&ContentId>,
    ) -> Result<()> {
        // Unlock with the underlying lifecycle
        self.lifecycle.unlock(resource, unlocker_id)?;
        
        // Update our state cache
        let resource_id = resource.id.clone();
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource_id.clone(), RegisterState::Active);
        }
        
        // Get domain from our indices
        let domain_id = {
            let mut domain_id = None;
            let domain_resources = self.resources_by_domain.read().unwrap();
            
            for (did, resources) in domain_resources.iter() {
                if resources.contains(&resource_id) {
                    domain_id = Some(did.clone());
                    break;
                }
            }
            
            domain_id
        };
        
        // Record the event
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource_id,
                ResourceLifecycleEvent::Unlocked,
                effect_id.clone(),
                domain_id,
            )
        } else {
            LifecycleEvent::new(
                resource_id,
                ResourceLifecycleEvent::Unlocked,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Consume a resource
    pub fn consume_resource(
        &self,
        resource: &mut ResourceRegister,
        effect_id: &EffectId,
    ) -> Result<()> {
        // Consume with the underlying lifecycle
        self.lifecycle.consume(resource)?;
        
        // Update our state cache
        let resource_id = resource.id.clone();
        {
            let mut states = self.resource_states.write().unwrap();
            states.insert(resource_id.clone(), RegisterState::Consumed);
        }
        
        // Get domain from our indices
        let domain_id = {
            let mut domain_id = None;
            let domain_resources = self.resources_by_domain.read().unwrap();
            
            for (did, resources) in domain_resources.iter() {
                if resources.contains(&resource_id) {
                    domain_id = Some(did.clone());
                    break;
                }
            }
            
            domain_id
        };
        
        // Record the event
        let event = if let Some(domain_id) = domain_id {
            LifecycleEvent::with_domain(
                resource_id,
                ResourceLifecycleEvent::Consumed,
                effect_id.clone(),
                domain_id,
            )
        } else {
            LifecycleEvent::new(
                resource_id,
                ResourceLifecycleEvent::Consumed,
                effect_id.clone(),
            )
        };
        
        self.record_event(event);
        
        Ok(())
    }
    
    /// Get the current state of a resource
    pub fn get_resource_state(&self, resource_id: &ContentId) -> Option<RegisterState> {
        let states = self.resource_states.read().unwrap();
        states.get(resource_id).cloned()
    }
    
    /// Check if a resource can perform an operation
    pub fn can_perform_operation(
        &self,
        resource: &ResourceRegister,
        operation: ResourceOperation,
    ) -> Result<bool> {
        self.lifecycle.can_perform_operation(resource, &operation)
    }
    
    /// Register an event handler
    pub fn register_event_handler<F>(&self, handler: F)
    where
        F: Fn(&LifecycleEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }
    
    /// Record a lifecycle event
    fn record_event(&self, event: LifecycleEvent) {
        // Add to pending events
        {
            let mut events = self.pending_events.write().unwrap();
            events.push(event.clone());
        }
        
        // Notify handlers
        let handlers = self.event_handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler(&event);
        }
    }
    
    /// Get all pending lifecycle events
    pub fn get_pending_events(&self) -> Vec<LifecycleEvent> {
        let events = self.pending_events.read().unwrap();
        events.clone()
    }
    
    /// Clear pending events
    pub fn clear_pending_events(&self) {
        let mut events = self.pending_events.write().unwrap();
        events.clear();
    }
}

/// Effect for managing resource lifecycle
#[derive(Debug)]
pub struct ResourceLifecycleEffect {
    /// Effect ID
    id: EffectId,
    
    /// Resource ID
    resource_id: ContentId,
    
    /// Operation to perform
    operation: ResourceOperation,
    
    /// Domain ID
    domain_id: Option<DomainId>,
    
    /// Controller ID (for lock operations)
    controller_id: Option<ContentId>,
    
    /// Transition reason
    reason: TransitionReason,
}

impl ResourceLifecycleEffect {
    /// Create a new resource lifecycle effect
    pub fn new(
        resource_id: ContentId,
        operation: ResourceOperation,
    ) -> Self {
        Self {
            id: EffectId::new(),
            resource_id,
            operation,
            domain_id: None,
            controller_id: None,
            reason: TransitionReason::UserRequested,
        }
    }
    
    /// Set the domain ID
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Set the controller ID
    pub fn with_controller(mut self, controller_id: ContentId) -> Self {
        self.controller_id = Some(controller_id);
        self
    }
    
    /// Set the transition reason
    pub fn with_reason(mut self, reason: TransitionReason) -> Self {
        self.reason = reason;
        self
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    /// Get the operation
    pub fn operation(&self) -> ResourceOperation {
        self.operation
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> Option<&DomainId> {
        self.domain_id.as_ref()
    }
}

/// Factory functions for creating common resource lifecycle effects
pub mod effects {
    use super::*;
    
    /// Create an effect to activate a resource
    pub fn activate_resource(resource_id: ContentId) -> ResourceLifecycleEffect {
        ResourceLifecycleEffect::new(resource_id, ResourceOperation::Update)
            .with_reason(TransitionReason::UserRequested)
    }
    
    /// Create an effect to lock a resource
    pub fn lock_resource(resource_id: ContentId, locker_id: Option<ContentId>) -> ResourceLifecycleEffect {
        let effect = ResourceLifecycleEffect::new(resource_id, ResourceOperation::Lock)
            .with_reason(TransitionReason::UserRequested);
            
        if let Some(locker_id) = locker_id {
            effect.with_controller(locker_id)
        } else {
            effect
        }
    }
    
    /// Create an effect to unlock a resource
    pub fn unlock_resource(resource_id: ContentId, unlocker_id: Option<ContentId>) -> ResourceLifecycleEffect {
        let effect = ResourceLifecycleEffect::new(resource_id, ResourceOperation::Unlock)
            .with_reason(TransitionReason::UserRequested);
            
        if let Some(unlocker_id) = unlocker_id {
            effect.with_controller(unlocker_id)
        } else {
            effect
        }
    }
    
    /// Create an effect to consume a resource
    pub fn consume_resource(resource_id: ContentId) -> ResourceLifecycleEffect {
        ResourceLifecycleEffect::new(resource_id, ResourceOperation::Consume)
            .with_reason(TransitionReason::UserRequested)
    }
    
    /// Create an effect to archive a resource
    pub fn archive_resource(resource_id: ContentId) -> ResourceLifecycleEffect {
        ResourceLifecycleEffect::new(resource_id, ResourceOperation::Archive)
            .with_reason(TransitionReason::UserRequested)
    }
} 