// Resource management system
//
// This module provides the core resource management system,
// including resource interfaces and SMT integration.

pub mod agent;
pub mod protocol;
pub mod storage;
pub mod validation;
pub mod query;
pub mod interface;
pub mod types;

#[cfg(test)]
pub mod tests;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

pub use interface::*;

pub use types::{
    ResourceTypeId,
    ResourceSchema,
    ResourceTypeCompatibility,
    ResourceTypeDefinition,
    ResourceTypeRegistry,
    ResourceTypeRegistryError,
    ResourceTypeRegistryResult,
    ContentAddressedResourceTypeRegistry,
    InMemoryResourceTypeRegistry,
    create_resource_type_registry,
};

pub use protocol::{
    CrossDomainResourceId,
    ResourceProjectionType,
    VerificationLevel,
    ResourceReference,
    VerificationResult,
    TransferStatus,
    ResourceTransferOperation,
    CrossDomainProtocolError,
    CrossDomainProtocolResult,
    CrossDomainResourceProtocol,
    DomainResourceAdapter,
    BasicCrossDomainResourceProtocol,
    create_cross_domain_protocol,
};

pub use storage::{
    ResourceStorage, ResourceStorageError, ResourceStorageResult,
    ResourceVersion, ResourceIndexEntry, ContentAddressedResourceStorage,
    InMemoryResourceStorage, ResourceStorageConfig, create_resource_storage,
};

// Re-export query system
pub use query::{
    ResourceQuery, QueryEngine, FilterExpression, FilterCondition, FilterOperator, FilterValue,
    Sort, SortDirection, Pagination, QueryError, QueryResult, QueryOptions, QueryExecution,
    ResourceIndex, InMemoryResourceIndex, BasicQueryEngine,
};

pub use agent::{
    AgentId,
    AgentType,
    AgentState,
    AgentRelationship,
    RelationshipType,
    AgentError,
    Agent,
    AgentImpl,
    AgentBuilder,
    Operation,
    OperationId,
    OperationType,
    OperationContext,
    OperationResult,
    OperationStatus,
    OperationError,
    OperationBuilder,
};

/// Resource error
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    /// Interface error
    #[error("Interface error: {0}")]
    InterfaceError(#[from] interface::ResourceError),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(#[from] validation::ValidationError),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
}

/// Resource result
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Interface configuration
    pub interface_config: interface::ResourceConfig,
    
    /// Validation configuration
    pub validation_config: Option<validation::ResourceValidatorConfig>,
    
    /// Resource metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            interface_config: interface::ResourceConfig::default(),
            validation_config: Some(validation::ResourceValidatorConfig::default()),
            metadata: HashMap::new(),
        }
    }
}

/// Resource manager
#[async_trait]
pub trait ResourceManager: Send + Sync + Debug {
    /// Get resource configuration
    fn get_config(&self) -> &ResourceConfig;
    
    /// Get resource interface
    async fn get_resource_interface(&self) -> ResourceResult<Arc<dyn ResourceInterface>>;
    
    /// Get resource validator
    async fn get_resource_validator(&self) -> ResourceResult<Arc<validation::ResourceValidator>>;
    
    /// Start resource manager
    async fn start(&self) -> ResourceResult<()>;
    
    /// Stop resource manager
    async fn stop(&self) -> ResourceResult<()>;
}

/// Resource manager factory
#[async_trait]
pub trait ResourceManagerFactory: Send + Sync + Debug {
    /// Create resource manager
    async fn create_manager(&self, config: ResourceConfig) -> ResourceResult<Arc<dyn ResourceManager>>;
    
    /// Get supported configurations
    fn supported_configs(&self) -> Vec<ResourceConfig>;
} 