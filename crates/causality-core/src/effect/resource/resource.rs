use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectType, HandlerResult, EffectHandler
};
use crate::effect::types::EffectTypeId;
use crate::resource::types::{ResourceId, ResourceType};
use crate::effect::{EffectError};

/// An operation on a resource
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    Create,
    Read,
    Update,
    Delete,
    Custom(String),
    /// Represents all operations (for capability granting)
    All,
}

/// A resource effect wrapping a resource operation
#[derive(Debug)]
pub struct ResourceEffect {
    resource_type: String,
    resource_id: String,
    operation: ResourceOperation,
    parameters: HashMap<String, String>,
}

impl ResourceEffect {
    /// Creates a new resource effect
    pub fn new(resource_type: &str, resource_id: &str, operation: ResourceOperation) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            operation,
            parameters: HashMap::new(),
        }
    }
    
    /// Creates a new create resource effect
    pub fn create(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, ResourceOperation::Create)
    }
    
    /// Creates a new read resource effect
    pub fn read(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, ResourceOperation::Read)
    }
    
    /// Creates a new update resource effect
    pub fn update(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, ResourceOperation::Update)
    }
    
    /// Creates a new delete resource effect
    pub fn delete(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, ResourceOperation::Delete)
    }
    
    /// Creates a new custom resource effect
    pub fn custom(resource_type: &str, resource_id: &str, operation_name: &str) -> Self {
        Self::new(resource_type, resource_id, ResourceOperation::Custom(operation_name.to_string()))
    }
    
    /// Adds a parameter to the effect
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }
    
    /// Get the operation
    pub fn operation(&self) -> &ResourceOperation {
        &self.operation
    }
    
    /// Get a parameter value
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }
    
    /// Get all parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

#[async_trait]
impl Effect for ResourceEffect {
    fn effect_type(&self) -> EffectType {
        match self.operation {
            ResourceOperation::Create => EffectType::Create,
            ResourceOperation::Read => EffectType::Read,
            ResourceOperation::Update => EffectType::Write,
            ResourceOperation::Delete => EffectType::Delete,
            ResourceOperation::Custom(_) => EffectType::Custom("resource:custom".to_string()),
            ResourceOperation::All => EffectType::Custom("resource:all".to_string()),
        }
    }
    
    fn description(&self) -> String {
        match &self.operation {
            ResourceOperation::Create => format!("Create resource {}", self.resource_id),
            ResourceOperation::Read => format!("Read resource {}", self.resource_id),
            ResourceOperation::Update => format!("Update resource {}", self.resource_id),
            ResourceOperation::Delete => format!("Delete resource {}", self.resource_id),
            ResourceOperation::Custom(name) => format!("Custom operation {} on resource {}", name, self.resource_id),
            ResourceOperation::All => format!("All operations on resource {}", self.resource_id),
        }
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Placeholder execution logic
        // In a real implementation, this would interact with the resource manager
        // based on self.operation, self.resource_id, self.resource_type, etc.

        match self.operation {
            ResourceOperation::Read | ResourceOperation::Update | ResourceOperation::Delete | ResourceOperation::Create => {
                // Simulate successful operation for now
                println!("Simulating {:?} on resource {} ({})", 
                         self.operation, self.resource_id, self.resource_type);
                Ok(EffectOutcome::success(HashMap::new()).with_message(format!("Operation {:?} simulated.", self.operation)))
            },
            ResourceOperation::Custom(ref name) => {
                // Simulate custom operation
                println!("Simulating custom operation '{}' on resource {} ({})", 
                         name, self.resource_id, self.resource_type);
                Ok(EffectOutcome::success(HashMap::new()).with_message(format!("Custom operation '{}' simulated.", name)))
            },
            ResourceOperation::All => {
                // 'All' is typically used for capability checks, not direct execution
                // Wrap the error string in Box::new(EffectError::Other(...))
                Ok(EffectOutcome::error(Box::new(EffectError::Other("Cannot execute All operation directly".to_string()))))
            }
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Resource effect handler
/// 
/// Handles resource-related effects by delegating to appropriate processors
#[derive(Debug, Clone)]
pub struct ResourceEffectHandler;

impl ResourceEffectHandler {
    /// Create a new resource effect handler
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EffectHandler for ResourceEffectHandler {
    fn supported_effect_types(&self) -> Vec<EffectTypeId> {
        vec![
            crate::effect::types::EffectTypeId::new("resource:create"),
            crate::effect::types::EffectTypeId::new("resource:read"),
            crate::effect::types::EffectTypeId::new("resource:update"),
            crate::effect::types::EffectTypeId::new("resource:delete"),
            crate::effect::types::EffectTypeId::new("resource:custom"),
        ]
    }
    
    async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> HandlerResult<EffectOutcome> {
        // Try to downcast to a ResourceEffect
        if let Some(resource_effect) = effect.as_any().downcast_ref::<ResourceEffect>() {
            // Handle based on operation type
            match &resource_effect.operation {
                ResourceOperation::Create => {
                    // Handle resource creation
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    result.insert("status".to_string(), "created".to_string());
                    Ok(EffectOutcome::success(result))
                },
                ResourceOperation::Read => {
                    // Handle resource read
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    // In a real implementation, we would read actual data
                    result.insert("data".to_string(), "sample_data".to_string());
                    Ok(EffectOutcome::success(result))
                },
                ResourceOperation::Update => {
                    // Handle resource update
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    result.insert("status".to_string(), "updated".to_string());
                    Ok(EffectOutcome::success(result))
                },
                ResourceOperation::Delete => {
                    // Handle resource deletion
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    result.insert("status".to_string(), "deleted".to_string());
                    Ok(EffectOutcome::success(result))
                },
                ResourceOperation::Custom(name) => {
                    // Handle custom operation
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    result.insert("operation".to_string(), name.clone());
                    result.insert("status".to_string(), "executed".to_string());
                    Ok(EffectOutcome::success(result))
                },
                ResourceOperation::All => {
                    // This is a special case used for capability granting, not for actual execution
                    let mut result = HashMap::new();
                    result.insert("resource_id".to_string(), resource_effect.resource_id.clone());
                    result.insert("status".to_string(), "permission_error".to_string());
                    result.insert("message".to_string(), "Cannot directly execute 'All' operation".to_string());
                    Ok(EffectOutcome::error(Box::new(crate::effect::EffectError::Other("Cannot execute All operation directly".to_string()))))
                },
            }
        } else {
            // Not a resource effect
            Ok(EffectOutcome::failure("Not a resource effect".to_string()))
        }
    }
}

/// Create a resource effect from resource identifiers and operation type
pub fn create_resource_effect(
    resource_id: &ResourceId,
    resource_type: &ResourceType,
    operation_type: EffectType,
) -> ResourceEffect {
    // Get strings needed for ResourceEffect constructor
    let type_name = resource_type.qualified_name();
    let id_str = resource_id.to_string();
    
    match operation_type {
        EffectType::Create => ResourceEffect::create(&type_name, &id_str),
        EffectType::Read => ResourceEffect::read(&type_name, &id_str),
        EffectType::Write => ResourceEffect::update(&type_name, &id_str),
        EffectType::Delete => ResourceEffect::delete(&type_name, &id_str),
        EffectType::Custom(name) => ResourceEffect::custom(&type_name, &id_str, &name),
    }
} 