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

// Use proper imports from causality_crypto and causality_types
use causality_crypto::ContentHash;
use causality_types::{ContentId, ContentAddressed};

pub use interface::*;
pub use types::*;
pub use validation::*;
pub use storage::*;
pub use agent::*;
pub use query::*;

// Re-export specific types from resource_types
pub use crate::resource_types::{ResourceId, ResourceType, ResourceTypeId};

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

// Re-export query system
pub use query::{
    ResourceQuery, QueryEngine, QueryResult, QueryOptions, QueryExecution,
    Filter, FilterExpression, FilterCondition, FilterOperator,
    Sort, SortDirection, SortOptions,
    Pagination, PaginationOptions, PaginationResult,
    ResourceIndex, IndexKey, IndexType, IndexEntry,
    QueryBuilder, FilterBuilder, SortBuilder,
    QueryError
};

// Re-export agent system
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
};

// Re-export agent operation submodule
pub use agent::operation::{
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

/// Resource trait
///
/// This trait defines the core interface for all resources in the system.
pub trait Resource: Send + Sync + Debug {
    /// Get the unique identifier for this resource
    fn id(&self) -> crate::resource_types::ResourceId;
    
    /// Get the type of this resource
    fn resource_type(&self) -> crate::resource_types::ResourceType;
    
    /// Get the current state of this resource
    fn state(&self) -> ResourceState;
    
    /// Get a specific metadata value
    fn get_metadata(&self, key: &str) -> Option<String>;
    
    /// Set a metadata value
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()>;
    
    /// Clone this resource into a boxed trait object
    fn clone_resource(&self) -> Box<dyn Resource>;
}

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