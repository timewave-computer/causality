// Resource Effect Integration
//
// This module provides integration between the effect system and resource management.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult, 
    EffectType, EffectHandler, HandlerResult
};
use crate::effect::types::EffectTypeId;
use crate::resource::{
    Resource, ResourceManager, ResourceState, ResourceError, ResourceResult,
    ResourceType, ResourceConfig,
    interface::{ResourceInterface, ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency, ResourceError as InterfaceResourceError}
};
// Import ResourceId from lib.rs which publicly re-exports it
use crate::ResourceId;

// Create placeholders for missing types
// Instead of using: use crate::resource::provider::ResourceDeployConfig;

/// Define ResourceRegistry as a trait instead of a struct
#[async_trait]
pub trait ResourceRegistry: Send + Sync + Debug {
    /// Get a resource manager for a resource type
    fn get_resource_manager(&self, resource_type: &str) -> Option<Arc<dyn ResourceManager>>;
    
    /// Get a resource interface for a resource ID
    async fn get_resource_interface(&self, resource_id: &str) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceInterface>>;
}

/// Create a placeholder implementation of ResourceRegistry
#[derive(Debug)]
pub struct SimpleResourceRegistry {
    resource_manager: Arc<dyn ResourceManager>,
}

impl SimpleResourceRegistry {
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            resource_manager,
        }
    }
}

#[async_trait]
impl ResourceRegistry for SimpleResourceRegistry {
    fn get_resource_manager(&self, _resource_type: &str) -> Option<Arc<dyn ResourceManager>> {
        Some(self.resource_manager.clone())
    }
    
    async fn get_resource_interface(&self, resource_id: &str) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceInterface>> {
        Ok(Arc::new(SimpleResourceInterface {
            resource_manager: self.resource_manager.clone(),
            resource_id: resource_id.to_string(),
        }))
    }
}

/// Simple resource interface implementation that delegates to a resource manager
#[derive(Debug)]
struct SimpleResourceInterface {
    resource_manager: Arc<dyn ResourceManager>,
    resource_id: String,
}

#[async_trait]
impl ResourceInterface for SimpleResourceInterface {
    async fn get_access(&self) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceAccess>> {
        Err(InterfaceResourceError::ResourceError("Not implemented".to_string()))
    }
    
    async fn get_lifecycle(&self) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceLifecycle>> {
        Err(InterfaceResourceError::ResourceError("Not implemented".to_string()))
    }
    
    async fn get_locking(&self) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceLocking>> {
        Err(InterfaceResourceError::ResourceError("Not implemented".to_string()))
    }
    
    async fn get_dependency(&self) -> crate::resource::interface::ResourceResult<Arc<dyn ResourceDependency>> {
        Err(InterfaceResourceError::ResourceError("Not implemented".to_string()))
    }
    
    async fn deploy(&self, data: HashMap<String, String>, config: HashMap<String, String>) -> crate::resource::interface::ResourceResult<()> {
        // For now, just create the resource with the data
        self.resource_manager.create_resource(
            &config.get("resource_type").ok_or_else(|| InterfaceResourceError::ResourceError("Missing resource_type".to_string()))?,
            &self.resource_id,
            data,
        ).await.map_err(|e| match e {
            ResourceError::AlreadyExists(id) => 
                InterfaceResourceError::AlreadyExists(causality_types::ContentId::new(id.to_string())),
            ResourceError::NotFound(id) => 
                InterfaceResourceError::NotFound(causality_types::ContentId::new(id.to_string())),
            _ => InterfaceResourceError::ResourceError(e.to_string())
        })
    }
    
    async fn update(&self, data: HashMap<String, String>) -> crate::resource::interface::ResourceResult<()> {
        // For now, just update the resource with the data
        let resource_type = match data.get("resource_type") {
            Some(rt) => rt.as_str(),
            None => return Err(InterfaceResourceError::ResourceError("Missing resource_type".to_string()))
        };
        
        self.resource_manager.update_resource(
            resource_type,
            &self.resource_id,
            data.clone(), // Clone to avoid the move
        ).await.map_err(|e| match e {
            ResourceError::AlreadyExists(id) => 
                InterfaceResourceError::AlreadyExists(causality_types::ContentId::new(id.to_string())),
            ResourceError::NotFound(id) => 
                InterfaceResourceError::NotFound(causality_types::ContentId::new(id.to_string())),
            _ => InterfaceResourceError::ResourceError(e.to_string())
        })
    }
    
