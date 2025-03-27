//! Resource integration for the Effect System
//!
//! This module integrates the resource system from causality-core with the
//! effect system in the engine. It provides effects for querying, storing,
//! and managing resources.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use causality_core::resource::{
    ResourceManager, ResourceResult, ResourceError, ResourceConfig,
    ResourceQuery, QueryEngine, QueryResult, QueryError, QueryOptions,
    ResourceInterface, Resource, ContentId, ResourceType
};

use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::context::Context;
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::error::{EffectError, EffectResult};
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::{Effect, EffectTypeId};

/// Resource effect error
#[derive(Error, Debug)]
pub enum ResourceEffectError {
    /// Resource error from core
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    /// Query error from core
    #[error("Query error: {0}")]
    QueryError(#[from] QueryError),
    
    /// Resource integration error
    #[error("Resource integration error: {0}")]
    IntegrationError(String),
}

impl From<ResourceEffectError> for EffectError {
    fn from(err: ResourceEffectError) -> Self {
        match err {
            ResourceEffectError::ResourceError(e) => 
                EffectError::SubsystemError(format!("Resource error: {}", e)),
            ResourceEffectError::QueryError(e) => 
                EffectError::SubsystemError(format!("Query error: {}", e)),
            ResourceEffectError::IntegrationError(e) => 
                EffectError::SubsystemError(format!("Resource integration error: {}", e)),
        }
    }
}

/// Resource effect manager for integrating with the engine
pub struct ResourceEffectManager {
    /// Resource manager from core
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceEffectManager {
    /// Create a new resource effect manager
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
    
    /// Get the resource manager
    pub fn resource_manager(&self) -> &Arc<dyn ResourceManager> {
        &self.resource_manager
    }
    
    /// Get the resource interface
    pub async fn get_resource_interface(&self) -> ResourceResult<Arc<dyn ResourceInterface>> {
        self.resource_manager.get_resource_interface().await
    }
    
    /// Register resource effects with the registry
    pub fn register_effects(&self, registry: &mut super::registry::EffectRegistry) {
        // Register resource query effect
        registry.register(
            ResourceQueryEffect::type_id(),
            Arc::new(ResourceQueryHandler::new(self.resource_manager.clone()))
        );
        
        // Register resource storage effects
        registry.register(
            ResourceStoreEffect::type_id(), 
            Arc::new(ResourceStoreHandler::new(self.resource_manager.clone()))
        );
        
        registry.register(
            ResourceGetEffect::type_id(),
            Arc::new(ResourceGetHandler::new(self.resource_manager.clone()))
        );
        
        registry.register(
            ResourceDeleteEffect::type_id(),
            Arc::new(ResourceDeleteHandler::new(self.resource_manager.clone()))
        );
    }
}

impl fmt::Debug for ResourceEffectManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceEffectManager")
            .finish()
    }
}

//
// Resource Query Effect
//

/// Resource query effect
#[derive(Debug, Clone)]
pub struct ResourceQueryEffect;

impl ResourceQueryEffect {
    /// Get effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.query")
    }
}

/// Parameters for resource query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryParams {
    /// The query to execute
    pub query: ResourceQuery,
    
    /// Query options
    pub options: Option<QueryOptions>,
}

/// Outcome of resource query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryOutcome {
    /// Query result data
    pub data: Vec<Box<dyn Resource>>,
    
    /// Pagination information
    pub pagination: Option<causality_core::resource::query::PaginationResult>,
    
    /// Query statistics
    pub stats: Option<QueryExecutionStats>,
}

/// Query execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionStats {
    /// Time taken in milliseconds
    pub execution_time_ms: u64,
    
    /// Number of resources matched
    pub resources_matched: usize,
    
    /// Number of resources returned
    pub resources_returned: usize,
}

impl Effect for ResourceQueryEffect {
    type Param = ResourceQueryParams;
    type Outcome = ResourceQueryOutcome;
    
    fn type_id(&self) -> EffectTypeId {
        Self::type_id()
    }
}

/// Handler for resource query effects
pub struct ResourceQueryHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceQueryHandler {
    /// Create a new resource query handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceQueryHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceQueryHandler")
            .finish()
    }
}

