// Storage adapter for resources
// Original file: src/resource/storage_adapter.rs

// Unified Storage Adapter - Compatibility Layer
//
// This module provides a compatibility layer for code that still uses the old
// storage adapter interfaces. It delegates to the new unified effect system for
// actual storage operations.
//
// @deprecated - Use the effect system directly with the resource lifecycle manager instead

use std::collections::HashSet;
use std::sync::Arc;

use causality_crypto::ContentId;
use causality_resource::{ResourceRegister, NullifierId};
use causality_resource_manager::ResourceRegisterLifecycleManager;
use causality_resource::RelationshipTracker;
use causality_resource::{StorageStrategy, StorageEffect};
use crate::effect::{EffectContext, EffectOutcome};
use causality_effects::EffectRuntime;
use crate::domain::{DomainId, DomainType, DomainRegistry};
use causality_types::Address;
use causality_types::{Error, Result};
use causality_crypto::Commitment;

/// Unified Storage Adapter
///
/// This adapter provides a compatibility layer that uses the new unified
/// architecture internally while maintaining the old interface for backward compatibility.
pub struct StorageAdapter {
    // Core services from unified architecture
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    relationship_tracker: Arc<RelationshipTracker>,
    domain_registry: Arc<DomainRegistry>,
    effect_runtime: Arc<EffectRuntime>,
    invoker: Address,
}

impl StorageAdapter {
    /// Create a new storage adapter using the unified architecture components
    pub fn new(
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
        domain_registry: Arc<DomainRegistry>,
        effect_runtime: Arc<EffectRuntime>,
        invoker: Address,
    ) -> Self {
        Self {
            lifecycle_manager,
            relationship_tracker,
            domain_registry,
            effect_runtime,
            invoker,
        }
    }
    
