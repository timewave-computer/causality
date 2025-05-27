// Effect capabilities module
//
// Provides capability-based security for effects in the Causality system.

use crate::effect::types::EffectTypeId;
use crate::capability::{Capability, CapabilityError, CapabilityGrants, ResourceId};
use std::marker::PhantomData;

/// A capability for executing a specific effect type
pub struct EffectCapability<E> {
    /// The underlying capability
    pub capability: Capability<E>,
    
    /// The effect type this capability grants access to
    pub effect_type_id: EffectTypeId,
}

impl<E> EffectCapability<E> {
    /// Create a new effect capability
    pub fn new(resource_id: ResourceId, effect_type_id: EffectTypeId, grants: CapabilityGrants) -> Self {
        Self {
            capability: Capability::new(resource_id, grants, None),
            effect_type_id,
        }
    }
    
    /// Check if this capability allows executing the effect
    pub fn allows_execution(&self) -> bool {
        self.capability.grants.allows_write()
    }
    
    /// Check if this capability allows inspecting the effect
    pub fn allows_inspection(&self) -> bool {
        self.capability.grants.allows_read()
    }
    
    /// Create a restricted capability with more limited grants
    pub fn restrict(&self, grants: CapabilityGrants) -> Result<Self, CapabilityError> {
        // Ensure the new grants are a subset of the current grants
        if (grants.can_read && !self.capability.grants.can_read) ||
           (grants.can_write && !self.capability.grants.can_write) ||
           (grants.can_delegate && !self.capability.grants.can_delegate) {
            return Err(CapabilityError::InvalidGrants(
                "Cannot escalate privileges in a restricted capability".to_string()
            ));
        }
        
        Ok(Self {
            capability: Capability::new(
                self.capability.id.clone(),
                grants,
                self.capability.origin.clone()
            ),
            effect_type_id: self.effect_type_id.clone(),
        })
    }
} 