#[async_trait]
impl :EffectRuntime:causality_core::effect::runtime::EffectRuntime::core::handler::EffectHandler for ResourceQueryHandler {
    async fn handle(
        &self,
        _effect_type: &EffectTypeId,
        param: serde_json::Value,
        context: &Context,
    ) -> EffectResult<serde_json::Value> {
        // Parse parameters
        let params: ResourceQueryParams = serde_json::from_value(param)
            .map_err(|e| EffectError::ParamParseError(e.to_string()))?;
        
        // Get resource interface
        let resource_interface = self.resource_manager
            .get_resource_interface()
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Execute query
        let query_engine = resource_interface.query_engine();
        
        // Get capability from context
        let capability = context.capability().cloned();
        
        // Execute query (with dynamic resource type)
        let result = query_engine
            .query::<Box<dyn Resource>>(
                &params.query,
                capability.as_ref(),
                params.options,
            )
            .await
            .map_err(|e| ResourceEffectError::QueryError(e))?;
        
        // Create outcome
        let outcome = ResourceQueryOutcome {
            data: result.resources,
            pagination: Some(result.pagination),
            stats: Some(QueryExecutionStats {
                execution_time_ms: result.stats.execution_time_ms,
                resources_matched: result.stats.resources_matched,
                resources_returned: result.stats.resources_returned,
            }),
        };
        
        // Convert to JSON
        serde_json::to_value(outcome)
            .map_err(|e| EffectError::OutcomeSerializationError(e.to_string()))
    }
}

//
// Resource Storage Effects
//

/// Resource store effect
#[derive(Debug, Clone)]
pub struct ResourceStoreEffect;

impl ResourceStoreEffect {
    /// Get effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.store")
    }
}

/// Parameters for resource store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStoreParams {
    /// The resource to store
    pub resource: Box<dyn Resource>,
    
    /// Whether to update if exists
    pub update: bool,
}

/// Outcome of resource store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStoreOutcome {
    /// Stored resource ID
    pub resource_id: ContentId,
    
    /// Whether the resource was created or updated
    pub was_updated: bool,
}

impl Effect for ResourceStoreEffect {
    type Param = ResourceStoreParams;
    type Outcome = ResourceStoreOutcome;
    
    fn type_id(&self) -> EffectTypeId {
        Self::type_id()
    }
}

/// Handler for resource store effects
pub struct ResourceStoreHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceStoreHandler {
    /// Create a new resource store handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceStoreHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceStoreHandler")
            .finish()
    }
}

#[async_trait]
impl :EffectRuntime:causality_core::effect::runtime::EffectRuntime::core::handler::EffectHandler for ResourceStoreHandler {
    async fn handle(
        &self,
        _effect_type: &EffectTypeId,
        param: serde_json::Value,
        context: &Context,
    ) -> EffectResult<serde_json::Value> {
        // Parse parameters
        let params: ResourceStoreParams = serde_json::from_value(param)
            .map_err(|e| EffectError::ParamParseError(e.to_string()))?;
        
        // Get resource interface
        let resource_interface = self.resource_manager
            .get_resource_interface()
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Store resource
        let resource_id = params.resource.resource_id().clone();
        let exists = resource_interface
            .get_resource(&resource_id)
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?
            .is_some();
        
        let was_updated = exists && params.update;
        
        if exists && !params.update {
            return Err(EffectError::SubsystemError(
                format!("Resource already exists: {}", resource_id)
            ));
        }
        
        // Store resource
        resource_interface
            .store_resource(params.resource.as_ref())
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Create outcome
        let outcome = ResourceStoreOutcome {
            resource_id,
            was_updated,
        };
        
        // Convert to JSON
        serde_json::to_value(outcome)
            .map_err(|e| EffectError::OutcomeSerializationError(e.to_string()))
    }
}

/// Resource get effect
#[derive(Debug, Clone)]
pub struct ResourceGetEffect;

impl ResourceGetEffect {
    /// Get effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.get")
    }
}

/// Parameters for resource get
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGetParams {
    /// The resource ID to get
    pub resource_id: ContentId,
    
    /// Resource type (optional for validation)
    pub resource_type: Option<ResourceType>,
}

/// Outcome of resource get
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGetOutcome {
    /// The resource (if found)
    pub resource: Option<Box<dyn Resource>>,
}

impl Effect for ResourceGetEffect {
    type Param = ResourceGetParams;
    type Outcome = ResourceGetOutcome;
    
    fn type_id(&self) -> EffectTypeId {
        Self::type_id()
    }
}

/// Handler for resource get effects
pub struct ResourceGetHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceGetHandler {
    /// Create a new resource get handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceGetHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceGetHandler")
            .finish()
    }
}

