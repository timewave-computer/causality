#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use crate::capability::BasicCapability;
    use crate::content::ContentId;
    use crate::effect::{
        Effect, EffectContext, EffectOutcome,
        context::BasicEffectContext,
        domain::{
            DomainId, DomainEffect, BasicDomainEffect, DomainEffectFactory,
        },
    };
    use crate::resource::{
        ResourceTypeId, CrossDomainResourceId, ResourceProjectionType,
        VerificationLevel, ResourceReference, ResourceTransferOperation,
    };
    
    use super::adapter::{
        TestDomainAdapterFactory, TestDomainEffectHandler, TestDomainResourceAdapter,
        create_test_domain_integration_layer,
    };
    
    use super::domain::{
        DomainEffectRouter, DomainResourceRouter, GenericDomainAdapter,
    };
    
    use crate::resource::protocol::create_cross_domain_protocol;
    
    #[tokio::test]
    async fn test_domain_integration_layer() {
        // Create a test domain integration layer
        let resource_type_registry = Arc::new(crate::resource::InMemoryResourceTypeRegistry::new());
        let cross_domain_protocol = create_cross_domain_protocol(resource_type_registry);
        
        let (effect_router, resource_router, adapter_factory) = 
            create_test_domain_integration_layer(cross_domain_protocol);
        
        // Verify that test domains are registered
        let domains = adapter_factory.supported_domains();
        assert_eq!(domains.len(), 4);
        
        // Test getting handlers for domains
        let test_domain = DomainId::new("test");
        let handler = effect_router.get_handler(&test_domain).await.unwrap();
        assert_eq!(handler.domain_id().to_string(), "test");
        
        // Test getting adapters for domains
        let adapter = resource_router.get_adapter(&test_domain).await.unwrap();
        assert_eq!(adapter.domain_id().to_string(), "test");
        
        // Create a basic effect context
        let context = BasicEffectContext::builder()
            .with_capability(BasicCapability::new("resource.create"))
            .with_capability(BasicCapability::new("resource.read"))
            .build();
        
        // Create a test domain effect
        let effect = DomainEffectFactory::create_domain_effect(
            test_domain.clone(),
            "test.effect",
        )
        .with_parameter("action", "create")
        .with_parameter("resource_id", "test-resource-1");
        
        // Route the effect to the appropriate handler
        let outcome = effect_router.route_effect(&effect, &context).await.unwrap();
        
        // Verify outcome
        assert!(outcome.is_success());
        
        if let EffectOutcome::Success(data) = outcome {
            assert_eq!(data.get("status"), Some(&"created".to_string()));
            assert_eq!(data.get("resource_id"), Some(&"test-resource-1".to_string()));
        } else {
            panic!("Expected successful outcome");
        }
    }
    
    #[tokio::test]
    async fn test_domain_validation() {
        // Create a test domain adapter
        let domain_id = DomainId::new("test");
        let handler = TestDomainEffectHandler::new(domain_id.clone());
        
        // Create a context with capabilities
        let context = BasicEffectContext::builder()
            .with_capability(BasicCapability::new("resource.create"))
            .build();
        
        // Create a valid effect
        let valid_effect = DomainEffectFactory::create_domain_effect(
            domain_id.clone(),
            "test.effect",
        )
        .with_parameter("action", "create")
        .with_parameter("resource_id", "test-resource-1");
        
        // Create an invalid effect (missing resource_id)
        let invalid_effect = DomainEffectFactory::create_domain_effect(
            domain_id.clone(),
            "test.effect",
        )
        .with_parameter("action", "create");
        
        // Handle the valid effect
        let outcome = handler.handle_domain_effect(&valid_effect, &context).await;
        assert!(outcome.is_ok());
        
        // Handle the invalid effect
        let outcome = handler.handle_domain_effect(&invalid_effect, &context).await;
        assert!(outcome.is_err());
        
        // Create an effect with invalid action
        let invalid_action_effect = DomainEffectFactory::create_domain_effect(
            domain_id.clone(),
            "test.effect",
        )
        .with_parameter("action", "invalid_action")
        .with_parameter("resource_id", "test-resource-1");
        
        // Handle the effect with invalid action
        let outcome = handler.handle_domain_effect(&invalid_action_effect, &context).await;
        assert!(outcome.is_err());
    }
    
    #[tokio::test]
    async fn test_capability_validation() {
        // Create a test domain adapter
        let domain_id = DomainId::new("test");
        let handler = TestDomainEffectHandler::new(domain_id.clone());
        
        // Create a context with read capability only
        let read_context = BasicEffectContext::builder()
            .with_capability(BasicCapability::new("resource.read"))
            .build();
        
        // Create a context with create capability only
        let create_context = BasicEffectContext::builder()
            .with_capability(BasicCapability::new("resource.create"))
            .build();
        
        // Create a read effect
        let read_effect = DomainEffectFactory::create_domain_effect(
            domain_id.clone(),
            "test.effect",
        )
        .with_parameter("action", "read")
        .with_parameter("resource_id", "test-resource-1");
        
        // Create a create effect
        let create_effect = DomainEffectFactory::create_domain_effect(
            domain_id.clone(),
            "test.effect",
        )
        .with_parameter("action", "create")
        .with_parameter("resource_id", "test-resource-1");
        
        // Handle the read effect with read context
        let outcome = handler.handle_domain_effect(&read_effect, &read_context).await;
        assert!(outcome.is_ok());
        
        // Handle the create effect with read context
        let outcome = handler.handle_domain_effect(&create_effect, &read_context).await;
        assert!(outcome.is_err());
        
        // Handle the create effect with create context
        let outcome = handler.handle_domain_effect(&create_effect, &create_context).await;
        assert!(outcome.is_ok());
        
        // Handle the read effect with create context
        let outcome = handler.handle_domain_effect(&read_effect, &create_context).await;
        assert!(outcome.is_err());
    }
    
    #[tokio::test]
    async fn test_domain_resource_adapter() {
        // Create a test domain resource adapter
        let domain_id = DomainId::new("test");
        let adapter = TestDomainResourceAdapter::new(domain_id.clone());
        
        // Create a context
        let context = BasicEffectContext::builder().build();
        
        // Create a resource reference
        let content_id = ContentId::from_bytes(&[1, 2, 3, 4]).unwrap();
        let resource_type = ResourceTypeId::new("document");
        let resource_id = CrossDomainResourceId::new(
            content_id.clone(),
            domain_id.clone(),
            resource_type.clone(),
        );
        
        // Check if resource exists (should not)
        let exists = adapter.has_resource(&resource_id, &context).await.unwrap();
        assert!(!exists);
        
        // Create a reference
        let reference = adapter.create_reference(
            resource_id.clone(),
            ResourceProjectionType::Shadow,
            VerificationLevel::Hash,
            None,
            &context,
        ).await.unwrap();
        
        // Check properties
        assert_eq!(reference.id, resource_id);
        assert_eq!(reference.projection_type, ResourceProjectionType::Shadow);
        assert_eq!(reference.verification_level, VerificationLevel::Hash);
        assert_eq!(reference.target_domain, domain_id);
        
        // Check if resource exists now (should)
        let exists = adapter.has_resource(&resource_id, &context).await.unwrap();
        assert!(exists);
        
        // Get the reference
        let retrieved = adapter.get_reference(&resource_id, &context).await.unwrap();
        assert!(retrieved.is_some());
        
        // Verify the reference
        let verified = adapter.verify_reference(&reference, &context).await.unwrap();
        assert!(verified);
    }
} 