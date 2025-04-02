#[cfg(test)]
mod standalone_tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use std::any::Any;
    use std::fmt::Debug;
    use std::error::Error;
    use async_trait::async_trait;

    use crate::effect::{
        Effect, EffectError, EffectOutcome, 
        handler::{EffectHandler, HandlerResult},
        types::EffectTypeId,
        context::EffectContext
    };

    // EffectType - simple enum for test purposes
    #[derive(Debug, Clone, PartialEq)]
    enum EffectType {
        Read,
        Write,
        Create,
        Delete,
        Custom(String),
    }

    impl EffectType {
        fn as_str(&self) -> &str {
            match self {
                EffectType::Read => "read",
                EffectType::Write => "write",
                EffectType::Create => "create",
                EffectType::Delete => "delete",
                EffectType::Custom(s) => s.as_str(),
            }
        }
    }

    // EffectError - error type for effects
    #[derive(Debug)]
    enum EffectError {
        NotFound(String),
        PermissionDenied(String),
        InvalidOperation(String),
        ExecutionError(String),
    }

    impl std::fmt::Display for EffectError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                EffectError::NotFound(s) => write!(f, "Resource not found: {}", s),
                EffectError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
                EffectError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
                EffectError::ExecutionError(s) => write!(f, "Execution error: {}", s),
            }
        }
    }

    impl Error for EffectError {}

    // EffectOutcome - represents the result of an effect
    #[derive(Debug, Clone)]
    enum EffectOutcome {
        Success {
            result: Option<String>,
            data: HashMap<String, String>,
        },
        Failure {
            error: String,
        },
        Pending,
    }

    impl EffectOutcome {
        fn success() -> Self {
            Self::Success {
                result: None,
                data: HashMap::new(),
            }
        }
        
        fn success_with_data(data: HashMap<String, String>) -> Self {
            Self::Success {
                result: None,
                data,
            }
        }
        
        fn success_with_result(result: impl Into<String>) -> Self {
            Self::Success {
                result: Some(result.into()),
                data: HashMap::new(),
            }
        }
        
        fn failure(error: impl Into<String>) -> Self {
            Self::Failure {
                error: error.into(),
            }
        }
        
        fn pending() -> Self {
            Self::Pending
        }
    }

    // Result type for effects
    type EffectResult = Result<EffectOutcome, EffectError>;

    // Context for effect execution
    trait EffectContext: Debug + Send + Sync {
        fn has_capability(&self, capability: &str) -> bool;
        fn get_data(&self, key: &str) -> Option<&String>;
        fn metadata(&self) -> &HashMap<String, String>;
    }

    // Simple context implementation
    #[derive(Debug, Default)]
    struct SimpleEffectContext {
        capabilities: Vec<String>,
        data: HashMap<String, String>,
        metadata: HashMap<String, String>,
    }

    impl SimpleEffectContext {
        fn new() -> Self {
            Self::default()
        }
        
        fn with_capability(mut self, capability: impl Into<String>) -> Self {
            self.capabilities.push(capability.into());
            self
        }
        
        fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.data.insert(key.into(), value.into());
            self
        }
        
        fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.metadata.insert(key.into(), value.into());
            self
        }
    }

    impl EffectContext for SimpleEffectContext {
        fn has_capability(&self, capability: &str) -> bool {
            self.capabilities.contains(&capability.to_string())
        }
        
        fn get_data(&self, key: &str) -> Option<&String> {
            self.data.get(key)
        }
        
        fn metadata(&self) -> &HashMap<String, String> {
            &self.metadata
        }
    }

    // Base Effect trait
    trait Effect: Debug + Send + Sync {
        fn effect_type(&self) -> EffectType;
        fn description(&self) -> String;
        fn execute(&self, context: &dyn EffectContext) -> EffectResult;
        fn as_any(&self) -> &dyn Any;
    }

    // Effect handler trait
    trait EffectHandler: Debug + Send + Sync {
        fn can_handle(&self, effect_type: &EffectType) -> bool;
        fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult;
    }

    // Simple Effect implementation for testing
    #[derive(Debug)]
    struct ResourceEffect {
        resource_type: String,
        resource_id: String,
        effect_type: EffectType,
        parameters: HashMap<String, String>,
    }

    impl ResourceEffect {
        fn new(resource_type: &str, resource_id: &str, effect_type: EffectType) -> Self {
            Self {
                resource_type: resource_type.to_string(),
                resource_id: resource_id.to_string(),
                effect_type,
                parameters: HashMap::new(),
            }
        }
        
        fn with_parameter(mut self, key: &str, value: &str) -> Self {
            self.parameters.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl Effect for ResourceEffect {
        fn effect_type(&self) -> EffectType {
            self.effect_type.clone()
        }

        fn description(&self) -> String {
            format!("{} {}:{}", 
                self.effect_type.as_str(),
                self.resource_type,
                self.resource_id
            )
        }

        fn execute(&self, context: &dyn EffectContext) -> EffectResult {
            // Check capability
            let capability = format!("{}.{}", self.resource_type, self.effect_type.as_str());
            if !context.has_capability(&capability) {
                return Err(EffectError::PermissionDenied(
                    format!("Missing capability: {}", capability)
                ));
            }
            
            // Execute based on operation type
            match self.effect_type {
                EffectType::Read => {
                    // Simulate reading data
                    let mut data = HashMap::new();
                    data.insert("type".to_string(), self.resource_type.clone());
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("content".to_string(), "Example content".to_string());
                    
                    Ok(EffectOutcome::Success {
                        result: Some("Read successful".to_string()),
                        data,
                    })
                },
                EffectType::Write => {
                    // Simulate writing data
                    let mut data = HashMap::new();
                    data.insert("type".to_string(), self.resource_type.clone());
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("content".to_string(), "Updated content".to_string());
                    
                    Ok(EffectOutcome::Success {
                        result: Some("Write successful".to_string()),
                        data,
                    })
                },
                EffectType::Create => {
                    // Simulate creating a resource
                    let mut data = HashMap::new();
                    data.insert("type".to_string(), self.resource_type.clone());
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("created".to_string(), "true".to_string());
                    
                    Ok(EffectOutcome::Success {
                        result: Some("Resource created".to_string()),
                        data,
                    })
                },
                EffectType::Delete => {
                    // Simulate deleting a resource
                    let mut data = HashMap::new();
                    data.insert("type".to_string(), self.resource_type.clone());
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("deleted".to_string(), "true".to_string());
                    
                    Ok(EffectOutcome::Success {
                        result: Some("Resource deleted".to_string()),
                        data,
                    })
                },
                EffectType::Custom(ref operation) => {
                    // Handle custom operations
                    Err(EffectError::InvalidOperation(format!(
                        "Custom operation not supported: {}", operation
                    )))
                }
            }
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Simple Effect handler implementation
    #[derive(Debug)]
    struct ResourceEffectHandler {
        supported_types: Vec<EffectType>,
    }

    impl ResourceEffectHandler {
        fn new(supported_types: Vec<EffectType>) -> Self {
            Self { supported_types }
        }
    }

    impl EffectHandler for ResourceEffectHandler {
        fn can_handle(&self, effect_type: &EffectType) -> bool {
            self.supported_types.contains(effect_type)
        }
        
        fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult {
            effect.execute(context)
        }
    }

    // Effect executor implementation
    #[derive(Debug, Default)]
    struct EffectExecutor {
        handlers: Vec<Arc<dyn EffectHandler>>,
    }

    impl EffectExecutor {
        fn new() -> Self {
            Self { handlers: Vec::new() }
        }
        
        fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) {
            self.handlers.push(handler);
        }
        
        fn find_handler(&self, effect: &dyn Effect) -> Option<Arc<dyn EffectHandler>> {
            let effect_type = effect.effect_type();
            for handler in &self.handlers {
                if handler.can_handle(&effect_type) {
                    return Some(handler.clone());
                }
            }
            None
        }
        
        fn execute(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult {
            if let Some(handler) = self.find_handler(effect) {
                handler.handle(effect, context)
            } else {
                Err(EffectError::ExecutionError(format!(
                    "No handler found for effect: {}", effect.description()
                )))
            }
        }
    }

    // Helper functions to create effects
    fn create_read_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
        ResourceEffect::new(resource_type, resource_id, EffectType::Read)
    }

    fn create_write_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
        ResourceEffect::new(resource_type, resource_id, EffectType::Write)
    }

    fn create_create_effect(resource_type: &str, resource_id: &str) -> ResourceEffect {
        ResourceEffect::new(resource_type, resource_id, EffectType::Create)
    }

    #[test]
    fn test_registry_integration() {
        // Create an executor
        let mut executor = EffectExecutor::new();
        
        // Create and register a handler
        let handler = Arc::new(ResourceEffectHandler::new(vec![
            EffectType::Read, 
            EffectType::Write, 
            EffectType::Create
        ]));
        executor.register_handler(handler);
        
        // Create a context with required capability
        let context = SimpleEffectContext::new()
            .with_capability("document.read")
            .with_capability("document.write");
        
        // Create an effect
        let read_effect = create_read_effect("document", "doc-123");
        let write_effect = create_write_effect("document", "doc-123");
        
        // Execute the effects
        let read_result = executor.execute(&read_effect, &context);
        let write_result = executor.execute(&write_effect, &context);
        
        // Check the results
        assert!(read_result.is_ok());
        assert!(write_result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = read_result {
            assert_eq!(result, Some("Read successful".to_string()));
            assert_eq!(data.get("type"), Some(&"document".to_string()));
            assert_eq!(data.get("id"), Some(&"doc-123".to_string()));
        }
    }
    
    #[test]
    fn test_effect_registry_handler_registration() {
        // Create an executor
        let mut executor = EffectExecutor::new();
        
        // Create handlers for different effect types
        let read_handler = Arc::new(ResourceEffectHandler::new(vec![EffectType::Read]));
        let write_handler = Arc::new(ResourceEffectHandler::new(vec![EffectType::Write]));
        let create_handler = Arc::new(ResourceEffectHandler::new(vec![EffectType::Create]));
        
        // Register the handlers
        executor.register_handler(read_handler);
        executor.register_handler(write_handler);
        executor.register_handler(create_handler);
        
        // Create effects
        let read_effect = create_read_effect("document", "doc-123");
        let write_effect = create_write_effect("document", "doc-123");
        let create_effect = create_create_effect("document", "doc-123");
        
        // Create contexts with required capabilities
        let read_context = SimpleEffectContext::new()
            .with_capability("document.read");
        let write_context = SimpleEffectContext::new()
            .with_capability("document.write");
        let create_context = SimpleEffectContext::new()
            .with_capability("document.create");
        
        // Execute the effects
        let read_result = executor.execute(&read_effect, &read_context);
        let write_result = executor.execute(&write_effect, &write_context);
        let create_result = executor.execute(&create_effect, &create_context);
        
        // Check the results
        assert!(read_result.is_ok());
        assert!(write_result.is_ok());
        assert!(create_result.is_ok());
        
        // Verify that missing capabilities cause errors
        let no_capability_context = SimpleEffectContext::new();
        let error_result = executor.execute(&read_effect, &no_capability_context);
        assert!(error_result.is_err());
        if let Err(EffectError::PermissionDenied(message)) = error_result {
            assert!(message.contains("Missing capability: document.read"));
        } else {
            panic!("Expected PermissionDenied error");
        }
    }
} 