#[async_trait]
impl :EffectRuntime:causality_core::effect::runtime::EffectRuntime::core::handler::EffectHandler for ResourceGetHandler {
    async fn handle(
        &self,
        _effect_type: &EffectTypeId,
        param: serde_json::Value,
        context: &Context,
    ) -> EffectResult<serde_json::Value> {
        // Parse parameters
        let params: ResourceGetParams = serde_json::from_value(param)
            .map_err(|e| EffectError::ParamParseError(e.to_string()))?;
        
        // Get resource interface
        let resource_interface = self.resource_manager
            .get_resource_interface()
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Get resource
        let resource = resource_interface
            .get_resource(&params.resource_id)
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Verify resource type if specified
        if let Some(expected_type) = &params.resource_type {
            if let Some(ref res) = resource {
                if res.resource_type() != expected_type {
                    return Err(EffectError::SubsystemError(
                        format!("Resource type mismatch: expected {:?}, got {:?}",
                                expected_type, res.resource_type())
                    ));
                }
            }
        }
        
        // Create outcome
        let outcome = ResourceGetOutcome { resource };
        
        // Convert to JSON
        serde_json::to_value(outcome)
            .map_err(|e| EffectError::OutcomeSerializationError(e.to_string()))
    }
}

/// Resource delete effect
#[derive(Debug, Clone)]
pub struct ResourceDeleteEffect;

impl ResourceDeleteEffect {
    /// Get effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.delete")
    }
}

/// Parameters for resource delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeleteParams {
    /// The resource ID to delete
    pub resource_id: ContentId,
}

/// Outcome of resource delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeleteOutcome {
    /// Whether the resource was deleted
    pub deleted: bool,
}

impl Effect for ResourceDeleteEffect {
    type Param = ResourceDeleteParams;
    type Outcome = ResourceDeleteOutcome;
    
    fn type_id(&self) -> EffectTypeId {
        Self::type_id()
    }
}

/// Handler for resource delete effects
pub struct ResourceDeleteHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceDeleteHandler {
    /// Create a new resource delete handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceDeleteHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceDeleteHandler")
            .finish()
    }
}

#[async_trait]
impl :EffectRuntime:causality_core::effect::runtime::EffectRuntime::core::handler::EffectHandler for ResourceDeleteHandler {
    async fn handle(
        &self,
        _effect_type: &EffectTypeId,
        param: serde_json::Value,
        context: &Context,
    ) -> EffectResult<serde_json::Value> {
        // Parse parameters
        let params: ResourceDeleteParams = serde_json::from_value(param)
            .map_err(|e| EffectError::ParamParseError(e.to_string()))?;
        
        // Get resource interface
        let resource_interface = self.resource_manager
            .get_resource_interface()
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Delete resource
        let deleted = resource_interface
            .delete_resource(&params.resource_id)
            .await
            .map_err(|e| ResourceEffectError::ResourceError(e))?;
        
        // Create outcome
        let outcome = ResourceDeleteOutcome { deleted };
        
        // Convert to JSON
        serde_json::to_value(outcome)
            .map_err(|e| EffectError::OutcomeSerializationError(e.to_string()))
    }
}

/// Resource capability verifier
pub struct ResourceCapabilityVerifier {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceCapabilityVerifier {
    /// Create a new resource capability verifier
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceCapabilityVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceCapabilityVerifier")
            .finish()
    }
}

#[async_trait]
impl super::capability::CapabilityVerifier for ResourceCapabilityVerifier {
    async fn verify_capability(
        &self,
        capability_id: &:EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::id::CapabilityId,
        context: &Context,
    ) -> EffectResult<()> {
        // Check if this is a resource capability
        let capability_string = capability_id.to_string();
        if !capability_string.starts_with("resource:") {
            // Not a resource capability, skip it
            return Ok(());
        }
        
        // Extract resource ID
        let parts: Vec<&str> = capability_string.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(EffectError::InvalidCapability(
                format!("Invalid resource capability ID format: {}", capability_string)
            ));
        }
        
        // Get the capability from the context
        let capability = match context.capability() {
            Some(cap) => cap,
            None => {
                return Err(EffectError::MissingCapability {
                    effect_type: :EffectRuntime:causality_core::effect::runtime::EffectRuntime::types::id::EffectTypeId::new("resource"),
                    capability: capability_string,
                });
            }
        };
        
        // If the capability has the resource ID in its scope, it's valid
        if capability.resource_id() == "*" || capability.resource_id() == parts[1] {
            return Ok(());
        }
        
        // Otherwise, fetch the resource validator and check the capability
        match self.resource_manager.get_resource_validator().await {
            Ok(validator) => {
                let validation_context = causality_core::resource::validation::ValidationContext::new();
                let resource_id = ContentId::from_string(parts[1]).map_err(|_| {
                    EffectError::InvalidCapability(
                        format!("Invalid resource ID in capability: {}", parts[1])
                    )
                })?;
                
                // Validate the capability
                validator.validate_capability(
                    &resource_id, 
                    capability,
                    &validation_context
                ).await
                .map_err(|e| {
                    ResourceEffectError::ValidationError(e).into()
                })
            },
            Err(e) => {
                Err(ResourceEffectError::ResourceError(e).into())
            }
        }
    }
} 