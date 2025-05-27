//-----------------------------------------------------------------------------
// Graph Error Types
//-----------------------------------------------------------------------------

use crate::core::error::ErrorCategory;
use thiserror::Error;

/// Errors related to graph operations
#[derive(Debug, Error)]
pub enum GraphError {
    /// Error when a node with the requested ID is not found
    #[error("Node not found with ID: {0}")]
    NodeNotFound(String),

    /// Error when an edge with the requested ID is not found
    #[error("Edge not found with ID: {0}")]
    EdgeNotFound(String),

    /// Error when attempting to create a node with an ID that already exists
    #[error("Node already exists with ID: {0}")]
    NodeAlreadyExists(String),

    /// Error when attempting to create an edge with an ID that already exists
    #[error("Edge already exists with ID: {0}")]
    EdgeAlreadyExists(String),

    /// Error when attempting to create an edge between non-existent nodes
    #[error(
        "Cannot create edge: source node {0} or target node {1} does not exist"
    )]
    InvalidEdgeNodes(String, String),

    /// General validation error
    #[error("Graph validation error: {0}")]
    ValidationError(String),

    /// Error category for this error type
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl GraphError {
    /// Get the error category for this error
    pub fn category(&self) -> ErrorCategory {
        match self {
            GraphError::NodeNotFound(_) => ErrorCategory::Resource,
            GraphError::EdgeNotFound(_) => ErrorCategory::Resource,
            GraphError::NodeAlreadyExists(_) => ErrorCategory::Validation,
            GraphError::EdgeAlreadyExists(_) => ErrorCategory::Validation,
            GraphError::InvalidEdgeNodes(_, _) => ErrorCategory::Validation,
            GraphError::ValidationError(_) => ErrorCategory::Validation,
            GraphError::Other(_) => ErrorCategory::General,
        }
    }
}
