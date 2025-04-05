// Resource management system
//
// This module provides the core resource management system,
// including resource interfaces and SMT integration.

pub mod adapter;
pub mod agent;
pub mod protocol;
pub mod storage;
// pub mod validation; // Temporarily disabled until we fix the compatibility issues
pub mod query;
pub mod interface;
pub mod types;
pub mod operation;

#[cfg(test)]
pub mod tests;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::any::Any;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Use proper imports
use causality_types::ContentId;

pub use interface::*;
pub use types::*;
pub use types::ResourceId;  // Explicit re-export to fix shadowing
// pub use validation::*; // Temporarily disabled until we fix the compatibility issues
pub use storage::*;
// pub use agent::*;
// pub use query::*;

// Re-export key types
// pub use manager::ResourceManager;
// pub use state::ResourceState;
// pub use types::*;

// Specific imports (might shadow glob exports if not careful)
// Remove this specific import to avoid shadowing the glob export from types::*
// use crate::resource_types::{ResourceTypeId};
// use crate::capabilities::CapabilityRegistry;
// use crate::error::ResourceError;
// use crate::identity::IdentityService;
// use crate::lifecycle::LifecycleManager;

// Re-export specific types from resource_types
// Use direct imports here instead of re-exporting
// Import ResourceType directly from our own module
pub use crate::resource::types::ResourceType;
// Explicitly re-export ResourceState
pub use crate::resource::interface::ResourceState;

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
    ValidationError(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(ContentId),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(ContentId),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Resource result
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Resource trait
///
/// This trait defines the core interface for all resources in the system.
pub trait Resource: Send + Sync + Debug + Any {
    /// Get the unique identifier for this resource
    fn id(&self) -> ResourceId;
    
    /// Get the type of this resource
    fn resource_type(&self) -> ResourceType;
    
    /// Get the current state of this resource
    fn state(&self) -> ResourceState;
    
    /// Get a specific metadata value
    fn get_metadata(&self, key: &str) -> Option<String>;
    
    /// Set a metadata value
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()>;
    
    /// Get all metadata as a map
    fn get_metadata_map(&self) -> Option<HashMap<String, String>> {
        None
    }
    
    /// Clone this resource into a boxed trait object
    fn clone_resource(&self) -> Box<dyn Resource>;
    
    /// Convert this resource to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert this resource to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Interface configuration
    pub interface_config: interface::ResourceConfig,
    
    /// Validation configuration
    pub validation_config: Option<HashMap<String, String>>,
    
    /// Resource metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            interface_config: interface::ResourceConfig::default(),
            validation_config: Some(HashMap::new()),
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
    async fn get_resource_validator(&self) -> ResourceResult<Arc<dyn Debug + Send + Sync>>;
    
    /// Start resource manager
    async fn start(&self) -> ResourceResult<()>;
    
    /// Stop resource manager
    async fn stop(&self) -> ResourceResult<()>;

    /// Check if a resource exists
    async fn resource_exists(&self, resource_type: &str, resource_id: &str) -> bool;
    
    /// Create a resource
    async fn create_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str, 
        params: HashMap<String, String>
    ) -> ResourceResult<()>;
    
    /// Get a resource
    async fn get_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str
    ) -> ResourceResult<Box<dyn Resource>>;
    
    /// Update a resource
    async fn update_resource(
        &self, 
        resource_type: &str, 
        resource_id: &str, 
        update_data: HashMap<String, String>
    ) -> ResourceResult<()>;
    
    /// Delete a resource
    async fn delete_resource(
        &self,
        resource_type: &str,
        resource_id: &str
    ) -> ResourceResult<()>;
    
    /// Execute an operation on a resource
    async fn execute_operation(
        &self,
        resource_type: &str,
        resource_id: &str,
        operation: &str,
        params: HashMap<String, String>
    ) -> ResourceResult<HashMap<String, String>>;
}

/// Resource manager factory
#[async_trait]
pub trait ResourceManagerFactory: Send + Sync + Debug {
    /// Create resource manager
    async fn create_manager(&self, config: ResourceConfig) -> ResourceResult<Arc<dyn ResourceManager>>;
    
    /// Get supported configurations
    fn supported_configs(&self) -> Vec<ResourceConfig>;
}

// Implement Resource for Box<dyn Resource> to allow using boxed resources directly
// This is needed for proper handling in query processing
impl Resource for Box<dyn Resource> {
    fn id(&self) -> ResourceId {
        (**self).id()
    }

    fn resource_type(&self) -> ResourceType {
        (**self).resource_type()
    }

    fn state(&self) -> ResourceState {
        (**self).state()
    }

    fn get_metadata(&self, key: &str) -> Option<String> {
        (**self).get_metadata(key)
    }

    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        (**self).set_metadata(key, value)
    }

    fn get_metadata_map(&self) -> Option<HashMap<String, String>> {
        (**self).get_metadata_map()
    }

    fn clone_resource(&self) -> Box<dyn Resource> {
        (**self).clone_resource()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Re-export key types
pub use crate::effect::resource::ResourceOperation; 