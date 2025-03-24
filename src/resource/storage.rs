//! Storage strategies and effects for the resource system
//!
//! This module defines the different storage strategies that can be used
//! for storing resources, as well as the storage effects that implement
//! the underlying storage operations.

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::any::Any;

use async_trait::async_trait;
#[cfg(feature = "tracing")]
use tracing::{debug, info, warn, error};
#[cfg(not(feature = "tracing"))]
use std::{println as info, println as debug, println as warn, println as error};

use crate::effect::{Effect, EffectContext, EffectOutcome, EffectId};
use crate::effect::boundary::ExecutionBoundary;
use crate::crypto::hash::ContentId;
use crate::domain::DomainType;
use crate::error::{Error, Result};
use crate::resource::resource_register::{ResourceRegister, StateVisibility};
use serde::{Serialize, Deserialize};

/// Storage strategy for resources
///
/// Defines where and how a resource should be stored,
/// which can vary based on the domain and specific requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageStrategy {
    /// Store resource fully on chain
    OnChain,
    /// Store only a commitment on chain
    Commitment,
    /// Store a nullifier on chain
    Nullifier,
    /// Read-only operation from storage
    ReadOnly,
}

/// Parameters used for storage operations
#[derive(Debug, Clone)]
pub struct StorageParams {
    /// Strategy for this storage operation
    pub strategy: StorageStrategy,
    /// Domain type where the storage operation will occur
    pub domain: DomainType,
    /// Additional parameters specific to the operation
    pub params: HashMap<String, String>,
}

/// Storage effect for resource storage operations
///
/// This effect provides an interface for storing resources
/// using different storage strategies.
#[derive(Debug, Clone)]
pub struct StorageEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// The resource being operated on
    resource: Option<ResourceRegister>,
    /// The resource ID for read operations
    resource_id: Option<ContentId>,
    /// Operation parameters
    params: StorageParams,
    /// Operation type
    operation: StorageOperation,
}

/// Storage operation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageOperation {
    /// Store a resource
    Store,
    /// Read a resource
    Read,
    /// Update a resource
    Update,
    /// Delete a resource
    Delete,
}

impl StorageStrategy {
    /// Get all available storage strategies
    pub fn all() -> Vec<StorageStrategy> {
        vec![
            StorageStrategy::OnChain,
            StorageStrategy::Commitment,
            StorageStrategy::Nullifier,
            StorageStrategy::ReadOnly,
        ]
    }
    
    /// Convert a string to a storage strategy
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "onchain" => Some(StorageStrategy::OnChain),
            "commitment" => Some(StorageStrategy::Commitment),
            "nullifier" => Some(StorageStrategy::Nullifier),
            "readonly" => Some(StorageStrategy::ReadOnly),
            _ => None,
        }
    }
    
    /// Convert a storage strategy to a string
    pub fn to_str(&self) -> &'static str {
        match self {
            StorageStrategy::OnChain => "onchain",
            StorageStrategy::Commitment => "commitment",
            StorageStrategy::Nullifier => "nullifier",
            StorageStrategy::ReadOnly => "readonly",
        }
    }
    
    /// Create a fully on-chain storage strategy with public visibility
    pub fn fully_on_chain() -> crate::resource::resource_register::StorageStrategy {
        crate::resource::resource_register::StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        }
    }
    
    /// Convert to the unified ResourceRegister storage strategy
    pub fn to_unified_strategy(&self) -> crate::resource::resource_register::StorageStrategy {
        match self {
            StorageStrategy::OnChain => {
                crate::resource::resource_register::StorageStrategy::FullyOnChain {
                    visibility: StateVisibility::Public,
                }
            },
            StorageStrategy::Commitment => {
                crate::resource::resource_register::StorageStrategy::CommitmentBased {
                    commitment: None,
                    nullifier: None,
                }
            },
            StorageStrategy::Nullifier => {
                crate::resource::resource_register::StorageStrategy::CommitmentBased {
                    commitment: None,
                    nullifier: None,
                }
            },
            StorageStrategy::ReadOnly => {
                crate::resource::resource_register::StorageStrategy::ReadOnly {
                    commitment: None,
                    nullifier: None,
                }
            },
        }
    }
}

impl StorageEffect {
    /// Create a new storage effect for storing a resource
    pub fn new_store(
        resource: ResourceRegister,
        strategy: StorageStrategy,
        domain: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource: Some(resource),
            resource_id: None,
            params: StorageParams {
                strategy,
                domain,
                params: HashMap::new(),
            },
            operation: StorageOperation::Store,
        }
    }

    /// Create a new storage effect for reading a resource
    pub fn new_read(
        resource_id: ContentId,
        strategy: StorageStrategy,
        domain: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource: None,
            resource_id: Some(resource_id),
            params: StorageParams {
                strategy,
                domain,
                params: HashMap::new(),
            },
            operation: StorageOperation::Read,
        }
    }

    /// Create a new storage effect for updating a resource
    pub fn new_update(
        resource: ResourceRegister,
        strategy: StorageStrategy,
        domain: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource: Some(resource),
            resource_id: None,
            params: StorageParams {
                strategy,
                domain,
                params: HashMap::new(),
            },
            operation: StorageOperation::Update,
        }
    }

    /// Create a new storage effect for deleting a resource
    pub fn new_delete(
        resource_id: ContentId,
        strategy: StorageStrategy,
        domain: DomainType,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource: None,
            resource_id: Some(resource_id),
            params: StorageParams {
                strategy,
                domain,
                params: HashMap::new(),
            },
            operation: StorageOperation::Delete,
        }
    }

    /// Add a parameter to the storage effect
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.params.insert(key.to_string(), value.to_string());
        self
    }
}

