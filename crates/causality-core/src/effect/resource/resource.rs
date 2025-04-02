use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::fmt;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult, EffectType, HandlerResult, EffectHandler
};
use crate::effect::types::EffectTypeId;
use crate::resource::{Resource, ResourceState, ResourceError, ResourceResult, ResourceId, ResourceType};

/// An operation on a resource
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    Create,
    Read,
    Update,
    Delete,
    Custom(String),
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
        }
    }
    
    fn description(&self) -> String {
        match &self.operation {
            ResourceOperation::Create => format!("Create resource {}", self.resource_id),
            ResourceOperation::Read => format!("Read resource {}", self.resource_id),
            ResourceOperation::Update => format!("Update resource {}", self.resource_id),
            ResourceOperation::Delete => format!("Delete resource {}", self.resource_id),
            ResourceOperation::Custom(name) => format!("Custom operation {} on resource {}", name, self.resource_id),
        }
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Basic implementation that just logs the operation
        println!("Executing resource effect: {}", self.description());
        
        // Create a simple result map
        let mut result = HashMap::new();
        result.insert("resource_id".to_string(), self.resource_id.clone());
        result.insert("operation".to_string(), format!("{:?}", self.operation));
        
        // In a real implementation, we would dispatch to a resource registry
        Ok(EffectOutcome::success(result))
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
            }
        } else {
            // Not a resource effect
            Ok(EffectOutcome::failure("Not a resource effect".to_string()))
        }
    }
}

/// Create a resource effect from a resource ID and operation type
pub fn create_resource_effect(
    resource: &dyn Resource,
    operation_type: EffectType,
) -> ResourceEffect {
    let resource_type = resource.resource_type().qualified_name();
    let resource_id = resource.id().to_string();
    
    match operation_type {
        EffectType::Create => ResourceEffect::create(&resource_type, &resource_id),
        EffectType::Read => ResourceEffect::read(&resource_type, &resource_id),
        EffectType::Write => ResourceEffect::update(&resource_type, &resource_id),
        EffectType::Delete => ResourceEffect::delete(&resource_type, &resource_id),
        EffectType::Custom(name) => ResourceEffect::custom(&resource_type, &resource_id, &name),
    }
} 