    async fn read_properties(&self, properties: Vec<String>) -> crate::resource::interface::ResourceResult<HashMap<String, String>> {
        // For now, just get the resource and return its properties
        let resource = self.resource_manager.get_resource(
            &properties.get(0).ok_or_else(|| InterfaceResourceError::ResourceError("Missing resource_type".to_string()))?,
            &self.resource_id,
        ).await.map_err(|e| match e {
            ResourceError::AlreadyExists(id) => 
                InterfaceResourceError::AlreadyExists(causality_types::ContentId::new(id.to_string())),
            ResourceError::NotFound(id) => 
                InterfaceResourceError::NotFound(causality_types::ContentId::new(id.to_string())),
            _ => InterfaceResourceError::ResourceError(e.to_string())
        })?;
        
        Ok(resource.state().properties())
    }
    
    async fn destroy(&self) -> crate::resource::interface::ResourceResult<()> {
        // For now, just delete the resource
        self.resource_manager.delete_resource(
            &self.resource_id.split(':').next().ok_or_else(|| InterfaceResourceError::ResourceError("Invalid resource_id".to_string()))?,
            &self.resource_id,
        ).await.map_err(|e| match e {
            ResourceError::AlreadyExists(id) => 
                InterfaceResourceError::AlreadyExists(causality_types::ContentId::new(id.to_string())),
            ResourceError::NotFound(id) => 
                InterfaceResourceError::NotFound(causality_types::ContentId::new(id.to_string())),
            _ => InterfaceResourceError::ResourceError(e.to_string())
        })
    }
    
    async fn execute_operation(&self, operation: &str, params: HashMap<String, String>) -> crate::resource::interface::ResourceResult<HashMap<String, String>> {
        // For now, just execute the operation on the resource
        let resource_type = match params.get("resource_type") {
            Some(rt) => rt.as_str(),
            None => return Err(InterfaceResourceError::ResourceError("Missing resource_type".to_string()))
        };
        
        self.resource_manager.execute_operation(
            resource_type,
            &self.resource_id,
            operation,
            params.clone(), // Clone to avoid the move
        ).await.map_err(|e| match e {
            ResourceError::AlreadyExists(id) => 
                InterfaceResourceError::AlreadyExists(causality_types::ContentId::new(id.to_string())),
            ResourceError::NotFound(id) => 
                InterfaceResourceError::NotFound(causality_types::ContentId::new(id.to_string())),
            _ => InterfaceResourceError::ResourceError(e.to_string())
        })
    }
}

/// Simple resource interface for registry interactions
/// This is a temporary definition to allow compilation until we fully integrate with the resource system
#[async_trait]
pub trait TemporaryResourceInterface: Send + Sync + Debug {
    /// Deploy a resource
    async fn deploy(&self, data: HashMap<String, String>, config: HashMap<String, String>) 
        -> Result<(), String>;
    
    /// Update a resource
    async fn update(&self, data: HashMap<String, String>) -> Result<(), String>;
    
    /// Read properties from a resource
    async fn read_properties(&self, properties: Vec<String>) -> Result<HashMap<String, String>, String>;
    
    /// Destroy a resource
    async fn destroy(&self) -> Result<(), String>;
    
    /// Execute a custom operation on a resource
    async fn execute_operation(&self, operation: &str, params: HashMap<String, String>) 
        -> Result<HashMap<String, String>, String>;
}

use super::resource::{ResourceEffect, ResourceOperation};

