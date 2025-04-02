#[cfg(test)]
mod standalone_tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use crate::effect::{
        EffectOutcome, EffectError,
        EffectContext, EffectHandler, SimpleEffectContext, HandlerResult,
        Effect, EffectType
    };
    use crate::effect::types::{EffectTypeId, EffectId, Right};
    use crate::effect::outcome::EffectResult;
    use async_trait::async_trait;

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
            
            if !context.has_capability(&crate::effect::context::Capability::new(
                causality_types::ContentId::new(&required_capability),
                Right::Read
            )) {
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
                Err(EffectError::ExecutionError(
                    format!("No handler found for effect type: {:?}", effect.effect_type())
                ))
            }
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

    #[tokio::test]
    async fn test_registry_integration() {
        // Create a registry
        let mut registry = TestRegistry::new();
        
        // Create handlers for different effect types
        let read_handler = Arc::new(TestHandler::new("reader", vec![EffectType::Read]));
        
        // Register the handlers
        registry.register_handler(read_handler);
        
        // Create a context with required capability
        let context = SimpleEffectContext::new(EffectId::from("test".to_string()))
            .with_capability("user.read");
        
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
        let read_context = SimpleEffectContext::new(EffectId::from("read_test".to_string()))
            .with_capability("user.read");
        let write_context = SimpleEffectContext::new(EffectId::from("write_test".to_string()))
            .with_capability("user.write");
        
        // Execute the effects
        let read_result = registry.execute(&read_effect, &read_context).await;
        let write_result = registry.execute(&write_effect, &write_context).await;
        
        // Check the results
        assert!(read_result.is_ok());
        assert!(write_result.is_ok());
    }
} 