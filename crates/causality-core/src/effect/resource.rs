// Resource Effect Interface
//
// This module provides a comprehensive interface for resource-specific effects,
// including standard operations like Create, Read, Update, Delete, and Transfer.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::capability::{Capability, CapabilityGrants};
use crate::resource::{ResourceId, ContentId};
use super::{Effect, EffectContext, EffectError, EffectOutcome, EffectResult};
use super::types::{EffectId, EffectTypeId};

/// Resource operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceOperation {
    /// Create a new resource
    Create {
        /// Resource type
        resource_type: String,
        /// Initial data
        initial_data: Option<String>,
    },
    
    /// Read a resource
    Read {
        /// Specific fields to read (empty means all)
        fields: Vec<String>,
    },
    
    /// Update a resource
    Update {
        /// Fields to update with their values
        updates: HashMap<String, String>,
        /// Whether this is a partial update
        partial: bool,
    },
    
    /// Delete a resource
    Delete,
    
    /// Transfer a resource to another owner
    Transfer {
        /// New owner
        new_owner: ContentId,
    },
    
    /// Clone a resource
    Clone {
        /// New resource ID
        new_resource_id: Option<ResourceId>,
    },
    
    /// Move a resource to a new location
    Move {
        /// New location
        new_location: String,
    },
    
    /// Custom operation with operation-specific data
    Custom {
        /// Operation name
        operation: String,
        /// Operation data
        data: Option<String>,
    },
}

impl fmt::Display for ResourceOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceOperation::Create { resource_type, .. } => {
                write!(f, "Create({})", resource_type)
            }
            ResourceOperation::Read { .. } => write!(f, "Read"),
            ResourceOperation::Update { .. } => write!(f, "Update"),
            ResourceOperation::Delete => write!(f, "Delete"),
            ResourceOperation::Transfer { .. } => write!(f, "Transfer"),
            ResourceOperation::Clone { .. } => write!(f, "Clone"),
            ResourceOperation::Move { .. } => write!(f, "Move"),
            ResourceOperation::Custom { operation, .. } => {
                write!(f, "Custom({})", operation)
            }
        }
    }
}

/// Outcome of a resource effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEffectOutcome {
    /// Resource ID
    pub resource_id: ResourceId,
    /// Operation that was performed
    pub operation: ResourceOperation,
    /// Whether the operation was successful
    pub success: bool,
    /// Result data (if any)
    pub result: Option<String>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Error specific to resource effects
#[derive(Error, Debug)]
pub enum ResourceEffectError {
    #[error("Resource not found: {0}")]
    NotFound(ResourceId),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(ResourceId),
    