/// Resource Manager Effect Handler
/// 
/// Handles resource management effects by delegating to a ResourceManager
#[derive(Debug)]
pub struct ResourceManagerEffectHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceManagerEffectHandler {
    /// Create a new resource manager effect handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            resource_manager,
        }
    }
    
    /// Execute a create effect
    async fn execute_create(
        &self, 
        effect: &ResourceEffect,
        _context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get the resource type and id
        let resource_type = effect.resource_type();
        let resource_id = effect.resource_id();
        
        // Check if the resource already exists
        if self.resource_manager.resource_exists(resource_type, resource_id).await {
            return Err(EffectError::ExecutionError(
                format!("Resource {}:{} already exists", resource_type, resource_id)
            ));
        }
        
        // Parse parameters and create resource
        let params = effect.parameters();
        
        // Try to create the resource
        match self.resource_manager.create_resource(resource_type, resource_id, params.clone()).await {
            Ok(_) => {
                // Return success with resource information
                let mut data = HashMap::new();
                data.insert("resource_type".to_string(), resource_type.to_string());
                data.insert("resource_id".to_string(), resource_id.to_string());
                data.insert("created".to_string(), "true".to_string());
                
                Ok(EffectOutcome::success(data))
            },
            Err(e) => Err(EffectError::ExecutionError(
                format!("Failed to create resource: {}", e)
            )),
        }
    }
    
    /// Execute a read effect
    async fn execute_read(
        &self, 
        effect: &ResourceEffect,
        _context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get the resource type and id
        let resource_type = effect.resource_type();
        let resource_id = effect.resource_id();
        
        // Check if the resource exists
        if !self.resource_manager.resource_exists(resource_type, resource_id).await {
            return Err(EffectError::NotFound(
                format!("Resource {}:{} does not exist", resource_type, resource_id)
            ));
        }
        
        // Try to read the resource
        match self.resource_manager.get_resource(resource_type, resource_id).await {
            Ok(resource) => {
                // Get resource state
                let state = resource.state();
                
                // Convert state to HashMap
                let mut data = HashMap::new();
                data.insert("resource_type".to_string(), resource_type.to_string());
                data.insert("resource_id".to_string(), resource_id.to_string());
                
                // Add state properties to data
                data.extend(state.properties().clone());
                
                Ok(EffectOutcome::success(data))
            },
            Err(e) => Err(EffectError::ExecutionError(
                format!("Failed to read resource: {}", e)
            )),
        }
    }
    
    /// Execute a write effect
    async fn execute_write(
        &self, 
        effect: &ResourceEffect,
        _context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get the resource type and id
        let resource_type = effect.resource_type();
        let resource_id = effect.resource_id();
        
        // Check if the resource exists
        if !self.resource_manager.resource_exists(resource_type, resource_id).await {
            return Err(EffectError::NotFound(
                format!("Resource {}:{} does not exist", resource_type, resource_id)
            ));
        }
        
        // Parse parameters for update
        let update_data = effect.parameters();
        
        // Try to update the resource
        match self.resource_manager.update_resource(resource_type, resource_id, update_data.clone()).await {
            Ok(_) => {
                // Return success with resource information
                let mut data = HashMap::new();
                data.insert("resource_type".to_string(), resource_type.to_string());
                data.insert("resource_id".to_string(), resource_id.to_string());
                data.insert("updated".to_string(), "true".to_string());
                
                Ok(EffectOutcome::success(data))
            },
            Err(e) => Err(EffectError::ExecutionError(
                format!("Failed to update resource: {}", e)
            )),
        }
    }
    
    /// Execute a delete effect
    async fn execute_delete(
        &self, 
        effect: &ResourceEffect,
        _context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get the resource type and id
        let resource_type = effect.resource_type();
        let resource_id = effect.resource_id();
        
        // Check if the resource exists
        if !self.resource_manager.resource_exists(resource_type, resource_id).await {
            return Err(EffectError::NotFound(
                format!("Resource {}:{} does not exist", resource_type, resource_id)
            ));
        }
        
        // Try to delete the resource
        match self.resource_manager.delete_resource(resource_type, resource_id).await {
            Ok(_) => {
                // Return success with resource information
                let mut data = HashMap::new();
                data.insert("resource_type".to_string(), resource_type.to_string());
                data.insert("resource_id".to_string(), resource_id.to_string());
                data.insert("deleted".to_string(), "true".to_string());
                
                Ok(EffectOutcome::success(data))
            },
            Err(e) => Err(EffectError::ExecutionError(
                format!("Failed to delete resource: {}", e)
            )),
        }
    }
    
    /// Execute an execute operation effect
    async fn execute_operation_effect(
        &self, 
        effect: &ResourceEffect,
        _context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get the resource type and id
        let resource_type = effect.resource_type();
        let resource_id = effect.resource_id();
        
        // Check if the resource exists
        if !self.resource_manager.resource_exists(resource_type, resource_id).await {
            return Err(EffectError::NotFound(
                format!("Resource {}:{} does not exist", resource_type, resource_id)
            ));
        }
        
        // Parse parameters for the operation
        let params = effect.parameters();
        
        // Get the operation name from the parameters
        let name = params.get("operation_name")
            .ok_or_else(|| EffectError::InvalidParameter(
                "Missing operation_name parameter".to_string()
            ))?;
        
        // Execute the operation on the resource
        match self.resource_manager.execute_operation(resource_type, resource_id, name, params.clone()).await {
            Ok(result) => {
                // Return success with operation result
                let mut data = HashMap::new();
                data.insert("resource_type".to_string(), resource_type.to_string());
                data.insert("resource_id".to_string(), resource_id.to_string());
                data.insert("operation".to_string(), name.to_string());
                data.insert("executed".to_string(), "true".to_string());
                
                // Add operation results to data
                for (key, value) in result {
                    data.insert(format!("result_{}", key), value);
                }
                
                Ok(EffectOutcome::success(data))
            },
            Err(e) => Err(EffectError::ExecutionError(
                format!("Failed to execute operation: {}", e)
            )),
        }
    }
}

