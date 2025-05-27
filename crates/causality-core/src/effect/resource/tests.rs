#[cfg(test)]
mod standalone_tests {
    use std::sync::Arc;
    use std::collections::{HashMap, HashSet};
    use std::any::Any;
    use crate::effect::{
        EffectOutcome, EffectError,
        EffectContext, EffectHandler, HandlerResult,
        Effect, EffectType
    };
    use crate::effect::context::{DefaultEffectContext, Capability};
    use crate::effect::types::{EffectTypeId, EffectId, Right};
    use crate::effect::outcome::EffectResult;
    use async_trait::async_trait;
    use crate::resource::types::ResourceId;

    // Simple Effect implementation for testing
    #[derive(Debug)]
    struct TestEffect {
        effect_type: EffectType,
        name: String,
        params: HashMap<String, String>,
    }

    impl TestEffect {
        fn new(name: &str, effect_type: EffectType) -> Self {
            Self {
                effect_type,
                name: name.to_string(),
                params: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl Effect for TestEffect {
        fn effect_type(&self) -> EffectType {
            self.effect_type.clone()
        }

        fn description(&self) -> String {
            format!("{:?} on {}", self.effect_type, self.name)
        }

        async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            // Simplified implementation - just check if we have a matching capability
            let required_capability = format!("{}.{:?}", self.name, self.effect_type).to_lowercase();
            
            let capability = Capability {
                resource_id: ResourceId::from_string(&required_capability).unwrap_or_else(|_| ResourceId::new_random()),
                right: Right::Read,
            };
            
            if !context.has_capability(&capability) {
                return Err(EffectError::PermissionDenied(
                    format!("Missing capability: {}", required_capability)
                ));
            }
            
            let mut data = HashMap::new();
            data.insert("name".to_string(), self.name.clone());
            data.insert("type".to_string(), format!("{:?}", self.effect_type));
            
            Ok(EffectOutcome::success_with_data(data))
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Simple Effect handler for testing
    #[derive(Debug)]
    struct TestHandler {
        name: String,
        supported_types: Vec<EffectType>,
    }

    impl TestHandler {
        fn new(name: &str, supported_types: Vec<EffectType>) -> Self {
            Self {
                name: name.to_string(),
                supported_types,
            }
        }
    }

    // Add type_id() method to EffectType
    impl EffectType {
        fn type_id(&self) -> EffectTypeId {
            match self {
                EffectType::Read => "read".into(),
                EffectType::Write => "write".into(),
                EffectType::Create => "create".into(),
                EffectType::Delete => "delete".into(),
                EffectType::Custom(name) => name.clone().into(),
            }
        }
    }

    #[async_trait]
    impl EffectHandler for TestHandler {
        fn supported_effect_types(&self) -> Vec<EffectTypeId> {
            self.supported_types.iter().map(|t| t.type_id()).collect()
        }
        
        async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> HandlerResult<EffectOutcome> {
            // Delegate to the effect's execute method
            effect.execute(context).await
        }
    }

    // Simple Registry that doesn't depend on domain effects
    #[derive(Debug, Default)]
    struct TestRegistry {
        handlers: Vec<Arc<dyn EffectHandler>>,
    }

    impl TestRegistry {
        fn new() -> Self {
            Self { handlers: Vec::new() }
        }
        
        fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) {
            self.handlers.push(handler);
        }
        
        fn get_handler_for(&self, effect: &dyn Effect) -> Option<Arc<dyn EffectHandler>> {
            let effect_type_id = effect.effect_type().type_id();
            for handler in &self.handlers {
                if handler.supported_effect_types().contains(&effect_type_id) {
                    return Some(handler.clone());
                }
            }
            None
        }
        
        async fn execute(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            if let Some(handler) = self.get_handler_for(effect) {
                handler.handle(effect, context).await
            } else {
                Err(EffectError::HandlerNotFound(
                    format!("No handler found for effect type: {:?}", effect.effect_type())
                ))
            }
        }
    }

    // Simple context implementation that doesn't use DefaultEffectContext
    #[derive(Debug, Clone)]
    struct SimpleEffectContext {
        capabilities: Vec<Capability>,
        resources: HashSet<ResourceId>,
        metadata: HashMap<String, String>,
        effect_id: EffectId,
        parent: Option<Arc<dyn EffectContext>>,
    }

    impl SimpleEffectContext {
        fn new() -> Self {
            Self {
                capabilities: Vec::new(),
                resources: HashSet::new(),
                metadata: HashMap::new(),
                effect_id: EffectId::from("default-id".to_string()),
                parent: None,
            }
        }
        
        fn with_capability(mut self, capability: Capability) -> Self {
            self.capabilities.push(capability);
            self
        }
        
        fn with_effect_id(mut self, effect_id: EffectId) -> Self {
            self.effect_id = effect_id;
            self
        }
        
        fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.metadata.insert(key.into(), value.into());
            self
        }
    }

    impl EffectContext for SimpleEffectContext {
        fn effect_id(&self) -> &EffectId {
            &self.effect_id
        }
        
        fn capabilities(&self) -> &[Capability] {
            &self.capabilities
        }
        
        fn resources(&self) -> &HashSet<ResourceId> {
            &self.resources
        }
        
        fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
            self.parent.as_ref()
        }
        
        fn has_capability(&self, capability: &Capability) -> bool {
            self.capabilities.iter().any(|cap| cap.resource_id == capability.resource_id && cap.right == capability.right)
        }
        
        fn metadata(&self) -> &HashMap<String, String> {
            &self.metadata
        }
        
        fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
            Box::new(Self {
                effect_id,
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                metadata: self.metadata.clone(),
                parent: Some(Arc::new(self.clone())),
            })
        }
        
        fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
            let mut new_caps = self.capabilities.clone();
            new_caps.extend(capabilities);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: new_caps,
                resources: self.resources.clone(),
                metadata: self.metadata.clone(),
                parent: self.parent.clone(),
            })
        }
        
        fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
            let mut new_resources = self.resources.clone();
            new_resources.extend(resources);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: self.capabilities.clone(),
                resources: new_resources,
                metadata: self.metadata.clone(),
                parent: self.parent.clone(),
            })
        }
        
        fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
            let mut new_metadata = self.metadata.clone();
            new_metadata.extend(metadata);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                metadata: new_metadata,
                parent: self.parent.clone(),
            })
        }
        
        fn clone_context(&self) -> Box<dyn EffectContext> {
            Box::new(self.clone())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Helper function to create a read effect
    fn create_read_effect(resource_type: &str) -> TestEffect {
        TestEffect::new(resource_type, EffectType::Read)
    }

    // Helper function to create a write effect
    fn create_write_effect(resource_type: &str) -> TestEffect {
        TestEffect::new(resource_type, EffectType::Write)
    }

    // Helper function to create a context with a capability
    fn create_context_with_capability(effect_id: &str, capability_str: &str) -> SimpleEffectContext {
        let capability = Capability {
            resource_id: ResourceId::from_string(capability_str).unwrap_or_else(|_| ResourceId::new_random()),
            right: Right::Read,
        };
        
        SimpleEffectContext::new()
            .with_effect_id(EffectId::from(effect_id.to_string()))
            .with_capability(capability)
    }

    #[tokio::test]
    async fn test_registry_integration() {
        // Create a registry
        let mut registry = TestRegistry::new();
        
        // Create handlers for different effect types
        let read_handler = Arc::new(TestHandler::new("reader", vec![EffectType::Read]));
        
        // Register the handlers
        registry.register_handler(read_handler);
        
        // Create a context with required capability
        let context = create_context_with_capability("test", "user.read");
        
        // Create an effect
        let effect = create_read_effect("user");
        
        // Execute the effect
        let result = registry.execute(&effect, &context).await;
        
        // Check the result
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_effect_registry_handler_registration() {
        // Create a registry
        let mut registry = TestRegistry::new();
        
        // Create handlers for different effect types
        let handler1 = Arc::new(TestHandler::new("handler1", vec![EffectType::Read]));
        let handler2 = Arc::new(TestHandler::new("handler2", vec![EffectType::Write]));
        
        // Register the handlers
        registry.register_handler(handler1);
        registry.register_handler(handler2);
        
        // Create effects
        let read_effect = create_read_effect("user");
        let write_effect = create_write_effect("user");
        
        // Create contexts with required capabilities
        let read_context = create_context_with_capability("read_test", "user.read");
        let write_context = create_context_with_capability("write_test", "user.write");
        
        // Execute the effects
        let read_result = registry.execute(&read_effect, &read_context).await;
        let write_result = registry.execute(&write_effect, &write_context).await;
        
        // Check the results
        assert!(read_result.is_ok());
        assert!(write_result.is_ok());
    }
    
    #[test]
    fn test_simple_effect_context() {
        // Create a basic context
        let context = SimpleEffectContext::new();
        
        // Verify the default values
        assert_eq!(context.effect_id().to_string(), "default-id");
        assert!(context.capabilities().is_empty());
        assert!(context.resources().is_empty());
        assert!(context.metadata().is_empty());
        assert!(context.parent_context().is_none());
        
        // Create a capability
        let resource_id = ResourceId::from_string("test-resource").unwrap_or_else(|_| ResourceId::new_random());
        let capability = Capability {
            resource_id: resource_id.clone(),
            right: Right::Read,
        };
        
        // Create a context with the capability
        let context_with_cap = context.clone().with_capability(capability.clone());
        
        // Verify the context has the capability
        assert!(!context_with_cap.capabilities().is_empty());
        assert!(context_with_cap.has_capability(&capability));
        
        // Test deriving a context
        let derived_context = context_with_cap.derive_context(EffectId::from("derived-id".to_string()));
        
        // Verify the derived context
        assert_eq!(derived_context.effect_id().to_string(), "derived-id");
        assert!(derived_context.has_capability(&capability));
        assert!(derived_context.parent_context().is_some());
    }
} 