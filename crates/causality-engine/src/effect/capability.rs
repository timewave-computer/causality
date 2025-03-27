//! Capability verification system
//!
//! This module provides the implementation for verifying capabilities
//! required by effects.

use std::fmt;
use std::sync::Arc;

use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::context::Context;
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::error::{EffectError, EffectResult};
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::Effect;
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::id::CapabilityId;

// Re-export from core capability system
use causality_core::capability::effect::{
    EffectCapability,
    EffectCapabilityType,
    EffectCapabilityRegistry,
    EffectCapabilityError,
};

// Re-export from effects
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::capability::{
    RequiresCapabilities,
    EffectCapabilityVerifier as EffectsCapabilityVerifier,
    extensions::convert_requirements_to_capability_types,
};

/// Manager for handling effect capabilities
///
/// This component is responsible for verifying that the context
/// has the required capabilities for an effect.
#[derive(Default)]
pub struct CapabilityManager {
    /// Core effect capability registry
    registry: Option<Arc<EffectCapabilityRegistry>>,
    
    /// Additional capability verifiers that can be registered
    verifiers: Vec<Arc<dyn CapabilityVerifier>>,
}

/// Trait for components that can verify capabilities
#[async_trait::async_trait]
pub trait CapabilityVerifier: Send + Sync {
    /// Verify that a capability is present and valid
    async fn verify_capability(
        &self,
        capability_id: &CapabilityId,
        context: &Context,
    ) -> EffectResult<()>;
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            registry: None,
            verifiers: Vec::new(),
        }
    }
    
    /// Set the core effect capability registry
    pub fn with_registry(mut self, registry: Arc<EffectCapabilityRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }
    
    /// Register a capability verifier
    pub fn register_verifier(&mut self, verifier: Arc<dyn CapabilityVerifier>) {
        self.verifiers.push(verifier);
    }
    
    /// Verify that the context has the required capabilities for an effect
    pub async fn verify_capabilities<E: Effect + RequiresCapabilities>(
        &self,
        effect: &E,
        context: &Context,
    ) -> EffectResult<()> {
        // Get the capability IDs required by this effect
        let required_capabilities = effect.required_capabilities();
        
        // Verify each required capability
        for (resource_id, rights) in &required_capabilities {
            for right in rights {
                // If we have a registry, verify with it first
                if let Some(registry) = &self.registry {
                    let capability_type = :EffectRuntime:causality_core::effect::runtime::EffectRuntime::capability::extensions::map_right_to_capability_type(right);
                    
                    // Create an identity from the context (simplified here)
                    let identity = causality_core::capability::IdentityId::new_with_name("context_identity");
                    
                    // Check if the identity has the capability in the registry
                    if let Ok(has_capability) = registry.has_required_capabilities(
                        &identity,
                        effect.type_id().to_string().as_str()
                    ) {
                        if !has_capability {
                            return Err(EffectError::MissingCapability {
                                effect_type: effect.type_id(),
                                capability: format!("{:?} for {:?}", right, resource_id),
                            });
                        }
                    }
                }
                
                // Then check the context
                if !context.has_capability(&CapabilityId::from(resource_id.clone())) {
                    return Err(EffectError::MissingCapability {
                        effect_type: effect.type_id(),
                        capability: format!("{:?} for {:?}", right, resource_id),
                    });
                }
            }
        }
        
        // Finally, run all registered verifiers
        for capability_id in get_capability_ids(effect) {
            for verifier in &self.verifiers {
                verifier.verify_capability(&capability_id, context).await?;
            }
        }
        
        Ok(())
    }
}

/// Get the capability IDs required by an effect
///
/// This function extracts CapabilityId instances from an effect.
fn get_capability_ids<E: Effect>(effect: &E) -> Vec<CapabilityId> {
    // For now, we just return a basic extraction
    // In a real implementation, this would use the RequiresCapabilities trait
    // and convert all resource IDs to capability IDs
    vec![CapabilityId::from(effect.type_id().to_string())]
}

impl fmt::Debug for CapabilityManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityManager")
            .field("has_registry", &self.registry.is_some())
            .field("verifier_count", &self.verifiers.len())
            .finish()
    }
}

/// Implementation of the CapabilityVerifier interface from causality-effects
#[async_trait::async_trait]
impl EffectsCapabilityVerifier for CapabilityManager {
    async fn verify_requirements<T: RequiresCapabilities + Send + Sync>(
        &self,
        object: &T,
        context: &Context,
    ) -> EffectResult<()> {
        // Convert requirements to capability types
        let requirements = object.required_capabilities();
        
        // Check if any requirements exist
        if requirements.is_empty() {
            return Ok(());
        }
        
        // Verify capabilities in the context
        for (resource_id, rights) in &requirements {
            for right in rights {
                let capability_id = CapabilityId::from(resource_id.clone());
                
                if !context.has_capability(&capability_id) {
                    return Err(EffectError::MissingCapability {
                        effect_type: Default::default(), // This is a limitation
                        capability: format!("{:?} for {:?}", right, resource_id),
                    });
                }
                
                // Run verifiers on this capability
                for verifier in &self.verifiers {
                    verifier.verify_capability(&capability_id, context).await?;
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::context::Context;
    use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::id::{CapabilityId, EffectTypeId};
    use async_trait::async_trait;
    
    // A simple test effect that requires a capability
    #[derive(Debug)]
    struct TestEffect;
    
    #[async_trait]
    impl Effect for TestEffect {
        type Param = ();
        type Outcome = ();
        
        fn type_id(&self) -> EffectTypeId {
            EffectTypeId::new("test.effect")
        }
        
        async fn execute(
            &self,
            _param: Self::Param,
            _context: &Context,
        ) -> Result<Self::Outcome, EffectError> {
            Ok(())
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    // A test capability verifier
    struct TestVerifier;
    
    #[async_trait::async_trait]
    impl CapabilityVerifier for TestVerifier {
        async fn verify_capability(
            &self,
            capability_id: &CapabilityId,
            _context: &Context,
        ) -> EffectResult<()> {
            // Just check if this is a test capability
            if capability_id.to_string() == "test_capability" {
                return Ok(());
            }
            
            Err(EffectError::InvalidCapability(format!(
                "Test verifier cannot verify: {}",
                capability_id
            )))
        }
    }
    
    #[tokio::test]
    async fn test_capability_verification() {
        // Create a test context with a capability
        let mut context = Context::new();
        context.add_capability(CapabilityId::from("test_capability"));
        
        // Create a capability manager
        let mut manager = CapabilityManager::new();
        manager.register_verifier(Arc::new(TestVerifier));
        
        // Create a test effect
        let effect = TestEffect;
        
        // Verify capabilities
        let result = manager.verify_capabilities(&effect, &context).await;
        
        // Should succeed because we have the required capability
        assert!(result.is_ok());
    }
} 