/// Async effect handler for resource operations
#[async_trait]
pub trait AsyncResourceEffectHandler: EffectHandler {
    /// Execute a resource effect asynchronously
    async fn execute_async(
        &self, 
        effect: &dyn Effect, 
        context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome>;
}

#[async_trait]
impl AsyncResourceEffectHandler for ResourceManagerEffectHandler {
    async fn execute_async(
        &self, 
        effect: &dyn Effect, 
        context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Try to downcast to ResourceEffect
        let resource_effect = match effect.as_any().downcast_ref::<ResourceEffect>() {
            Some(effect) => effect,
            None => return Err(EffectError::ExecutionError(
                "Effect is not a ResourceEffect".to_string()
            )),
        };
        
        // Execute the effect based on its type
        match resource_effect.effect_type() {
            EffectType::Create => {
                self.execute_create(resource_effect, context).await
            },
            EffectType::Read => {
                self.execute_read(resource_effect, context).await
            },
            EffectType::Write => {
                self.execute_write(resource_effect, context).await
            },
            EffectType::Delete => {
                self.execute_delete(resource_effect, context).await
            },
            EffectType::Custom(name) => {
                self.execute_operation_effect(resource_effect, context).await
            },
            _ => Err(EffectError::InvalidOperation(
                format!("Unsupported effect type: {:?}", resource_effect.effect_type())
            )),
        }
    }
}

/// Execute a resource deployment effect
pub async fn execute_deploy_resource_effect(
    resource_type: &str,
    resource_id: &str,
    data: HashMap<String, String>,
    config: HashMap<String, String>,
    registry: Arc<dyn ResourceRegistry>,
) -> EffectResult<EffectOutcome> {
    // Get the resource interface from the registry
    let interface = registry.get_resource_interface(resource_id)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to get resource interface: {}", e)))?;
        
    // Execute the deploy method
    interface.deploy(data, config)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to deploy resource: {}", e)))?;
        
    // Return success
    let mut result = HashMap::new();
    result.insert("resource_id".to_string(), resource_id.to_string());
    result.insert("resource_type".to_string(), resource_type.to_string());
    
    Ok(EffectOutcome::success(result))
}

/// Execute a resource update effect
pub async fn execute_update_resource_effect(
    resource_type: &str,
    resource_id: &str,
    update_data: HashMap<String, String>,
    registry: Arc<dyn ResourceRegistry>,
) -> EffectResult<EffectOutcome> {
    // Get the resource interface from the registry
    let interface = registry.get_resource_interface(resource_id)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to get resource interface: {}", e)))?;
        
    // Execute the update method
    interface.update(update_data)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to update resource: {}", e)))?;
        
    // Return success
    let mut result = HashMap::new();
    result.insert("resource_id".to_string(), resource_id.to_string());
    result.insert("resource_type".to_string(), resource_type.to_string());
    
    Ok(EffectOutcome::success(result))
}

/// Execute a resource read effect
pub async fn execute_read_resource_effect(
    resource_type: &str,
    resource_id: &str,
    properties: Vec<String>,
    registry: Arc<dyn ResourceRegistry>,
) -> EffectResult<EffectOutcome> {
    // Get the resource interface from the registry
    let interface = registry.get_resource_interface(resource_id)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to get resource interface: {}", e)))?;
        
    // Read properties
    let properties_data = interface.read_properties(properties)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to read resource properties: {}", e)))?;
    
    // Return success with properties
    let mut result = HashMap::new();
    result.insert("resource_id".to_string(), resource_id.to_string());
    result.insert("resource_type".to_string(), resource_type.to_string());
    result.extend(properties_data);
    
    Ok(EffectOutcome::success(result))
}

/// Execute a resource destroy effect
pub async fn execute_destroy_resource_effect(
    resource_type: &str,
    resource_id: &str,
    registry: Arc<dyn ResourceRegistry>,
) -> EffectResult<EffectOutcome> {
    // Get the resource interface from the registry
    let interface = registry.get_resource_interface(resource_id)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to get resource interface: {}", e)))?;
        
    // Execute the destroy method
    interface.destroy()
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to destroy resource: {}", e)))?;
        
    // Return success
    let mut result = HashMap::new();
    result.insert("resource_id".to_string(), resource_id.to_string());
    result.insert("resource_type".to_string(), resource_type.to_string());
    
    Ok(EffectOutcome::success(result))
}

/// Execute a custom resource operation
pub async fn execute_custom_resource_operation(
    _resource_type: &str,
    resource_id: &str,
    operation_name: &str,
    operation_params: HashMap<String, String>,
    registry: Arc<dyn ResourceRegistry>,
) -> EffectResult<EffectOutcome> {
    // Get the resource interface from the registry
    let interface = registry.get_resource_interface(resource_id)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Failed to get resource interface: {}", e)))?;
    
    // Execute the custom operation
    let operation_result = interface.execute_operation(operation_name, operation_params)
        .await
        .map_err(|e| EffectError::ExecutionError(format!("Unsupported custom operation {}: {}", operation_name, e)))?;
            
    // Return success with operation result
    let mut result = HashMap::new();
    result.insert("resource_id".to_string(), resource_id.to_string());
    result.insert("operation".to_string(), operation_name.to_string());
    result.extend(operation_result);
    
    Ok(EffectOutcome::success(result))
}

#[async_trait]
impl EffectHandler for ResourceManagerEffectHandler {
    fn supported_effect_types(&self) -> Vec<EffectTypeId> {
        vec![
            crate::effect::types::EffectTypeId("resource:create".to_string()),
            crate::effect::types::EffectTypeId("resource:read".to_string()),
            crate::effect::types::EffectTypeId("resource:update".to_string()),
            crate::effect::types::EffectTypeId("resource:delete".to_string()),
            crate::effect::types::EffectTypeId("resource:list".to_string()),
        ]
    }
    