    /// Compatibility method for storing a resource on-chain
    pub async fn store_resource(&self, resource_id: &ContentId, domain_id: &DomainId) -> Result<String> {
        // Get the resource from the lifecycle manager
        let resource_state = self.lifecycle_manager.get_state(resource_id)?;
        
        // Get the resource register
        let register = self.lifecycle_manager
            .get_resource(resource_id)
            .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))?;
        
        // Get the domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
        
        // Create the domain type from domain info
        let domain_type = DomainType::from_domain_id(domain_id);
        
        // Create fields to store based on the storage strategy
        let fields = match &register.storage_strategy {
            causality_resource::StorageStrategy::FullyOnChain { .. } => {
                // Store all fields
                register.all_fields()
            },
            causality_resource::StorageStrategy::Hybrid { on_chain_fields, .. } => {
                // Store only specified fields
                on_chain_fields.clone()
            },
            causality_resource::StorageStrategy::CommitmentBased { .. } => {
                // Store minimal fields for commitment-based storage
                let mut fields = HashSet::new();
                fields.insert("id".to_string());
                fields
            }
        };
        
        // Create the storage effect
        let effect = StorageEffect::new_store(
            register.clone(),
            StorageStrategy::OnChain,
            domain_type,
        );
        
        // Create the effect context
        let mut context = EffectContext::default();
        
        // If there are any relationships, add them to the context
        let relationships = self.relationship_tracker.get_relationships_for_resource(resource_id);
        
        // Execute the effect
        let outcome = self.effect_runtime.execute_effect(Arc::new(effect), context)
            .await
            .map_err(|e| Error::EffectError(e.to_string()))?;
        
        // Process the outcome
        if outcome.success {
            if let Some(tx_id) = outcome.data.get("transaction_id") {
                return Ok(tx_id.to_string());
            }
            
            Ok("success".to_string())
        } else {
            if let Some(error) = outcome.error {
                Err(Error::EffectError(error))
            } else {
                Err(Error::EffectError("Unknown error in effect execution".to_string()))
            }
        }
    }
    
    /// Compatibility method for storing a commitment
    pub async fn store_commitment(&self, 
        resource_id: &ContentId, 
        commitment: Commitment, 
        domain_id: &DomainId
    ) -> Result<String> {
        // Get the resource from the lifecycle manager
        let register = self.lifecycle_manager
            .get_resource(resource_id)
            .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))?;
        
        // Create the domain type from domain info
        let domain_type = DomainType::from_domain_id(domain_id);
        
        // Create a storage effect with commitment strategy
        let mut effect = StorageEffect::new_store(
            register.clone(),
            StorageStrategy::Commitment,
            domain_type,
        );
        
        // Add commitment as a parameter
        effect = effect.with_param("commitment", &commitment.to_string());
        
        // Create the effect context
        let context = EffectContext::default();
        
        // Execute the effect
        let outcome = self.effect_runtime.execute_effect(Arc::new(effect), context)
            .await
            .map_err(|e| Error::EffectError(e.to_string()))?;
        
        // Process the outcome
        if outcome.success {
            if let Some(tx_id) = outcome.data.get("transaction_id") {
                return Ok(tx_id.to_string());
            }
            
            Ok("success".to_string())
        } else {
            if let Some(error) = outcome.error {
                Err(Error::EffectError(error))
            } else {
                Err(Error::EffectError("Unknown error in effect execution".to_string()))
            }
        }
    }
    
    /// Compatibility method for storing a nullifier
    pub async fn store_nullifier(&self, 
        resource_id: &ContentId, 
        nullifier: NullifierId, 
        domain_id: &DomainId
    ) -> Result<String> {
        // Get the resource from the lifecycle manager
        let register = self.lifecycle_manager
            .get_resource(resource_id)
            .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))?;
        
        // Create the domain type from domain info
        let domain_type = DomainType::from_domain_id(domain_id);
        
        // Create a storage effect with nullifier strategy
        let mut effect = StorageEffect::new_store(
            register.clone(),
            StorageStrategy::Nullifier,
            domain_type,
        );
        
        // Add nullifier as a parameter
        effect = effect.with_param("nullifier", &nullifier.to_string());
        
        // Create the effect context
        let context = EffectContext::default();
        
        // Execute the effect
        let outcome = self.effect_runtime.execute_effect(Arc::new(effect), context)
            .await
            .map_err(|e| Error::EffectError(e.to_string()))?;
        
        // Process the outcome
        if outcome.success {
            if let Some(tx_id) = outcome.data.get("transaction_id") {
                return Ok(tx_id.to_string());
            }
            
            Ok("success".to_string())
        } else {
            if let Some(error) = outcome.error {
                Err(Error::EffectError(error))
            } else {
                Err(Error::EffectError("Unknown error in effect execution".to_string()))
            }
        }
    }
    
    /// Compatibility method for reading a resource from storage
    pub async fn read_resource(&self, 
        resource_id: &ContentId, 
        domain_id: &DomainId
    ) -> Result<ResourceRegister> {
        // Create the domain type from domain info
        let domain_type = DomainType::from_domain_id(domain_id);
        
        // Create a storage effect for reading
        let effect = StorageEffect::new_read(
            resource_id.clone(),
            StorageStrategy::OnChain,
            domain_type,
        );
        
        // Create the effect context
        let context = EffectContext::default();
        
        // Execute the effect
        let outcome = self.effect_runtime.execute_effect(Arc::new(effect), context)
            .await
            .map_err(|e| Error::EffectError(e.to_string()))?;
        
        // Process the outcome
        if outcome.success {
            // In a real implementation, we would deserialize the register from the outcome data
            // For now, we'll retrieve it from the lifecycle manager
            let register = self.lifecycle_manager
                .get_resource(resource_id)
                .ok_or_else(|| Error::NotFound(format!("Resource {} not found", resource_id)))?;
                
            Ok(register)
        } else {
            if let Some(error) = outcome.error {
                Err(Error::EffectError(error))
            } else {
                Err(Error::EffectError("Unknown error in effect execution".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::RegisterState;
    
    #[tokio::test]
    async fn test_storage_adapter() {
        // This test would verify that the compatibility layer works correctly
        // It would create instances of the new architecture components and verify
        // that the adapter correctly delegates to them
    }
} 
