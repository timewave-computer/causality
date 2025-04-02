//! Resource integration for the Effect System
//!
//! This module integrates the resource system from causality-core with the
//! effect system in the engine. It provides effects for querying, storing,
//! and managing resources.

use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use causality_core::resource::{
    ResourceManager, ResourceResult, ResourceError, ResourceConfig,
    ResourceQuery, QueryEngine, QueryResult, QueryError, QueryOptions,
    ResourceInterface, Resource, ResourceType
};
use causality_types::ContentId;

use causality_core::effect::{Effect, EffectResult};
use causality_core::effect::context::EffectContext as Context;
use causality_core::effect::{EffectOutcome, EffectType, EffectError};
// Create a local definition for EffectTypeId
use crate::effect::capability::EffectTypeId;

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
            ResourceEffectError::ResourceError(e) => EffectError::Internal(e.to_string()),
            ResourceEffectError::QueryError(e) => EffectError::Internal(e.to_string()),
            ResourceEffectError::IntegrationError(e) => EffectError::Internal(e),
        }
    }
}

/// Manager for resource effects
pub struct ResourceEffectManager {
    /// Resource manager from core
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceEffectManager {
    /// Create a new resource effect manager
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
    
    /// Get the underlying resource manager
    pub fn resource_manager(&self) -> &Arc<dyn ResourceManager> {
        &self.resource_manager
    }
    
    /// Get the resource interface for direct operations
    pub async fn get_resource_interface(&self) -> ResourceResult<Arc<dyn ResourceInterface>> {
        self.resource_manager.get_resource_interface().await
    }
    
    /// Register all resource effects with the registry
    pub fn register_effects(&self, registry: &mut super::registry::EffectRegistry) {
        // Create handlers
        let query_handler = Arc::new(ResourceQueryHandler::new(self.resource_manager.clone()));
        let store_handler = Arc::new(ResourceStoreHandler::new(self.resource_manager.clone()));
        let get_handler = Arc::new(ResourceGetHandler::new(self.resource_manager.clone()));
        let delete_handler = Arc::new(ResourceDeleteHandler::new(self.resource_manager.clone()));
        
        // Register with the registry
        registry.register(ResourceQueryEffect::type_id(), query_handler);
        registry.register(ResourceStoreEffect::type_id(), store_handler);
        registry.register(ResourceGetEffect::type_id(), get_handler);
        registry.register(ResourceDeleteEffect::type_id(), delete_handler);
    }
}

impl fmt::Debug for ResourceEffectManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceEffectManager")
            .finish_non_exhaustive()
    }
}

/// Effect for querying resources
#[derive(Debug)]
pub struct ResourceQueryEffect;

impl ResourceQueryEffect {
    /// Get the effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.query")
    }
}

/// Parameters for resource query effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryParams {
    /// The query to execute
    pub query: ResourceQuery,
    
    /// Query options
    pub options: Option<QueryOptions>,
}

/// Outcome of resource query effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryOutcome {
    /// Query result data
    pub data: Vec<Box<dyn Resource>>,
    
    /// Pagination information
    pub pagination: Option<causality_core::resource::query::PaginationResult>,
    
    /// Query statistics
    pub stats: Option<QueryExecutionStats>,
}

/// Statistics about query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionStats {
    /// Time taken in milliseconds
    pub execution_time_ms: u64,
    
    /// Number of resources matched
    pub resources_matched: usize,
    
    /// Number of resources returned
    pub resources_returned: usize,
}

#[async_trait]
impl Effect for ResourceQueryEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(Self::type_id().to_string())
    }
    
    fn description(&self) -> String {
        "Query resources from the resource database".to_string()
    }
    
    async fn execute(&self, context: &dyn causality_core::effect::EffectContext) -> EffectResult<EffectOutcome> {
        // Simplified implementation - normally would parse params from context
        // and execute the query, but for now we just return success
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for resource query effects
pub struct ResourceQueryHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceQueryHandler {
    /// Create a new query handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceQueryHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceQueryHandler").finish()
    }
}

impl causality_core::effect::handler::EffectHandler for ResourceQueryHandler {
    fn supported_effect_types(&self) -> Vec<causality_core::effect::EffectTypeId> {
        vec![ResourceQueryEffect::type_id()]
    }
    
    async fn handle(
        &self,
        effect: &dyn causality_core::Effect,
        context: &dyn causality_core::effect::context::EffectContext,
    ) -> causality_core::effect::outcome::EffectResult<causality_core::effect::outcome::EffectOutcome> {
        // Simplified implementation
        Ok(causality_core::effect::outcome::EffectOutcome {
            effect_id: Some(causality_core::effect::EffectId(effect.effect_type().to_string())),
            status: causality_core::effect::outcome::EffectStatus::Success,
            data: std::collections::HashMap::new(),
            result: causality_core::effect::outcome::ResultData::None,
            error_message: None,
            affected_resources: vec![],
            child_outcomes: vec![],
            content_hash: None,
        })
    }
}

/// Effect for storing resources
#[derive(Debug)]
pub struct ResourceStoreEffect;

impl ResourceStoreEffect {
    /// Get the effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.store")
    }
}

/// Parameters for resource store effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStoreParams {
    /// The resource to store
    pub resource: Box<dyn Resource>,
    
    /// Whether to update if exists
    pub update: bool,
}

/// Outcome of resource store effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStoreOutcome {
    /// Stored resource ID
    pub resource_id: ContentId,
    
    /// Whether the resource was created or updated
    pub was_updated: bool,
}