    async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> HandlerResult<EffectOutcome> {
        // Try to downcast to ResourceEffect
        if let Some(resource_effect) = effect.as_any().downcast_ref::<super::resource::ResourceEffect>() {
            // Handle based on operation type
            match resource_effect.operation() {
                ResourceOperation::Create => {
                    self.execute_create(resource_effect, context).await
                },
                ResourceOperation::Read => {
                    self.execute_read(resource_effect, context).await
                },
                ResourceOperation::Update => {
                    self.execute_write(resource_effect, context).await
                },
                ResourceOperation::Delete => {
                    self.execute_delete(resource_effect, context).await
                },
                ResourceOperation::Custom(ref name) => {
                    self.execute_operation_effect(resource_effect, context).await
                }
            }
        } else {
            // Return error if not a ResourceEffect
            Ok(EffectOutcome::failure(format!("Expected ResourceEffect, got {:?}", effect)))
        }
    }
}

/// Simple resource manager implementation
#[derive(Debug, Clone)]
pub struct SimpleResourceManager {
    config: ResourceConfig,
    resources: Arc<RwLock<HashMap<String, Box<dyn Resource>>>>,
}

impl SimpleResourceManager {
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            config,
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ResourceManager for SimpleResourceManager {
    fn get_config(&self) -> &ResourceConfig {
        &self.config
    }
    
    async fn get_resource_interface(&self) -> ResourceResult<Arc<dyn ResourceInterface>> {
        Ok(Arc::new(SimpleResourceInterface {
            resource_manager: Arc::new(self.clone()),
            resource_id: "".to_string(),
        }))
    }
    
    async fn get_resource_validator(&self) -> ResourceResult<Arc<dyn Debug + Send + Sync>> {
        Ok(Arc::new(()))
    }
    
    async fn start(&self) -> ResourceResult<()> {
        Ok(())
    }
    
    async fn stop(&self) -> ResourceResult<()> {
        Ok(())
    }
    
    async fn resource_exists(&self, resource_type: &str, resource_id: &str) -> bool {
        if let Ok(resources) = self.resources.read() {
            resources.contains_key(resource_id)
        } else {
            false
        }
    }
    
    async fn create_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        data: HashMap<String, String>,
    ) -> ResourceResult<()> {
        let mut resources = match self.resources.write() {
            Ok(guard) => guard,
            Err(_) => return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::Internal("Failed to acquire lock".to_string())
            )),
        };
        