    #[error("Resource type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid field value: {0}")]
    InvalidField(String),
    
    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),
    
    #[error("Resource validation error: {0}")]
    ValidationError(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Dependency error: {0}")]
    DependencyError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Convert ResourceEffectError to EffectError
impl From<ResourceEffectError> for EffectError {
    fn from(err: ResourceEffectError) -> Self {
        EffectError::ExecutionError(err.to_string())
    }
}

/// Base trait for resource effects
#[async_trait]
pub trait ResourceEffect: Effect {
    /// Get the resource ID this effect operates on
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the resource type
    fn resource_type(&self) -> &str;
    
    /// Get the operation this effect will perform
    fn operation(&self) -> &ResourceOperation;
    
    /// Handle the resource effect
    async fn handle_resource_effect(&self, context: &dyn EffectContext) -> Result<ResourceEffectOutcome, ResourceEffectError>;
}

/// Basic resource effect implementation
#[derive(Debug)]
pub struct BasicResourceEffect {
    /// Effect ID
    id: EffectId,
    /// Resource ID
    resource_id: ResourceId,
    /// Resource type
    resource_type: String,
    /// Operation to perform
    operation: ResourceOperation,
}

impl BasicResourceEffect {
    /// Create a new basic resource effect
    pub fn new(
        id: EffectId,
        resource_id: ResourceId,
        resource_type: String,
        operation: ResourceOperation,
    ) -> Self {
        Self {
            id,
            resource_id,
            resource_type,
            operation,
        }
    }
    
    /// Create a result with success
    fn success_result(&self, result: Option<String>) -> ResourceEffectOutcome {
        ResourceEffectOutcome {
            resource_id: self.resource_id.clone(),
            operation: self.operation.clone(),
            success: true,
            result,
            error: None,
        }
    }
    
    /// Create a result with error
    fn error_result(&self, error: String) -> ResourceEffectOutcome {
        ResourceEffectOutcome {
            resource_id: self.resource_id.clone(),
            operation: self.operation.clone(),
            success: false,
            result: None,
            error: Some(error),
        }
    }
    
    /// Check if the context has the required capability for this operation
    fn check_capability(&self, context: &dyn EffectContext) -> Result<(), ResourceEffectError> {
        let required_grants = match &self.operation {
            ResourceOperation::Read { .. } => CapabilityGrants {
                read: true,
                write: false,
                delegate: false,
            },
            ResourceOperation::Create { .. } |
            ResourceOperation::Update { .. } |
            ResourceOperation::Delete |
            ResourceOperation::Clone { .. } |
            ResourceOperation::Move { .. } |
            ResourceOperation::Custom { .. } => CapabilityGrants {
                read: true,
                write: true,
                delegate: false,
            },
            ResourceOperation::Transfer { .. } => CapabilityGrants {
                read: true,
                write: true,
                delegate: true,
            },
        };
        
        let has_capability = context.verify_resource_capability(&self.resource_id, &required_grants)
            .map_err(|e| ResourceEffectError::InternalError(e.to_string()))?;
            
        if !has_capability {
            return Err(ResourceEffectError::Unauthorized(
                format!("Missing required capability for {}", self.operation)
            ));
        }
        
        Ok(())
    }
}

#[async_trait]
impl Effect for BasicResourceEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new(&format!("resource.{}", self.resource_type))
    }
    
    fn name(&self) -> String {
        format!("ResourceEffect::{}", self.operation)
    }
    
    fn dependencies(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn modifications(&self) -> Vec<ResourceId> {
        match self.operation {
            ResourceOperation::Read { .. } => vec![],
            _ => vec![self.resource_id.clone()],
        }
    }
    
    fn clone_effect(&self) -> Box<dyn Effect> {
        Box::new(Self {
            id: self.id.clone(),
            resource_id: self.resource_id.clone(),
            resource_type: self.resource_type.clone(),
            operation: self.operation.clone(),
        })
    }
}

#[async_trait]
impl ResourceEffect for BasicResourceEffect {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    fn operation(&self) -> &ResourceOperation {
        &self.operation
    }
    
    async fn handle_resource_effect(&self, context: &dyn EffectContext) -> Result<ResourceEffectOutcome, ResourceEffectError> {
        // Check if we have the required capability
        self.check_capability(context)?;
        
        match &self.operation {
            ResourceOperation::Create { resource_type, initial_data } => {
                // In a real implementation, we would create the resource here
                self.success_result(Some(format!("Created resource of type {}", resource_type)))
            },
            ResourceOperation::Read { fields } => {
                // In a real implementation, we would read the resource here
                self.success_result(Some(format!("Read fields: {:?}", fields)))
            },
            ResourceOperation::Update { updates, partial } => {
                // In a real implementation, we would update the resource here
                let update_type = if *partial { "partial" } else { "full" };
                self.success_result(Some(format!("Applied {} update with {} fields", update_type, updates.len())))
            },
            ResourceOperation::Delete => {
                // In a real implementation, we would delete the resource here
                self.success_result(Some("Resource deleted".to_string()))
            },
            ResourceOperation::Transfer { new_owner } => {
                // In a real implementation, we would transfer the resource here
                self.success_result(Some(format!("Resource transferred to {}", new_owner)))
            },
            ResourceOperation::Clone { new_resource_id } => {
                // In a real implementation, we would clone the resource here
                let id_info = new_resource_id.as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "auto-generated ID".to_string());
                self.success_result(Some(format!("Resource cloned with {}", id_info)))
            },
            ResourceOperation::Move { new_location } => {
                // In a real implementation, we would move the resource here
                self.success_result(Some(format!("Resource moved to {}", new_location)))
            },
            ResourceOperation::Custom { operation, data } => {
                // In a real implementation, we would perform the custom operation here
                let data_info = data.as_ref()
                    .map(|d| format!(" with data: {}", d))
                    .unwrap_or_default();
                self.success_result(Some(format!("Custom operation '{}'{}", operation, data_info)))
            },
        }
    }
} 