#[async_trait]
impl Effect for ResourceStoreEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(Self::type_id().to_string())
    }
    
    fn description(&self) -> String {
        "Store a resource in the resource database".to_string()
    }
    
    async fn execute(&self, context: &dyn causality_core::effect::EffectContext) -> EffectResult<EffectOutcome> {
        // Simplified implementation
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for resource store effects
pub struct ResourceStoreHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceStoreHandler {
    /// Create a new store handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceStoreHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceStoreHandler").finish()
    }
}

impl causality_core::effect::handler::EffectHandler for ResourceStoreHandler {
    fn supported_effect_types(&self) -> Vec<causality_core::effect::EffectTypeId> {
        vec![ResourceStoreEffect::type_id()]
    }
    
    async fn handle(
        &self,
        effect: &dyn causality_core::Effect,
        context: &dyn causality_core::effect::context::EffectContext,
    ) -> causality_core::effect::outcome::EffectResult<causality_core::effect::outcome::EffectOutcome> {
        // Simplified implementation
        Ok(causality_core::effect::outcome::EffectOutcome {
            effect_id: Some(causality_core::effect::EffectId(effect.effect_type().to_string())),
            status: causality_core::effect::outcome::EffectStatus::Success,
            data: std::collections::HashMap::new(),
            result: causality_core::effect::outcome::ResultData::None,
            error_message: None,
            affected_resources: vec![],
            child_outcomes: vec![],
            content_hash: None,
        })
    }
}

/// Effect for getting a resource
#[derive(Debug)]
pub struct ResourceGetEffect;

impl ResourceGetEffect {
    /// Get the effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.get")
    }
}

/// Parameters for resource get effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGetParams {
    /// The resource ID to get
    pub resource_id: ContentId,
    
    /// Resource type (optional for validation)
    pub resource_type: Option<ResourceType>,
}

/// Outcome of resource get effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGetOutcome {
    /// The resource (if found)
    pub resource: Option<Box<dyn Resource>>,
}

#[async_trait]
impl Effect for ResourceGetEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(Self::type_id().to_string())
    }
    
    fn description(&self) -> String {
        "Get a resource from the resource database".to_string()
    }
    
    async fn execute(&self, context: &dyn causality_core::effect::EffectContext) -> EffectResult<EffectOutcome> {
        // Simplified implementation
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for resource get effects
pub struct ResourceGetHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceGetHandler {
    /// Create a new get handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceGetHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceGetHandler").finish()
    }
}

impl causality_core::effect::handler::EffectHandler for ResourceGetHandler {
    fn supported_effect_types(&self) -> Vec<causality_core::effect::EffectTypeId> {
        vec![ResourceGetEffect::type_id()]
    }
    
    async fn handle(
        &self,
        effect: &dyn causality_core::Effect,
        context: &dyn causality_core::effect::context::EffectContext,
    ) -> causality_core::effect::outcome::EffectResult<causality_core::effect::outcome::EffectOutcome> {
        // Simplified implementation
        Ok(causality_core::effect::outcome::EffectOutcome {
            effect_id: Some(causality_core::effect::EffectId(effect.effect_type().to_string())),
            status: causality_core::effect::outcome::EffectStatus::Success,
            data: std::collections::HashMap::new(),
            result: causality_core::effect::outcome::ResultData::None,
            error_message: None,
            affected_resources: vec![],
            child_outcomes: vec![],
            content_hash: None,
        })
    }
}

/// Effect for deleting a resource
#[derive(Debug)]
pub struct ResourceDeleteEffect;

impl ResourceDeleteEffect {
    /// Get the effect type ID
    pub fn type_id() -> EffectTypeId {
        EffectTypeId::new("resource.delete")
    }
}

/// Parameters for resource delete effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeleteParams {
    /// The resource ID to delete
    pub resource_id: ContentId,
}

/// Outcome of resource delete effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeleteOutcome {
    /// Whether the resource was deleted
    pub deleted: bool,
}

#[async_trait]
impl Effect for ResourceDeleteEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(Self::type_id().to_string())
    }
    
    fn description(&self) -> String {
        "Delete a resource from the resource database".to_string()
    }
    
    async fn execute(&self, context: &dyn causality_core::effect::EffectContext) -> EffectResult<EffectOutcome> {
        // Simplified implementation
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for resource delete effects
pub struct ResourceDeleteHandler {
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl ResourceDeleteHandler {
    /// Create a new delete handler
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

impl fmt::Debug for ResourceDeleteHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceDeleteHandler").finish()
    }
}

impl causality_core::effect::handler::EffectHandler for ResourceDeleteHandler {
    fn supported_effect_types(&self) -> Vec<causality_core::effect::EffectTypeId> {
        vec![ResourceDeleteEffect::type_id()]
    }
    
    async fn handle(
        &self,
        effect: &dyn causality_core::Effect,
        context: &dyn causality_core::effect::context::EffectContext,
    ) -> causality_core::effect::outcome::EffectResult<causality_core::effect::outcome::EffectOutcome> {
        // Simplified implementation
        Ok(causality_core::effect::outcome::EffectOutcome {
            effect_id: Some(causality_core::effect::EffectId(effect.effect_type().to_string())),
            status: causality_core::effect::outcome::EffectStatus::Success,
            data: std::collections::HashMap::new(),
            result: causality_core::effect::outcome::ResultData::None,
            error_message: None,
            affected_resources: vec![],
            child_outcomes: vec![],
            content_hash: None,
        })
    }
}

/// Verifier for resource capabilities
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
        f.debug_struct("ResourceCapabilityVerifier").finish()
    }
}

// Simplified implementation of the capability verification
#[async_trait]
impl super::capability::CapabilityVerifier for ResourceCapabilityVerifier {
    async fn verify_capability(
        &self,
        capability_id: &super::capability::CapabilityId,
        context: &dyn causality_core::effect::context::EffectContext,
    ) -> EffectResult<()> {
        // Simplified implementation
        Ok(())
    }
} 