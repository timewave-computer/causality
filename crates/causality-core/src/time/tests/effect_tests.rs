#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::Utc;
    
    use crate::crypto::KeyPair;
    use crate::effect::{
        EffectBuilder, EffectContext, Effect, EffectRegistry,
        registry::EffectRegistryBuilder,
    };
    use crate::time::{
        effect::{CausalTimeEffect, ClockTimeEffect, TemporalQueryEffect},
        implementations::{
            MemoryTimeService, MemoryTimeAttestationStore, MemoryFactTimeStore
        },
        integration::{TimeEffectIntegration, TimeEffectBuilderExt},
    };
    use crate::types::{DomainId, FactId};

    #[tokio::test]
    async fn test_causal_time_effect() {
        // Create a time effect integration
        let key_pair = KeyPair::generate().unwrap();
        let integration = TimeEffectIntegration::new_memory_based(key_pair);
        
        // Create the effect registry
        let mut registry_builder = EffectRegistryBuilder::new();
        integration.register_handlers(&mut registry_builder);
        let registry = registry_builder.build();
        
        // Create a domain ID and some facts
        let domain_id = DomainId::new("test_domain");
        let fact1 = FactId::new("fact1");
        let fact2 = FactId::new("fact2");
        
        // Create a causal time effect
        let causal_effect = integration.create_causal_time_effect(
            domain_id.clone(),
            vec![fact1.clone()],
        ).await.unwrap();
        
        // Create a context
        let context = EffectContext::new();
        
        // Execute the effect
        let outcome = registry.execute(&causal_effect, &context).await.unwrap();
        
        // Check that the outcome is successful
        assert!(outcome.is_success());
    }

    #[tokio::test]
    async fn test_clock_time_effect() {
        // Create a time effect integration
        let key_pair = KeyPair::generate().unwrap();
        let integration = TimeEffectIntegration::new_memory_based(key_pair);
        
        // Create the effect registry
        let mut registry_builder = EffectRegistryBuilder::new();
        integration.register_handlers(&mut registry_builder);
        let registry = registry_builder.build();
        
        // Create a domain ID
        let domain_id = DomainId::new("test_domain");
        
        // Create a clock time effect
        let clock_effect = integration.create_clock_time_effect(
            domain_id.clone(),
        ).await.unwrap();
        
        // Create a context
        let context = EffectContext::new();
        
        // Execute the effect
        let outcome = registry.execute(&clock_effect, &context).await.unwrap();
        
        // Check that the outcome is successful
        assert!(outcome.is_success());
    }

    #[tokio::test]
    async fn test_builder_extensions() {
        // Create a time effect integration
        let key_pair = KeyPair::generate().unwrap();
        let integration = TimeEffectIntegration::new_memory_based(key_pair);
        
        // Create the effect registry
        let mut registry_builder = EffectRegistryBuilder::new();
        integration.register_handlers(&mut registry_builder);
        let registry = registry_builder.build();
        
        // Create a domain ID and some facts
        let domain_id = DomainId::new("test_domain");
        let fact1 = FactId::new("fact1");
        let fact2 = FactId::new("fact2");
        
        // Create effects
        let causal_effect = integration.create_causal_time_effect(
            domain_id.clone(),
            vec![fact1.clone()],
        ).await.unwrap();
        
        let clock_effect = integration.create_clock_time_effect(
            domain_id.clone(),
        ).await.unwrap();
        
        // Create a composite effect using the builder extensions
        let composite_effect = EffectBuilder::new("composite_effect")
            .add_causal_time_effect(causal_effect)
            .add_clock_time_effect(clock_effect)
            .build();
        
        // Create a context
        let context = EffectContext::new();
        
        // Execute the composite effect
        let outcome = registry.execute(&composite_effect, &context).await.unwrap();
        
        // Check that the outcome is successful
        assert!(outcome.is_success());
    }
} 