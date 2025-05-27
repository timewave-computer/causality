#[cfg(test)]
mod standalone_tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use std::any::Any;
    use std::fmt::Debug;
    use std::error::Error;
    use async_trait::async_trait;

    use crate::effect::{
        Effect as CoreEffect, EffectError as CoreEffectError, EffectOutcome as CoreEffectOutcome, 
        handler::{EffectHandler as CoreEffectHandler, HandlerResult},
        types::EffectTypeId,
        context::EffectContext as CoreEffectContext
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

    // TestEffectError - error type for effects
    #[derive(Debug)]
    enum TestEffectError {
        NotFound(String),
        PermissionDenied(String),
        InvalidOperation(String),
        ExecutionError(String),
    }

    impl std::fmt::Display for TestEffectError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestEffectError::NotFound(s) => write!(f, "Resource not found: {}", s),
                TestEffectError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
                TestEffectError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
                TestEffectError::ExecutionError(s) => write!(f, "Execution error: {}", s),
            }
        }
    }

    impl Error for TestEffectError {}

    // TestEffectOutcome - represents the result of an effect
    #[derive(Debug, Clone)]
    enum TestEffectOutcome {
        Success {
            result: Option<String>,
            data: HashMap<String, String>,
        },
        Failure {
            error: String,
        },
        Pending,
    }

    impl TestEffectOutcome {
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
    type TestEffectResult = Result<TestEffectOutcome, TestEffectError>;

    // Context for effect execution
    trait TestEffectContext: Debug + Send + Sync {
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

    impl TestEffectContext for SimpleEffectContext {
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

    // Base TestEffect trait
    trait TestEffect: Debug + Send + Sync {
        fn effect_type(&self) -> EffectType;
        fn description(&self) -> String;
        fn execute(&self, context: &dyn TestEffectContext) -> TestEffectResult;
        fn as_any(&self) -> &dyn Any;
    }

    // TestEffect handler trait
    trait TestEffectHandler: Debug + Send + Sync {
        fn can_handle(&self, effect_type: &EffectType) -> bool;
        fn handle(&self, effect: &dyn TestEffect, context: &dyn TestEffectContext) -> TestEffectResult;
    }

    // Simple TestEffect implementation for testing
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

    impl TestEffect for ResourceEffect {
        fn effect_type(&self) -> EffectType {
            self.effect_type.clone()
        }
        
        fn description(&self) -> String {
            format!(
                "{} resource {} of type {}",
                self.effect_type.as_str(),
                self.resource_id,
                self.resource_type
            )
        }
        
        fn execute(&self, context: &dyn TestEffectContext) -> TestEffectResult {
            if !context.has_capability(&format!("resource:{}", self.effect_type.as_str())) {
                return Err(TestEffectError::PermissionDenied(
                    format!("Missing capability for {}", self.effect_type.as_str())
                ));
            }
            
            match self.effect_type {
                EffectType::Read => {
                    // Simulate finding the resource
                    if self.resource_id == "not_found" {
                        return Err(TestEffectError::NotFound(
                            format!("Resource {} not found", self.resource_id)
                        ));
                    }
                    
                    // Return success with simulated data
                    let mut data = HashMap::new();
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("type".to_string(), self.resource_type.clone());
                    
                    Ok(TestEffectOutcome::success_with_data(data))
                },
                EffectType::Write => {
                    // Simulate writing to the resource
                    if self.resource_id == "read_only" {
                        return Err(TestEffectError::PermissionDenied(
                            "Resource is read-only".to_string()
                        ));
                    }
                    
                    // Get content from parameters
                    let content = self.parameters.get("content")
                        .cloned()
                        .unwrap_or_else(|| "Default content".to_string());
                    
                    // Return success with updated content
                    let mut data = HashMap::new();
                    data.insert("content".to_string(), content);
                    data.insert("updated".to_string(), "true".to_string());
                    
                    Ok(TestEffectOutcome::success_with_data(data))
                },
                EffectType::Create => {
                    // Simulate creating a new resource
                    let mut data = HashMap::new();
                    data.insert("id".to_string(), self.resource_id.clone());
                    data.insert("type".to_string(), self.resource_type.clone());
                    data.insert("created".to_string(), "true".to_string());
                    
                    Ok(TestEffectOutcome::success_with_data(data))
                },
                EffectType::Delete => {
                    // Simulate deleting a resource
                    if self.resource_id == "permanent" {
                        return Err(TestEffectError::InvalidOperation(
                            "Cannot delete permanent resource".to_string()
                        ));
                    }
                    
                    Ok(TestEffectOutcome::success())
                },
                EffectType::Custom(ref op) => {
                    // Handle custom operations
                    if op == "error" {
                        return Err(TestEffectError::ExecutionError(
                            "Custom operation failed".to_string()
                        ));
                    }
                    
                    Ok(TestEffectOutcome::success_with_result(
                        format!("Custom operation '{}' executed", op)
                    ))
                }
            }
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Resource effect handler
    #[derive(Debug)]
    struct ResourceEffectHandler {
        supported_types: Vec<EffectType>,
    }

    impl ResourceEffectHandler {
        fn new(supported_types: Vec<EffectType>) -> Self {
            Self { supported_types }
        }
    }

    impl TestEffectHandler for ResourceEffectHandler {
        fn can_handle(&self, effect_type: &EffectType) -> bool {
            self.supported_types.contains(effect_type)
        }
        
        fn handle(&self, effect: &dyn TestEffect, context: &dyn TestEffectContext) -> TestEffectResult {
            effect.execute(context)
        }
    }

    // Effect executor
    struct EffectExecutor {
        handlers: Vec<Arc<dyn TestEffectHandler>>,
    }

    impl EffectExecutor {
        fn new() -> Self {
            Self { handlers: Vec::new() }
        }
        
        fn register_handler(&mut self, handler: Arc<dyn TestEffectHandler>) {
            self.handlers.push(handler);
        }
        
        fn find_handler(&self, effect: &dyn TestEffect) -> Option<Arc<dyn TestEffectHandler>> {
            let effect_type = effect.effect_type();
            
            for handler in &self.handlers {
                if handler.can_handle(&effect_type) {
                    return Some(Arc::clone(handler));
                }
            }
            
            None
        }
        
        fn execute(&self, effect: &dyn TestEffect, context: &dyn TestEffectContext) -> TestEffectResult {
            if let Some(handler) = self.find_handler(effect) {
                handler.handle(effect, context)
            } else {
                Err(TestEffectError::InvalidOperation(
                    format!("No handler found for effect type: {:?}", effect.effect_type())
                ))
            }
        }
    }

    // Helper functions
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
        // Create test context
        let context = SimpleEffectContext::new()
            .with_capability("resource:read")
            .with_capability("resource:write")
            .with_capability("resource:create");
        
        // Create effect handlers
        let read_handler = Arc::new(ResourceEffectHandler::new(vec![
            EffectType::Read
        ]));
        
        let write_handler = Arc::new(ResourceEffectHandler::new(vec![
            EffectType::Write,
            EffectType::Create
        ]));
        
        // Create executor and register handlers
        let mut executor = EffectExecutor::new();
        executor.register_handler(read_handler);
        executor.register_handler(write_handler);
        
        // Execute read effect
        let read_effect = create_read_effect("document", "doc123");
        let result = executor.execute(&read_effect, &context);
        assert!(result.is_ok());
        
        // Execute write effect
        let write_effect = create_write_effect("document", "doc123")
            .with_parameter("content", "Updated content");
        let result = executor.execute(&write_effect, &context);
        assert!(result.is_ok());
        
        // Execute create effect
        let create_effect = create_create_effect("document", "new_doc");
        let result = executor.execute(&create_effect, &context);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_effect_registry_handler_registration() {
        let mut executor = EffectExecutor::new();
        
        // Create and register handlers for read and write
        let read_handler = Arc::new(ResourceEffectHandler::new(vec![EffectType::Read]));
        executor.register_handler(read_handler);
        
        // Create a read effect
        let read_effect = create_read_effect("document", "doc456");
        
        // We should find a handler for read effects
        assert!(executor.find_handler(&read_effect).is_some());
        
        // Create a write effect
        let write_effect = create_write_effect("document", "doc456");
        
        // We should not find a handler for write effects
        assert!(executor.find_handler(&write_effect).is_none());
        
        // Now register a write handler
        let write_handler = Arc::new(ResourceEffectHandler::new(vec![EffectType::Write]));
        executor.register_handler(write_handler);
        
        // Now we should find handlers for both read and write
        assert!(executor.find_handler(&read_effect).is_some());
        assert!(executor.find_handler(&write_effect).is_some());
    }
} 