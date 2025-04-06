use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::any::Any;
use std::error::Error;

/// A simple effect type
#[derive(Debug, Clone)]
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

/// Simple effect error
#[derive(Debug)]
enum EffectError {
    NotFound(String),
    PermissionDenied(String),
    InvalidOperation(String),
    Other(String),
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::NotFound(s) => write!(f, "Resource not found: {}", s),
            EffectError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            EffectError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
            EffectError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl Error for EffectError {}

/// Simple effect outcome
#[derive(Debug)]
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

/// Type for effect results
type EffectResult<T = EffectOutcome> = Result<T, EffectError>;

/// Trait for effects
trait Effect: Debug + Send + Sync {
    fn effect_type(&self) -> EffectType;
    fn description(&self) -> String;
    fn execute(&self, context: &dyn EffectContext) -> EffectResult;
    fn as_any(&self) -> &dyn Any;
}

/// Context for effect execution
trait EffectContext: Debug + Send + Sync {
    fn has_capability(&self, capability: &str) -> bool;
    fn get_data(&self, key: &str) -> Option<&String>;
    fn metadata(&self) -> &HashMap<String, String>;
}

/// Basic implementation of a context
#[derive(Debug)]
struct BasicEffectContext {
    capabilities: Vec<String>,
    data: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

impl BasicEffectContext {
    fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }
    
    fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }
    
    fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl EffectContext for BasicEffectContext {
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

/// Simple resource operation
#[derive(Debug, Clone)]
struct ResourceOperation {
    resource_type: String,
    resource_id: String,
    operation: EffectType,
    parameters: HashMap<String, String>,
}

/// Resource effect implementation
#[derive(Debug)]
struct ResourceEffect {
    operation: ResourceOperation,
}

impl ResourceEffect {
    fn new(
        resource_type: &str,
        resource_id: &str,
        operation: EffectType,
    ) -> Self {
        let operation = ResourceOperation {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            operation,
            parameters: HashMap::new(),
        };
        
        Self { operation }
    }
    
    fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.operation.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

impl Effect for ResourceEffect {
    fn effect_type(&self) -> EffectType {
        self.operation.operation.clone()
    }
    
    fn description(&self) -> String {
        format!(
            "{} {}:{}", 
            self.operation.operation.as_str(),
            self.operation.resource_type,
            self.operation.resource_id,
        )
    }
    
    fn execute(&self, context: &dyn EffectContext) -> EffectResult {
        // Check capability
        let capability = format!("{}.{}", self.operation.resource_type, self.operation.operation.as_str());
        if !context.has_capability(&capability) {
            return Err(EffectError::PermissionDenied(format!(
                "Missing capability: {}", capability
            )));
        }
        
        // Execute based on operation type
        match self.operation.operation {
            EffectType::Read => {
                // Simulate reading data
                let mut data = HashMap::new();
                data.insert("type".to_string(), self.operation.resource_type.clone());
                data.insert("id".to_string(), self.operation.resource_id.clone());
                data.insert("content".to_string(), "Example content".to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some("Read successful".to_string()),
                    data,
                })
            },
            EffectType::Write => {
                // Simulate writing data
                let content = self.operation.parameters.get("content")
                    .cloned()
                    .unwrap_or_else(|| "Default content".to_string());
                
                let mut data = HashMap::new();
                data.insert("type".to_string(), self.operation.resource_type.clone());
                data.insert("id".to_string(), self.operation.resource_id.clone());
                data.insert("content".to_string(), content);
                
                Ok(EffectOutcome::Success {
                    result: Some("Write successful".to_string()),
                    data,
                })
            },
            EffectType::Create => {
                // Simulate creating resource
                let mut data = HashMap::new();
                data.insert("type".to_string(), self.operation.resource_type.clone());
                data.insert("id".to_string(), self.operation.resource_id.clone());
                data.insert("created".to_string(), "true".to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some("Resource created".to_string()),
                    data,
                })
            },
            EffectType::Delete => {
                // Simulate deleting resource
                let mut data = HashMap::new();
                data.insert("type".to_string(), self.operation.resource_type.clone());
                data.insert("id".to_string(), self.operation.resource_id.clone());
                data.insert("deleted".to_string(), "true".to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some("Resource deleted".to_string()),
                    data,
                })
            },
            EffectType::Custom(ref operation) => {
                // Handle custom operation
                let mut data = HashMap::new();
                data.insert("type".to_string(), self.operation.resource_type.clone());
                data.insert("id".to_string(), self.operation.resource_id.clone());
                data.insert("operation".to_string(), operation.clone());
                
                Ok(EffectOutcome::Success {
                    result: Some(format!("Custom operation '{}' executed", operation)),
                    data,
                })
            },
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Effect handler trait
trait EffectHandler: Debug + Send + Sync {
    fn can_handle(&self, effect_type: &EffectType) -> bool;
    fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult;
}

/// Simple effect handler implementation
#[derive(Debug)]
struct SimpleEffectHandler {
    handled_types: Vec<EffectType>,
}

impl SimpleEffectHandler {
    fn new(handled_types: Vec<EffectType>) -> Self {
        Self { handled_types }
    }
}

impl EffectHandler for SimpleEffectHandler {
    fn can_handle(&self, effect_type: &EffectType) -> bool {
        self.handled_types.iter().any(|t| match (t, effect_type) {
            (EffectType::Custom(a), EffectType::Custom(b)) => a == b,
            (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
        })
    }
    
    fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult {
        // Just delegate to the effect for now
        effect.execute(context)
    }
}

/// Effect executor
#[derive(Debug)]
struct EffectExecutor {
    handlers: Vec<Arc<dyn EffectHandler>>,
}

impl EffectExecutor {
    fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) {
        self.handlers.push(handler);
    }
    
    fn find_handler(&self, effect: &dyn Effect) -> Option<Arc<dyn EffectHandler>> {
        let effect_type = effect.effect_type();
        self.handlers.iter()
            .find(|handler| handler.can_handle(&effect_type))
            .cloned()
    }
    
    fn execute(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult {
        if let Some(handler) = self.find_handler(effect) {
            handler.handle(effect, context)
        } else {
            Err(EffectError::InvalidOperation(format!(
                "No handler found for effect type: {:?}", effect.effect_type()
            )))
        }
    }
}

/// Time effect example
#[derive(Debug, Clone)]
enum TimeOperation {
    GetCurrentTime,
    AdvanceTime(u64),
    GetTimeDifference(u64, u64),
}

/// Time effect implementation
#[derive(Debug)]
struct TimeEffect {
    operation: TimeOperation,
}

impl TimeEffect {
    fn new(operation: TimeOperation) -> Self {
        Self { operation }
    }
}

impl Effect for TimeEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("time".to_string())
    }
    
    fn description(&self) -> String {
        match &self.operation {
            TimeOperation::GetCurrentTime => "get current time".to_string(),
            TimeOperation::AdvanceTime(amount) => format!("advance time by {}", amount),
            TimeOperation::GetTimeDifference(t1, t2) => format!("get time difference between {} and {}", t1, t2),
        }
    }
    
    fn execute(&self, context: &dyn EffectContext) -> EffectResult {
        // Check capability
        if !context.has_capability("time.manage") {
            return Err(EffectError::PermissionDenied(
                "Missing capability: time.manage".to_string()
            ));
        }
        
        // Execute based on operation type
        match self.operation {
            TimeOperation::GetCurrentTime => {
                let mut data = HashMap::new();
                // Simulate getting current time
                let current_time = 1000;
                data.insert("current_time".to_string(), current_time.to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some(format!("Current time: {}", current_time)),
                    data,
                })
            },
            TimeOperation::AdvanceTime(amount) => {
                let mut data = HashMap::new();
                // Simulate current time
                let current_time = 1000;
                let new_time = current_time + amount;
                
                data.insert("old_time".to_string(), current_time.to_string());
                data.insert("new_time".to_string(), new_time.to_string());
                data.insert("amount".to_string(), amount.to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some(format!("Advanced time from {} to {}", current_time, new_time)),
                    data,
                })
            },
            TimeOperation::GetTimeDifference(t1, t2) => {
                let mut data = HashMap::new();
                let difference = if t2 > t1 { t2 - t1 } else { t1 - t2 };
                
                data.insert("time1".to_string(), t1.to_string());
                data.insert("time2".to_string(), t2.to_string());
                data.insert("difference".to_string(), difference.to_string());
                
                Ok(EffectOutcome::Success {
                    result: Some(format!("Time difference: {}", difference)),
                    data,
                })
            },
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() {
    println!("Effect-based programming demonstration");
    
    // Create a context with capabilities
    let context = BasicEffectContext::new()
        .with_capability("document.read")
        .with_capability("document.write")
        .with_capability("document.create")
        .with_capability("document.delete")
        .with_capability("document.custom")
        .with_capability("time.manage");
    
    // Create an effect executor with handlers
    let mut executor = EffectExecutor::new();
    
    // Register handlers
    let resource_handler = SimpleEffectHandler::new(vec![
        EffectType::Read,
        EffectType::Write,
        EffectType::Create,
        EffectType::Delete,
        EffectType::Custom("archive".to_string()),
    ]);
    
    let time_handler = SimpleEffectHandler::new(vec![
        EffectType::Custom("time".to_string()),
    ]);
    
    executor.register_handler(Arc::new(resource_handler));
    executor.register_handler(Arc::new(time_handler));
    
    // Create and execute some effects
    let resource_effects: Vec<Box<dyn Effect>> = vec![
        Box::new(ResourceEffect::new("document", "doc-123", EffectType::Create)),
        Box::new(ResourceEffect::new("document", "doc-123", EffectType::Write)
            .with_parameter("content", "Hello World")),
        Box::new(ResourceEffect::new("document", "doc-123", EffectType::Read)),
        Box::new(ResourceEffect::new("document", "doc-123", EffectType::Custom("archive".to_string()))),
        Box::new(ResourceEffect::new("document", "doc-123", EffectType::Delete)),
    ];
    
    // Execute each resource effect
    println!("\n=== RESOURCE EFFECTS ===");
    for effect in resource_effects {
        println!("\nExecuting effect: {}", effect.description());
        
        match executor.execute(effect.as_ref(), &context) {
            Ok(EffectOutcome::Success { result, data }) => {
                println!("  Success: {}", result.unwrap_or_default());
                println!("  Data: {:?}", data);
            },
            Ok(EffectOutcome::Failure { error }) => {
                println!("  Failed: {}", error);
            },
            Ok(EffectOutcome::Pending) => {
                println!("  Pending execution");
            },
            Err(err) => {
                println!("  Error: {}", err);
            },
        }
    }
    
    // Create and execute time effects
    let time_effects: Vec<Box<dyn Effect>> = vec![
        Box::new(TimeEffect::new(TimeOperation::GetCurrentTime)),
        Box::new(TimeEffect::new(TimeOperation::AdvanceTime(500))),
        Box::new(TimeEffect::new(TimeOperation::GetTimeDifference(1000, 1500))),
    ];
    
    // Execute each time effect
    println!("\n=== TIME EFFECTS ===");
    for effect in time_effects {
        println!("\nExecuting effect: {}", effect.description());
        
        match executor.execute(effect.as_ref(), &context) {
            Ok(EffectOutcome::Success { result, data }) => {
                println!("  Success: {}", result.unwrap_or_default());
                println!("  Data: {:?}", data);
            },
            Ok(EffectOutcome::Failure { error }) => {
                println!("  Failed: {}", error);
            },
            Ok(EffectOutcome::Pending) => {
                println!("  Pending execution");
            },
            Err(err) => {
                println!("  Error: {}", err);
            },
        }
    }
    
    println!("\nDemonstration completed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_effect_execution() {
        // Create a context with capabilities
        let context = BasicEffectContext::new()
            .with_capability("document.read")
            .with_capability("document.write");
        
        // Create effects
        let read_effect = ResourceEffect::new("document", "doc-123", EffectType::Read);
        let write_effect = ResourceEffect::new("document", "doc-123", EffectType::Write)
            .with_parameter("content", "Hello World");
        
        // Execute read effect directly
        let read_result = read_effect.execute(&context);
        assert!(read_result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = read_result {
            assert_eq!(result, Some("Read successful".to_string()));
            assert_eq!(data.get("type"), Some(&"document".to_string()));
            assert_eq!(data.get("id"), Some(&"doc-123".to_string()));
            assert_eq!(data.get("content"), Some(&"Example content".to_string()));
        } else {
            panic!("Expected EffectOutcome::Success");
        }
        
        // Execute write effect directly
        let write_result = write_effect.execute(&context);
        assert!(write_result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = write_result {
            assert_eq!(result, Some("Write successful".to_string()));
            // Check that our parameter value was used
            assert_eq!(data.get("content"), Some(&"Hello World".to_string()));
        }
    }
    
    #[test]
    fn test_effect_executor() {
        // Create a context with capability
        let context = BasicEffectContext::new()
            .with_capability("document.read");
        
        // Create an executor
        let mut executor = EffectExecutor::new();
        
        // Register a handler
        let handler = SimpleEffectHandler::new(vec![
            EffectType::Read,
            EffectType::Write,
        ]);
        executor.register_handler(Arc::new(handler));
        
        // Create an effect
        let read_effect = ResourceEffect::new("document", "doc-123", EffectType::Read);
        
        // Execute via executor
        let result = executor.execute(&read_effect, &context);
        assert!(result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = result {
            assert_eq!(result, Some("Read successful".to_string()));
            assert_eq!(data.get("type"), Some(&"document".to_string()));
            assert_eq!(data.get("id"), Some(&"doc-123".to_string()));
        } else {
            panic!("Expected EffectOutcome::Success");
        }
    }
    
    #[test]
    fn test_effect_registry_handler_registration() {
        // Create an executor
        let mut executor = EffectExecutor::new();
        
        // Create handlers for different effect types
        let read_handler = SimpleEffectHandler::new(vec![EffectType::Read]);
        let write_handler = SimpleEffectHandler::new(vec![EffectType::Write]);
        
        // Register the handlers
        executor.register_handler(Arc::new(read_handler));
        executor.register_handler(Arc::new(write_handler));
        
        // Create effects
        let read_effect = ResourceEffect::new("document", "doc-123", EffectType::Read);
        let write_effect = ResourceEffect::new("document", "doc-123", EffectType::Write);
        
        // Create contexts with required capabilities
        let read_context = BasicEffectContext::new()
            .with_capability("document.read");
        let write_context = BasicEffectContext::new()
            .with_capability("document.write");
        
        // Execute the effects
        let read_result = executor.execute(&read_effect, &read_context);
        let write_result = executor.execute(&write_effect, &write_context);
        
        // Check the results
        assert!(read_result.is_ok());
        assert!(write_result.is_ok());
        
        // Verify that missing capabilities cause errors
        let no_capability_context = BasicEffectContext::new();
        let error_result = executor.execute(&read_effect, &no_capability_context);
        assert!(error_result.is_err());
        if let Err(EffectError::PermissionDenied(message)) = error_result {
            assert!(message.contains("Missing capability: document.read"));
        } else {
            panic!("Expected PermissionDenied error");
        }
    }
    
    #[test]
    fn test_time_effect_execution() {
        // Create a context with capability
        let context = BasicEffectContext::new()
            .with_capability("time.manage");
        
        // Create time effects
        let current_time_effect = TimeEffect::new(TimeOperation::GetCurrentTime);
        let advance_time_effect = TimeEffect::new(TimeOperation::AdvanceTime(500));
        let time_diff_effect = TimeEffect::new(TimeOperation::GetTimeDifference(1000, 1500));
        
        // Execute current time effect
        let current_time_result = current_time_effect.execute(&context);
        assert!(current_time_result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = current_time_result {
            assert!(result.unwrap().contains("Current time:"));
            assert!(data.contains_key("current_time"));
        } else {
            panic!("Expected EffectOutcome::Success");
        }
        
        // Execute time diff effect
        let time_diff_result = time_diff_effect.execute(&context);
        assert!(time_diff_result.is_ok());
        
        if let Ok(EffectOutcome::Success { result, data }) = time_diff_result {
            assert_eq!(result, Some("Time difference: 500".to_string()));
            assert_eq!(data.get("difference"), Some(&"500".to_string()));
            assert_eq!(data.get("time1"), Some(&"1000".to_string()));
            assert_eq!(data.get("time2"), Some(&"1500".to_string()));
        } else {
            panic!("Expected EffectOutcome::Success");
        }
    }
}