        if resources.contains_key(resource_id) {
            return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::AlreadyExists(
                    causality_types::ContentId::new(resource_id.to_string())
                )
            ));
        }
        
        let resource = Box::new(SimpleResource {
            id: resource_id.to_string(),
            resource_type: resource_type.to_string(),
            state: ResourceState::Created,
            metadata: HashMap::new(),
        });
        
        resources.insert(resource_id.to_string(), resource);
        Ok(())
    }
    
    async fn get_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> ResourceResult<Box<dyn Resource>> {
        let resources = match self.resources.read() {
            Ok(guard) => guard,
            Err(_) => return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::Internal("Failed to acquire lock".to_string())
            )),
        };
        
        let resource = resources.get(resource_id).ok_or_else(|| {
            ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::NotFound(
                    causality_types::ContentId::new(resource_id.to_string())
                )
            )
        })?;
            
        Ok(Box::new(SimpleResource {
            id: resource.id().to_string(),
            resource_type: resource.resource_type().to_string(),
            state: resource.state(),
            metadata: resource.get_metadata_map().unwrap_or_default(),
        }))
    }
    
    async fn update_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        data: HashMap<String, String>,
    ) -> ResourceResult<()> {
        let mut resources = match self.resources.write() {
            Ok(guard) => guard,
            Err(_) => return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::Internal("Failed to acquire lock".to_string())
            )),
        };
        
        let resource = resources.get_mut(resource_id).ok_or_else(|| {
            ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::NotFound(
                    causality_types::ContentId::new(resource_id.to_string())
                )
            )
        })?;
            
        // Update metadata from data
        for (key, value) in data {
            resource.set_metadata(&key, &value)?;
        }
        
        Ok(())
    }
    
    async fn delete_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> ResourceResult<()> {
        let mut resources = match self.resources.write() {
            Ok(guard) => guard,
            Err(_) => return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::Internal("Failed to acquire lock".to_string())
            )),
        };
        
        resources.remove(resource_id).ok_or_else(|| {
            ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::NotFound(
                    causality_types::ContentId::new(resource_id.to_string())
                )
            )
        })?;
            
        Ok(())
    }
    
    async fn execute_operation(
        &self,
        resource_type: &str,
        resource_id: &str,
        operation: &str,
        params: HashMap<String, String>,
    ) -> ResourceResult<HashMap<String, String>> {
        let mut resources = match self.resources.write() {
            Ok(guard) => guard,
            Err(_) => return Err(ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::Internal("Failed to acquire lock".to_string())
            )),
        };
        
        let resource = resources.get_mut(resource_id).ok_or_else(|| {
            ResourceError::InterfaceError(
                crate::resource::interface::ResourceError::NotFound(
                    causality_types::ContentId::new(resource_id.to_string())
                )
            )
        })?;
            
        // For now, just return the metadata
        Ok(resource.get_metadata_map().unwrap_or_default())
    }
}

/// Simple resource implementation
#[derive(Debug, Clone)]
struct SimpleResource {
    id: String,
    resource_type: String,
    state: ResourceState,
    metadata: HashMap<String, String>,
}

impl Resource for SimpleResource {
    fn id(&self) -> ResourceId {
        ResourceId::from_string(&self.id).unwrap_or_else(|_| {
            ResourceId::new(causality_types::crypto_primitives::ContentHash::new("blake3", self.id.as_bytes().to_vec()))
        })
    }
    
    fn resource_type(&self) -> ResourceType {
        ResourceType::new(&self.resource_type, "1.0")
    }
    
    fn state(&self) -> ResourceState {
        self.state
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn get_metadata_map(&self) -> Option<HashMap<String, String>> {
        Some(self.metadata.clone())
    }
    
    fn clone_resource(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
} 