#[async_trait]
impl Effect for StorageEffect {
    /// Get the unique identifier for this effect
    fn id(&self) -> EffectId {
        self.id.clone()
    }

    /// Get the boundary layer for this effect
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }

    /// Execute the storage effect
    async fn execute(&self, context: &EffectContext) -> Result<EffectOutcome> {
        let mut outcome = EffectOutcome::success(self.id());

        // In a real implementation, this would connect to the actual storage backend
        // based on the domain type and execute the appropriate operation

        match self.operation {
            StorageOperation::Store => {
                // Ensure we have a resource to store
                let resource = self.resource.as_ref().ok_or_else(|| {
                    Error::ValidationError("No resource provided for store operation".to_string())
                })?;

                // Log the store operation
                info!(
                    "Storing resource {} with strategy {:?} in domain {:?}",
                    resource.id(),
                    self.params.strategy,
                    self.params.domain
                );

                // In a real implementation, we would store the resource in the relevant backend
                // For now, just simulate a successful store operation
                outcome = outcome.with_data("transaction_id", "tx_12345");
                outcome = outcome.with_data("resource_id", &resource.id().to_string());
            }
            StorageOperation::Read => {
                // Ensure we have a resource ID to read
                let resource_id = self.resource_id.as_ref().ok_or_else(|| {
                    Error::ValidationError("No resource ID provided for read operation".to_string())
                })?;

                // Log the read operation
                info!(
                    "Reading resource {} with strategy {:?} from domain {:?}",
                    resource_id,
                    self.params.strategy,
                    self.params.domain
                );

                // In a real implementation, we would read the resource from the relevant backend
                // For now, just simulate a successful read operation
                outcome = outcome.with_data("found", "true");
                outcome = outcome.with_data("resource_id", &resource_id.to_string());
            }
            StorageOperation::Update => {
                // Ensure we have a resource to update
                let resource = self.resource.as_ref().ok_or_else(|| {
                    Error::ValidationError("No resource provided for update operation".to_string())
                })?;

                // Log the update operation
                info!(
                    "Updating resource {} with strategy {:?} in domain {:?}",
                    resource.id(),
                    self.params.strategy,
                    self.params.domain
                );

                // In a real implementation, we would update the resource in the relevant backend
                // For now, just simulate a successful update operation
                outcome = outcome.with_data("transaction_id", "tx_update_67890");
                outcome = outcome.with_data("resource_id", &resource.id().to_string());
            }
            StorageOperation::Delete => {
                // Ensure we have a resource ID to delete
                let resource_id = self.resource_id.as_ref().ok_or_else(|| {
                    Error::ValidationError("No resource ID provided for delete operation".to_string())
                })?;

                // Log the delete operation
                info!(
                    "Deleting resource {} with strategy {:?} from domain {:?}",
                    resource_id,
                    self.params.strategy,
                    self.params.domain
                );

                // In a real implementation, we would delete the resource from the relevant backend
                // For now, just simulate a successful delete operation
                outcome = outcome.with_data("transaction_id", "tx_delete_24680");
                outcome = outcome.with_data("resource_id", &resource_id.to_string());
            }
        }

        Ok(outcome)
    }

    /// Get a description of this effect
    fn description(&self) -> String {
        match self.operation {
            StorageOperation::Store => {
                if let Some(resource) = &self.resource {
                    format!(
                        "Store resource {} with strategy {:?} in domain {:?}",
                        resource.id(),
                        self.params.strategy,
                        self.params.domain
                    )
                } else {
                    "Store resource (no resource provided)".to_string()
                }
            }
            StorageOperation::Read => {
                if let Some(resource_id) = &self.resource_id {
                    format!(
                        "Read resource {} with strategy {:?} from domain {:?}",
                        resource_id,
                        self.params.strategy,
                        self.params.domain
                    )
                } else {
                    "Read resource (no resource ID provided)".to_string()
                }
            }
            StorageOperation::Update => {
                if let Some(resource) = &self.resource {
                    format!(
                        "Update resource {} with strategy {:?} in domain {:?}",
                        resource.id(),
                        self.params.strategy,
                        self.params.domain
                    )
                } else {
                    "Update resource (no resource provided)".to_string()
                }
            }
            StorageOperation::Delete => {
                if let Some(resource_id) = &self.resource_id {
                    format!(
                        "Delete resource {} with strategy {:?} from domain {:?}",
                        resource_id,
                        self.params.strategy,
                        self.params.domain
                    )
                } else {
                    "Delete resource (no resource ID provided)".to_string()
                }
            }
        }
    }

    /// Validate this effect before execution
    async fn validate(&self, _context: &EffectContext) -> Result<()> {
        // Validate based on operation type
        match self.operation {
            StorageOperation::Store | StorageOperation::Update => {
                // Ensure we have a resource for store/update operations
                if self.resource.is_none() {
                    return Err(Error::ValidationError(
                        "Resource is required for store/update operations".to_string(),
                    ));
                }
            }
            StorageOperation::Read | StorageOperation::Delete => {
                // Ensure we have a resource ID for read/delete operations
                if self.resource_id.is_none() {
                    return Err(Error::ValidationError(
                        "Resource ID is required for read/delete operations".to_string(),
                    ));
                }
            }
        }

        // Validate domain is provided
        if self.params.domain == DomainType::Unknown {
            return Err(Error::ValidationError(
                "Valid domain must be specified for storage operations".to_string(),
            ));
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 
